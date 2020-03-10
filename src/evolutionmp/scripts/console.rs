use crate::runtime::{Script, ScriptEnv, Runtime, TaskQueue};
use crate::game::ui::{BASE_WIDTH, BASE_HEIGHT, Font, TextInput};
use crate::game::{Rgba, GameState};
use crate::game::controls::{Group as ControlGroup, Control};
use crate::win::input::{InputEvent, KeyboardEvent};
use crate::events::{ScriptEvent, NativeEvent};
use std::time::{Instant, SystemTime};
use std::collections::VecDeque;
use cgmath::{Vector2, Array};
use std::time::Duration;
use std::ffi::CString;
use widestring::WideCStr;
use std::sync::atomic::AtomicBool;
use clipboard::{ClipboardContext, ClipboardProvider};
use std::ops::Range;
use winapi::um::winuser::VK_ESCAPE;
use std::cell::RefCell;
use crate::native::ThreadSafe;

pub const FONT: Font = Font::ChaletLondon;
pub const CONSOLE_WIDTH: f32 = BASE_WIDTH;
pub const CONSOLE_HEIGHT: f32 = BASE_HEIGHT / 3.0;
pub const INPUT_HEIGHT: f32 = 20.0;
pub const LINES_PER_PAGE: usize = 16;

pub const PAGE_COLOR: Rgba = Rgba::WHITE;
pub const OUTPUT_COLOR: Rgba = Rgba::WHITE;
pub const BACKGROUND_COLOR: Rgba = Rgba::new(0, 0, 0, 127);

static OPEN: AtomicBool = AtomicBool::new(false);
static LAST_CLOSED: ThreadSafe<RefCell<SystemTime>> = ThreadSafe::new(RefCell::new(SystemTime::UNIX_EPOCH));

pub struct ScriptConsole {
    current_page: usize,
    line_history: Vec<String>,
    input: TextInput,
    tasks: TaskQueue
}

impl Script for ScriptConsole {
    fn prepare(&mut self, mut env: ScriptEnv) {}

    fn frame(&mut self, mut env: ScriptEnv, game_state: GameState) {
        if game_state == GameState::Playing {
            self.tasks.process(&mut env);
            if is_open() {
                lock_controls();
                self.draw();
            } else if get_last_closed() > SystemTime::now() {
                lock_controls();
            }
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        let open = is_open();
        match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(event) => {
                        match event {
                            KeyboardEvent::Key { key, alt, shift, control, is_up, .. } if !*is_up => {
                                const VK_KEY_T: i32 = 0x54;
                                match *key {
                                    VK_ESCAPE if open => {
                                        self.tasks.push(|env| set_open(false));
                                        return false;
                                    },
                                    VK_KEY_T if !open => {
                                        self.tasks.push(|env| set_open(true));
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

pub fn set_open(open: bool) {
    use std::sync::atomic::Ordering;
    OPEN.store(open, Ordering::SeqCst);
    if !open {
        LAST_CLOSED.replace(SystemTime::now() + Duration::from_millis(200));
        lock_controls();
    }
}

pub fn get_last_closed() -> SystemTime {
    *LAST_CLOSED.borrow()
}

pub fn lock_controls() {
    use crate::game::controls;
    controls::disable_all_actions(ControlGroup::Move);
    controls::enable_action(ControlGroup::Move, Control::LookLeftRight, true);
    controls::enable_action(ControlGroup::Move, Control::LookUpDown, true);
    controls::enable_action(ControlGroup::Move, Control::LookUpOnly, true);
    controls::enable_action(ControlGroup::Move, Control::LookDownOnly, true);
    controls::enable_action(ControlGroup::Move, Control::LookLeftOnly, true);
    controls::enable_action(ControlGroup::Move, Control::LookRightOnly, true);
}

impl ScriptConsole {
    pub fn new() -> ScriptConsole {
        ScriptConsole {
            current_page: 1,
            line_history: Vec::new(),
            input: TextInput::new(String::new(), CONSOLE_WIDTH, INPUT_HEIGHT, FONT),
            tasks: TaskQueue::new()
        }
    }

    fn draw(&self) {
        use crate::game::ui::{draw_rect, draw_text, get_text_width};

        let scale = Vector2::from_value(0.35);
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