use std::ffi::{CString, OsString};
use std::ptr::null_mut;
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{HKL, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::winuser::{CallWindowProcW, FindWindowA, GET_WHEEL_DELTA_WPARAM, GetAsyncKeyState, GetKeyboardLayout, GetKeyboardState, GetWindowThreadProcessId, GWLP_WNDPROC, MapVirtualKeyExW, MAPVK_VSC_TO_VK, SetWindowLongPtrW, ToUnicodeEx, VK_CONTROL, VK_DELETE, VK_SHIFT, WM_CHAR, WM_INPUTLANGCHANGE, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSCHAR, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDPROC, WM_NCHITTEST, WM_SETCURSOR, WM_GETICON};
use wio::wide::FromWide;

use crate::events::ScriptEvent;
use crate::Window;
use std::convert::TryFrom;

static mut EVENT_POOL: Option<Sender<InputEvent>> = None;
static mut WND_PROC: WNDPROC = None;

#[derive(Debug, Clone)]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
}

#[derive(Debug, Clone)]
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
        is_up: bool,
    },
    Char(String),
}

#[derive(Debug, Copy, Clone)]
pub enum MouseEvent {
    Click(MouseButton, bool),
    Wheel(f32),
    Move(i16, i16),
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

static mut LAST_LAYOUT: Option<HKL> = None;
static mut LAST_LAYOUT_CHANGE: Option<Instant> = None;

fn push_event(event: InputEvent) {
    unsafe {
        EVENT_POOL.as_ref().unwrap().send(event).expect("failed to push input event");
    }
}

#[no_mangle]
pub unsafe extern "system" fn process_event(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
            let is_up = msg == WM_SYSKEYUP || msg == WM_KEYUP;
            let event = KeyboardEvent::Key {
                key: wparam as i32,
                repeats: (lparam & 0xFFFF) as u16,
                scan_code: ((lparam >> 16) & 0xFF) as u8,
                is_extended: ((lparam >> 24) & 1) == 1,
                alt: msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP,
                shift: (GetAsyncKeyState(VK_SHIFT) as usize & 0x8000) != 0,
                control: (GetAsyncKeyState(VK_CONTROL) as usize & 0x8000) != 0,
                was_down_before: ((lparam >> 30) & 1) == 1,
                is_up,
            };

            push_event(InputEvent::Keyboard(event));

            if wparam as i32 == VK_DELETE && !is_up {
                push_event(InputEvent::Keyboard(KeyboardEvent::Char(String::from("\u{007F}"))));
            }
        }
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
            push_event(InputEvent::Mouse(MouseEvent::Click(button, down)))
        }
        WM_MOUSEWHEEL => {
            let scroll = if GET_WHEEL_DELTA_WPARAM(wparam) > 0 { 1.0 } else { -1.0 };
            push_event(InputEvent::Mouse(MouseEvent::Wheel(scroll)))
        }
        WM_MOUSEMOVE => {
            let x = lparam as i16;
            let y = (lparam >> 16) as i16;
            push_event(InputEvent::Mouse(MouseEvent::Move(x, y)))
        }
        WM_CHAR | WM_SYSCHAR => {
            let target_thread = GetWindowThreadProcessId(hwnd, null_mut());
            let layout = LAST_LAYOUT.unwrap_or_else(|| GetKeyboardLayout(target_thread));
            let scan_code = ((lparam >> 16) & 0xFF) as u8;
            let vk = MapVirtualKeyExW(scan_code as u32, MAPVK_VSC_TO_VK, layout);
            let mut key_state = [0u8; 256];
            GetKeyboardState(key_state.as_mut_ptr());
            let mut buf = [0u16; 2];
            let len = ToUnicodeEx(vk, scan_code as u32, key_state.as_mut_ptr(), buf.as_mut_ptr(), 2, 0, layout);
            let chars = OsString::from_wide_ptr(buf.as_ptr(), len as usize).into_string().expect("chars conversation failed");
            if len != 0 {
                push_event(InputEvent::Keyboard(KeyboardEvent::Char(chars)))
            } else {
                match char::try_from(wparam as u32) {
                    Ok(c) => push_event(InputEvent::Keyboard(KeyboardEvent::Char(c.to_string()))),
                    Err(e) => error!("Invalid character input: {:?}", e)
                }
            }
        }
        WM_INPUTLANGCHANGE => {
            let layout = lparam as HKL;
            if let Some(last_layout) = LAST_LAYOUT {
                if layout != last_layout {
                    let last_change = LAST_LAYOUT_CHANGE.unwrap();
                    if Instant::now().duration_since(last_change) > Duration::from_millis(50) {
                        LAST_LAYOUT = Some(layout);
                        LAST_LAYOUT_CHANGE = Some(Instant::now());
                    }
                }
            } else {
                LAST_LAYOUT = Some(layout);
                LAST_LAYOUT_CHANGE = Some(Instant::now());
            }
        }
        _ => {}
    }

    let start = Instant::now();
    let ret = CallWindowProcW(WND_PROC, hwnd, msg, wparam, lparam);
    let elapsed = start.elapsed();
    if elapsed > Duration::from_millis(20) {
        warn!("Window event 0x{:08X} took {} ms. Result was 0x{:08X}", msg, elapsed.as_millis(), ret);
    }
    ret
}


pub unsafe fn hook(window: &Window) {
    let (sender, receiver) = channel::<InputEvent>();
    EVENT_POOL = Some(sender);
    WND_PROC = window.set_event_processor(Some(process_event));
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            let mut event_senders = crate::native::script::EVENT_SENDERS.lock().unwrap();
            for sender in event_senders.iter_mut() {
                sender.send(ScriptEvent::UserInput(event.clone())).expect("event sending failed");
            }
        }
    });
}

pub unsafe fn unhook() {
    if let Some(proc) = WND_PROC {
        let window = CString::new("grcWindow").unwrap();
        let handle = FindWindowA(window.as_ptr() as *const _, std::ptr::null());
        SetWindowLongPtrW(handle, GWLP_WNDPROC, std::mem::transmute(proc));
    }
}