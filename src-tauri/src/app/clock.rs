use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::clock::Timestamp;

#[must_use]
pub fn now() -> Timestamp {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        });
    Timestamp::from_unix_millis(millis)
}

#[cfg(test)]
mod tests {
    use super::now;

    #[test]
    fn system_time_is_after_the_project_started() {
        assert!(now().unix_millis() > 1_790_000_000_000);
    }
}
