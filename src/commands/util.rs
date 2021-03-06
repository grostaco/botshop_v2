use chrono::{DateTime, Duration, Utc};

pub fn get_tomorrow() -> DateTime<Utc> {
    (Utc::now() + Duration::days(1)).date().and_hms(0, 0, 0)
}

pub fn get_today() -> DateTime<Utc> {
    Utc::now().date().and_hms(0, 0, 0)
}
