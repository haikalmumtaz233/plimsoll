pub const MAX_FILE_BYTES: u64 = 1024 * 1024;
pub const KEPT_FILES: usize = 3;

const STEM: &str = "plimsoll";
const EXTENSION: &str = "log";

#[must_use]
pub fn file_name(generation: usize) -> String {
    if generation == 0 {
        format!("{STEM}.{EXTENSION}")
    } else {
        format!("{STEM}.{generation}.{EXTENSION}")
    }
}

#[must_use]
pub const fn needs_rotation(current: u64, incoming: u64, limit: u64) -> bool {
    current > 0 && current.saturating_add(incoming) > limit
}

#[must_use]
pub fn shifts() -> Vec<(usize, usize)> {
    (0..KEPT_FILES - 1)
        .rev()
        .map(|generation| (generation, generation + 1))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{KEPT_FILES, MAX_FILE_BYTES, file_name, needs_rotation, shifts};

    #[test]
    fn the_current_file_has_no_generation_suffix() {
        assert_eq!(file_name(0), "plimsoll.log");
        assert_eq!(file_name(1), "plimsoll.1.log");
        assert_eq!(file_name(2), "plimsoll.2.log");
    }

    #[test]
    fn rotates_only_when_the_next_line_would_overflow() {
        assert!(!needs_rotation(0, 10, 100));
        assert!(!needs_rotation(90, 10, 100));
        assert!(needs_rotation(91, 10, 100));
        assert!(!needs_rotation(0, 500, 100));
        assert!(needs_rotation(u64::MAX, u64::MAX, MAX_FILE_BYTES));
    }

    #[test]
    fn shifts_move_older_files_first_and_drop_the_oldest() {
        assert_eq!(shifts(), vec![(1, 2), (0, 1)]);
        assert!(
            shifts()
                .iter()
                .all(|&(from, to)| to == from + 1 && to < KEPT_FILES)
        );
    }

    #[test]
    fn total_log_size_stays_within_three_megabytes() {
        assert!(MAX_FILE_BYTES * KEPT_FILES as u64 <= 3 * 1024 * 1024);
    }
}
