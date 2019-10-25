use winapi::shared::windef::HWND;
use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT};
use winapi::um::winuser::{WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, CallWindowProcA, WNDPROC, FindWindowA, SetWindowLongPtrA, GWLP_WNDPROC};
use winapi::um::sysinfoapi::GetTickCount;
use std::sync::{Arc, Mutex};
use std::cell::UnsafeCell;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::CString;
use winapi::shared::ntdef::NULL;
use std::time::Duration;
use std::sync::atomic::AtomicPtr;

static mut EVENT_POOL: Option<EventPool> = None;
static mut WND_PROC: WNDPROC = None;

struct EventPool {
    sender: Sender<KeyEvent>
}

impl EventPool {
    fn send(&mut self, event: KeyEvent) {
        self.sender.send(event).expect("Unable to sync keyboard event")
    }
}

#[derive(Debug, Copy, Clone)]
pub struct KeyEvent {
    pub key: i32,
    pub repeats: u16,
    pub scan_code: u8,
    pub is_extended: bool,
    pub alt: bool,
    pub was_down_before: bool,
    pub is_up: bool
}

#[no_mangle]
pub unsafe extern "stdcall" fn WndProc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let alt = msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP;
    if msg == WM_KEYDOWN || msg == WM_KEYUP || alt {
        let event = KeyEvent {
            key: wparam as i32,
            repeats: (lparam & 0xFFFF) as u16,
            scan_code: ((lparam >> 16) & 0xFF) as u8,
            is_extended: ((lparam >> 24) & 1) == 1,
            alt,
            was_down_before: ((lparam >> 30) & 1) == 1,
            is_up: msg == WM_SYSKEYUP || msg == WM_KEYUP
        };
        EVENT_POOL.as_mut().unwrap().send(event)
    }
    CallWindowProcA(WND_PROC, hwnd, msg, wparam, lparam)
}

pub struct InputHook {
    receiver: Receiver<KeyEvent>
}

impl InputHook {
    pub unsafe fn new() -> Option<InputHook> {
        let (sender, receiver) = channel::<KeyEvent>();
        EVENT_POOL.replace(EventPool { sender });
        let mut handle: HWND = std::ptr::null_mut();
        let window = CString::new("grcWindow").unwrap();
        while handle.is_null() {
            handle = FindWindowA(window.as_ptr() as *const _, std::ptr::null());
            std::thread::sleep(Duration::from_millis(100));
        }
        let proc = std::mem::transmute(SetWindowLongPtrA(handle, GWLP_WNDPROC, WndProc as u64 as LONG_PTR));
        WND_PROC = proc;
        if proc.is_none() {
            None
        } else {
            Some(InputHook {
                receiver
            })
        }
    }

    pub fn next_event(&mut self) -> Option<KeyEvent> {
        self.receiver.try_recv().ok()
    }
}

impl std::ops::Drop for InputHook {
    fn drop(&mut self) {
        unsafe {
            let window = CString::new("grcWindow").unwrap();
            let handle = FindWindowA(window.as_ptr() as *const _, std::ptr::null());
            SetWindowLongPtrA(handle, GWLP_WNDPROC, std::mem::transmute(WND_PROC.unwrap()));
        }
    }
}