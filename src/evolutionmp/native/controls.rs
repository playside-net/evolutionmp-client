use crate::invoke;
use crate::game::Handle;

pub fn disable_action(group: u32, control: u32, disable: bool) {
    invoke!((), 0xFE99B66D079CF6BC, group, control, disable)
}

pub fn enable_action(group: u32, control: u32, enable: bool) {
    invoke!((), 0x351220255D64C155, group, control, enable)
}

pub fn disable_all_actions(group: u32) {
    invoke!((), 0x5F4B6931816E599B, group)
}

pub fn enable_all_actions(group: u32) {
    invoke!((), 0xA5FFE9B05F199DE7, group)
}

pub fn is_pressed(group: u32, control: u32) -> bool {
    invoke!(bool, 0xF3A21BCD95725A4A, group, control)
}