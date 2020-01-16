use crate::invoke;
use crate::hash::Hash;

pub fn set_int(hash: Hash, value: i32, save: bool) -> bool {
    invoke!(bool, 0xB3271D7AB655B441, hash, value, save)
}

pub fn set_float(hash: Hash, value: f32, save: bool) -> bool {
    invoke!(bool, 0x4851997F37FE9B3C, hash, value, save)
}

pub fn set_bool(hash: Hash, value: bool, save: bool) -> bool {
    invoke!(bool, 0x4B33C4243DE0C432, hash, value, save)
}

pub fn get_int(hash: Hash, output: &mut i32, default: i32) -> bool {
    invoke!(bool, 0x767FBC2AC802EF3D, hash, output, default)
}

pub fn get_float(hash: Hash, output: &mut f32, default: f32) -> bool {
    invoke!(bool, 0xD7AE6C9C9C6AC54C, hash, output, default)
}

pub fn get_bool(hash: Hash, output: &mut bool, default: bool) -> bool {
    invoke!(bool, 0x11B5E6D2AE73F48E, hash, output, default)
}