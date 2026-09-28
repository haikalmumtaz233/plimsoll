use std::fs;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use notify::RecommendedWatcher;
use tauri::async_runtime::{self, JoinHandle};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::sync::Notify;

use super::engine::{Engine, Report};
use super::startup::Startup;
use super::view::{AlertView, UsageView, ViewContext};
use super::{clock, locale};
use crate::diagnostics;
use crate::domain::alerts::Alert;
use crate::domain::clock::{Span, Timestamp};
use crate::domain::limit::{LimitKind, Utilization};
use crate::domain::preferences::{Language, Preferences};
use crate::domain::refresh::{
    Activity, Attempts, MIN_SPACING, ManualRefresh, adaptive_delay, manual_refresh,
};
use crate::error::AppError;
use crate::i18n::Text;
use crate::sources::jsonl::{self, scanner::JsonlSource, watch};
use crate::sources::oauth::credentials::{self, CredentialsError};
use crate::sources::oauth::poll::{self, PollResult};
use crate::sources::oauth::schedule::{Jitter, PollSchedule};
use crate::sources::oauth::status::OAuthStatus;
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
const POPUP_FRESHNESS: Span = Span::minutes(2);
const CODING_FRESHNESS: Span = Span::minutes(5);

#[derive(Debug, Clone, Copy, Default)]
struct SyncState {
    activity: Activity,
    attempts: Attempts,
}

struct Shared {
    engine: Mutex<Engine>,
    poller: Mutex<Option<JoinHandle<()>>>,
    wake: Arc<Notify>,
    sync: Mutex<SyncState>,
    watcher: Mutex<Option<RecommendedWatcher>>,
    refresh: Sender<()>,
    menu_language: Mutex<Option<Language>>,
    startup: Option<Startup>,
}

pub fn start<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let engine = Engine::new(open_database(app)?)?;
    if engine.migrate_settings()? {
        diagnostics::info(
            "store",
            "moved the legacy 1 minute poll interval to adaptive",
        );
    }
    let accurate_mode = engine.accurate_mode()?;
    if let Err(error) = engine.prune(clock::now()) {
        diagnostics::error("store", &format!("failed to prune old usage: {error}"));
    }
    let (refresh, changes) = mpsc::channel();
    app.manage(Shared {
        engine: Mutex::new(engine),
        poller: Mutex::new(None),
        wake: Arc::new(Notify::new()),
        sync: Mutex::new(SyncState::default()),
        watcher: Mutex::new(None),
        refresh,
        menu_language: Mutex::new(None),
        startup: Startup::current(&app.package_info().name)
            .inspect_err(|error| {
                diagnostics::warn(
                    "startup",
                    &format!("start with windows is unavailable: {error}"),
                );
            })
            .ok(),
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

pub fn note_popup_opened<R: Runtime>(app: &AppHandle<R>) {
    let now = clock::now();
    update_sync(app, |state| state.activity.popup_opened_at = Some(now));
    wake_if_stale(app, POPUP_FRESHNESS, now);
}

pub fn current_view<R: Runtime>(app: &AppHandle<R>) -> Option<UsageView> {
    let now = clock::now();
    report(app, now).map(|report| view_of(app, &report, now))
}

pub fn refresh_now<R: Runtime>(app: &AppHandle<R>) -> Option<UsageView> {
    let shared = app.try_state::<Shared>()?;
    shared.refresh.send(()).ok();
    let now = clock::now();
    if manual_state(app, now) == ManualRefresh::Ready {
        update_sync(app, |state| state.attempts.requested = Some(now));
        shared.wake.notify_one();
        diagnostics::info("oauth", "manual refresh requested");
    }
    publish(app)
}

fn view_of<R: Runtime>(app: &AppHandle<R>, report: &Report, now: Timestamp) -> UsageView {
    UsageView::from_report(
        report,
        ViewContext {
            language: locale::resolve(report.preferences.language),
            autostart: autostart_enabled(app),
            refresh: manual_state(app, now),
            now,
        },
    )
}

fn manual_state<R: Runtime>(app: &AppHandle<R>, now: Timestamp) -> ManualRefresh {
    let healthy = with_engine(app, |engine| {
        Ok(engine.accurate_mode()? && engine.status() == OAuthStatus::Active)
    })
    .unwrap_or(false);
    manual_refresh(healthy, sync_state(app).attempts, now)
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

pub fn set_manual_reading<R: Runtime>(
    app: &AppHandle<R>,
    kind: LimitKind,
    utilization: Option<Utilization>,
) -> Option<UsageView> {
    with_engine(app, |engine| {
        engine.set_manual_reading(kind, utilization, clock::now())
    })?;
    publish(app)
}

pub fn set_autostart<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Option<UsageView> {
    let shared = app.try_state::<Shared>()?;
    let startup = shared.startup.as_ref()?;
    let result = if enabled {
        startup.enable()
    } else {
        startup.disable()
    };
    if let Err(error) = result {
        diagnostics::error(
            "startup",
            &format!("failed to change start with windows: {error}"),
        );
        return None;
    }
    publish(app)
}

fn autostart_enabled<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<Shared>()
        .and_then(|shared| shared.startup.as_ref().map(Startup::is_enabled))
        .unwrap_or(false)
}

fn open_database<R: Runtime>(app: &AppHandle<R>) -> Result<Database, DatabaseError> {
    let Some(path) = database_path(app) else {
        return Database::open_in_memory();
    };
    Database::open(&path).or_else(|error| {
        diagnostics::error(
            "store",
            &format!("failed to open usage database, keeping usage in memory: {error}"),
        );
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
        Err(error) => diagnostics::error(
            "jsonl",
            &format!("failed to watch claude code usage: {error}"),
        ),
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
            if let Some(latest) = events.iter().map(|keyed| keyed.event.at).max() {
                note_coding(app, latest);
            }
        }
        Err(error) => diagnostics::warn(
            "jsonl",
            &format!("failed to read claude code usage: {error}"),
        ),
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
    let handle = app.clone();
    *poller = Some(async_runtime::spawn(async move {
        poll_oauth(handle).await;
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

async fn poll_oauth<R: Runtime>(app: AppHandle<R>) {
    let Some(wake) = wake_handle(&app) else {
        return;
    };
    tokio::time::sleep(restart_pause(&app)).await;
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
    poll::run(
        &source,
        PollSchedule::default(),
        clock::now,
        || cadence(&app),
        Jitter::random,
        &wake,
        |result| {
            record(&app, &result);
            ControlFlow::Continue(())
        },
    )
    .await;
}

fn wake_handle<R: Runtime>(app: &AppHandle<R>) -> Option<Arc<Notify>> {
    app.try_state::<Shared>()
        .map(|shared| Arc::clone(&shared.wake))
}

fn cadence<R: Runtime>(app: &AppHandle<R>) -> Duration {
    let interval = with_engine(app, |engine| engine.preferences())
        .unwrap_or_default()
        .poll_interval;
    let activity = sync_state(app).activity;
    interval
        .fixed_duration()
        .unwrap_or_else(|| adaptive_delay(activity, clock::now()).to_duration())
}

fn restart_pause<R: Runtime>(app: &AppHandle<R>) -> Duration {
    let now = clock::now();
    sync_state(app)
        .attempts
        .last
        .map_or(Duration::ZERO, |at| (at + MIN_SPACING - now).to_duration())
}

fn note_coding<R: Runtime>(app: &AppHandle<R>, latest: Timestamp) {
    let now = clock::now();
    if now - latest > CODING_FRESHNESS {
        return;
    }
    update_sync(app, |state| state.activity.coding_at = Some(latest));
    wake_if_stale(app, CODING_FRESHNESS, now);
}

fn wake_if_stale<R: Runtime>(app: &AppHandle<R>, freshness: Span, now: Timestamp) {
    let stale = with_engine(app, |engine| {
        let last = engine.last_official_at()?;
        Ok(last.is_none_or(|at| now - at > freshness))
    });
    if stale == Some(true)
        && manual_state(app, now) == ManualRefresh::Ready
        && let Some(wake) = wake_handle(app)
    {
        wake.notify_one();
    }
}

fn sync_state<R: Runtime>(app: &AppHandle<R>) -> SyncState {
    update_sync(app, |_| {}).unwrap_or_default()
}

fn update_sync<R: Runtime>(
    app: &AppHandle<R>,
    change: impl FnOnce(&mut SyncState),
) -> Option<SyncState> {
    let shared = app.try_state::<Shared>()?;
    let mut state = shared.sync.lock().unwrap_or_else(PoisonError::into_inner);
    change(&mut state);
    Some(*state)
}

fn record<R: Runtime>(app: &AppHandle<R>, result: &PollResult) {
    let now = clock::now();
    update_sync(app, |state| state.attempts.last = Some(now));
    let previous = with_engine(app, |engine| Ok(engine.status()));
    log_oauth(previous, result);
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

fn log_oauth(previous: Option<OAuthStatus>, result: &PollResult) {
    match result {
        Ok(snapshots) if previous != Some(OAuthStatus::Active) => diagnostics::info(
            "oauth",
            &format!("usage refreshed with {} limits", snapshots.len()),
        ),
        Ok(_) => {}
        Err(error) => diagnostics::warn(
            "oauth",
            &format!("{:?}: {error}", OAuthStatus::from_error(error)),
        ),
    }
}

fn announce<R: Runtime>(app: &AppHandle<R>, alert: &Alert, text: Text, now: Timestamp) {
    let message = toast::message(alert, text, now);
    if let Err(error) = toast::show(app, &message) {
        diagnostics::error("toast", &format!("failed to show a notification: {error}"));
    }
    if let Err(error) = app.emit(ALERT_EVENT, AlertView::new(alert, message)) {
        diagnostics::error("app", &format!("failed to publish an alert: {error}"));
    }
}

fn publish<R: Runtime>(app: &AppHandle<R>) -> Option<UsageView> {
    let now = clock::now();
    let report = report(app, now)?;
    let language = locale::resolve(report.preferences.language);
    let view = view_of(app, &report, now);
    if let Err(error) = app.emit(USAGE_EVENT, &view) {
        diagnostics::error("app", &format!("failed to publish usage: {error}"));
    }
    apply_menu_language(app, language);
    let reading = TrayReading::from_summary(&report.summary);
    if let Err(error) = tray::show_reading(
        app,
        &reading,
        report.preferences.thresholds,
        Text::new(language),
        now,
    ) {
        diagnostics::error("tray", &format!("failed to update the tray icon: {error}"));
    }
    Some(view)
}

fn apply_menu_language<R: Runtime>(app: &AppHandle<R>, language: Language) {
    let Some(shared) = app.try_state::<Shared>() else {
        return;
    };
    let mut applied = shared
        .menu_language
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if *applied == Some(language) {
        return;
    }
    match tray::set_language(app, language) {
        Ok(()) => *applied = Some(language),
        Err(error) => {
            diagnostics::error(
                "tray",
                &format!("failed to translate the tray menu: {error}"),
            );
        }
    }
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
            diagnostics::error("store", &format!("usage database error: {error}"));
            None
        }
    }
}
