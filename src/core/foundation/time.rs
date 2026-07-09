use chrono::{DateTime, SecondsFormat, Utc};

pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_utc_ms() -> i64 {
    now_utc().timestamp_millis()
}

pub fn now_unix_seconds() -> i64 {
    now_utc().timestamp()
}

pub fn now_utc_iso() -> String {
    now_utc().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_values_are_utc_and_positive() {
        assert!(now_utc_ms() > 0);
        assert!(now_unix_seconds() > 0);
        assert!(now_utc_iso().ends_with('Z'));
    }
}
