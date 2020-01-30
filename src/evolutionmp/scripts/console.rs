use crate::runtime::{Script, ScriptEnv, Runtime};
use crate::game::ui::{BASE_WIDTH, BASE_HEIGHT, Font};
use crate::game::Rgba;
use crate::game::controls::{Group as ControlGroup, Control};
use crate::win::input::{InputEvent, KeyboardEvent};
use crate::events::{ScriptEvent, NativeEvent};
use std::time::{Instant, UNIX_EPOCH, SystemTime};
use std::collections::VecDeque;
use std::os::raw::c_int;
use winapi::um::winuser::{VK_BACK, VK_DELETE, VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_UP, VK_DOWN, VK_ESCAPE, VK_RETURN};
use cgmath::Vector2;
use std::time::Duration;
use std::ffi::CString;
use widestring::WideCStr;
use winapi::_core::sync::atomic::AtomicBool;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::ops::Range;

pub const FONT: Font = Font::ChaletLondon;
pub const CONSOLE_WIDTH: f32 = BASE_WIDTH;
pub const CONSOLE_HEIGHT: f32 = BASE_HEIGHT / 3.0;
pub const INPUT_HEIGHT: f32 = 20.0;
pub const LINES_PER_PAGE: usize = 16;

pub const INPUT_COLOR: Rgba = Rgba::WHITE;
pub const INPUT_COLOR_BUSY: Rgba = Rgba::DARK_GRAY;
pub const OUTPUT_COLOR: Rgba = Rgba::WHITE;
pub const PREFIX_COLOR: Rgba = Rgba::new(52, 152, 219, 255);
pub const BACKGROUND_COLOR: Rgba = Rgba::new(0, 0, 0, 127);
pub const ALT_BACKGROUND_COLOR: Rgba = Rgba::new(52, 73, 94, 127);
pub const SELECTION_COLOR: Rgba = Rgba::new(0, 0, 255, 127);
pub const CURSOR_COLOR: Rgba = Rgba::new(255, 255, 255, 127);

static OPEN: AtomicBool = AtomicBool::new(false);

pub struct ScriptConsole {
    selection: Range<usize>,
    command_pos: usize,
    current_page: usize,
    input: String,
    line_history: Vec<String>,
    command_history: Vec<String>,
    last_closed: Instant,
    last_selection_changed: Instant,
}

impl Script for ScriptConsole {
    fn prepare(&mut self, mut env: ScriptEnv) {

    }

    fn frame(&mut self, mut env: ScriptEnv) {
        if is_open() {
            self.lock_controls();
            self.draw();
        } else if self.get_last_closed() > Instant::now() {
            self.lock_controls();
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(event) => {
                        let open = is_open();

                        match event {
                            KeyboardEvent::Key { key, alt, shift, control, is_up, .. } if *is_up => {
                                const VK_KEY_C: c_int = 0x43;
                                const VK_KEY_X: c_int = 0x58;
                                const VK_KEY_V: c_int = 0x56;
                                const VK_KEY_T: c_int = 0x54;

                                match *key {
                                    VK_LEFT if open => {
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
                                    },
                                    VK_RIGHT if open => {
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
                                    VK_HOME if open => {
                                        if *shift {
                                            self.selection.end = 0;
                                        } else {
                                            self.selection = 0..0;
                                        }
                                        self.last_selection_changed = Instant::now();
                                    },
                                    VK_END if open => {
                                        let len = self.get_input().chars().count();
                                        if *shift {
                                            self.selection.end = len;
                                        } else {
                                            self.selection = len..len;
                                        }
                                        self.last_selection_changed = Instant::now();
                                    },
                                    VK_UP if open => {
                                        if self.command_pos < self.command_history.len() {
                                            self.command_pos += 1;
                                            let len = self.get_input().chars().count();
                                            self.selection = len..len;
                                            self.last_selection_changed = Instant::now();
                                        }
                                    },
                                    VK_DOWN if open => {
                                        if self.command_pos > 0 {
                                            self.command_pos -= 1;
                                            let len = self.get_input().chars().count();
                                            self.selection = len..len;
                                            self.last_selection_changed = Instant::now();
                                        }
                                    },
                                    VK_ESCAPE if open => {
                                        self.set_open(false);
                                    },
                                    VK_RETURN if open => {
                                        if self.command_pos == 0 {
                                            if !self.input.is_empty() {
                                                let mut input = String::new();
                                                std::mem::swap(&mut self.input, &mut input);
                                                self.selection = 0..0;
                                                self.command_history.push(input.clone());
                                                output.push_back(ScriptEvent::ConsoleInput(input));
                                            }
                                        } else {
                                            let input = self.command_history[self.command_pos - 1].clone();
                                            self.input = String::new();
                                            self.selection = 0..0;
                                            self.command_pos = 0;
                                            output.push_back(ScriptEvent::ConsoleInput(input));
                                        }
                                    },
                                    VK_KEY_C if open && *control => {
                                        let start = self.selection.start;
                                        let end = self.selection.end;
                                        let from = start.min(end);
                                        let to = start.max(end);
                                        if to > from {
                                            let mut context = ClipboardContext::new()
                                                .expect("clipboard context creation failed");
                                            context.set_contents(self.get_input().clone())
                                                .expect("clipboard text update failed");
                                        }
                                    },
                                    VK_KEY_X if open && *control => {
                                        let start = self.selection.start;
                                        let end = self.selection.end;
                                        let from = start.min(end);
                                        let to = start.max(end);
                                        if to > from {
                                            let mut context = ClipboardContext::new()
                                                .expect("clipboard context creation failed");
                                            let mut input = String::new();
                                            std::mem::swap(self.get_input_mut(), &mut input);
                                            context.set_contents(input)
                                                .expect("clipboard text update failed");
                                        }
                                    },
                                    VK_KEY_V if open && *control => {
                                        let start = self.selection.start;
                                        let end = self.selection.end;
                                        let from = start.min(end);
                                        let to = start.max(end);
                                        if to > from {
                                            let mut context = ClipboardContext::new()
                                                .expect("clipboard context creation failed");
                                            let mut input = context.get_contents()
                                                .expect("clipboard text getting failed");
                                            std::mem::swap(self.get_input_mut(), &mut input);
                                        }
                                    },
                                    VK_KEY_T if !open && !crate::game::ui::is_cursor_active_this_frame() => {
                                        self.set_open(true);
                                    }
                                    _ => {}
                                }
                            },
                            KeyboardEvent::Char(c) if open => {
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
            },
            ScriptEvent::NativeEvent(event) => {
                self.line_history.push(format!("Native event received: {:?}", event));
                return true;
            },
            ScriptEvent::ConsoleOutput(line) => {
                self.line_history.push(line.clone());
                return true;
            }
            _ => {}
        }
        false
    }
}

pub fn is_open() -> bool {
    use std::sync::atomic::Ordering;
    OPEN.load(Ordering::SeqCst)
}

impl ScriptConsole {
    pub fn new() -> ScriptConsole {
        ScriptConsole {
            selection: 0..0,
            command_pos: 0,
            current_page: 1,
            input: String::new(),
            line_history: Vec::new(),
            command_history: Vec::new(),
            last_closed: Instant::now(),
            last_selection_changed: Instant::now()
        }
    }

    fn set_open(&mut self, open: bool) {
        use std::sync::atomic::Ordering;
        OPEN.store(open, Ordering::SeqCst);
        if !open {
            self.last_closed = Instant::now() + Duration::from_millis(200);
            self.lock_controls();
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

    fn get_last_closed(&self) -> Instant {
        self.last_closed
    }
    
    fn lock_controls(&self) {
        use crate::game::controls;
        controls::disable_all_actions(ControlGroup::Move);
        controls::enable_action(ControlGroup::Move, Control::LookLeftRight, true);
        controls::enable_action(ControlGroup::Move, Control::LookUpDown, true);
        controls::enable_action(ControlGroup::Move, Control::LookUpOnly, true);
        controls::enable_action(ControlGroup::Move, Control::LookDownOnly, true);
        controls::enable_action(ControlGroup::Move, Control::LookLeftOnly, true);
        controls::enable_action(ControlGroup::Move, Control::LookRightOnly, true);
    }

    fn draw(&self) {
        use crate::game::ui::{draw_rect, draw_text, get_text_width};

        let now = Instant::now();
        let scale = Vector2::new(0.35, 0.35);
        // Draw background
        draw_rect([0.0, 0.0], [CONSOLE_WIDTH, CONSOLE_HEIGHT], BACKGROUND_COLOR);
        // Draw input field
        //draw_rect([0.0, CONSOLE_HEIGHT], [CONSOLE_WIDTH, INPUT_HEIGHT], ALT_BACKGROUND_COLOR);
        draw_rect([0.0, CONSOLE_HEIGHT + INPUT_HEIGHT], [80.0, INPUT_HEIGHT], ALT_BACKGROUND_COLOR);
        // Draw input prefix
        draw_text(">", [0.0, CONSOLE_HEIGHT], PREFIX_COLOR, FONT, scale);
        // Draw input text
        draw_text(self.get_input(), [25.0, CONSOLE_HEIGHT], INPUT_COLOR, FONT, scale);
        // Draw page information
        let total_pages = ((self.line_history.len() + (LINES_PER_PAGE - 1)) / LINES_PER_PAGE).max(1);
        draw_text(format!("Page {}/{}", self.current_page, total_pages), [5.0, CONSOLE_HEIGHT + INPUT_HEIGHT], INPUT_COLOR, FONT, scale);

        // Draw blinking cursor
        let start = self.selection.start;
        let end = self.selection.end;

        if start == end {
            if now.duration_since(self.last_selection_changed).subsec_millis() < 500 {
                let prefix = self.get_input().chars().take(start).collect::<String>();
                let x = get_text_width(&prefix, FONT, scale) * CONSOLE_WIDTH;
                let x = if prefix.is_empty() { x - 0.5 } else { x - 4.0 };
                draw_rect([25.0 + x, CONSOLE_HEIGHT + 2.0], [1.5, INPUT_HEIGHT - 4.0], CURSOR_COLOR);
            }
        } else {
            let from = start.min(end);
            let to = start.max(end);
            let prefix = self.get_input().chars().take(from).collect::<String>();
            let x = get_text_width(&prefix, FONT, scale) * CONSOLE_WIDTH;
            let selected = self.get_input().chars().skip(from).take(to - from).collect::<String>();
            let width = get_text_width(&selected, FONT, scale) * CONSOLE_WIDTH;
            let x = if prefix.is_empty() { x - 0.5 } else { x - 4.0 };
            draw_rect([25.0 + x, CONSOLE_HEIGHT + 2.0], [width, INPUT_HEIGHT - 4.0], SELECTION_COLOR);
        }

        // Draw console history text
        let history_offset = self.line_history.len() as i32 - (LINES_PER_PAGE * self.current_page) as i32;
        let history_length = (history_offset + LINES_PER_PAGE as i32) as usize;
        for i in (history_offset.max(0) as usize)..history_length {
            draw_text(&self.line_history[i], [2.0, ((i as i32 - history_offset) * 14) as f32], OUTPUT_COLOR, FONT, scale);
        }
    }

    fn get_input(&self) -> &String {
        if self.command_pos == 0 {
            &self.input
        } else {
            &self.command_history[self.command_pos - 1]
        }
    }

    fn get_input_mut(&mut self) -> &mut String {
        if self.command_pos == 0 {
            &mut self.input
        } else {
            &mut self.command_history[self.command_pos - 1]
        }
    }
}