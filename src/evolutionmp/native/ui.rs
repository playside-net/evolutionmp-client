use crate::invoke;
use cgmath::Vector2;
use crate::game::Rgba;
use crate::game::ui::CursorSprite;

pub fn set_credits_active(active: bool) {
    invoke!((), 0xB938B7E6D3C0620C, active);
}

pub fn is_loading_screen_active() -> bool {
    invoke!(bool, 0x10D0A8F259E93EC9)
}

pub fn hide_loading_prompt() {
    invoke!((), 0x10D373323E5B9C0D)
}

pub fn is_loading_prompt_visible() -> bool {
    invoke!(bool, 0xD422FCC5F239A915)
}

pub fn set_loading_prompt_text_entry(entry: &str) {
    invoke!((), 0xABA17D7CE615ADBF, entry)
}

pub fn show_loading_prompt(spinner_type: u32) {
    invoke!((), 0xBD12F8228410D9B4, spinner_type)
}

pub fn push_string(string: &str) {
    for s in string.as_bytes().chunks(99).map(|c| unsafe { std::str::from_utf8_unchecked(c) }) {
        invoke!((), 0x6C188BE134E074AA, s)
    }
}

pub fn set_cursor_sprite(sprite: u32) {
    invoke!((), 0x8DB8CFFD58B62552, sprite)
}

pub fn get_cursor_sprite() -> CursorSprite {
    unsafe { super::CURSOR_SPRITE.load(std::sync::atomic::Ordering::SeqCst).read() }
}

pub fn set_notification_text_entry(ty: &str) {
    invoke!((), 0x202709F4C58A0424, ty)
}

pub fn show_notification(duration: bool, immediately: bool) {
    invoke!((), 0x9D77056A530643F6, duration, immediately)
}

pub fn set_map_revealed(revealed: bool) {
    invoke!((), 0xF8DEE0A5600CBB93, revealed)
}

pub fn set_big_map_active(toggle: bool, full: bool) {
    invoke!((), 0x231C8F89D0539D8F, toggle, full)
}

pub fn is_big_map_active() -> bool {
    unsafe { super::EXPANDED_RADAR.load(std::sync::atomic::Ordering::SeqCst).read() }
}

pub fn is_big_map_full() -> bool {
    unsafe { super::REVEAL_FULL_MAP.load(std::sync::atomic::Ordering::SeqCst).read() }
}

pub fn draw_rect(pos: Vector2<f32>, size: Vector2<f32>, color: Rgba) {
    invoke!((), 0x3A618A217E5154F0, pos, size, color)
}

pub fn set_text_font(font: u32) {
    invoke!((), 0x66E0276CC5F6B9DA, font)
}

pub fn set_text_scale(scale: Vector2<f32>) {
    invoke!((), 0x07C837F9A01C34C9, scale)
}

pub fn set_text_color(color: Rgba) {
    invoke!((), 0xBE6B23FFA53FB442, color)
}

pub fn begin_text_command_draw(ty: &str) {
    invoke!((), 0x25FBB336DF1804CB, ty)
}

pub fn end_text_command_draw(pos: Vector2<f32>) {
    invoke!((), 0xCD015E5BB0D96A57, pos)
}

pub fn begin_text_command_width(ty: &str) {
    invoke!((), 0x54CE8AC98E120CAB, ty)
}

pub fn end_text_command_width(unknown: bool) -> f32 {
    invoke!(f32, 0x85F061DA64ED2F67, unknown)
}