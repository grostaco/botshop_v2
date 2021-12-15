use chrono::{DateTime, Duration, Utc};

pub fn get_tomorrow_midnight() -> DateTime<Utc> {
    (Utc::now() + Duration::days(1)).date().and_hms(0, 0, 0)
}
