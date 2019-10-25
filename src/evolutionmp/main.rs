#![feature(abi_thiscall)]
#![feature(maybe_uninit_ref)]

extern crate winapi;
extern crate ntapi;
#[macro_use]
extern crate detour;
extern crate dirs;
extern crate libc;
extern crate widestring;
#[macro_use]
extern crate field_offset;

use winapi::shared::minwindef::{HINSTANCE, DWORD, LPVOID, BOOL, TRUE, HMODULE};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibraryAndExitThread};
use crate::win::user::{MessageBoxButtons, MessageBoxIcon, message_box};
use crate::win::ps::get_current_process;
use std::ptr::null_mut;
use ntapi::winapi::_core::time::Duration;
use crate::pattern::{MemoryRegion, RegionIterator};
use ntapi::winapi::_core::panic::PanicInfo;
use winapi::um::tlhelp32::TH32CS_SNAPMODULE;
use winapi::ctypes::c_void;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::env::current_dir;
use dirs::desktop_dir;
use std::collections::HashMap;
use crate::game::vehicle::Vehicle;
use crate::hash::joaat;
use crate::game::player::Player;
use crate::game::entity::Entity;
use crate::game::ui::{CursorSprite, LoadingPrompt};
use crate::win::input::{InputHook, KeyEvent};
use crate::game::scaleform::Scaleform;
use crate::script::{Script, Wait};
use std::time::Instant;
use winapi::um::winuser::{VK_BACK, VK_NUMPAD5};

pub mod win;
pub mod hash;
pub mod native;
pub mod script;
pub mod mappings;
pub mod game;
pub mod pattern;
pub mod process;
pub mod registry;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_THREAD_ATTACH: u32 = 2;
const DLL_THREAD_DETACH: u32 = 3;
const DLL_PROCESS_DETACH: u32 = 0;

pub(crate) static mut GAME_STATE: *const GameState = std::ptr::null_mut();

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub enum GameState {
    Playing,
    Intro,
    Legals = 3,
    MainMenu = 5,
    LoadingSpMp = 6
}

fn attach(instance: HMODULE) {
    unsafe {
        DisableThreadLibraryCalls(instance);
        let mem = MemoryRegion::image();

        //global_region.find("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").next().expect("launcher").nop(21); //Disable launcher check
        //global_region.find("72 1F E8 ? ? ? ? 8B 0D").next().expect("legals").nop(2); //Disable legals
        mem.find_first_await("70 6C 61 74 66 6F 72 6D 3A 2F 6D 6F 76", 50, 1000)
            .expect("movie").nop(13); //Disable movie
        /*mem.find("72 6F 63 6B 73 74 61 72 5F 6C 6F 67 6F 73 00 62 69 6B")
            .next().expect("news").replace("32 73 65 63 6F 6E 64 73 62 6C 61 63 6B 2E 62 69 6B 00");*/

        native::init(&mem);

        std::thread::spawn(move || {
            let mut input = win::input::InputHook::new().expect("Input hooking failed");

            GAME_STATE = mem.find_first_await("83 3D ? ? ? ? ? 8A D9 74 0A", 50, 1000)
                    .expect("game state")
                    .add(2)
                    .read_ptr(5)
                    .get::<GameState>();

            while *GAME_STATE != GameState::MainMenu {
                std::thread::sleep(Duration::from_millis(50));
            }
            script::init(&mem);
            info!("Scripts initialized", "Info");
            script::register(ScriptTest {});

            loop {
                while let Some(event) = input.next_event() {
                    for s in &mut script::SCRIPTS {
                        s.key(event);
                    }
                }
            }
        });
    }
}

fn detach() {

}

#[macro_export]
macro_rules! error {
    ($text:expr,$caption:expr) => {
        use crate::win::user::*;
        unsafe { message_box(None, $text, $caption, MessageBoxButtons::Ok, Some(MessageBoxIcon::Error)) };
    };
}

#[macro_export]
macro_rules! info {
    ($text:expr,$caption:expr) => {
        use crate::win::user::*;
        unsafe { message_box(None, $text, $caption, MessageBoxButtons::Ok, Some(MessageBoxIcon::Information)) };
    };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::panic::set_hook(Box::new(|info: &PanicInfo| {
                //let backtrace = Backtrace::new();

                let thread = std::thread::current();
                let thread = thread.name().unwrap_or("unnamed");

                let reason = self::downcast_str(info.payload());

                let location = match info.location() {
                    Some(location) => format!(": {}:{}", location.file(), location.line()).replace("\\", "/"),
                    None => String::from("")
                };

                let message = format!("thread '{}' panicked at '{}'{}", thread, reason, location);

                error!(message, "EvolutionMP Error");

                /*let s = format!("{:?}", backtrace);

                for line in s.lines(){
                    debug!(target: LOG_PANIC, "{}", line);
                }*/
            }));
            attach(instance)
        }
        DLL_PROCESS_DETACH => {
            detach();
        }
        _ => {},
    }
    TRUE
}

pub fn downcast_str(string: &(dyn std::any::Any + Send)) -> &str  {
    match string.downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => {
            match string.downcast_ref::<String>() {
                Some(s) => &**s,
                None => {
                    "Box<Any>"
                }
            }
        }
    }
}

pub struct ScriptTest {

}

impl Script for ScriptTest {
    fn tick(&mut self, wait: &Wait, delta_time: f64) {

    }

    fn render(&self, game_state: GameState) {

    }

    fn on_key(&mut self, key: KeyEvent, time_caught: Instant) {
        if key.key == VK_NUMPAD5 {
            let handle = unsafe { native::player::get_local_handle() };
            game::ui::show_loading_prompt(LoadingPrompt::LoadingRight, &format!("Kek {}", handle));
        }
    }
}