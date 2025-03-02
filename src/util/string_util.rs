use chrono::{DateTime, Datelike, Timelike};

pub fn is_start_with<T: Eq>(vec: &Vec<T>, p: &Vec<T>) -> bool {
    for i in 0..p.len() {
        if vec.get(i) != p.get(i) {
            return false;
        }
    }
    true
}

pub fn is_legal_username(username: impl AsRef<str>) -> bool {
    username
        .as_ref()
        .chars()
        .all(|c| (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9'))
}

pub fn time_to_string<T: chrono::TimeZone>(time: &DateTime<T>) -> String {
    format!(
        "{:02}{:02}{:02}-{:02}:{:02}",
        time.year() % 2000,
        time.month(),
        time.day(),
        time.hour(),
        time.minute(),
    )
}
