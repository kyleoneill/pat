use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time since the unix epoch should never fail")
        .as_secs() as i64
}
