use crate::runtime::{Script, ScriptEnv, Runtime};
use crate::game::ui::{BASE_WIDTH, BASE_HEIGHT, Font, TextInput};
use crate::game::Rgba;
use crate::game::controls::{Group as ControlGroup, Control};
use crate::win::input::{InputEvent, KeyboardEvent};
use crate::events::{ScriptEvent, NativeEvent};
use std::time::Instant;
use std::collections::VecDeque;
use cgmath::Vector2;
use std::time::Duration;
use std::ffi::CString;
use widestring::WideCStr;
use winapi::_core::sync::atomic::AtomicBool;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::ops::Range;
use winapi::um::winuser::VK_ESCAPE;

pub const FONT: Font = Font::ChaletLondon;
pub const CONSOLE_WIDTH: f32 = BASE_WIDTH;
pub const CONSOLE_HEIGHT: f32 = BASE_HEIGHT / 3.0;
pub const INPUT_HEIGHT: f32 = 20.0;
pub const LINES_PER_PAGE: usize = 16;

pub const PAGE_COLOR: Rgba = Rgba::WHITE;
pub const OUTPUT_COLOR: Rgba = Rgba::WHITE;
pub const BACKGROUND_COLOR: Rgba = Rgba::new(0, 0, 0, 127);

static OPEN: AtomicBool = AtomicBool::new(false);

pub struct ScriptConsole {
    current_page: usize,
    line_history: Vec<String>,
    last_closed: Instant,
    input: TextInput
}

impl Script for ScriptConsole {
    fn prepare(&mut self, mut env: ScriptEnv) {}

    fn frame(&mut self, mut env: ScriptEnv) {
        if is_open() {
            self.lock_controls();
            self.draw();
        } else if self.get_last_closed() > Instant::now() {
            self.lock_controls();
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        let open = is_open();
        match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(event) => {
                        match event {
                            KeyboardEvent::Key { key, alt, shift, control, is_up, .. } if *is_up => {
                                const VK_KEY_T: i32 = 0x54;
                                match *key {
                                    VK_ESCAPE if open => {
                                        self.set_open(false);
                                        return false;
                                    },
                                    VK_KEY_T if !open && !crate::game::ui::is_cursor_active_this_frame() => {
                                        self.set_open(true);
                                        return false;
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                if open {
                    if let Some(input) = self.input.input(event) {
                        output.push_back(ScriptEvent::ConsoleInput(input));
                    }
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
            current_page: 1,
            line_history: Vec::new(),
            last_closed: Instant::now(),
            input: TextInput::new(String::new(), CONSOLE_WIDTH, INPUT_HEIGHT, FONT)
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

        let scale = Vector2::new(0.35, 0.35);
        // Draw background
        draw_rect([0.0, 0.0], [CONSOLE_WIDTH, CONSOLE_HEIGHT], BACKGROUND_COLOR);
        let total_pages = ((self.line_history.len() + (LINES_PER_PAGE - 1)) / LINES_PER_PAGE).max(1);
        draw_text(format!("Page {}/{}", self.current_page, total_pages), [5.0, CONSOLE_HEIGHT + INPUT_HEIGHT], PAGE_COLOR, FONT, scale);

        self.input.draw(0.0, CONSOLE_HEIGHT);

        // Draw console history text
        let history_offset = self.line_history.len() as i32 - (LINES_PER_PAGE * self.current_page) as i32;
        let history_length = (history_offset + LINES_PER_PAGE as i32) as usize;
        for i in (history_offset.max(0) as usize)..history_length {
            draw_text(&self.line_history[i], [2.0, ((i as i32 - history_offset) * 14) as f32], OUTPUT_COLOR, FONT, scale);
        }
    }
}