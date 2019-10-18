use crate::invoke;
use std::ffi::CString;
use std::mem::ManuallyDrop;

pub unsafe fn get_nickname<'a>() -> &'a str {
    invoke!(&str, 0x198D161F458ECC7F)
}