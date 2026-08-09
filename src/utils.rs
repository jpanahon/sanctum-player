use chrono::{DateTime, Local};
use std::time::SystemTime;

pub fn format_timestamp(timestamp: u64) -> String {
    let minutes = timestamp / 60;
    let seconds = timestamp % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

pub fn format_date(created: SystemTime) -> String {
    let age = match SystemTime::now().duration_since(created) {
        Ok(d) => d,
        Err(_) => {
            let date_time: DateTime<Local> = created.into();
            return date_time.format("%d/%m/%y").to_string();
        }
    };

    let secs = age.as_secs();

    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;

    if secs < MIN {
        "just now".to_string()
    } else if secs < HOUR {
        format!("{} minutes ago", secs / MIN)
    } else if secs < DAY {
        format!("{} hours ago", secs / HOUR)
    } else if secs < WEEK {
        format!("{} days ago", secs / DAY)
    } else {
        let date_time: DateTime<Local> = created.into();
        date_time.format("%d/%m/%y").to_string()
    }
}
