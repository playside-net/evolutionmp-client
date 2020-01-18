use crate::{invoke, native};
use crate::game::Rgba;
use cgmath::Vector2;

pub const BASE_WIDTH: f32 = 1280.0;
pub const BASE_HEIGHT: f32 = 720.0;

pub fn show_loading_prompt(ty: LoadingPrompt, text: &str) {
    invoke!((), 0xABA17D7CE615ADBF, "STRING");
    push_string(text);
    invoke!((), 0xBD12F8228410D9B4, ty as u32);
}

pub fn show_subtitle(text: &str, duration: i32, immediately: bool) {
    invoke!((), 0xB87A37EEB7FAA67D, "STRING");
    crate::native::ui::push_string(text);
    invoke!((), 0x9D77056A530643F6, duration, immediately);
}

pub fn draw_rect<P, S, C>(pos: P, size: S, color: C)
    where P: Into<Vector2<f32>>, S: Into<Vector2<f32>>, C: Into<Rgba>
{
    let pos = pos.into();
    let pos = Vector2::new(pos.x / BASE_WIDTH, pos.y / BASE_HEIGHT);
    let size = size.into();
    let size = Vector2::new(size.x / BASE_WIDTH, size.y / BASE_HEIGHT);

    invoke!((), 0x3A618A217E5154F0, pos + (size * 0.5), size, color.into())
}

pub fn draw_text<T, P, S>(text: T, pos: P, color: Rgba, font: Font, scale: S)
    where T: AsRef<str>, P: Into<Vector2<f32>>, S: Into<Vector2<f32>>
{
    let pos = pos.into();
    let pos = Vector2::new(pos.x / BASE_WIDTH, pos.y / BASE_HEIGHT);
    set_text_font(font as u32);
    set_text_scale(scale.into());
    set_text_color(color);
    begin_text_command_draw("CELL_EMAIL_BCON");
    push_string(text.as_ref());
    end_text_command_draw(pos.into())
}

pub fn get_text_width<T>(text: T, font: Font, scale: Vector2<f32>) -> f32 where T: AsRef<str> {
    set_text_font(font as u32);
    set_text_scale(scale);
    begin_text_command_width("CELL_EMAIL_BCON");
    push_string(text.as_ref());
    end_text_command_width(true)
}

#[derive(Debug, Copy, Clone)]
pub enum Font {
    ChaletLondon,
    HouseScript,
    Monospace,
    ChaletComprimeCologne = 4,
    Pricedown = 7
}

pub enum LoadingPrompt {
    LoadingLeft,
    LoadingLeft2,
    LoadingLeft3,
    SavingLeft,
    LoadingRight
}

#[repr(C)]
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

pub enum HudComponent {
    Main,
    WantedStars,
    WeaponIcon,
    Cash,
    MpCash,
    MpMessage,
    VehicleName,
    AreaName,
    Unused,
    StreetName,
    HelpText,
    FloatingHelpText1,
    FloatingHelpText2,
    CashChange,
    Reticle,
    SubtitleText,
    RadioStationsWheel,
    Saving,
    GameStreamUnused,
    WeaponWheel,
    WeaponWheelStats,
    DrugsPurse01,
    DrugsPurse02,
    DrugsPurse03,
    DrugsPurse04,
    MpTagCashFromBank,
    MpTagPackages,
    MpTagCuffKeys,
    MpTagDownloadData,
    MpTagIfPedFollowing,
    MpTagKeyCard,
    MpTagRandomObject,
    MpTagRemoteControl,
    MpTagCashFromSafe,
    MpTagWeaponsPackage,
    MpTagKeys,
    MpVehicle,
    MpVehicleHelicopter,
    MpVehiclePlane,
    PlayerSwitchAlert,
    MpRankBar,
    DirectorMode,
    ReplayController,
    ReplayMouse,
    ReplayHeader,
    ReplayOptions,
    ReplayHelpText,
    ReplayMiscText,
    ReplayTopLine,
    ReplayBottomLine,
    ReplayLeftBar,
    ReplayTimer
}

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

fn set_text_font(font: u32) {
    invoke!((), 0x66E0276CC5F6B9DA, font)
}

fn set_text_scale(scale: Vector2<f32>) {
    invoke!((), 0x07C837F9A01C34C9, scale)
}

fn set_text_color(color: Rgba) {
    invoke!((), 0xBE6B23FFA53FB442, color)
}

fn begin_text_command_draw(ty: &str) {
    invoke!((), 0x25FBB336DF1804CB, ty)
}

fn end_text_command_draw(pos: Vector2<f32>) {
    invoke!((), 0xCD015E5BB0D96A57, pos)
}

fn begin_text_command_width(ty: &str) {
    invoke!((), 0x54CE8AC98E120CAB, ty)
}

fn end_text_command_width(unknown: bool) -> f32 {
    invoke!(f32, 0x85F061DA64ED2F67, unknown)
}