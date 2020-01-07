use crate::native;

pub fn set_time(hour: u32, minute: u32, second: u32) {
    unsafe { native::clock::set_time(hour, minute, second) }
}

pub fn pause(toggle: bool) {
    unsafe { native::clock::pause(toggle) }
}

pub fn advance_time_to(hour: u32, minute: u32, second: u32) {
    unsafe { native::clock::advance_time_to(hour, minute, second) }
}

pub fn add_time(hours: u32, minutes: u32, seconds: u32) {
    unsafe { native::clock::add_time(hours, minutes, seconds) }
}

pub fn get_hours() -> u32 {
    unsafe { native::clock::get_hours() }
}

pub fn get_minutes() -> u32 {
    unsafe { native::clock::get_minutes() }
}

pub fn get_seconds() -> u32 {
    unsafe { native::clock::get_seconds() }
}

pub fn set_date(day: u32, month: u32, year: u32) {
    unsafe { native::clock::set_date(day, month, year) }
}

pub fn get_day_of_week() -> u32 {
    unsafe { native::clock::get_day_of_week() }
}

pub fn get_day_of_month() -> u32 {
    unsafe { native::clock::get_day_of_month() }
}

pub fn get_month() -> u32 {
    unsafe { native::clock::get_month() }
}

pub fn get_year() -> u32 {
    unsafe { native::clock::get_year() }
}

pub fn get_millis_per_game_minute() -> u32 {
    unsafe { native::clock::get_millis_per_game_minute() }
}

pub fn get_posix_time() -> Time {
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    unsafe {
        native::clock::get_posix_time(&mut year, &mut month, &mut day, &mut hour, &mut minute, &mut second)
    }
    Time {
        year, month, day, hour, minute, second
    }
}

pub fn get_utc_time() -> Time {
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    unsafe {
        native::clock::get_utc_time(&mut year, &mut month, &mut day, &mut hour, &mut minute, &mut second)
    }
    Time {
        year, month, day, hour, minute, second
    }
}

pub fn get_local_time() -> Time {
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    unsafe {
        native::clock::get_local_time(&mut year, &mut month, &mut day, &mut hour, &mut minute, &mut second)
    }
    Time {
        year, month, day, hour, minute, second
    }
}

pub struct Time {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}