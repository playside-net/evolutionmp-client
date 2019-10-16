use super::NativeStackValue;
use crate::invoke;
use std::ffi::CString;
use widestring::WideCString;

pub unsafe fn set_credits_active(active: bool) {
    invoke!((), 0xB938B7E6D3C0620C, active);
}

pub unsafe fn is_loading_screen_active() -> bool {
    invoke!(bool, 0x10D0A8F259E93EC9)
}

pub unsafe fn hide_loading_prompt() {
    invoke!((), 0x10D373323E5B9C0D)
}

pub unsafe fn is_loading_prompt_visible() -> bool {
    invoke!(bool, 0xD422FCC5F239A915)
}

pub unsafe fn set_loading_prompt_text_entry(entry: CString) {
    invoke!((), 0xABA17D7CE615ADBF, entry)
}

pub unsafe fn show_loading_prompt(spinner_type: u32) {
    invoke!((), 0xBD12F8228410D9B4, spinner_type)
}

pub unsafe fn push_string(string: CString) {
    invoke!((), 0x6C188BE134E074AA, string)
}

pub unsafe fn set_cursor_sprite(sprite: u32) {
    invoke!((), 0x8DB8CFFD58B62552, sprite)
}