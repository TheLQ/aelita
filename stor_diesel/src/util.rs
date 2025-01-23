use chrono::{DateTime, FixedOffset, SecondsFormat};

pub fn to_stor_date_format(date: DateTime<FixedOffset>) -> String {
    date.to_rfc3339_opts(SecondsFormat::Secs, false)
}
