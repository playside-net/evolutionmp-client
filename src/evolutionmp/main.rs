use crate::win::ps::get_current_process;
use crate::pattern::{MemoryRegion, RegionIterator};
use crate::game::vehicle::Vehicle;
use crate::hash::joaat;
use crate::game::player::Player;
use crate::game::entity::Entity;
use crate::game::ui::{CursorSprite, LoadingPrompt};
use crate::win::input::{InputHook, KeyboardEvent};
use crate::game::scaleform::Scaleform;
use crate::runtime::Script;
use crate::game::GameState;
use std::ptr::null_mut;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::File;
use std::env::current_dir;
use std::collections::HashMap;
use std::time::Instant;
use std::panic::PanicInfo;
use std::io::stdout;
use backtrace::Backtrace;
use winapi::shared::minwindef::{HINSTANCE, DWORD, LPVOID, BOOL, TRUE, HMODULE};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibraryAndExitThread, GetModuleHandleW};
use winapi::ctypes::c_void;
use winapi::um::winuser::{VK_BACK, VK_NUMPAD5};
use dirs::desktop_dir;
use colored::{Color, Colorize};
use fern::colors::ColoredLevelConfig;
use fern::Dispatch;
use log::{info, debug, error};
use serde_derive::{Serialize, Deserialize};

#[cfg(target_os = "windows")]
pub mod win;
#[cfg(target_os = "windows")]
pub mod native;
#[cfg(target_os = "windows")]
pub mod runtime;
#[cfg(target_os = "windows")]
pub mod mappings;
#[cfg(target_os = "windows")]
pub mod game;
#[cfg(target_os = "windows")]
pub mod pattern;
#[cfg(target_os = "windows")]
pub mod process;
#[cfg(target_os = "windows")]
pub mod registry;
#[cfg(target_os = "windows")]
pub mod multiplayer;

pub mod network;
pub mod hash;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_THREAD_ATTACH: u32 = 2;
const DLL_THREAD_DETACH: u32 = 3;
const DLL_PROCESS_DETACH: u32 = 0;

pub const LOG_ROOT: &'static str = "root";
pub const LOG_PANIC: &'static str = "panic";

#[cfg(target_os = "windows")]
fn attach(instance: HMODULE) {
    unsafe {
        info!("Injection successful");

        let mem = MemoryRegion::image();

        //mem.find("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").next().expect("launcher").nop(21); //Disable launcher check
        mem.find_await("70 6C 61 74 66 6F 72 6D 3A 2F 6D 6F 76", 50, 1000)
            .expect("movie").nop(13); //Disable movie

        let game_state = mem.find_await("83 3D ? ? ? ? ? 8A D9 74 0A", 50, 1000)
            .expect("game state")
            .add(2)
            .read_ptr(5);

        let get_game_state = move || {
            *game_state.get::<GameState>()
        };

        std::thread::spawn(move || {
            let mut input = win::input::InputHook::new().expect("Input hooking failed");
            info!("Input hooked. Waiting for game being loaded...");

            while !get_game_state().is_loaded() {
                std::thread::sleep(Duration::from_millis(50));
            }

            info!("Initializing natives");

            native::init(&mem);

            info!("Natives initialized. Waiting for game being started...");

            while get_game_state() != GameState::Playing {
                game::ui::show_loading_prompt(LoadingPrompt::LoadingRight, "Loading Evolution MP");
                std::thread::sleep(Duration::from_millis(50));
            }

            info!("Game started. Starting runtime...");

            runtime::start(&mem, input);
        });
    }
}

#[cfg(target_os = "windows")]
fn detach() {

}

#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! error_message {
    ($caption:expr,$($arg:tt)*) => {
        use crate::win::user::*;
        unsafe { message_box(None, format!($($arg)*), $caption, MessageBoxButtons::Ok, Some(MessageBoxIcon::Error)) };
    };
}

#[cfg(target_os = "windows")]
#[macro_export]
macro_rules! info_message {
    ($caption:expr,$($arg:tt)*) => {
        use crate::win::user::*;
        unsafe { message_box(None, format!($($arg)*), $caption, MessageBoxButtons::Ok, Some(MessageBoxIcon::Information)) };
    };
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(instance) };
            setup_logger("client", true);
            attach(instance)
        }
        DLL_PROCESS_DETACH => {
            detach();
        }
        _ => {},
    }
    TRUE
}

pub fn launcher_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Missing home directory");
    let launcher_dir = home_dir.join(".evolutionmp");
    if !launcher_dir.exists() {
        std::fs::create_dir(&launcher_dir).expect("Directory creation failed");
    }
    launcher_dir
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

#[cfg(windows)]
#[inline]
fn is_ansi_supported() -> bool {
    ansi_term::enable_ansi_support().is_ok()
}

#[cfg(not(windows))]
#[inline]
fn is_ansi_supported() -> bool {
    true
}

pub fn setup_logger(prefix: &str, debug: bool) {
    if !is_ansi_supported() {
        colored::control::set_override(false);
    }

    let colors = ColoredLevelConfig::new()
        .info(Color::Blue)
        .warn(Color::Yellow)
        .error(Color::Red)
        .debug(Color::BrightBlue);

    Dispatch::new()
        .format(move |out, message, record| {
            let time = chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]");
            match record.target() {
                LOG_ROOT => {
                    let level = format!("{}", colors.color(record.level()));
                    out.finish(format_args!(
                        "{}[{}] {}",
                        time,
                        (&*level).bold(),
                        message
                    ))
                },
                LOG_PANIC => {
                    let message = format!("{}", message);
                    out.finish(format_args!(
                        "{} {}",
                        time,
                        (&*message).red()
                    ))
                },
                _ => {
                    let level = format!("{}", colors.color(record.level()));
                    out.finish(format_args!(
                        "{}[{}][{}] {}",
                        time,
                        record.target(),
                        (&*level).bold(),
                        message
                    ))
                }
            }
        })
        .level(if debug { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .chain(fern::log_file(launcher_dir().join("latest.log")).unwrap())
        .chain(stdout())
        .apply().expect("Logger setup failed");

    std::panic::set_hook(Box::new(|info: &PanicInfo| {
        let backtrace = Backtrace::new();

        let thread = std::thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let reason = self::downcast_str(info.payload());

        let location = match info.location() {
            Some(location) => format!(": {}:{}", location.file(), location.line()),
            None => String::from("")
        };

        error!(target: LOG_PANIC, "thread '{}' panicked at '{}'{}", thread, reason, location);

        let s = format!("{:?}", backtrace);

        for line in s.lines(){
            debug!(target: LOG_PANIC, "{}", line);
        }
    }));
}