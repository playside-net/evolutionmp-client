use crate::runtime::{Script, ScriptEnv};
use crate::game::ui::{BASE_WIDTH, BASE_HEIGHT, Font, LoadingPrompt};
use crate::game::Rgba;
use crate::game::controls::{Group as ControlGroup, Control};
use crate::pattern::MemoryRegion;
use crate::win::input::{InputEvent, KeyboardEvent};
use std::time::Instant;
use std::sync::MutexGuard;
use std::collections::VecDeque;
use std::os::raw::c_int;
use winapi::um::winuser::{VK_BACK, VK_DELETE, VK_LEFT, VK_RIGHT, VK_HOME, VK_END, VK_UP, VK_DOWN, VK_ESCAPE, VK_RETURN};
use cgmath::Vector2;

pub const CONSOLE_WIDTH: f32 = BASE_WIDTH;
pub const CONSOLE_HEIGHT: f32 = BASE_HEIGHT / 3.0;
pub const INPUT_HEIGHT: f32 = 20.0;
pub const LINES_PER_PAGE: usize = 16;

pub const INPUT_COLOR: Rgba = Rgba::WHITE;
pub const INPUT_COLOR_BUSY: Rgba = Rgba::DARK_GRAY;
pub const OUTPUT_COLOR: Rgba = Rgba::WHITE;
pub const PREFIX_COLOR: Rgba = Rgba::new(52, 152, 219, 255);
pub const BACKGROUND_COLOR: Rgba = Rgba::BLACK;
pub const ALT_BACKGROUND_COLOR: Rgba = Rgba::new(52, 73, 94, 200);

pub unsafe fn init(mem: &MemoryRegion) {
    crate::runtime::register_script("console", ScriptConsole {
        cursor_pos: 0,
        command_pos: -1,
        current_page: 1,
        input: String::new(),
        line_history: Vec::new(),
        command_history: Vec::new()
    });
}

pub struct ScriptConsole {
    cursor_pos: usize,
    command_pos: i32,
    current_page: usize,
    input: String,
    line_history: Vec<String>,
    command_history: Vec<String>
}

impl Script for ScriptConsole {
    fn prepare(&mut self, mut env: ScriptEnv) {

    }

    fn frame(&mut self, mut env: ScriptEnv) {
        self.line_history.extend(self.take_lines().into_iter());
        crate::game::ui::show_loading_prompt(LoadingPrompt::LoadingLeft3, &format!("Input: {}", self.input));
        if self.is_open() {
            self.lock_controls();
            self.draw();
        } else if self.get_last_closed() > Instant::now() {
            self.lock_controls();
        }
    }

    fn input(&mut self, mut env: ScriptEnv, event: InputEvent, time_caught: Instant) {
        match event {
            InputEvent::Keyboard(event) => {
                match event {
                    KeyboardEvent::Key { key, alt, shift, control, .. } => {
                        const VK_KEY_C: c_int = 0x43;
                        const VK_KEY_X: c_int = 0x58;
                        const VK_KEY_V: c_int = 0x56;
                        const VK_KEY_T: c_int = 0x54;

                        match key {
                            VK_BACK => {
                                if self.input.len() > 0 && self.cursor_pos > 0 {
                                    self.input.remove(self.cursor_pos - 1);
                                    self.cursor_pos -= 1;
                                }
                            },
                            VK_DELETE => {
                                self.is_open();
                                if self.input.len() > 0 && self.cursor_pos < self.input.len() {
                                    self.input.remove(self.cursor_pos);
                                }
                            },
                            VK_LEFT => {
                                if control {

                                } else {

                                }
                            },
                            VK_RIGHT => {
                                if control {

                                } else {

                                }
                            },
                            VK_HOME => {

                            },
                            VK_END => {

                            },
                            VK_UP => {

                            },
                            VK_DOWN => {

                            },
                            VK_ESCAPE => {
                                if self.is_open() {
                                   self.set_open(false);
                                }
                            },
                            VK_RETURN => {

                            },
                            VK_KEY_C if control => {

                            },
                            VK_KEY_X if control => {

                            },
                            VK_KEY_V if control => {

                            },
                            VK_KEY_T => {
                                if !self.is_open() {
                                    self.set_open(true);
                                }
                            }
                            _ => {}
                        }
                    },
                    KeyboardEvent::Char(c) => {
                        if !c.is_control() {
                            self.input.push(c);
                            self.cursor_pos += 1;
                        }
                    }
                }
            },
            _ => {}
        }
    }
}

impl ScriptConsole {
    fn take_lines(&self) -> Vec<String> {
        unsafe { crate::runtime::CONSOLE.as_ref().expect("Missing console").take_lines() }
    }

    fn is_open(&self) -> bool {
        unsafe { crate::runtime::CONSOLE.as_ref().expect("Missing console").is_open() }
    }

    fn set_open(&self, open: bool) {
        unsafe { crate::runtime::CONSOLE.as_mut().expect("Missing console").set_open(open) };
        if !open {
            self.lock_controls();
        }
    }

    fn get_last_closed(&self) -> Instant {
        unsafe { crate::runtime::CONSOLE.as_ref().expect("Missing console").get_last_closed() }
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
        let font = Font::ChaletLondon;
        let scale = Vector2::new(0.35, 0.35);
        // Draw background
        draw_rect([0.0, 0.0], [CONSOLE_WIDTH, CONSOLE_HEIGHT], BACKGROUND_COLOR);
        // Draw input field
        draw_rect([0.0, CONSOLE_HEIGHT], [CONSOLE_WIDTH, INPUT_HEIGHT], ALT_BACKGROUND_COLOR);
        draw_rect([0.0, CONSOLE_HEIGHT + INPUT_HEIGHT], [80.0, INPUT_HEIGHT], ALT_BACKGROUND_COLOR);
        // Draw input prefix
        draw_text(">", [0.0, CONSOLE_HEIGHT], PREFIX_COLOR, font, scale);
        // Draw input text
        draw_text(&self.input, [25.0, CONSOLE_HEIGHT], INPUT_COLOR, font, scale);
        // Draw page information
        let total_pages = ((self.line_history.len() + (LINES_PER_PAGE - 1)) / LINES_PER_PAGE).max(1);
        draw_text(format!("Page {}/{}", self.current_page, total_pages), [5.0, CONSOLE_HEIGHT + INPUT_HEIGHT], INPUT_COLOR, font, scale);

        // Draw blinking cursor
        if now.elapsed().subsec_millis() < 500 {
            let length = get_text_width(&self.input[0..self.cursor_pos], font, scale);
            draw_text("~w~~h~|~w~", [25.0 + (length * CONSOLE_WIDTH) - 4.0, CONSOLE_HEIGHT], INPUT_COLOR, font, scale);
        }

        // Draw console history text
        let history_offset = self.line_history.len() as i32 - (LINES_PER_PAGE * self.current_page) as i32;
        let history_length = (history_offset + LINES_PER_PAGE as i32) as usize;
        for i in (history_offset.max(0) as usize)..history_length {
            draw_text(&self.line_history[i], [2.0, ((i as i32 - history_offset) * 14) as f32], OUTPUT_COLOR, font, scale);
        }
    }
}