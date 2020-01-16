use crate::native;
pub use native::clock::{
    set_time, pause, advance_time_to, add_time, get_hours, get_minutes, get_seconds, set_date,
    get_day_of_week, get_day_of_month, get_month, get_year, get_millis_per_game_minute
};

pub fn get_posix_time() -> Time {
    let mut year = 0;
    let mut month = 0;
    let mut day = 0;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0;
    native::clock::get_posix_time(&mut year, &mut month, &mut day, &mut hour, &mut minute, &mut second);
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
    native::clock::get_utc_time(&mut year, &mut month, &mut day, &mut hour, &mut minute, &mut second);
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
    native::clock::get_local_time(&mut year, &mut month, &mut day, &mut hour, &mut minute, &mut second);
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