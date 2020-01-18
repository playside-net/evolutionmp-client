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

pub fn is_enabled(group: u32, control: u32) -> bool {
    invoke!(bool, 0x1CEA6BFDF248E5D9, group, control)
}

pub fn is_just_pressed(group: u32, control: u32) -> bool {
    invoke!(bool, 0x580417101DDB492F, group, control)
}

pub fn is_just_released(group: u32, control: u32) -> bool {
    invoke!(bool, 0x50F940259D3841E6, group, control)
}

pub fn is_pressed(group: u32, control: u32) -> bool {
    invoke!(bool, 0xF3A21BCD95725A4A, group, control)
}

pub fn is_released(group: u32, control: u32) -> bool {
    invoke!(bool, 0x648EE3E7F38877DD, group, control)
}

pub fn is_disabled_just_pressed(group: u32, control: u32) -> bool {
    invoke!(bool, 0x91AEF906BCA88877, group, control)
}

pub fn is_disabled_just_released(group: u32, control: u32) -> bool {
    invoke!(bool, 0x305C8DCD79DA8B0F, group, control)
}

pub fn is_disabled_pressed(group: u32, control: u32) -> bool {
    invoke!(bool, 0xE2587F8CBBD87B1D, group, control)
}

pub fn is_disabled_released(group: u32, control: u32) -> bool {
    invoke!(bool, 0xFB6C4072E9A32E92, group, control)
}