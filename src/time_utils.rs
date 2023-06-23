use std::time::{SystemTime, UNIX_EPOCH};

const SECS_PER_MIN: i32 = 60;
const SECS_PER_HOUR: i32 = 60 * SECS_PER_MIN;
const SECS_PER_DAY: i32 = 24 * SECS_PER_HOUR;

const EPOCH_YEAR: i32 = 1970;
const EPOCH_MONTH: i32 = 1;


fn is_leap_year(year: i32) -> bool {
    if year % 4 == 0 {
        if year % 100 == 0 {
            if year % 400 == 0 {
                return true;
            }
            return false;
        }
        return true;
    }
    return false;
}


fn get_days_for_year(year: i32) -> i32 {
    if is_leap_year(year) {
        return 366;
    }
    return 365;
}


fn get_days_per_month(year: i32) -> Vec<i32> {
    let mut days_per_month = Vec::new();
    for i in 1..13 {
        match i {
            1 => days_per_month.push(31),
            2 => {
                if is_leap_year(year) {
                    days_per_month.push(29);
                } else {
                    days_per_month.push(28);
                }
            },
            3 => days_per_month.push(31),
            4 => days_per_month.push(30),
            5 => days_per_month.push(31),
            6 => days_per_month.push(30),
            7 => days_per_month.push(31),
            8 => days_per_month.push(31),
            9 => days_per_month.push(30),
            10 => days_per_month.push(31),
            11 => days_per_month.push(30),
            12 => days_per_month.push(31),
            _ => (),
        }
    }
    days_per_month
}


pub(crate) fn get_date_time_string(now: SystemTime) -> String {
    let mut days = now.duration_since(UNIX_EPOCH).unwrap().as_secs() / SECS_PER_DAY as u64;
    let mut year = EPOCH_YEAR;
    while days >= get_days_for_year(year) as u64 {
        days -= get_days_for_year(year) as u64;
        year += 1;
    }
    let days_per_month = get_days_per_month(year);
    let mut month = EPOCH_MONTH;
    while days >= days_per_month[(month - 1) as usize] as u64 {
        days -= days_per_month[(month - 1) as usize] as u64;
        month += 1;
    }
    let day = days + 1;
    let secs = now.duration_since(UNIX_EPOCH).unwrap().as_secs() % SECS_PER_DAY as u64;
    let hour = secs / SECS_PER_HOUR as u64;
    let secs = secs % SECS_PER_HOUR as u64;
    let min = secs / SECS_PER_MIN as u64;
    let secs = secs % SECS_PER_MIN as u64;
    let mut date_time_string = String::new();
    date_time_string.push_str(
        &format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, min, secs
            )
        );
    date_time_string
}
