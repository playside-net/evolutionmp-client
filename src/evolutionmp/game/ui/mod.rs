use crate::{invoke, native};
use crate::game::{Rgba, Handle};
use cgmath::Vector2;
use crate::runtime::ScriptEnv;
use crate::game::controls::{Group, Control};
use crate::pattern::{MemoryRegion, RET, NOP, XOR_32_64};
use std::sync::atomic::AtomicBool;
use std::time::Instant;
use std::ops::Range;
use crate::win::input::{InputEvent, KeyboardEvent};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::os::raw::c_int;

pub mod notification;

pub const BASE_WIDTH: f32 = 1280.0;
pub const BASE_HEIGHT: f32 = 720.0;

type GetWarnResult = extern "C" fn(bool, u32) -> FrontendButtons;
static mut GET_WARN_RESULT: *const () = std::ptr::null();

pub(crate) static MOUSE_VISIBLE: AtomicBool = AtomicBool::new(false);

pub unsafe fn init(mem: &MemoryRegion) {
    GET_WARN_RESULT = mem.find("33 D2 33 C9 E8 ? ? ? ? 48 83 F8 04 0F 84")
        .next().expect("get_warn_result").add(4).get_call();

    let no_slowmo = mem.find("38 51 64 74 19")
        .next().expect("no_slowmo");

    no_slowmo.add(26).read_ptr(4).write_bytes(&[RET, NOP, NOP, NOP, NOP]); //No vignette

    no_slowmo.add(8).nop(5); //Vignetting call patch

    no_slowmo.add(34).write_bytes(&[XOR_32_64, 0xD2]); //Timescale override patch
}

pub fn show_loading_prompt(ty: LoadingPrompt, text: &str) {
    invoke!((), 0xABA17D7CE615ADBF, "STRING");
    push_string(text);
    invoke!((), 0xBD12F8228410D9B4, ty as u32);
}

pub fn show_subtitle(text: &str, duration: i32, immediately: bool) {
    invoke!((), 0xB87A37EEB7FAA67D, "STRING");
    push_string(text);
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
    set_text_font(font);
    set_text_scale(scale.into());
    set_text_color(color);
    begin_text_command_draw("CELL_EMAIL_BCON");
    push_string(text.as_ref());
    end_text_command_draw(pos.into())
}

pub fn get_text_width<T>(text: T, font: Font, scale: Vector2<f32>) -> f32 where T: AsRef<str> {
    set_text_font(font);
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

#[repr(C)]
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

#[repr(C)]
pub enum HudElement {
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

pub fn push_string(value: &str) {
    for s in value.as_bytes().chunks(99).map(|c| unsafe { std::str::from_utf8_unchecked(c) }) {
        invoke!((), 0x6C188BE134E074AA, s)
    }
}

pub fn push_int(value: u32) {
    invoke!((), 0x03B504CF259931BC)
}

pub fn set_cursor_sprite(sprite: u32) {
    invoke!((), 0x8DB8CFFD58B62552, sprite)
}

pub fn get_cursor_sprite() -> CursorSprite {
    unsafe { native::CURSOR_SPRITE.load(std::sync::atomic::Ordering::SeqCst).read() }
}

pub fn set_cursor_active_this_frame() {
    invoke!((), 0xAAE7CE1D63167423)
}

pub fn is_cursor_active_this_frame() -> bool {
    use std::sync::atomic::Ordering;
    MOUSE_VISIBLE.load(Ordering::SeqCst)
}

pub fn set_cursor_position(pos: Vector2<f32>) {
    invoke!((), 0xFC695459D4D0E219, pos)
}

pub fn set_map_revealed(revealed: bool) {
    invoke!((), 0xF8DEE0A5600CBB93, revealed)
}

pub fn set_big_map_active(toggle: bool, full: bool) {
    invoke!((), 0x231C8F89D0539D8F, toggle, full)
}

pub fn is_big_map_active() -> bool {
    unsafe { native::EXPANDED_RADAR.load(std::sync::atomic::Ordering::SeqCst).read() }
}

pub fn is_big_map_full() -> bool {
    unsafe { native::REVEAL_FULL_MAP.load(std::sync::atomic::Ordering::SeqCst).read() }
}

pub fn is_hud_element_active(element: HudElement) -> bool {
    invoke!(bool, 0xBC4C9EA5391ECC0D, element as u32)
}

pub fn set_hud_element_visible_this_frame(element: HudElement, visible: bool) {
    if visible {
        invoke!((), 0x0B4DF1FA60C0E664, element as u32)
    } else {
        invoke!((), 0x6806C51AD12B83B8, element as u32)
    }
}

fn set_text_font(font: Font) {
    invoke!((), 0x66E0276CC5F6B9DA, font as u32)
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

pub fn prompt(env: &mut ScriptEnv, title: &str, placeholder: &str, max_length: u32) -> Option<String> {
    super::locale::set_translation("FMMC_KEY_TIP10", title);
    invoke!((), 0x00DC833F2568DBF6, 1u32, "FMMC_KEY_TIP10", "", placeholder, "", "", "", max_length);
    loop {
        match invoke!(u32, 0x0CF2B696BBF945AE) {
            1 => {
                let input = invoke!(&str, 0x8362B09B91893647);
                break Some(input.to_owned());
            },
            2 => {
                break None;
            },
            _ => {
                env.wait(0);
            }
        }
    }
}

fn get_warn_result() -> FrontendButtons {
    let getter: GetWarnResult = unsafe { std::mem::transmute(GET_WARN_RESULT) };
    getter(true, 0)
}

pub fn warn(env: &mut ScriptEnv, title: &str, line1: &str, line2: &str, buttons: FrontendButtons, background: bool) -> FrontendButtons {
    super::locale::set_translation("WNMC_TITLE", title);
    super::locale::set_translation("WNMC_LINE1", line1);
    super::locale::set_translation("WNMC_LINE2", line2);
    let buttons = buttons as u32;
    loop {
        env.wait(0);
        invoke!((), 0xDC38CC1E35B6A5D7, "WNMC_TITLE", "WNMC_LINE1", buttons, "WNMC_LINE2", 0, -1, false, 0, true);
        let result = get_warn_result();
        if result != FrontendButtons::None {
            break result;
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialOrd, PartialEq, Hash)]
pub enum FrontendButtons {
    None = 0,
    Select = 1,
    Ok = 2,
    Yes = 4,
    Back = 8,
    BackSelect = 9,
    BackOk = 10,
    BackYes = 12,
    Cancel = 16,
    CancelSelect = 17,
    CancelOk = 18,
    CancelYes = 20,
    No = 32,
    NoSelect = 33,
    NoOk = 34,
    YesNo = 36,
    Retry = 64,
    RetrySelect = 65,
    RetryOk = 66,
    RetryYes = 68,
    RetryBack = 72,
    RetryBackSelect = 73,
    RetryBackOk = 74,
    RetryBackYes = 76,
    RetryCancel = 80,
    RetryCancelSelect = 81,
    RetryCancelOk = 82,
    RetryCancelYes = 84,
    Skip = 256,
    SkipSelect = 257,
    SkipOk = 258,
    SkipYes = 260,
    SkipBack = 264,
    SkipBackSelect = 265,
    SkipBackOk = 266,
    SkipBackYes = 268,
    SkipCancel = 272,
    SkipCancelSelect = 273,
    SkipCancelOk = 274,
    SkipCancelYes = 276,
    Continue = 16384,
    BackContinue = 16392,
    CancelContinue = 16400,
    LoadingSpinner = 134217728,
    SelectLoadingSpinner = 134217729,
    OkLoadingSpinner = 134217730,
    YesLoadingSpinner = 134217732,
    BackLoadingSpinner = 134217736,
    BackSelectLoadingSpinner = 134217737,
    BackOkLoadingSpinner = 134217738,
    BackYesLoadingSpinner = 134217740,
    CancelLoadingSpinner = 134217744,
    CancelSelectLoadingSpinner = 134217745,
    CancelOkLoadingSpinner = 134217746,
    CancelYesLoadingSpinner = 134217748
}

pub struct TextInput {
    selection: Range<usize>,
    input: String,
    command_pos: usize,
    command_history: Vec<String>,
    last_selection_changed: Instant,
    width: f32,
    height: f32,
    font: Font
}

impl TextInput {
    pub fn new(input: String, width: f32, height: f32, font: Font) -> TextInput {
        TextInput {
            selection: 0..0,
            input,
            command_pos: 0,
            command_history: vec![],
            last_selection_changed: Instant::now(),
            width,
            height,
            font
        }
    }

    pub fn input(&mut self, event: &InputEvent) -> Option<String> {
        use winapi::um::winuser::{VK_BACK, VK_DELETE, VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_UP, VK_DOWN, VK_RETURN};
        match event {
            InputEvent::Keyboard(event) => {
                match event {
                    KeyboardEvent::Key { key, alt, shift, control, is_up, .. } if *is_up => {
                        const VK_KEY_C: c_int = 0x43;
                        const VK_KEY_X: c_int = 0x58;
                        const VK_KEY_V: c_int = 0x56;

                        match *key {
                            VK_LEFT => {
                                if *control {con
                                    if *shift {
                                        self.selection.end = 0;
                                    } else {
                                        self.selection = 0..0;
                                    }
                                } else if *shift {
                                    if self.selection.end > 0 {
                                        self.selection.end -= 1;
                                    }
                                } else {
                                    let to = self.selection.start.min(self.selection.end);
                                    if to > 0 {
                                        self.selection = (to - 1)..(to - 1);
                                    } else {
                                        self.selection = to..to;
                                    }
                                }
                                self.last_selection_changed = Instant::now();
                            },
                            VK_RIGHT => {
                                if *control {
                                    let len = self.get_input().chars().count();
                                    if *shift {
                                        self.selection.end = len;
                                    } else {
                                        self.selection = len..len;
                                    }
                                } else if *shift {
                                    if self.selection.end < self.get_input().chars().count() {
                                        self.selection.end += 1;
                                    }
                                } else {
                                    let len = self.get_input().chars().count();
                                    let to = self.selection.start.max(self.selection.end);
                                    if to < len {
                                        self.selection = (to + 1)..(to + 1);
                                    } else {
                                        self.selection = to..to;
                                    }
                                }
                                self.last_selection_changed = Instant::now();
                            },
                            VK_HOME => {
                                if *shift {
                                    self.selection.end = 0;
                                } else {
                                    self.selection = 0..0;
                                }
                                self.last_selection_changed = Instant::now();
                            },
                            VK_END => {
                                let len = self.get_input().chars().count();
                                if *shift {
                                    self.selection.end = len;
                                } else {
                                    self.selection = len..len;
                                }
                                self.last_selection_changed = Instant::now();
                            },
                            VK_UP => {
                                if self.command_pos < self.command_history.len() {
                                    self.command_pos += 1;
                                    let len = self.get_input().chars().count();
                                    self.selection = len..len;
                                    self.last_selection_changed = Instant::now();
                                }
                            },
                            VK_DOWN => {
                                if self.command_pos > 0 {
                                    self.command_pos -= 1;
                                    let len = self.get_input().chars().count();
                                    self.selection = len..len;
                                    self.last_selection_changed = Instant::now();
                                }
                            },
                            VK_RETURN => {
                                if self.command_pos == 0 {
                                    if !self.input.is_empty() {
                                        let mut input = String::new();
                                        std::mem::swap(&mut self.input, &mut input);
                                        self.selection = 0..0;
                                        self.command_history.push(input.clone());
                                        return Some(input);
                                    }
                                } else {
                                    let input = self.command_history[self.command_pos - 1].clone();
                                    self.input = String::new();
                                    self.selection = 0..0;
                                    self.command_pos = 0;
                                    return Some(input);
                                }
                            },
                            VK_KEY_C if *control => {
                                let start = self.selection.start;
                                let end = self.selection.end;
                                let selected = self.get_chars(start, end);
                                if !selected.is_empty() {
                                    let mut context = ClipboardContext::new()
                                        .expect("clipboard context creation failed");
                                    context.set_contents(selected)
                                        .expect("clipboard text update failed");
                                }
                            },
                            VK_KEY_X if *control => {
                                let start = self.selection.start;
                                let end = self.selection.end;
                                let selected = self.get_chars(start, end);
                                if !selected.is_empty() {
                                    let mut context = ClipboardContext::new()
                                        .expect("clipboard context creation failed");
                                    context.set_contents(selected)
                                        .expect("clipboard text update failed");
                                    self.replace_chars(start, end, "")
                                }
                            },
                            VK_KEY_V if *control => {
                                let start = self.selection.start;
                                let end = self.selection.end;
                                let mut context = ClipboardContext::new()
                                    .expect("clipboard context creation failed");
                                let mut input = context.get_contents()
                                    .expect("clipboard text getting failed");
                                self.replace_chars(start, end, &input);
                            },
                            _ => {}
                        }
                    },
                    KeyboardEvent::Char(c) => {
                        match c {
                            &'\u{0008}' => self.erase_left(),
                            &'\u{007F}' => self.erase_right(),
                            c if !c.is_control() => self.enter_char(*c),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        None
    }

    pub fn draw(&self, start_x: f32, start_y: f32) {
        const INPUT_COLOR: Rgba = Rgba::WHITE;
        const INPUT_COLOR_BUSY: Rgba = Rgba::DARK_GRAY;
        const PREFIX_COLOR: Rgba = Rgba::new(52, 152, 219, 255);
        const ALT_BACKGROUND_COLOR: Rgba = Rgba::new(52, 73, 94, 127);
        const SELECTION_COLOR: Rgba = Rgba::new(0, 0, 255, 127);
        const CURSOR_COLOR: Rgba = Rgba::new(255, 255, 255, 127);

        let now = Instant::now();
        let scale = Vector2::new(0.35, 0.35);

        // Draw blinking cursor
        let start = self.selection.start;
        let end = self.selection.end;

        // Draw input field
        draw_rect([start_x, start_y], [self.width, self.height], ALT_BACKGROUND_COLOR);
        draw_rect([start_x, start_y + self.height], [80.0, self.height], ALT_BACKGROUND_COLOR);
        // Draw input prefix
        draw_text(">", [start_x, start_y], PREFIX_COLOR, self.font, scale);
        // Draw input text
        draw_text(self.get_input(), [start_x + 25.0, start_y], INPUT_COLOR, self.font, scale);
        // Draw page information
        if start == end {
            if now.duration_since(self.last_selection_changed).subsec_millis() < 500 {
                let prefix = self.get_input().chars().take(start).collect::<String>();
                let x = get_text_width(&prefix, self.font, scale) * self.width;
                let x = if prefix.is_empty() { x - 0.5 } else { x - 4.0 };
                draw_rect([25.0 + start_x + x, start_y + 2.0], [1.5, self.height - 4.0], CURSOR_COLOR);
            }
        } else {
            let from = start.min(end);
            let to = start.max(end);
            let prefix = self.get_input().chars().take(from).collect::<String>();
            let x = get_text_width(&prefix, self.font, scale) * self.width;
            let selected = self.get_input().chars().skip(from).take(to - from).collect::<String>();
            let width = get_text_width(&selected, self.font, scale) * self.width;
            let x = if prefix.is_empty() { x - 0.5 } else { x - 4.0 };
            draw_rect([25.0 + start_x + x, start_y + 2.0], [width, self.height - 4.0], SELECTION_COLOR);
        }
    }

    fn erase_left(&mut self) {
        let start = self.selection.start;
        let end = self.selection.end;
        if start == end && start > 0 {
            self.replace_chars(start - 1, start, "");
        } else {
            self.replace_chars(start, end, "");
        }
    }

    fn erase_right(&mut self) {
        let start = self.selection.start;
        let end = self.selection.end;
        let len = self.get_input().chars().count();
        if start == end && end < len {
            self.replace_chars(end, end + 1, "");
        } else {
            self.replace_chars(start, end, "");
        }
    }

    fn enter_char(&mut self, c: char) {
        let start = self.selection.start;
        let end = self.selection.end;
        self.replace_chars(start, end, &format!("{}", c));
        let pos = start.min(end) + 1;
        self.selection = pos..pos;
    }

    fn get_chars(&self, start: usize, end: usize) -> String {
        let from = start.min(end);
        let to = start.max(end);
        self.get_input().chars().skip(from).take(to - from).collect::<String>()
    }

    fn replace_chars(&mut self, start: usize, end: usize, replacement: &str) {
        let bytes_len = self.get_input().len();
        let len = self.get_input().chars().count();
        let from = start.min(end);
        let to = start.max(end);
        let old_input = self.get_input().chars().collect::<Vec<_>>();
        let mut new_input = String::with_capacity(bytes_len - (to - from) + replacement.len());
        for i in 0..from {
            new_input.push(old_input[i])
        }
        new_input.push_str(replacement);
        for i in to..len {
            new_input.push(old_input[i])
        }
        std::mem::replace(self.get_input_mut(), new_input);
        self.selection = from..from;
        self.last_selection_changed = Instant::now();
    }

    pub fn get_input(&self) -> &String {
        if self.command_pos == 0 {
            &self.input
        } else {
            &self.command_history[self.command_pos - 1]
        }
    }

    pub fn get_input_mut(&mut self) -> &mut String {
        if self.command_pos == 0 {
            &mut self.input
        } else {
            &mut self.command_history[self.command_pos - 1]
        }
    }
}