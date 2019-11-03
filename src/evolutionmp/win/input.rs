use winapi::shared::windef::{HWND, POINT};
use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT};
use winapi::um::winuser::{WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEWHEEL, WM_MOUSEMOVE, CallWindowProcA, WNDPROC, FindWindowA, SetWindowLongPtrA, GWLP_WNDPROC, GET_WHEEL_DELTA_WPARAM, GetAsyncKeyState, VK_SHIFT, VK_CONTROL, WM_CHAR, WM_UNICHAR, SetWindowLongPtrW, CallWindowProcW, TranslateMessage, GetMessageW, MSG};
use winapi::um::sysinfoapi::GetTickCount;
use std::sync::{Arc, Mutex};
use std::cell::UnsafeCell;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::ffi::CString;
use winapi::shared::ntdef::NULL;
use std::time::Duration;
use std::sync::atomic::AtomicPtr;
use crate::win::input::InputEvent::Mouse;

static mut EVENT_POOL: Option<EventPool> = None;
static mut WND_PROC: WNDPROC = None;

struct EventPool {
    sender: Sender<InputEvent>
}

impl EventPool {
    fn send(&mut self, event: InputEvent) {
        self.sender.send(event).expect("Unable to sync keyboard event")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent)
}

#[derive(Debug, Copy, Clone)]
pub enum KeyboardEvent {
    Key {
        key: i32,
        repeats: u16,
        scan_code: u8,
        is_extended: bool,
        alt: bool,
        shift: bool,
        control: bool,
        was_down_before: bool,
        is_up: bool
    },
    Char(char)
}

#[derive(Debug, Copy, Clone)]
pub enum MouseEvent {
    Click(MouseButton, bool),
    Wheel(f32),
    Move(i16, i16)
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButton {
    Left, Right, Middle
}

#[no_mangle]
pub unsafe extern "stdcall" fn WndProc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
            let event = KeyboardEvent::Key {
                key: wparam as i32,
                repeats: (lparam & 0xFFFF) as u16,
                scan_code: ((lparam >> 16) & 0xFF) as u8,
                is_extended: ((lparam >> 24) & 1) == 1,
                alt: msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP,
                shift: (GetAsyncKeyState(VK_SHIFT) as usize & 0x8000) != 0,
                control: (GetAsyncKeyState(VK_CONTROL) as usize & 0x8000) != 0,
                was_down_before: ((lparam >> 30) & 1) == 1,
                is_up: msg == WM_SYSKEYUP || msg == WM_KEYUP
            };
            EVENT_POOL.as_mut().unwrap().send(InputEvent::Keyboard(event))
        },
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN | WM_MBUTTONUP => {
            let down = match msg {
                WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => true,
                _ => false
            };
            let button = match msg {
                WM_LBUTTONDOWN | WM_LBUTTONUP => MouseButton::Left,
                WM_RBUTTONDOWN | WM_RBUTTONUP => MouseButton::Left,
                _ => MouseButton::Middle
            };
            EVENT_POOL.as_mut().unwrap().send(InputEvent::Mouse(MouseEvent::Click(button, down)))
        },
        WM_MOUSEWHEEL => {
            let scroll = if GET_WHEEL_DELTA_WPARAM(wparam) > 0 { 1.0 } else { -1.0 };
            EVENT_POOL.as_mut().unwrap().send(InputEvent::Mouse(MouseEvent::Wheel(scroll)))
        },
        WM_MOUSEMOVE => {
            let x = lparam as i16;
            let y = (lparam >> 16) as i16;
            EVENT_POOL.as_mut().unwrap().send(InputEvent::Mouse(MouseEvent::Move(x, y)))
        },
        WM_CHAR => {
            let mut msg = MSG {
                hwnd,
                message: msg,
                wParam: wparam,
                lParam: lparam,
                time: GetTickCount(),
                pt: POINT { x: 0, y: 0 }
            };
            TranslateMessage(&msg as *const MSG);
            let wparam = msg.wParam;
            if wparam > 0 && wparam < 0x10000 {
                let c = std::mem::transmute(wparam as u32);
                EVENT_POOL.as_mut().unwrap().send(InputEvent::Keyboard(KeyboardEvent::Char(c)))
            }
        }
        _ => {}
    }
    CallWindowProcW(WND_PROC, hwnd, msg, wparam, lparam)
}

pub struct InputHook {
    receiver: Receiver<InputEvent>
}

impl InputHook {
    pub unsafe fn new() -> Option<InputHook> {
        let (sender, receiver) = channel::<InputEvent>();
        EVENT_POOL.replace(EventPool { sender });
        let mut handle: HWND = std::ptr::null_mut();
        let window = CString::new("grcWindow").unwrap();
        while handle.is_null() {
            handle = FindWindowA(window.as_ptr() as *const _, std::ptr::null());
            std::thread::sleep(Duration::from_millis(100));
        }
        let proc = std::mem::transmute(SetWindowLongPtrW(handle, GWLP_WNDPROC, WndProc as u64 as LONG_PTR));
        WND_PROC = proc;
        if proc.is_none() {
            None
        } else {
            Some(InputHook {
                receiver
            })
        }
    }

    pub fn next_event(&mut self) -> Option<InputEvent> {
        self.receiver.try_recv().ok()
    }
}

impl std::ops::Drop for InputHook {
    fn drop(&mut self) {
        unsafe {
            let window = CString::new("grcWindow").unwrap();
            let handle = FindWindowA(window.as_ptr() as *const _, std::ptr::null());
            SetWindowLongPtrW(handle, GWLP_WNDPROC, std::mem::transmute(WND_PROC.unwrap()));
        }
    }
}