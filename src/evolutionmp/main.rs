#![feature(llvm_asm, core_intrinsics, link_llvm_intrinsics, abi_thiscall)]

extern crate backtrace;
#[macro_use]
extern crate lazy_static;

use std::ffi::{CString, OsString};
use std::io::stdout;
use std::panic::PanicInfo;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use backtrace::{Backtrace, SymbolName};
use colored::{Color, Colorize};
use detour::RawDetour;
use fern::colors::ColoredLevelConfig;
use fern::Dispatch;
use log::{debug, error, info};
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, LPVOID, MAX_PATH, TRUE};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::errhandlingapi::AddVectoredExceptionHandler;
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibrary, GetModuleFileNameW, GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::VirtualQuery;
use winapi::um::winnt::{EXCEPTION_POINTERS, LONG, LPWSTR, MEMORY_BASIC_INFORMATION};
use winapi::um::winuser::{GWLP_WNDPROC, IsWindow, IsWindowVisible, SetWindowLongPtrW, WNDPROC};
use wio::wide::FromWide;

use crate::game::GameState;

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
#[cfg(target_os = "windows")]
pub mod jni;

pub mod hash;
#[cfg(target_os = "windows")]
pub mod console;

#[repr(u32)]
#[derive(Debug)]
pub enum DllCallReason {
    ProcessDetach,
    ProcessAttach,
    ThreadAttach,
    ThreadDetach,
}

pub const LOG_ROOT: &'static str = "root";
pub const LOG_PANIC: &'static str = "panic";

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
                debug!(target: LOG_PANIC, " at '{}' + 0x{:X} ({} in {}:{})", name, offset, symbol_name, filename.display(), line)
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
        if code != 0x40010006 /*Debugger shit*/ && code != 0xE06D7363 /*NVIDIA shit*/ && code != 0x406D1388 {
            let native = crate::native::CURRENT_NATIVE.load(Ordering::SeqCst);
            if native != 0 {
                error!(target: LOG_PANIC, "Error occurred while invoking native `0x{:016X}` (address: {:p}, code: 0x{:X})", native, addr, code);
            } else {
                error!(target: LOG_PANIC, "Unhandled exception occurred at address {:p} (code: 0x{:X})", addr, code);
            }
            print_address_info(addr, 0, None, SymbolName::new(b"<unknown>\0"));
            let backtrace = Backtrace::new();

            for frame in backtrace.frames().iter()/*.skip_while(|f| f.symbol_address() != addr)*/ {
                for symbol in frame.symbols() {
                    let name = symbol.name().unwrap_or(SymbolName::new(b"<unknown>\0"));
                    let addr = symbol.addr().unwrap_or(std::ptr::null_mut());
                    let line = symbol.lineno().unwrap_or(0);
                    print_address_info(addr, line, symbol.filename(), name);
                }
            }
        }
        0 //EXCEPTION_CONTINUE_SEARCH
    }
}

fn add_dll_directory(java_exe: &Path) {
    let java_libs_root = java_exe.parent().unwrap().parent().unwrap();
    let old_path = std::env::var("PATH").unwrap();
    std::env::set_var("PATH", format!("{}\\{}", old_path, java_libs_root.display()));
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Window {
    ptr: HWND
}

impl Window {
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    pub fn is_valid(&self) -> bool {
        unsafe {
            IsWindow(self.ptr) == TRUE
        }
    }

    pub fn is_visible(&self) -> bool {
        unsafe {
            IsWindowVisible(self.ptr) == TRUE
        }
    }

    pub fn set_event_processor(&self, event_processor: WNDPROC) -> WNDPROC {
        unsafe {
            std::mem::transmute(
                SetWindowLongPtrW(self.ptr, GWLP_WNDPROC, std::mem::transmute(event_processor))
            )
        }
    }
}

macro_rules! proc_detour {
    ($name:ident,$module:literal,$proc:literal,$repl:expr,$abi:literal,fn($($arg:ty),*)->$ret:ty) => {
        lazy_static::lazy_static! {
            static ref $name: extern $abi fn($($arg),*) -> $ret = unsafe {
                let module = CString::new($module).unwrap();
                let module = GetModuleHandleA(module.as_ptr());
                let proc = CString::new($proc).unwrap();
                let proc = GetProcAddress(module, proc.as_ptr());
                let detour = RawDetour::new(proc as _, $repl as _)
                    .expect(concat!("error detouring ", $proc));
                detour.enable().expect(concat!("error enabling detour for ", $proc));
                let trampoline = detour.trampoline() as *const ();
                std::mem::forget(detour);
                std::mem::transmute(trampoline)
            };
        }
    };
}

unsafe extern "system" fn create_window(ex_style: DWORD, class_name: LPWSTR, window_name: LPWSTR,
                                        style: DWORD, x: i32, y: i32, w: i32, h: i32, parent: Window,
                                        menu: HMENU, instance: HINSTANCE, param: LPVOID) -> Window {
    let window = CREATE_WINDOW(ex_style, class_name, window_name, style, x, y, w, h, parent, menu, instance, param);
    if parent.is_null() {
        let title = OsString::from_wide_ptr_null(class_name as _);
        if title == "grcWindow" {
            initialize(&window);
        }
    }
    window
}

proc_detour!(CREATE_WINDOW, "user32.dll", "CreateWindowExW", create_window, "system",
    fn(DWORD, LPWSTR, LPWSTR, DWORD, i32, i32, i32, i32, Window, HMENU, HINSTANCE, LPVOID) -> Window
);

unsafe fn initialize(window: &Window) {
    console::attach();
    info!("Hooking user input...");
    crate::win::input::hook(window);

    info!("Applying patches...");

    lazy_static::initialize(&GAME_STATE);

    //mem!("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").expect("launcher").nop(21); //Disable launcher check
    /*mem.find_str("platform:/movies").expect("movie")
        .write_bytes(b"platform:/movies/2secondsblack.bik\0"); //Disable movie*/
    mem!("70 6C 61 74 66 6F 72 6D 3A").expect("logos").write_bytes(&[0xC3]); //Disable movie
    /*mem!("72 1F E8 ? ? ? ? 8B 0D").expect("legals")
        .nop(2); //Disable legals*/
    mem!("48 83 3D ? ? ? ? 00 88 05 ? ? ? ? 75 0B").expect("force offline")
        .add(8).nop(6);
    let focus_pause = mem!("0F 95 05 ? ? ? ? E8 ? ? ? ? 48 85 C0").expect("focus pause");
    focus_pause.add(3).read_ptr(4).write_bytes(&[0]);
    focus_pause.nop(7);
    bind_field!(DEVICE_LIMIT, "C7 05 ? ? ? ? 64 00 00 00 48 8B", 6, u32);
    *DEVICE_LIMIT.as_mut() *= 5;
    mem!("C6 80 F0 00 00 00 01 E8 ? ? ? ? E8").expect("no relative device sorting")
        .add(12).nop(5);
    /*mem!("48 85 C0 0F 84 ? ? ? ? 8B 48 50").expect("unlock objects")
        .nop(24);*/

    //*HEAP_SIZE.as_mut() = 650 * 1024 * 1024; //Increase heap size to 650MB

    info!("Initializing FS");
    native::fs::pre_init();

    info!("Initializing natives");
    native::pre_init();

    info!("Initializing game hooks");
    crate::game::pre_init();
    game::init();

    info!("Initializing core scripts");
    crate::scripts::init();
}

#[cfg(target_os = "windows")]
fn attach() {
    unsafe {
        if AddVectoredExceptionHandler(0, Some(except)).is_null() {
            panic!("Unable to set exception handler");
        }
        lazy_static::initialize(&CREATE_WINDOW);
    }
}

#[cfg(target_os = "windows")]
fn detach() {
    unsafe {
        crate::win::input::unhook();
    }
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
pub extern "stdcall" fn DllMain(instance: HINSTANCE, reason: DllCallReason, _reserved: LPVOID) -> BOOL {
    match reason {
        DllCallReason::ProcessAttach => {
            unsafe { DisableThreadLibraryCalls(instance) };
            setup_logger("client", true);
            attach()
        }
        DllCallReason::ProcessDetach => {
            unsafe { FreeLibrary(instance) };
            detach();
        }
        _ => {
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

pub fn downcast_str(string: &(dyn std::any::Any + Send)) -> &str {
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
                }
                LOG_PANIC => {
                    let message = format!("{}", message);
                    out.finish(format_args!(
                        "{} {}",
                        time,
                        (&*message).red()
                    ))
                }
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

        for line in s.lines() {
            debug!(target: LOG_PANIC, "{}", line);
        }
    }));
}