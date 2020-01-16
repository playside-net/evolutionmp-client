use crate::invoke;

pub fn set_time(hour: u32, minute: u32, second: u32) {
    invoke!((), 0x47C3B5848C3E45D8, hour, minute, second)
}

pub fn pause(toggle: bool) {
    invoke!((), 0x4055E40BD2DBEC1D, toggle)
}

pub fn advance_time_to(hour: u32, minute: u32, second: u32) {
    invoke!((), 0xC8CA9670B9D83B3B, hour, minute, second)
}

pub fn add_time(hours: u32, minutes: u32, seconds: u32) {
    invoke!((), 0xD716F30D8C8980E2, hours, minutes, seconds)
}

pub fn get_hours() -> u32 {
    invoke!(u32, 0x25223CA6B4D20B7F)
}

pub fn get_minutes() -> u32 {
    invoke!(u32, 0x13D2B8ADD79640F2)
}

pub fn get_seconds() -> u32 {
    invoke!(u32, 0x494E97C2EF27C470)
}

pub fn set_date(day: u32, month: u32, year: u32) {
    invoke!((), 0xB096419DF0D06CE7, day, month, year)
}

pub fn get_day_of_week() -> u32 {
    invoke!(u32, 0xD972E4BD7AEB235F)
}

pub fn get_day_of_month() -> u32 {
    invoke!(u32, 0x3D10BC92A4DB1D35)
}

pub fn get_month() -> u32 {
    invoke!(u32, 0xBBC72712E80257A1)
}

pub fn get_year() -> u32 {
    invoke!(u32, 0x961777E64BDAF717)
}

pub fn get_millis_per_game_minute() -> u32 {
    invoke!(u32, 0x2F8B4D1C595B11DB)
}

pub fn get_posix_time(year: &mut u32, month: &mut u32, day: &mut u32, hour: &mut u32, minute: &mut u32, second: &mut u32) {
    invoke!((), 0xDA488F299A5B164E, year, month, day, hour, minute, second)
}

pub fn get_utc_time(year: &mut u32, month: &mut u32, day: &mut u32, hour: &mut u32, minute: &mut u32, second: &mut u32) {
    invoke!((), 0x8117E09A19EEF4D3, year, month, day, hour, minute, second)
}

pub fn get_local_time(year: &mut u32, month: &mut u32, day: &mut u32, hour: &mut u32, minute: &mut u32, second: &mut u32) {
    invoke!((), 0x50C7A99057A69748, year, month, day, hour, minute, second)
}