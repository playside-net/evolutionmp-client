use crate::invoke;

pub fn set_flag(flag: &str, value: bool) {
    invoke!((), 0xB9EFD5C25018725A, flag, value)
}

pub fn set_mobile_radio_enabled(enabled: bool) {
    invoke!((), 0x1098355A16064BB3, enabled)
}

pub fn set_mobile_radio_state(state: bool) {
    invoke!((), 0xBF286C554784F3DF, state)
}