use winapi::um::consoleapi::AllocConsole;
use detour::RawDetour;
use winapi::um::wincon::FreeConsole;
use winapi::shared::minwindef::{BOOL, FALSE};
use winapi::um::libloaderapi::{GetProcAddress, GetModuleHandleA};

extern {
    #[link_name = "llvm.returnaddress"]
    fn return_address(param: i32) -> *const u8;
}

unsafe extern "C" fn free_console() -> BOOL {
    let addr = return_address(0);
    crate::info!("{:p} is trying to deallocate console", addr);
    FALSE
}

pub(crate) unsafe fn attach() {
    AllocConsole();
    ansi_term::enable_ansi_support().expect("enabling console ansi support failed");
    let kernel = GetModuleHandleA(b"Kernel32.dll\0".as_ptr() as _);
    let proc = GetProcAddress(kernel, b"FreeConsole\0".as_ptr() as _);
    let d = RawDetour::new(proc as _, free_console as _)
        .expect("detour creation failed");
    d.enable().expect("detour enabling failed");
    std::mem::forget(d);
}