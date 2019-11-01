pub fn show_loading_prompt(ty: LoadingPrompt, text: &str) {
    unsafe {
        crate::native::ui::set_loading_prompt_text_entry("STRING");
        crate::native::ui::push_string(text);
        crate::native::ui::show_loading_prompt(ty as u32);
    }
}

pub fn show_subtitle(text: &str, duration: i32, immediately: bool) {
    unsafe {
        use crate::invoke;
        invoke!((), 0xB87A37EEB7FAA67D, "STRING");
        crate::native::ui::push_string(text);
        invoke!((), 0x9D77056A530643F6, duration, immediately);
    }
}

pub enum LoadingPrompt {
    LoadingLeft,
    LoadingLeft2,
    LoadingLeft3,
    SavingLeft,
    LoadingRight
}

pub fn hide_loading_prompt() {
    unsafe {
        crate::native::ui::hide_loading_prompt();
    }
}

pub fn set_cursor_sprite(sprite: CursorSprite) {
    unsafe {
        crate::native::ui::set_cursor_sprite(sprite as u32)
    }
}

pub enum CursorSprite {
    None,
    Normal,
    TransparentNormal,
    PreGrab,
    Grab,
    MiddleFinger,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    HorizontalExpand,
    Add,
    Remove
}