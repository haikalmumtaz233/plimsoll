use std::path::Path;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::sources::SourceError;

const EXTENSION: &str = "jsonl";

pub fn start<F>(root: &Path, on_change: F) -> Result<RecommendedWatcher, SourceError>
where
    F: Fn() + Send + 'static,
{
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<Event>| {
        if result.is_ok_and(|event| is_relevant(&event)) {
            on_change();
        }
    })?;
    watcher.watch(root, RecursiveMode::Recursive)?;
    Ok(watcher)
}

#[must_use]
pub fn is_relevant(event: &Event) -> bool {
    matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
        && event.paths.iter().any(|path| {
            path.extension()
                .is_some_and(|extension| extension == EXTENSION)
        })
}

#[cfg(test)]
mod tests {
    use super::{is_relevant, start};
    use notify::event::{CreateKind, ModifyKind, RemoveKind};
    use notify::{Event, EventKind};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;

    fn event(kind: EventKind, path: &str) -> Event {
        Event::new(kind).add_path(PathBuf::from(path))
    }

    #[test]
    fn only_jsonl_writes_are_relevant() {
        assert!(is_relevant(&event(
            EventKind::Modify(ModifyKind::Any),
            "C:\\p\\s.jsonl"
        )));
        assert!(is_relevant(&event(
            EventKind::Create(CreateKind::File),
            "C:\\p\\s.jsonl"
        )));
        assert!(!is_relevant(&event(
            EventKind::Modify(ModifyKind::Any),
            "C:\\p\\MEMORY.md"
        )));
        assert!(!is_relevant(&event(
            EventKind::Remove(RemoveKind::File),
            "C:\\p\\s.jsonl"
        )));
    }

    #[test]
    fn notifies_when_a_session_file_changes() {
        let root = std::env::temp_dir().join(format!("plimsoll-watch-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).expect("reset scratch dir");
        }
        fs::create_dir_all(&root).expect("create scratch dir");

        let (sender, receiver) = mpsc::channel();
        let watcher = start(&root, move || {
            sender.send(()).ok();
        })
        .expect("start watcher");
        fs::write(root.join("session.jsonl"), b"{}\n").expect("write session file");

        assert!(receiver.recv_timeout(Duration::from_secs(10)).is_ok());
        drop(watcher);
        fs::remove_dir_all(&root).ok();
    }
}
