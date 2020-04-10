#![feature(asm, core_intrinsics, link_llvm_intrinsics, abi_thiscall)]

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
use backtrace::{Backtrace, BacktraceFmt, BacktraceFrame, SymbolName};
use winapi::shared::minwindef::{HINSTANCE, LPVOID, BOOL, TRUE, MAX_PATH};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibrary, GetModuleFileNameW};
use colored::{Color, Colorize};
use fern::colors::ColoredLevelConfig;
use fern::Dispatch;
use log::{info, debug, error};
use std::sync::atomic::{AtomicPtr, Ordering};
use crate::hash::Hashable;
use winapi::um::errhandlingapi::AddVectoredExceptionHandler;
use winapi::um::winnt::{EXCEPTION_POINTERS, LONG, MEMORY_BASIC_INFORMATION, MAX_PACKAGE_NAME};
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::ctypes::c_void;
use winapi::um::memoryapi::VirtualQuery;
use std::ffi::{CStr, CString};

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
#[cfg(target_os = "windows")]
pub mod console;

#[repr(u32)]
#[derive(Debug)]
pub enum DllCallReason {
    ProcessDetach, ProcessAttach, ThreadAttach, ThreadDetach
}

pub const LOG_ROOT: &'static str = "root";
pub const LOG_PANIC: &'static str = "panic";

//bind_fn!(RUN_INIT_STATE, "32 DB EB 02 B3 01 E8 ? ? ? ? 48 8B", 6, "C", fn() -> ());
//bind_fn!(SKIP_INIT, "32 DB EB 02 B3 01 E8 ? ? ? ? 48 8B", -9, "C", fn(u32) -> bool);

/*extern "C" fn run_init_state() {
    RUN_INIT_STATE()
}*/

/*extern "C" fn skip_init(stage: u32) -> bool {
    info!("skipping init {}", stage);
    SKIP_INIT(stage)
}*/

bind_field_ip!(INIT_STATE, "BA 07 00 00 00 8D 41 FC 83 F8 01", 2, u32);
bind_field_ip!(DIGITAL_DISTRIBUTION, "BA 07 00 00 00 8D 41 FC 83 F8 01", -26, bool);
bind_field_ip!(GAME_STATE, "83 3D ? ? ? ? ? 8A D9 74 0A", 2, GameState, 5);
bind_field_ip!(HEAP_SIZE, "83 C8 01 48 8D 0D ? ? ? ? 41 B1 01 45 33 C0", 17, u32);

unsafe fn print_address_info(addr: *mut c_void, line: u32, filename: Option<&Path>, symbol_name: SymbolName) {
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    let size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    if VirtualQuery(addr, &mut mbi, size) == size {
        let mut name = [0; MAX_PATH];
        let len = GetModuleFileNameW(mbi.AllocationBase.cast(), name.as_mut_ptr(), MAX_PATH as u32);
        if len != 0 {
            let name = widestring::WideCStr::from_ptr_with_nul(name.as_ptr(), len as usize).to_string_lossy();
            let offset = addr as u64 - mbi.AllocationBase as u64;
            if let Some(filename) = filename {
                debug!(target: LOG_PANIC, " at '{}' + 0x{:X} ({} at {} line {})", name, offset, filename.display(), line, symbol_name)
            } else {
                debug!(target: LOG_PANIC, " at '{}' + 0x{:X} ({})", name, offset, symbol_name)
            }
        }
    }
}

extern "system" fn except(info: *mut EXCEPTION_POINTERS) -> LONG {
    unsafe {
        let info = &mut *info;
        let rec = &mut *info.ExceptionRecord;
        let addr = rec.ExceptionAddress;
        let code = rec.ExceptionCode;
        //if code != 0x40010006 /*Debugger shit*/ && code != 0xE06D7363 /*NVIDIA shit*/ {
            error!(target: LOG_PANIC, "Unhandled exception occurred at address {:p} (code: 0x{:X})", addr, code);
            let backtrace = Backtrace::new();

            for frame in backtrace.frames().iter()/*.skip_while(|f| f.symbol_address() != addr)*/ {
                for symbol in frame.symbols() {
                    let name = symbol.name().unwrap_or(SymbolName::new(b"<unknown>\0"));
                    let addr = symbol.addr().unwrap_or(std::ptr::null_mut());
                    let line = symbol.lineno().unwrap_or(0);
                    print_address_info(addr, line, symbol.filename(), name);
                }
            }
            //1 //EXCEPTION_EXECUTE_HANDLER
        //} else {
            0 //EXCEPTION_CONTINUE_SEARCH
        //}
    }
}

#[cfg(target_os = "windows")]
fn attach(instance: HINSTANCE) {
    unsafe {
        std::thread::spawn(move || {
            info!("Injection successful");

            if AddVectoredExceptionHandler(0, Some(except)).is_null() {
                panic!("Unable to set exception handler");
            }

            let mem = MemoryRegion::image();

            //mem.find("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").next().expect("launcher").nop(21); //Disable launcher check
            mem.find_str("platform:/movies").next() //platform:/movies/rockstar_logos.bik
                .expect("movie").nop(16); //Disable movie

            mem.find("70 6C 61 74 66 6F 72 6D 3A").next()
                .expect("logos").write_bytes(&[0xC3]);

            mem.find("72 1F E8 ? ? ? ? 8B 0D").next()
                .expect("legals").nop(2);

            lazy_static::initialize(&GAME_STATE);

            //*HEAP_SIZE.as_mut() = 650 * 1024 * 1024; //Increase heap size to 650MB

            native::fs::pre_init(&mem);

            let input = InputHook::new();

            info!("Input hooked.");

            native::pre_init();

            info!("Waiting for game being loaded...");

            while !GAME_STATE.is_loaded() {
                std::thread::sleep(Duration::from_millis(50));
            }

            console::attach();

            info!("Initializing game hooks");

            game::init();

            info!("Initializing natives");

            native::init();

            info!("Starting runtime");

            runtime::start(input);

            info!("Waiting for game being initialized");

            while native::script::is_thread_pool_empty() {
                std::thread::sleep(Duration::from_millis(50));
            }
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
pub extern "stdcall" fn DllMain(instance: HINSTANCE, reason: DllCallReason, reserved: LPVOID) -> BOOL {
    match reason {
        DllCallReason::ProcessAttach => {
            unsafe { DisableThreadLibraryCalls(instance) };
            setup_logger("client", true);
            attach(instance)
        },
        DllCallReason::ProcessDetach => {
            unsafe { FreeLibrary(instance) };
        }
        other => {
            //crate::info!("{:?}: {:?}", other, unsafe { GetCurrentThreadId() })
        }
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
    if !is_ansi_supported() || prefix == "client" {
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