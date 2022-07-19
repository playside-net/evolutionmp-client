use std::ops::Range;
use std::os::raw::c_int;
use std::time::Instant;

use cgmath::{Vector2, Vector3};
use clipboard::{ClipboardContext, ClipboardProvider};

use crate::{invoke, native};
use crate::{bind_fn_ip, bind_field_ip, mem};
use crate::game::Rgba;
use crate::hash::{Hashable, Hash};
use crate::win::input::{InputEvent, KeyboardEvent};
use crate::client::native::alloc::RageVec;
use std::ffi::{CStr, OsStr};
use crate::pattern::{RET, NOP};

pub mod notification;

pub const BASE_WIDTH: f32 = 1280.0;
pub const BASE_HEIGHT: f32 = 720.0;

#[repr(C, packed(1))]
pub struct UIMenuItem {
    index: u32,
    title: Hash,
    unk_vec: RageVec<()>,
    setting_id: u8,
    action_ty: u8,
    ty: u8,
    state_flags: u8,
    pad1: [u8; 4]
}

#[repr(C, packed(1))]
pub struct UIMenu {
    c_instance: *mut (),
    items: RageVec<Box<UIMenuItem>>,
    unk: *mut (),
    pad1: [u8; 16],
    name: *mut u8,
    pad2: [u8; 8],
    id: u32,
    unk1: u32,
    unk_flag: u16,
    scroll_flags: u16,
    pad3: [u8; 4]
}

impl UIMenu {
    pub fn empty() -> UIMenu {
        UIMenu {
            c_instance: std::ptr::null_mut(),
            items: RageVec::empty(),
            unk: std::ptr::null_mut(),
            pad1: [0; 16],
            name: std::ptr::null_mut(),
            pad2: [0; 8],
            id: 0,
            unk1: 0,
            unk_flag: 0,
            scroll_flags: 0,
            pad3: [0; 4]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.c_instance.is_null() || self.name.is_null()
    }

    pub fn get_name(&self) -> &OsStr {
        unsafe { std::mem::transmute(CStr::from_ptr(self.name.cast()).to_bytes()) }
    }
}

#[repr(u32)]
pub enum DynamicMenuAction {
    Slider,
    Toggle,
    AimMode,
    GamepadLayout,
    LowMedHi,
    AudioOutput,
    FadeRadio,
    SelfRadioMode,
    OffOnBlips,
    SimpleComplex,
    Language,
    LowHi
}

bind_fn_ip!(GET_WARN_RESULT, "33 D2 33 C9 E8 ? ? ? ? 48 83 F8 04 0F 84", 5, (bool, u32) -> FrontendButtons);
bind_field_ip!(ACTIVE_MENU_POOL, "0F B7 54 51 ?", -4, RageVec<UIMenu>);

pub fn hook() {
    info!("Hooking UI...");
    assert_eq!(std::mem::size_of::<UIMenuItem>(), 0x20);
    assert_eq!(std::mem::size_of::<UIMenu>(), 0x50);
    lazy_static::initialize(&GET_WARN_RESULT);
    lazy_static::initialize(&ACTIVE_MENU_POOL);
}

pub fn init() {
    unsafe {
        //let no_slowmo = mem!("32 C0 F3 0F 11 09").expect("no_slowmo");
        //no_slowmo.nop(6);
        let no_slowmo = mem!("38 51 64 74 19")
           .expect("no_slowmo");

        no_slowmo.add(26).read_ptr(4).write_bytes(&[RET, NOP, NOP, NOP, NOP]); //No vignette

        no_slowmo.add(8).nop(5); //Vignetting call patch

        no_slowmo.add(34).write_bytes(&[0x31, 0xD2]); //Timescale override patch
    }
}

pub fn print_menus() {
    for menu in unsafe { ACTIVE_MENU_POOL.as_mut() }.iter_mut() {
        if !menu.is_empty() {
            let name = menu.get_name();
            let instance = menu.c_instance;
            let id = menu.id;
            // let items = &*menu.items;
            // warn!("{:p} menu \"{}\": {} with {} elements", instance, name.to_string_lossy(), id, items.len());
            //if name == "PAUSE_MENU_PAGES_GAME" {
            //     warn!("unk {:p} unk1 {} unk_flag {} scroll_flags {}", menu.unk, menu.unk1, menu.unk_flag, menu.scroll_flags);
                //menu.items = RageVec::empty();
            //}
        }
    }
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

pub fn show_help(text: &str, looping: bool, beep: bool, duration: Option<u32>) {
    invoke!((), 0x8509B634FBE7DA11, "STRING");
    push_string(text);
    invoke!((), 0x238FFE5C7B0498A6, 0, looping, beep, duration.unwrap_or(u32::MAX))
}

pub fn show_help_this_frame(text: &str) {
    invoke!((), 0x960C9FF8F616E41C, text, 0)
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
    Pricedown = 7,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum LoadingPrompt {
    LoadingLeft,
    LoadingLeft2,
    LoadingLeft3,
    SavingLeft,
    LoadingRight,
}

#[repr(C)]
#[derive(Copy, Clone)]
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
    Remove,
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
    ReplayTimer,
}

pub fn at_origin<F>(origin: Vector3<f32>, task: F) where F: FnOnce() {
    invoke!((), 0xAA0008F3BBB8F416, origin, 0);
    task();
    invoke!((), 0xFF0B610F6BE0D7AF);
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
    invoke!((), 0x03B504CF259931BC, value)
}

pub fn is_pause_menu_active() -> bool {
    invoke!(bool, 0xB0034A223497FFCB)
}

pub fn set_pause_menu_active(active: bool) {
    invoke!((), 0xDF47FC56C71569CF, active)
}

pub fn set_frontend_active(active: bool) {
    invoke!((), 0x745711A75AB09277, active)
}

pub fn activate_frontend_menu<H>(menu: H, toggle_pause: bool, component: i32) where H: Hashable {
    invoke!((), 0xEF01D36B9C9D0C7B, menu.joaat(), toggle_pause, component)
}

pub fn set_cursor_sprite(sprite: u32) {
    invoke!((), 0x8DB8CFFD58B62552, sprite)
}

pub fn get_cursor_sprite() -> CursorSprite {
    *native::CURSOR_SPRITE.as_ref()
}

pub fn set_cursor_active_this_frame() {
    invoke!((), 0xAAE7CE1D63167423)
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
    *native::EXPANDED_RADAR.as_ref()
}

pub fn is_big_map_full() -> bool {
    *native::REVEAL_FULL_MAP.as_ref()
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

pub fn set_ability_bar_visible(visible: bool) {
    invoke!((), 0x1DFEDD15019315A9, visible)
}

pub fn set_director_mode(enabled: bool) {
    invoke!((), 0x808519373FD336A3, enabled)
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

pub fn prompt(title: &str, placeholder: &str, max_length: u32) -> Option<String> {
    super::locale::set_translation("FMMC_KEY_TIP10", title);
    invoke!((), 0x00DC833F2568DBF6, 1u32, "FMMC_KEY_TIP10", "", placeholder, "", "", "", max_length);
    loop {
        match invoke!(u32, 0x0CF2B696BBF945AE) {
            1 => {
                let input = invoke!(&str, 0x8362B09B91893647);
                break Some(input.to_owned());
            }
            2 => {
                break None;
            }
            _ => {
                super::script::wait(0);
            }
        }
    }
}

pub fn warn(title: &str, line1: &str, line2: &str, buttons: FrontendButtons, background: bool) -> FrontendButtons {
    super::locale::set_translation("WNMC_TITLE", title);
    super::locale::set_translation("WNMC_LINE1", line1);
    super::locale::set_translation("WNMC_LINE2", line2);
    let buttons = buttons.bits;
    loop {
        super::script::wait(0);
        invoke!((), 0xDC38CC1E35B6A5D7, "WNMC_TITLE", "WNMC_LINE1", buttons, "WNMC_LINE2", 0, -1, false, 0, background);
        let result = GET_WARN_RESULT(true, 0);
        if result != FrontendButtons::NONE {
            break result;
        }
    }
}

bitflags! {
    #[repr(C)]
    pub struct FrontendButtons: u32 {
        const NONE = 0;
        const SELECT = 1;
        const OK = 2;
        const YES = 4;
        const BACK = 8;
        const CANCEL = 16;
        const NO = 32;
        const RETRY = 64;
        const UNK128 = 128;
        const SKIP = 256;
        const CONTINUE = 16384;
        const LOADING_SPINNER = 134217728;
    }
}

pub struct TextInput {
    selection: Range<usize>,
    input: String,
    history_pos: usize,
    history: Vec<String>,
    last_selection_changed: Instant,
    width: f32,
    height: f32,
    font: Font,
}

impl TextInput {
    pub fn new(input: String, width: f32, height: f32, font: Font) -> TextInput {
        TextInput {
            selection: 0..0,
            input,
            history_pos: 0,
            history: vec![],
            last_selection_changed: Instant::now(),
            width,
            height,
            font,
        }
    }

    pub fn input(&mut self, event: &InputEvent) -> Option<String> {
        use winapi::um::winuser::{VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_UP, VK_DOWN, VK_RETURN};
        match event {
            InputEvent::Keyboard(event) => {
                match event {
                    KeyboardEvent::Key { key, shift, control, is_up, .. } if !*is_up => {
                        const VK_KEY_A: c_int = 0x41;
                        const VK_KEY_C: c_int = 0x43;
                        const VK_KEY_X: c_int = 0x58;
                        const VK_KEY_V: c_int = 0x56;

                        match *key {
                            VK_LEFT => {
                                if *control {
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
                            }
                            VK_RIGHT => {
                                if *control {
                                    let len = self.len();
                                    if *shift {
                                        self.selection.end = len;
                                    } else {
                                        self.selection = len..len;
                                    }
                                } else if *shift {
                                    if self.selection.end < self.len() {
                                        self.selection.end += 1;
                                    }
                                } else {
                                    let len = self.len();
                                    let to = self.selection.start.max(self.selection.end);
                                    if to < len {
                                        self.selection = (to + 1)..(to + 1);
                                    } else {
                                        self.selection = to..to;
                                    }
                                }
                                self.last_selection_changed = Instant::now();
                            }
                            VK_HOME => {
                                if *shift {
                                    self.selection.end = 0;
                                } else {
                                    self.selection = 0..0;
                                }
                                self.last_selection_changed = Instant::now();
                            }
                            VK_END => {
                                let len = self.len();
                                if *shift {
                                    self.selection.end = len;
                                } else {
                                    self.selection = len..len;
                                }
                                self.last_selection_changed = Instant::now();
                            }
                            VK_UP => {
                                if self.history_pos < self.history.len() {
                                    self.history_pos += 1;
                                    let len = self.len();
                                    self.selection = len..len;
                                    self.last_selection_changed = Instant::now();
                                }
                            }
                            VK_DOWN => {
                                if self.history_pos > 0 {
                                    self.history_pos -= 1;
                                    let len = self.len();
                                    self.selection = len..len;
                                    self.last_selection_changed = Instant::now();
                                }
                            }
                            VK_RETURN => {
                                if self.history_pos == 0 {
                                    if !self.input.is_empty() {
                                        let mut input = String::new();
                                        std::mem::swap(&mut self.input, &mut input);
                                        self.selection = 0..0;
                                        self.history.push(input.clone());
                                        return Some(input);
                                    }
                                } else {
                                    let input = self.history[self.history_pos - 1].clone();
                                    self.reset();
                                    return Some(input);
                                }
                            }
                            VK_KEY_A if *control => {
                                let len = self.len();
                                self.selection = 0..len;
                                self.last_selection_changed = Instant::now();
                            }
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
                            }
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
                            }
                            VK_KEY_V if *control => {
                                let start = self.selection.start;
                                let end = self.selection.end;
                                let mut context = ClipboardContext::new()
                                    .expect("clipboard context creation failed");
                                let input = context.get_contents()
                                    .expect("clipboard text getting failed");
                                self.replace_chars(start, end, &input);
                                let pos = start.min(end) + input.chars().count();
                                self.selection = pos..pos;
                                self.last_selection_changed = Instant::now();
                            }
                            _ => {}
                        }
                    }
                    KeyboardEvent::Char(c) => {
                        match c.as_str() {
                            "\u{0008}" => self.erase_left(),
                            "\u{007F}" => self.erase_right(),
                            c => self.enter_char(c)
                        }
                    }
                    _ => {}
                }
            }
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
        draw_text(self.get_input().replace("~", "\\~"), [start_x + 25.0, start_y], INPUT_COLOR, self.font, scale);
        // Draw page information
        if start == end {
            if now.duration_since(self.last_selection_changed).subsec_millis() < 500 {
                let prefix = self.get_input().chars().take(start).collect::<String>();
                let x = get_text_width(prefix.replace("~", "\\~"), self.font, scale) * self.width;
                let x = if prefix.is_empty() { x - 0.5 } else { x - 4.0 };
                draw_rect([25.0 + start_x + x, start_y + 2.0], [1.5, self.height - 4.0], CURSOR_COLOR);
            }
        } else {
            let from = start.min(end);
            let to = start.max(end);
            let prefix = self.get_input().chars().take(from).collect::<String>();
            let x = get_text_width(prefix.replace("~", "\\~"), self.font, scale) * self.width;
            let selected = self.get_input().chars().skip(from).take(to - from).collect::<String>();
            let width = get_text_width(selected.replace("~", "\\~"), self.font, scale) * self.width;
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
        let len = self.len();
        if start == end && end < len {
            self.replace_chars(end, end + 1, "");
        } else {
            self.replace_chars(start, end, "");
        }
    }

    fn enter_char(&mut self, c: &str) {
        let start = self.selection.start;
        let end = self.selection.end;
        self.replace_chars(start, end, c);
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
        let len = self.len();
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
        let _ = std::mem::replace(self.get_input_mut(), new_input);
        self.selection = from..from;
        self.last_selection_changed = Instant::now();
    }

    pub fn reset(&mut self) {
        self.selection = 0..0;
        self.last_selection_changed = Instant::now();
        self.input = String::new();
        self.history_pos = 0;
    }

    pub fn len(&self) -> usize {
        self.get_input().chars().count()
    }

    pub fn get_display_input(&self) -> String {
        self.get_input().replace("~", "\\~")
    }

    pub fn get_input(&self) -> &String {
        if self.history_pos == 0 {
            &self.input
        } else {
            &self.history[self.history_pos - 1]
        }
    }

    pub fn get_input_mut(&mut self) -> &mut String {
        if self.history_pos == 0 {
            &mut self.input
        } else {
            &mut self.history[self.history_pos - 1]
        }
    }
}