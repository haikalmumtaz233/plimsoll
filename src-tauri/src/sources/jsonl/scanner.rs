use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::line;
use super::reader::{MAX_LINE_BYTES, read_appended_lines};
use crate::domain::record::{EventKey, KeyedEvent};
use crate::sources::SourceError;

const MAX_DEPTH: usize = 4;
const EXTENSION: &str = "jsonl";

#[derive(Debug)]
pub struct JsonlSource {
    root: PathBuf,
    offsets: HashMap<PathBuf, u64>,
    seen: HashSet<EventKey>,
}

impl JsonlSource {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            offsets: HashMap::new(),
            seen: HashSet::new(),
        }
    }

    #[must_use]
    pub fn with_offsets(root: PathBuf, offsets: Vec<(PathBuf, u64)>) -> Self {
        Self {
            root,
            offsets: offsets.into_iter().collect(),
            seen: HashSet::new(),
        }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn offsets(&self) -> Vec<(PathBuf, u64)> {
        let mut offsets: Vec<(PathBuf, u64)> = self
            .offsets
            .iter()
            .map(|(path, offset)| (path.clone(), *offset))
            .collect();
        offsets.sort();
        offsets
    }

    pub fn poll(&mut self) -> Result<Vec<KeyedEvent>, SourceError> {
        let mut events = Vec::new();
        for path in discover(&self.root)? {
            let offset = self.offsets.get(&path).copied().unwrap_or(0);
            let seen = &mut self.seen;
            let result =
                read_appended_lines(&path, offset, MAX_LINE_BYTES, &mut |bytes: &[u8]| {
                    if let Some(keyed) = line::parse(bytes)
                        && seen.insert(keyed.key.clone())
                    {
                        events.push(keyed);
                    }
                });
            match result {
                Ok(next) => {
                    self.offsets.insert(path, next);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    self.offsets.remove(&path);
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(events)
    }
}

fn discover(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if root.is_dir() {
        collect(root, 0, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect(directory: &Path, depth: usize, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() && depth < MAX_DEPTH {
            collect(&path, depth + 1, files)?;
        } else if kind.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == EXTENSION)
        {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::JsonlSource;
    use serde_json::json;
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("plimsoll-source-{name}-{}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).expect("reset scratch dir");
        }
        fs::create_dir_all(dir.join("project-a")).expect("create scratch dir");
        dir
    }

    fn usage_line(message_id: &str, output: u64) -> String {
        let line = json!({
            "type": "assistant",
            "timestamp": "2026-09-25T01:35:37.212Z",
            "requestId": format!("req_{message_id}"),
            "cwd": "C:\\work\\project-a",
            "message": {
                "id": message_id,
                "model": "claude-opus-5",
                "usage": { "input_tokens": 1, "output_tokens": output }
            }
        });
        format!("{line}\n")
    }

    fn append(path: &Path, text: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("open session file");
        file.write_all(text.as_bytes()).expect("append line");
    }

    #[test]
    fn polls_incrementally_and_deduplicates_messages() {
        let root = scratch_dir("poll");
        let session = root.join("project-a").join("session.jsonl");
        append(
            &session,
            &[
                usage_line("msg_1", 10),
                usage_line("msg_1", 10),
                usage_line("msg_2", 20),
            ]
            .concat(),
        );
        append(&root.join("project-a").join("notes.md"), "ignored\n");

        let mut source = JsonlSource::new(root.clone());
        let first = source.poll().expect("first poll");
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].event.tokens.output, 10);
        assert_eq!(first[1].event.tokens.output, 20);

        assert!(source.poll().expect("idle poll").is_empty());

        append(
            &session,
            &[usage_line("msg_2", 20), usage_line("msg_3", 30)].concat(),
        );
        let next = source.poll().expect("incremental poll");
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].event.tokens.output, 30);

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn finds_nested_session_files() {
        let root = scratch_dir("nested");
        let nested = root.join("project-a").join("session").join("subagents");
        fs::create_dir_all(&nested).expect("create nested dir");
        append(&nested.join("agent.jsonl"), &usage_line("msg_9", 90));

        let mut source = JsonlSource::new(root.clone());
        let events = source.poll().expect("poll");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event.project, "project-a");

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn resumes_from_persisted_offsets() {
        let root = scratch_dir("resume");
        let session = root.join("project-a").join("session.jsonl");
        append(&session, &usage_line("msg_1", 10));

        let mut first = JsonlSource::new(root.clone());
        assert_eq!(first.poll().expect("first poll").len(), 1);
        let offsets = first.offsets();
        assert_eq!(offsets.len(), 1);

        append(&session, &usage_line("msg_2", 20));
        let mut resumed = JsonlSource::with_offsets(root.clone(), offsets);
        let events = resumed.poll().expect("resumed poll");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key.message_id, "msg_2");

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn missing_root_yields_no_events() {
        let mut source = JsonlSource::new(PathBuf::from("Z:\\plimsoll-missing-root"));
        assert!(source.poll().expect("poll").is_empty());
        assert_eq!(source.root(), Path::new("Z:\\plimsoll-missing-root"));
    }
}
