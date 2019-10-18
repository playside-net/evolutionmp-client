extern crate winapi;
extern crate dirs;
extern crate libc;
extern crate widestring;

use winapi::shared::minwindef::{HINSTANCE, DWORD, LPVOID, BOOL, TRUE, HMODULE};
use winapi::um::processthreadsapi::{GetCurrentProcess, CreateThread};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibraryAndExitThread};
use crate::win::user::{MessageBoxButtons, MessageBoxIcon, message_box};
use crate::win::ps::get_current_process;
use std::ptr::null_mut;
use ntapi::winapi::_core::time::Duration;
use crate::pattern::{Region, RegionIterator};
use winapi::um::winuser::{MessageBeep, MB_ICONSTOP, VK_RETURN, VK_NUMPAD5, VK_TAB, VK_SPACE};
use ntapi::winapi::_core::panic::PanicInfo;
use winapi::um::tlhelp32::TH32CS_SNAPMODULE;
use winapi::ctypes::c_void;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use ntapi::winapi::_core::cell::UnsafeCell;
use std::sync::Mutex;
use ntapi::winapi::_core::mem::MaybeUninit;
use crate::GameState::MainMenu;
use crate::natives::NATIVES;
use std::env::current_dir;
use dirs::desktop_dir;
use std::collections::HashMap;
use crate::game::vehicle::Vehicle;
use crate::hash::joaat;
use crate::game::player::Player;
use crate::game::entity::Entity;
use crate::game::ui::{CursorSprite, LoadingPrompt};
use crate::win::input::{InputHook, KeyEvent};

pub mod win;
pub mod hash;
pub mod natives;
pub mod mappings;
pub mod game;
pub mod pattern;
pub mod process;
pub mod registry;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_THREAD_ATTACH: u32 = 2;
const DLL_THREAD_DETACH: u32 = 3;
const DLL_PROCESS_DETACH: u32 = 0;

#[derive(Clone, Debug, PartialEq)]
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
        let global_region = Region::get_global();

        //global_region.find("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").next().expect("launcher").nop(21); //Disable launcher check
        //global_region.find("72 1F E8 ? ? ? ? 8B 0D").next().expect("legals").nop(2); //Disable legals
        global_region.find("70 6C 61 74 66 6F 72 6D 3A 2F 6D 6F 76")
            .next().expect("movie").nop(13); //Disable movie
        global_region.find("72 6F 63 6B 73 74 61 72 5F 6C 6F 67 6F 73 00 62 69 6B")
            .next().expect("news").replace("32 73 65 63 6F 6E 64 73 62 6C 61 63 6B 2E 62 69 6B 00");

        natives::init(&global_region);

        std::thread::spawn(move || {
            let mut input = win::input::InputHook::new().expect("Input hooking failed");
            let mut game_state = global_region.find("83 3D ? ? ? ? ? 8A D9 74 0A")
                .next().expect("game state").add(2).rip(5).get::<GameState>();

            while *game_state != GameState::MainMenu {
                std::thread::sleep(Duration::from_millis(50));
            }

            loop {
                while let Some(event) = input.next_event() {
                    if !event.is_up {
                        match event.key {
                            VK_SPACE => {
                                let ped = game::ped::Ped::local();
                                game::ui::show_loading_prompt(LoadingPrompt::LoadingRight, &format!("Привет, {}", ped.get_handle()));
                                let pos = ped.get_position();
                                let heading = ped.get_heading();
                                Vehicle::new(0x91CA96EE, pos, heading, false, false);
                            },
                            VK_TAB => {
                                let ped = game::ped::Ped::local();
                                let veh = ped.is_in_any_vehicle(false);
                                game::ui::show_subtitle(&format!("Vehicle: {}", veh), 50, true)
                            }
                            _ => {}
                        }
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