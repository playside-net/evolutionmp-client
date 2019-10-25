use winapi::_core::iter::once;
use std::string::FromUtf16Error;
use winapi::shared::minwindef::{HMODULE, DWORD};
use std::ffi::{OsStr, OsString};
use winapi::um::winbase::SetDllDirectoryW;
use std::path::PathBuf;
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER;
use winapi::um::libloaderapi::GetModuleFileNameW;
use std::os::windows::ffi::OsStringExt;

pub mod user;
pub mod ps;
pub mod thread;
pub mod input;

fn fill_utf16_buf<F1, F2, T>(mut f1: F1, f2: F2) -> std::io::Result<T>
    where F1: FnMut(*mut u16, DWORD) -> DWORD,
          F2: FnOnce(&[u16]) -> T
{
    let mut stack_buf = [0u16; 512];
    let mut heap_buf = Vec::new();
    unsafe {
        let mut n = stack_buf.len();
        loop {
            let buf = if n <= stack_buf.len() {
                &mut stack_buf[..]
            } else {
                let extra = n - heap_buf.len();
                heap_buf.reserve(extra);
                heap_buf.set_len(n);
                &mut heap_buf[..]
            };

            SetLastError(0);
            let k = match f1(buf.as_mut_ptr(), n as DWORD) {
                0 if GetLastError() == 0 => 0,
                0 => return Err(std::io::Error::last_os_error()),
                n => n,
            } as usize;
            if k == n && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                n *= 2;
            } else if k >= n {
                n = k;
            } else {
                return Ok(f2(&buf[..k]))
            }
        }
    }
}

fn os2path(s: &[u16]) -> PathBuf {
    PathBuf::from(OsString::from_wide(s))
}

pub fn get_module_name(module: HMODULE) -> std::io::Result<PathBuf> {
    fill_utf16_buf(|buf, sz| unsafe {
        GetModuleFileNameW(module, buf, sz)
    }, os2path)
}