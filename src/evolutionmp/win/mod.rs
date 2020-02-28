use winapi::shared::minwindef::{HMODULE, DWORD};
use std::ffi::OsString;
use std::path::PathBuf;
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::shared::winerror::ERROR_INSUFFICIENT_BUFFER;
use winapi::um::libloaderapi::GetModuleFileNameW;
use std::os::windows::ffi::OsStringExt;

pub mod user;
pub mod ps;
pub mod thread;
pub mod input;