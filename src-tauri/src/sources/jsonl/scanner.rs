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
    roots: Vec<PathBuf>,
    offsets: HashMap<PathBuf, u64>,
    seen: HashSet<EventKey>,
}

impl JsonlSource {
    #[must_use]
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self::with_offsets(roots, Vec::new())
    }

    #[must_use]
    pub fn with_offsets(roots: Vec<PathBuf>, offsets: Vec<(PathBuf, u64)>) -> Self {
        Self {
            roots,
            offsets: offsets.into_iter().collect(),
            seen: HashSet::new(),
        }
    }

    #[must_use]
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }

    pub fn set_roots(&mut self, roots: Vec<PathBuf>) {
        self.roots = roots;
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
        for path in discover(&self.roots)? {
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

fn discover(roots: &[PathBuf]) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in roots.iter().filter(|root| root.is_dir()) {
        collect(root, 0, &mut files)?;
    }
    files.sort();
    files.dedup();
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

        let mut source = JsonlSource::new(vec![root.clone()]);
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

        let mut source = JsonlSource::new(vec![root.clone()]);
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

        let mut first = JsonlSource::new(vec![root.clone()]);
        assert_eq!(first.poll().expect("first poll").len(), 1);
        let offsets = first.offsets();
        assert_eq!(offsets.len(), 1);

        append(&session, &usage_line("msg_2", 20));
        let mut resumed = JsonlSource::with_offsets(vec![root.clone()], offsets);
        let events = resumed.poll().expect("resumed poll");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key.message_id, "msg_2");

        fs::remove_dir_all(&root).expect("cleanup");
    }

    #[test]
    fn missing_root_yields_no_events() {
        let missing = PathBuf::from("Z:\\plimsoll-missing-root");
        let mut source = JsonlSource::new(vec![missing.clone()]);
        assert!(source.poll().expect("poll").is_empty());
        assert_eq!(source.roots(), [missing]);
    }

    #[test]
    fn reads_every_root_and_follows_root_changes() {
        let first = scratch_dir("root-one");
        let second = scratch_dir("root-two");
        append(
            &first.join("project-a").join("a.jsonl"),
            &usage_line("msg_a", 1),
        );
        append(
            &second.join("project-a").join("b.jsonl"),
            &usage_line("msg_b", 2),
        );

        let mut source = JsonlSource::new(vec![first.clone()]);
        assert_eq!(source.poll().expect("first root").len(), 1);
        source.set_roots(vec![first.clone(), second.clone(), first.clone()]);
        let added = source.poll().expect("both roots");
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].key.message_id, "msg_b");

        fs::remove_dir_all(&first).expect("cleanup");
        fs::remove_dir_all(&second).expect("cleanup");
    }
}
