use std::fs;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use notify::RecommendedWatcher;
use tauri::async_runtime::{self, JoinHandle};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use super::engine::{Engine, Report};
use super::view::{AlertView, UsageView};
use super::{clock, locale};
use crate::domain::alerts::Alert;
use crate::domain::clock::Timestamp;
use crate::domain::preferences::Preferences;
use crate::error::AppError;
use crate::i18n::Text;
use crate::sources::jsonl::{self, scanner::JsonlSource, watch};
use crate::sources::oauth::credentials::{self, CredentialsError};
use crate::sources::oauth::poll::{self, PollResult};
use crate::sources::oauth::schedule::{Jitter, PollSchedule};
use crate::sources::oauth::transport::HttpsTransport;
use crate::sources::oauth::{OAuthError, OAuthUsageSource};
use crate::store::{Database, DatabaseError};
use crate::toast;
use crate::tray::{self, reading::TrayReading};

pub const USAGE_EVENT: &str = "usage://updated";
pub const ALERT_EVENT: &str = "usage://alert";

const DATABASE_FILE: &str = "plimsoll.sqlite";
const TICK: Duration = Duration::from_secs(60);
const DEBOUNCE: Duration = Duration::from_millis(300);

struct Shared {
    engine: Mutex<Engine>,
    poller: Mutex<Option<JoinHandle<()>>>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    refresh: Sender<()>,
}

pub fn start<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let engine = Engine::new(open_database(app)?)?;
    let accurate_mode = engine.accurate_mode()?;
    if let Err(error) = engine.prune(clock::now()) {
        eprintln!("failed to prune old usage: {error}");
    }
    let (refresh, changes) = mpsc::channel();
    app.manage(Shared {
        engine: Mutex::new(engine),
        poller: Mutex::new(None),
        watcher: Mutex::new(None),
        refresh,
    });
    let root = jsonl::projects_root();
    if let Some(root) = &root {
        watch_projects(app, root);
    }
    spawn_jsonl_worker(app.clone(), root, changes);
    if accurate_mode {
        start_poller(app);
    }
    Ok(())
}

pub fn current_view<R: Runtime>(app: &AppHandle<R>) -> Option<UsageView> {
    let now = clock::now();
    report(app, now).map(|report| {
        UsageView::from_report(&report, locale::resolve(report.preferences.language), now)
    })
}

pub fn set_accurate_mode<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Option<UsageView> {
    with_engine(app, |engine| engine.set_accurate_mode(enabled))?;
    if enabled {
        start_poller(app);
    } else {
        stop_poller(app);
    }
    publish(app)
}

pub fn set_preferences<R: Runtime>(
    app: &AppHandle<R>,
    preferences: Preferences,
) -> Option<UsageView> {
    let accurate_mode = with_engine(app, |engine| {
        engine.set_preferences(preferences)?;
        engine.accurate_mode()
    })?;
    if accurate_mode {
        start_poller(app);
    }
    publish(app)
}

fn open_database<R: Runtime>(app: &AppHandle<R>) -> Result<Database, DatabaseError> {
    let Some(path) = database_path(app) else {
        return Database::open_in_memory();
    };
    Database::open(&path).or_else(|error| {
        eprintln!("failed to open usage database, keeping usage in memory: {error}");
        Database::open_in_memory()
    })
}

fn database_path<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let directory = app.path().app_local_data_dir().ok()?;
    fs::create_dir_all(&directory).ok()?;
    Some(directory.join(DATABASE_FILE))
}

fn watch_projects<R: Runtime>(app: &AppHandle<R>, root: &Path) {
    let Some(shared) = app.try_state::<Shared>() else {
        return;
    };
    let refresh = shared.refresh.clone();
    match watch::start(root, move || {
        refresh.send(()).ok();
    }) {
        Ok(watcher) => {
            *shared
                .watcher
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = Some(watcher);
        }
        Err(error) => eprintln!("failed to watch claude code usage: {error}"),
    }
}

fn spawn_jsonl_worker<R: Runtime>(app: AppHandle<R>, root: Option<PathBuf>, changes: Receiver<()>) {
    thread::spawn(move || {
        let offsets = with_engine(&app, |engine| engine.offsets()).unwrap_or_default();
        let mut source = root.map(|root| JsonlSource::with_offsets(root, offsets));
        loop {
            if let Some(source) = source.as_mut() {
                ingest(&app, source);
            }
            publish(&app);
            match changes.recv_timeout(TICK) {
                Ok(()) => settle(&changes),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
    });
}

fn settle(changes: &Receiver<()>) {
    thread::sleep(DEBOUNCE);
    while changes.try_recv().is_ok() {}
}

fn ingest<R: Runtime>(app: &AppHandle<R>, source: &mut JsonlSource) {
    let before = source.offsets();
    match source.poll() {
        Ok(events) => {
            let after = source.offsets();
            if !events.is_empty() || after != before {
                with_engine(app, |engine| engine.store_jsonl(&events, &after));
            }
        }
        Err(error) => eprintln!("failed to read claude code usage: {error}"),
    }
}

fn start_poller<R: Runtime>(app: &AppHandle<R>) {
    let Some(shared) = app.try_state::<Shared>() else {
        return;
    };
    let mut poller = shared.poller.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(previous) = poller.take() {
        previous.abort();
    }
    let interval = with_engine(app, |engine| engine.preferences())
        .unwrap_or_default()
        .poll_interval;
    let handle = app.clone();
    *poller = Some(async_runtime::spawn(async move {
        poll_oauth(handle, PollSchedule::new(interval.duration())).await;
    }));
}

fn stop_poller<R: Runtime>(app: &AppHandle<R>) {
    if let Some(shared) = app.try_state::<Shared>()
        && let Some(poller) = shared
            .poller
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    {
        poller.abort();
    }
}

async fn poll_oauth<R: Runtime>(app: AppHandle<R>, schedule: PollSchedule) {
    let Some(path) = credentials::credentials_path() else {
        record(
            &app,
            &Err(OAuthError::Credentials(CredentialsError::Missing)),
        );
        return;
    };
    let transport = match HttpsTransport::new() {
        Ok(transport) => transport,
        Err(error) => {
            record(&app, &Err(error.into()));
            return;
        }
    };
    let source = OAuthUsageSource::new(transport, path);
    poll::run(&source, schedule, clock::now, Jitter::random, |result| {
        record(&app, &result);
        ControlFlow::Continue(())
    })
    .await;
}

fn record<R: Runtime>(app: &AppHandle<R>, result: &PollResult) {
    let now = clock::now();
    let outcome = with_engine(app, |engine| {
        engine.record_oauth(result, now)?;
        Ok((engine.take_alerts(now)?, engine.preferences()?.language))
    });
    publish(app);
    if let Some((alerts, language)) = outcome {
        let text = locale::text(language);
        for alert in &alerts {
            announce(app, alert, text, now);
        }
    }
}

fn announce<R: Runtime>(app: &AppHandle<R>, alert: &Alert, text: Text, now: Timestamp) {
    let message = toast::message(alert, text, now);
    if let Err(error) = toast::show(app, &message) {
        eprintln!("failed to show a notification: {error}");
    }
    if let Err(error) = app.emit(ALERT_EVENT, AlertView::new(alert, message)) {
        eprintln!("failed to publish an alert: {error}");
    }
}

fn publish<R: Runtime>(app: &AppHandle<R>) -> Option<UsageView> {
    let now = clock::now();
    let report = report(app, now)?;
    let language = locale::resolve(report.preferences.language);
    let view = UsageView::from_report(&report, language, now);
    if let Err(error) = app.emit(USAGE_EVENT, &view) {
        eprintln!("failed to publish usage: {error}");
    }
    let reading = TrayReading::from_summary(&report.summary);
    if let Err(error) = tray::show_reading(
        app,
        &reading,
        report.preferences.thresholds,
        Text::new(language),
        now,
    ) {
        eprintln!("failed to update the tray icon: {error}");
    }
    Some(view)
}

fn report<R: Runtime>(app: &AppHandle<R>, now: Timestamp) -> Option<Report> {
    with_engine(app, |engine| engine.report(now))
}

fn with_engine<R, T, F>(app: &AppHandle<R>, action: F) -> Option<T>
where
    R: Runtime,
    F: FnOnce(&mut Engine) -> Result<T, DatabaseError>,
{
    let shared = app.try_state::<Shared>()?;
    let mut engine = shared.engine.lock().unwrap_or_else(PoisonError::into_inner);
    match action(&mut engine) {
        Ok(value) => Some(value),
        Err(error) => {
            eprintln!("usage database error: {error}");
            None
        }
    }
}
