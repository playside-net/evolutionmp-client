use crate::invoke;

pub fn set_flag(flag: &str, value: bool) {
    invoke!((), 0xB9EFD5C25018725A, flag, value)
}