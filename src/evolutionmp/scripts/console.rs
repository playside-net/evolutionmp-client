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

static OPEN: AtomicBool = AtomicBool::new(false);

pub struct ScriptConsole {
    cursor_pos: usize,
    command_pos: usize,
    current_page: usize,
    input: String,
    line_history: Vec<String>,
    command_history: Vec<String>,
    last_closed: Instant
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
                                    //VK_BACK if open => self.erase_left(),
                                    //VK_DELETE if open => self.erase_right(),
                                    VK_LEFT if open => {
                                        if *control {

                                        } else {
                                            if self.cursor_pos > 0 {
                                                self.cursor_pos -= 1;
                                            }
                                        }
                                    },
                                    VK_RIGHT if open => {
                                        if *control {

                                        } else {
                                            if self.cursor_pos < self.get_input().len() {
                                                self.cursor_pos += 1;
                                            }
                                        }
                                    },
                                    VK_HOME if open => {

                                    },
                                    VK_END if open => {

                                    },
                                    VK_UP if open => {
                                        if self.command_pos < self.command_history.len() {
                                            self.command_pos += 1;
                                            self.cursor_pos = self.get_input().len();
                                        }
                                    },
                                    VK_DOWN if open => {
                                        if self.command_pos > 0 {
                                            self.command_pos -= 1;
                                            self.cursor_pos = self.get_input().len();
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
                                                self.cursor_pos = 0;
                                                self.command_history.push(input.clone());
                                                output.push_back(ScriptEvent::ConsoleInput(input));
                                            }
                                        } else {
                                            let input = self.command_history[self.command_pos - 1].clone();
                                            self.input = String::new();
                                            self.cursor_pos = 0;
                                            self.command_pos = 0;
                                            output.push_back(ScriptEvent::ConsoleInput(input));
                                        }
                                    },
                                    VK_KEY_C if open && *control => {
                                        let mut context = ClipboardContext::new()
                                            .expect("clipboard context creation failed");
                                        context.set_contents(self.get_input().clone())
                                            .expect("clipboard text update failed");
                                    },
                                    VK_KEY_X if open && *control => {
                                        let mut context = ClipboardContext::new()
                                            .expect("clipboard context creation failed");
                                        let mut input = String::new();
                                        std::mem::swap(self.get_input_mut(), &mut input);
                                        context.set_contents(input)
                                            .expect("clipboard text update failed");
                                    },
                                    VK_KEY_V if open && *control => {
                                        let mut context = ClipboardContext::new()
                                            .expect("clipboard context creation failed");
                                        let mut input = context.get_contents()
                                            .expect("clipboard text getting failed");
                                        std::mem::swap(self.get_input_mut(), &mut input);
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
                                    c if !c.is_control() => {
                                        self.get_input_mut().push(*c);
                                        self.cursor_pos += 1;
                                    },
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
            cursor_pos: 0,
            command_pos: 0,
            current_page: 1,
            input: String::new(),
            line_history: Vec::new(),
            command_history: Vec::new(),
            last_closed: Instant::now()
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
        let pos = self.cursor_pos;
        let len = self.get_input().len();
        if len > 0 && pos > 0 {
            let mut input = String::with_capacity(len - 1);
            for (i, c) in self.get_input().chars().enumerate() {
                if i != pos - 1 {
                    input.push(c);
                }
            }
            std::mem::replace(self.get_input_mut(), input);
            self.cursor_pos -= 1;
        }
    }

    fn erase_right(&mut self) {
        let pos = self.cursor_pos;
        let len = self.get_input().len();
        if len > 0 && pos < len {
            let mut input = String::with_capacity(len - 1);
            for (i, c) in self.get_input().chars().enumerate() {
                if i != pos {
                    input.push(c);
                }
            }
            std::mem::replace(self.get_input_mut(), input);
        }
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

        let now = SystemTime::now();
        let scale = Vector2::new(0.35, 0.35);
        // Draw background
        draw_rect([0.0, 0.0], [CONSOLE_WIDTH, CONSOLE_HEIGHT], BACKGROUND_COLOR);
        // Draw input field
        draw_rect([0.0, CONSOLE_HEIGHT], [CONSOLE_WIDTH, INPUT_HEIGHT], ALT_BACKGROUND_COLOR);
        draw_rect([0.0, CONSOLE_HEIGHT + INPUT_HEIGHT], [80.0, INPUT_HEIGHT], ALT_BACKGROUND_COLOR);
        // Draw input prefix
        draw_text(">", [0.0, CONSOLE_HEIGHT], PREFIX_COLOR, FONT, scale);
        // Draw input text
        draw_text(self.get_input(), [25.0, CONSOLE_HEIGHT], INPUT_COLOR, FONT, scale);
        // Draw page information
        let total_pages = ((self.line_history.len() + (LINES_PER_PAGE - 1)) / LINES_PER_PAGE).max(1);
        draw_text(format!("Page {}/{}", self.current_page, total_pages), [5.0, CONSOLE_HEIGHT + INPUT_HEIGHT], INPUT_COLOR, FONT, scale);

        // Draw blinking cursor
        if now.duration_since(UNIX_EPOCH).unwrap().subsec_millis() < 500 {
            let prefix = self.get_input().chars().take(self.cursor_pos).collect::<String>();
            let width = get_text_width(prefix, FONT, scale);
            draw_text("~w~~h~|~w~", [25.0 + (width * CONSOLE_WIDTH) - 4.0, CONSOLE_HEIGHT], INPUT_COLOR, FONT, scale);
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