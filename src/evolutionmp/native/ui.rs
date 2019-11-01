use crate::invoke;

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

pub unsafe fn set_loading_prompt_text_entry(entry: &str) {
    invoke!((), 0xABA17D7CE615ADBF, entry)
}

pub unsafe fn show_loading_prompt(spinner_type: u32) {
    invoke!((), 0xBD12F8228410D9B4, spinner_type)
}

pub unsafe fn push_string(string: &str) {
    for s in string.as_bytes().chunks(99).map(|c|std::str::from_utf8_unchecked(c)) {
        invoke!((), 0x6C188BE134E074AA, s)
    }
}

pub unsafe fn set_cursor_sprite(sprite: u32) {
    invoke!((), 0x8DB8CFFD58B62552, sprite)
}

pub unsafe fn get_cursor_sprite() -> u32 {
    super::CURSOR_SPRITE.read()
}

pub unsafe fn set_notification_text_entry(ty: &str) {
    invoke!((), 0x202709F4C58A0424, ty)
}

pub unsafe fn show_notification(duration: bool, immediately: bool) {
    invoke!((), 0x9D77056A530643F6, duration, immediately)
}

pub unsafe fn set_map_revealed(revealed: bool) {
    invoke!((), 0xF8DEE0A5600CBB93, revealed)
}

pub unsafe fn set_big_map_active(toggle: bool, full: bool) {
    invoke!((), 0x231C8F89D0539D8F, toggle, full)
}

pub unsafe fn is_big_map_active() -> bool {
    super::EXPANDED_RADAR.read()
}

pub unsafe fn is_big_map_full() -> bool {
    super::REVEAL_FULL_MAP.read()
}