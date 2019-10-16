use crate::invoke;
use std::ffi::CString;
use std::mem::ManuallyDrop;

pub unsafe fn get_nickname() -> CString {
    invoke!(CString, 0x198D161F458ECC7F)
}