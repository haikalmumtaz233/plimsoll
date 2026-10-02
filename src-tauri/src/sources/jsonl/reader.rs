use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

pub const MAX_LINE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineRead {
    End,
    Complete(u64),
    Oversized(u64),
}

pub fn read_line_bounded<R: BufRead>(
    reader: &mut R,
    line: &mut Vec<u8>,
    limit: usize,
) -> io::Result<LineRead> {
    line.clear();
    let mut consumed: u64 = 0;
    let mut oversized = false;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(LineRead::End);
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let chunk = newline.map_or(available, |index| &available[..=index]);
        let chunk_len = chunk.len();
        if !oversized && line.len() + chunk_len > limit {
            oversized = true;
            line.clear();
        }
        if !oversized {
            line.extend_from_slice(chunk);
        }
        reader.consume(chunk_len);
        consumed += chunk_len as u64;
        if newline.is_some() {
            return Ok(if oversized {
                LineRead::Oversized(consumed)
            } else {
                LineRead::Complete(consumed)
            });
        }
    }
}

pub fn read_appended_lines<F>(
    path: &Path,
    offset: u64,
    limit: usize,
    visit: &mut F,
) -> io::Result<u64>
where
    F: FnMut(&[u8]),
{
    let mut file = File::open(path)?;
    let start = if file.metadata()?.len() < offset {
        0
    } else {
        offset
    };
    file.seek(SeekFrom::Start(start))?;
    let mut reader = BufReader::new(file);
    let mut line = Vec::new();
    let mut position = start;
    loop {
        match read_line_bounded(&mut reader, &mut line, limit)? {
            LineRead::End => return Ok(position),
            LineRead::Complete(length) => {
                position += length;
                visit(&line);
            }
            LineRead::Oversized(length) => position += length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LineRead, read_appended_lines, read_line_bounded};
    use std::fs::{self, OpenOptions};
    use std::io::{BufReader, Cursor, Write};
    use std::path::{Path, PathBuf};

    fn scratch_file(name: &str, content: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "plimsoll-reader-{name}-{}.jsonl",
            std::process::id()
        ));
        fs::write(&path, content).expect("write scratch file");
        path
    }

    fn collect(path: &Path, offset: u64, limit: usize) -> (Vec<Vec<u8>>, u64) {
        let mut lines = Vec::new();
        let next = read_appended_lines(path, offset, limit, &mut |line: &[u8]| {
            lines.push(line.to_vec());
        })
        .expect("read lines");
        (lines, next)
    }

    #[test]
    fn reads_lines_across_small_buffers() {
        let mut reader = BufReader::with_capacity(2, Cursor::new(b"abc\nde\n".to_vec()));
        let mut line = Vec::new();
        assert_eq!(
            read_line_bounded(&mut reader, &mut line, 64).expect("read"),
            LineRead::Complete(4)
        );
        assert_eq!(line, b"abc\n");
        assert_eq!(
            read_line_bounded(&mut reader, &mut line, 64).expect("read"),
            LineRead::Complete(3)
        );
        assert_eq!(line, b"de\n");
        assert_eq!(
            read_line_bounded(&mut reader, &mut line, 64).expect("read"),
            LineRead::End
        );
    }

    #[test]
    fn unterminated_line_is_not_consumed() {
        let mut reader = Cursor::new(b"partial".to_vec());
        let mut line = Vec::new();
        assert_eq!(
            read_line_bounded(&mut reader, &mut line, 64).expect("read"),
            LineRead::End
        );
    }

    #[test]
    fn oversized_line_is_skipped_but_consumed() {
        let mut reader = BufReader::with_capacity(3, Cursor::new(b"abcdefgh\nok\n".to_vec()));
        let mut line = Vec::new();
        assert_eq!(
            read_line_bounded(&mut reader, &mut line, 4).expect("read"),
            LineRead::Oversized(9)
        );
        assert_eq!(line, b"");
        assert_eq!(
            read_line_bounded(&mut reader, &mut line, 4).expect("read"),
            LineRead::Complete(3)
        );
        assert_eq!(line, b"ok\n");
    }

    #[test]
    fn resumes_from_offset_after_append() {
        let path = scratch_file("append", b"one\ntw");
        let (lines, offset) = collect(&path, 0, 64);
        assert_eq!(lines, vec![b"one\n".to_vec()]);
        assert_eq!(offset, 4);

        let mut file = OpenOptions::new()
            .append(true)
            .open(&path)
            .expect("open for append");
        file.write_all(b"o\nthree\n").expect("append");
        drop(file);

        let (lines, offset) = collect(&path, offset, 64);
        assert_eq!(lines, vec![b"two\n".to_vec(), b"three\n".to_vec()]);
        assert_eq!(offset, 14);
        fs::remove_file(&path).expect("cleanup");
    }

    #[test]
    fn truncated_file_is_read_from_the_start() {
        let path = scratch_file("truncate", b"short\n");
        let (lines, offset) = collect(&path, 100, 64);
        assert_eq!(lines, vec![b"short\n".to_vec()]);
        assert_eq!(offset, 6);
        fs::remove_file(&path).expect("cleanup");
    }

    #[test]
    fn oversized_lines_in_files_advance_the_offset() {
        let path = scratch_file("oversized", b"0123456789\nok\n");
        let (lines, offset) = collect(&path, 0, 4);
        assert_eq!(lines, vec![b"ok\n".to_vec()]);
        assert_eq!(offset, 14);
        fs::remove_file(&path).expect("cleanup");
    }
}
