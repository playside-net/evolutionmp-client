use std::ffi::{CStr, CString, OsString};
use std::path::Path;
use std::sync::atomic::Ordering;

use backtrace::{Backtrace, SymbolName};
use detour::RawDetour;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{BOOL, DWORD, HINSTANCE, HLOCAL, HMODULE, LPVOID, MAX_PATH, TRUE};
use winapi::shared::windef::{HHOOK, HMENU, HWND};
use winapi::um::errhandlingapi::{AddVectoredExceptionHandler, GetLastError};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibrary, GetModuleFileNameW, GetModuleHandleA, GetProcAddress, LoadLibraryA};
use winapi::um::memoryapi::VirtualQuery;
use winapi::um::winbase::{
    FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_HMODULE,
    FORMAT_MESSAGE_FROM_SYSTEM, FormatMessageW, LocalFree,
};
use winapi::um::winnt::{EXCEPTION_POINTERS, EXCEPTION_RECORD, LANG_NEUTRAL, LONG, LPCSTR, LPWSTR, MAKELANGID, MEMORY_BASIC_INFORMATION, STATUS_ACCESS_VIOLATION, STATUS_IN_PAGE_ERROR, SUBLANG_DEFAULT};
use winapi::um::winuser::{GWLP_WNDPROC, IsWindow, IsWindowVisible, SetWindowLongPtrW, WNDPROC, HOOKPROC};
use wio::wide::FromWide;

use game::GameState;

use crate::{bind_field, bind_field_ip, LOG_PANIC, mem};
use crate::client::pattern::RET;

pub mod win;
pub mod native;
pub mod runtime;
pub mod events;
pub mod mappings;
pub mod game;
pub mod pattern;
pub mod registry;
pub mod scripts;
pub mod jni;
pub mod console;

bind_field_ip!(GAME_STATE, "83 3D ? ? ? ? ? 8A D9 74 0A", 2, GameState, 5);
bind_field_ip!(HEAP_SIZE, "83 C8 01 48 8D 0D ? ? ? ? 41 B1 01 45 33 C0", 17, u32);

unsafe fn print_address_info(addr: *mut c_void, line: Option<u32>, symbol_name: SymbolName) {
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    let size = std::mem::size_of::<MEMORY_BASIC_INFORMATION>();
    if VirtualQuery(addr, &mut mbi, size) == size {
        let mut name = [0; MAX_PATH];
        let len = GetModuleFileNameW(mbi.AllocationBase.cast(), name.as_mut_ptr(), MAX_PATH as u32);
        if len != 0 {
            let name = OsString::from_wide_ptr(name.as_ptr(), len as usize);
            let offset = addr as u64 - mbi.AllocationBase as u64;
            if let Some(line) = line {
                debug!(target: LOG_PANIC, " at {} (line: {}) in '{}' + 0x{:X}", symbol_name, line, name.to_string_lossy(), offset)
            } else {
                debug!(target: LOG_PANIC, " at {} in '{}' + 0x{:X}", symbol_name, name.to_string_lossy(), offset)
            }
        }
    }
}

fn get_op(code: usize) -> &'static str {
    match code {
        0 => "reading",
        8 => "DEP",
        _ => "writing"
    }
}

unsafe fn get_error_code_message(ntdll: HMODULE, rec: &EXCEPTION_RECORD) -> String {
    match rec.ExceptionCode {
        STATUS_ACCESS_VIOLATION => {
            let address = rec.ExceptionInformation[1];
            if rec.NumberParameters == 3 {
                let op = get_op(rec.ExceptionInformation[0]);
                format!("STATUS_ACCESS_VIOLATION {} 0x{:08X}", op, address)
            } else {
                String::from("STATUS_ACCESS_VIOLATION")
            }
        }
        STATUS_IN_PAGE_ERROR => {
            let address = rec.ExceptionInformation[1];
            if rec.NumberParameters == 3 {
                let op = get_op(rec.ExceptionInformation[0]);
                let code = rec.ExceptionInformation[3];
                format!("STATUS_IN_PAGE_ERROR {} 0x{:08X} with code 0x{:08X}", op, address, code)
            } else {
                String::from("STATUS_IN_PAGE_ERROR")
            }
        }
        code => {
            let mut buffer: LPWSTR = std::ptr::null_mut();
            let strlen = FormatMessageW(FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_HMODULE,
                                        ntdll as _,
                                        code,
                                        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as _,
                                        (&mut buffer as *mut LPWSTR) as LPWSTR,
                                        0,
                                        std::ptr::null_mut());

            if buffer.is_null() {
                let err = GetLastError();
                format!("UNKNOWN (FormatMessageW() returned 0x{:08X})", err)
            } else {
                let message = OsString::from_wide_ptr(buffer, strlen as usize);
                LocalFree(buffer as HLOCAL);
                message.to_string_lossy().trim_matches(|c| c == '\r' || c == '\n').to_string()
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
        let ntdll = LoadLibraryA(c_str!("ntdll.dll").as_ptr());
        let message = get_error_code_message(ntdll, rec);
        let native = crate::native::CURRENT_NATIVE.load(Ordering::SeqCst);
        let active_script = crate::native::script::get_active_thread().as_ref();
        if let Some(active_script) = active_script {
            error!(target: LOG_PANIC, "Script {} crashed with an exception", active_script.context.script_hash);
        }
        if native != 0 {
            error!(target: LOG_PANIC, "Unhandled exception at 0x{:08X} caused by native invocation `0x{:016X}`: 0x{:08X} ({})", addr as u64, native, code, message);
        } else {
            error!(target: LOG_PANIC, "Unhandled exception at 0x{:08X}: 0x{:08X} ({})", addr as u64, code, message);
        }

        let backtrace = Backtrace::new();

        for frame in backtrace.frames().iter() {
            for symbol in frame.symbols() {
                if let Some(addr) = symbol.addr().clone() {
                    let name = symbol.name().unwrap_or_else(|| SymbolName::new(b"<unknown>"));
                    print_address_info(addr, symbol.lineno(), name);
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
    ($name:ident,$module:literal,$proc:literal,$repl:expr,($($arg:ty),*)->$ret:ty) => {
        lazy_static::lazy_static! {
            static ref $name: extern fn($($arg),*) -> $ret = unsafe {
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

unsafe extern fn create_window(ex_style: DWORD, class_name: LPWSTR, window_name: LPWSTR,
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

proc_detour!(CREATE_WINDOW, "user32.dll", "CreateWindowExW", create_window,
    (DWORD, LPWSTR, LPWSTR, DWORD, i32, i32, i32, i32, Window, HMENU, HINSTANCE, LPVOID) -> Window
);

proc_detour!(SET_WINDOWS_HOOK, "user32.dll", "SetWindowsHookExW", set_windows_hook,
    (u32, HOOKPROC, HINSTANCE, DWORD) -> HHOOK
);

proc_detour!(OUTPUT_DEBUG_STRING_A, "kernel32.dll", "OutputDebugStringA", debug_a,
    (LPCSTR) -> ()
);

proc_detour!(OUTPUT_DEBUG_STRING_W, "kernel32.dll", "OutputDebugStringW", debug_w,
    (LPWSTR) -> ()
);

unsafe extern fn set_windows_hook(id: u32, handler: HOOKPROC, module: HINSTANCE, thread_id: DWORD) -> HHOOK {
    //if crate::game::is_loaded() {
        warn!("Hook installation requested: {}, {:?}, {:p}, {}", id, handler.map(|f| f as *mut ()), module, thread_id);
        SET_WINDOWS_HOOK(id, handler, module, thread_id)
    //} else {
    //    std::ptr::null_mut()
    //}
}

unsafe extern fn debug_a(text: LPCSTR) {
    let text = CStr::from_ptr(text);
    info!("Debugger output: {}", text.to_string_lossy())
}

unsafe extern fn debug_w(text: LPWSTR) {
    let text = OsString::from_wide_ptr_null(text);
    info!("Debugger output: {}", text.to_string_lossy())
}

unsafe fn initialize(window: &Window) {
    console::attach();

    lazy_static::initialize(&GAME_STATE);
    info!("Hooking DirectX...");
    crate::win::direct::hook();
    info!("Hooking user input...");
    crate::win::input::hook(window);

    info!("Applying patches...");

    //mem!("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").expect("launcher").nop(21); //Disable launcher check
    /*mem.find_str("platform:/movies").expect("movie")
        .write_bytes(b"platform:/movies/2secondsblack.bik\0"); //Disable movie*/

    mem!("70 6C 61 74 66 6F 72 6D 3A").expect("logos").write_bytes(&[RET]); //Disable movie
    /*mem!("72 1F E8 ? ? ? ? 8B 0D").expect("legals")
        .nop(2); //Disable legals*/
    mem!("48 83 3D ? ? ? ? 00 88 05 ? ? ? ? 75 0B").expect("force offline")
        .add(8).nop(6);
    let focus_pause = mem!("0F 95 05 ? ? ? ? E8 ? ? ? ? 48 85 C0").expect("focus pause");
    focus_pause.add(3).read_ptr(4).write_bytes(&[0]);
    focus_pause.nop(7);
    bind_field!(DEVICE_LIMIT, "C7 05 ? ? ? ? 64 00 00 00 48 8B", 6, u32);
    *DEVICE_LIMIT.as_mut() *= 15;
    mem!("C6 80 F0 00 00 00 01 E8 ? ? ? ? E8").expect("no relative device sorting")
        .add(12).nop(5);
    /*mem!("48 85 C0 0F 84 ? ? ? ? 8B 48 50").expect("unlock objects")
        .nop(24);*/

    /*bind_field_ip!(DEBUGGER_HOOK, "48 8D 15 ? ? ? ? 41 8D 49 0D", 3, HHOOK);
    lazy_static::initialize(&DEBUGGER_HOOK);

    std::thread::spawn(|| {
        loop {
            unsafe {
                let hook = **DEBUGGER_HOOK;
                if !hook.is_null() {
                    if UnhookWindowsHookEx(hook) == 1 {
                        info!("unhooked {:p}", hook);
                    }
                    Sleep(0);
                }
            }
        }
    });*/

    //*HEAP_SIZE.as_mut() = 650 * 1024 * 1024; //Increase heap size to 650MB

    native::fs::hook();
    native::hook();

    game::hook();
    game::init();

    crate::scripts::init();
}

#[cfg(target_os = "windows")]
fn attach() {
    unsafe {
        if AddVectoredExceptionHandler(0, Some(except)).is_null() {
            panic!("Unable to set exception handler");
        }
        lazy_static::initialize(&OUTPUT_DEBUG_STRING_A);
        lazy_static::initialize(&OUTPUT_DEBUG_STRING_W);
        lazy_static::initialize(&CREATE_WINDOW);
        lazy_static::initialize(&SET_WINDOWS_HOOK);
    }
}

#[cfg(target_os = "windows")]
fn detach() {
    unsafe {
        crate::win::input::unhook();
    }
}

#[repr(u32)]
#[derive(Debug)]
pub enum DllCallReason {
    ProcessDetach,
    ProcessAttach,
    ThreadAttach,
    ThreadDetach,
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn DllMain(instance: HINSTANCE, reason: DllCallReason, _reserved: LPVOID) -> BOOL {
    match reason {
        DllCallReason::ProcessAttach => {
            unsafe { DisableThreadLibraryCalls(instance) };
            crate::setup_logger("client", true);
            attach()
        }
        DllCallReason::ProcessDetach => {
            unsafe { FreeLibrary(instance) };
            detach();
        }
        _ => {
            //info!("{:?}: {:?}", other, unsafe { GetCurrentThreadId() })
        }
    }
    TRUE
}