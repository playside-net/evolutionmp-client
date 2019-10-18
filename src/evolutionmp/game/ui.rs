use std::ffi::CString;
use widestring::WideCString;
use winapi::_core::mem::ManuallyDrop;

pub fn show_loading_prompt(ty: LoadingPrompt, text: &str) {
    unsafe {
        crate::natives::ui::set_loading_prompt_text_entry("STRING");
        crate::natives::ui::push_string(text);
        crate::natives::ui::show_loading_prompt(ty as u32);
    }
}

pub fn show_subtitle(text: &str, duration: i32, immediately: bool) {
    unsafe {
        use crate::invoke;
        invoke!((), 0xB87A37EEB7FAA67D, "STRING");
        crate::natives::ui::push_string(text);
        invoke!((), 0x9D77056A530643F6, duration, immediately);
    }
}

pub enum LoadingPrompt {
    LoadingLeft = 0,
    LoadingLeft2 = 1,
    LoadingLeft3 = 2,
    SavingLeft = 3,
    LoadingRight = 4
}

pub fn hide_loading_prompt() {
    unsafe {
        crate::natives::ui::hide_loading_prompt();
    }
}

pub fn set_cursor_sprite(sprite: CursorSprite) {
    unsafe {
        crate::natives::ui::set_cursor_sprite(sprite as u32)
    }
}

pub enum CursorSprite {
    None = 0,
    Normal = 1,
    TransparentNormal = 2,
    PreGrab = 3,
    Grab = 4,
    MiddleFinger = 5,
    LeftArrow = 6,
    RightArrow = 7,
    UpArrow = 8,
    DownArrow = 9,
    HorizontalExpand = 10,
    Add = 11,
    Remove = 12
}