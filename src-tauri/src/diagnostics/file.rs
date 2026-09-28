use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::rotation::{MAX_FILE_BYTES, file_name, needs_rotation, shifts};

#[derive(Debug)]
pub struct LogFile {
    directory: PathBuf,
    limit: u64,
    file: Option<File>,
    size: u64,
}

impl LogFile {
    pub fn open(directory: PathBuf) -> io::Result<Self> {
        Self::open_with_limit(directory, MAX_FILE_BYTES)
    }

    pub fn open_with_limit(directory: PathBuf, limit: u64) -> io::Result<Self> {
        fs::create_dir_all(&directory)?;
        let file = open_current(&directory)?;
        let size = file.metadata()?.len();
        Ok(Self {
            directory,
            limit,
            file: Some(file),
            size,
        })
    }

    pub fn append(&mut self, line: &str) -> io::Result<()> {
        let incoming = u64::try_from(line.len()).unwrap_or(u64::MAX);
        if needs_rotation(self.size, incoming, self.limit) {
            self.rotate()?;
        }
        let file = match &mut self.file {
            Some(file) => file,
            None => self.file.insert(open_current(&self.directory)?),
        };
        file.write_all(line.as_bytes())?;
        self.size = self.size.saturating_add(incoming);
        Ok(())
    }

    fn rotate(&mut self) -> io::Result<()> {
        self.file = None;
        for (from, to) in shifts() {
            let source = self.directory.join(file_name(from));
            if source.exists() {
                fs::rename(source, self.directory.join(file_name(to)))?;
            }
        }
        self.file = Some(open_current(&self.directory)?);
        self.size = 0;
        Ok(())
    }
}

fn open_current(directory: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join(file_name(0)))
}

#[cfg(test)]
mod tests {
    use super::LogFile;
    use crate::diagnostics::rotation::{KEPT_FILES, file_name};
    use std::fs;
    use std::path::{Path, PathBuf};

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("plimsoll-log-{name}-{}", std::process::id()));
        fs::remove_dir_all(&directory).ok();
        directory
    }

    fn sizes(directory: &Path) -> Vec<u64> {
        (0..=KEPT_FILES)
            .filter_map(|generation| fs::metadata(directory.join(file_name(generation))).ok())
            .map(|metadata| metadata.len())
            .collect()
    }

    #[test]
    fn appends_lines_to_the_current_file_across_reopens() {
        let directory = scratch("append");
        LogFile::open(directory.clone())
            .expect("open")
            .append("one\n")
            .expect("write");
        LogFile::open(directory.clone())
            .expect("reopen")
            .append("two\n")
            .expect("write");
        assert_eq!(
            fs::read_to_string(directory.join(file_name(0))).expect("read"),
            "one\ntwo\n"
        );
        fs::remove_dir_all(directory).expect("cleanup");
    }

    #[test]
    fn rotates_into_at_most_three_bounded_files() {
        let directory = scratch("rotate");
        let mut log = LogFile::open_with_limit(directory.clone(), 100).expect("open");
        for index in 0..40 {
            log.append(&format!("line {index:02} padded to twenty\n"))
                .expect("write");
        }
        let sizes = sizes(&directory);
        assert_eq!(sizes.len(), KEPT_FILES);
        assert!(sizes.iter().all(|&size| size <= 100), "{sizes:?}");
        let newest = fs::read_to_string(directory.join(file_name(0))).expect("read");
        assert!(newest.ends_with("line 39 padded to twenty\n"));
        let oldest = fs::read_to_string(directory.join(file_name(2))).expect("read");
        assert!(oldest.starts_with("line 28"));
        fs::remove_dir_all(directory).expect("cleanup");
    }
}
