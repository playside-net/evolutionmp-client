#![feature(asm, set_stdio, core_intrinsics)]

#[macro_use]
extern crate lazy_static;
extern crate backtrace;

use crate::pattern::MemoryRegion;
use crate::game::entity::Entity;
use crate::win::input::InputHook;
use crate::game::GameState;
use std::ptr::null_mut;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::panic::PanicInfo;
use std::io::stdout;
use backtrace::Backtrace;
use winapi::shared::minwindef::{HINSTANCE, LPVOID, BOOL, TRUE};
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use colored::{Color, Colorize};
use fern::colors::ColoredLevelConfig;
use fern::Dispatch;
use log::{info, debug, error};
use std::sync::atomic::{AtomicPtr, Ordering};
use crate::hash::Hashable;
use winapi::_core::ops::RangeInclusive;

#[cfg(target_os = "windows")]
pub mod win;
#[cfg(target_os = "windows")]
pub mod native;
#[cfg(target_os = "windows")]
pub mod runtime;
#[cfg(target_os = "windows")]
pub mod events;
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
pub mod scripts;

pub mod network;
pub mod hash;

#[repr(C)]
pub enum DllCallReason {
    ProcessDetach, ProcessAttach, ThreadAttach, ThreadDetach
}

pub const LOG_ROOT: &'static str = "root";
pub const LOG_PANIC: &'static str = "panic";

static GAME_STATE: AtomicPtr<GameState> = AtomicPtr::new(null_mut());

fn get_game_state() -> GameState {
    unsafe { GAME_STATE.load(Ordering::SeqCst).read() }
}

type RunInitState = extern "C" fn();
static mut RUN_INIT_STATE: *const () = std::ptr::null();

unsafe extern "C" fn run_init_state() {
    let origin: RunInitState = std::mem::transmute(RUN_INIT_STATE);
    origin()
}

type SkipInit = extern "C" fn(u32) -> bool;
static mut SKIP_INIT: *const () = std::ptr::null();

unsafe extern "C" fn skip_init(stage: u32) -> bool {
    let origin: SkipInit = std::mem::transmute(SKIP_INIT);
    info!("skipping init {}", stage);
    origin(stage)
}

static mut INIT_STATE: *mut u32 = std::ptr::null_mut();
static mut DIGITAL_DISTRIBUTION: bool = false;

#[cfg(target_os = "windows")]
fn attach(instance: HINSTANCE) {
    unsafe {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));

            info!("Injection successful");

            let mem = MemoryRegion::image();

            //mem.find("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").next().expect("launcher").nop(21); //Disable launcher check
            mem.find("70 6C 61 74 66 6F 72 6D 3A 2F 6D 6F 76").next()
                .expect("movie").nop(13); //Disable movie

            mem.find("72 1F E8 ? ? ? ? 8B 0D").next()
                .expect("legals").nop(2);

            GAME_STATE.store(mem.find("83 3D ? ? ? ? ? 8A D9 74 0A").next()
                .expect("game state")
                .add(2)
                .read_ptr(5).get_mut(), Ordering::SeqCst);

            let r = mem.find("32 DB EB 02 B3 01 E8 ? ? ? ? 48 8B")
                .next().expect("run_init_state");
            RUN_INIT_STATE = r.add(6).detour(run_init_state as _);
            //SKIP_INIT = r.offset(-9).detour(skip_init as _);
            let s = mem.find("BA 07 00 00 00 8D 41 FC 83 F8 01")
                .next().expect("init state");
            INIT_STATE = s.add(2).read_ptr(4).get_mut();
            DIGITAL_DISTRIBUTION = s.offset(-26).get::<u8>().read() == 3;
            info!("Digital distribution: {}", DIGITAL_DISTRIBUTION);

            let input = InputHook::new(&mem);
            info!("Input hooked. Waiting for game being loaded...");

            while !get_game_state().is_loaded() {
                std::thread::sleep(Duration::from_millis(50));
            }

            info!("Initializing game hooks");

            game::init(&mem);

            info!("Initializing natives");

            native::init(&mem);

            info!("Waiting for game being started...");

            while get_game_state() != GameState::Playing {
                std::thread::sleep(Duration::from_millis(50));
            }

            info!("Starting runtime");

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

#[no_mangle]
pub extern fn set_io(print: Option<Box<dyn Write + Send>>, panic: Option<Box<dyn Write + Send>>) {
    std::io::set_print(print);
    std::io::set_panic(panic);
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HINSTANCE, reason: DllCallReason, reserved: LPVOID) -> BOOL {
    std::io::stdout();
    match reason {
        DllCallReason::ProcessAttach => {
            unsafe { DisableThreadLibraryCalls(instance) };
            setup_logger("client", true);
            attach(instance)
        }
        DllCallReason::ProcessDetach => {
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
        .chain(fern::log_file(launcher_dir().join(&format!("{}.log", prefix))).unwrap())
        .chain(stdout())
        .apply().expect("Logger setup failed");

    std::panic::set_hook(Box::new(|info: &PanicInfo| {
        let backtrace = Backtrace::new();

        let thread = std::thread::current();
        let thread = thread.name().unwrap_or("unnamed");

        let reason = self::downcast_str(info.payload());

        let location = match info.location() {
            Some(location) => format!(": {}:{}:{}", location.file(), location.line(), location.column()),
            None => String::from("")
        };

        error!(target: LOG_PANIC, "thread '{}' panicked at '{}'{}", thread, reason, location);

        let s = format!("{:?}", backtrace);

        for line in s.lines(){
            debug!(target: LOG_PANIC, "{}", line);
        }
    }));
}