use winapi::shared::windef::{HWND, POINT};
use winapi::shared::basetsd::LONG_PTR;
use winapi::shared::minwindef::{UINT, WPARAM, LPARAM, LRESULT, HKL};
use winapi::um::winuser::{WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEWHEEL, WM_MOUSEMOVE, CallWindowProcA, WNDPROC, FindWindowA, SetWindowLongPtrA, GWLP_WNDPROC, GET_WHEEL_DELTA_WPARAM, GetAsyncKeyState, VK_SHIFT, VK_CONTROL, WM_CHAR, WM_UNICHAR, SetWindowLongPtrW, CallWindowProcW, TranslateMessage, GetMessageW, MSG, WM_SYSCHAR, WM_KEYFIRST, WM_KEYLAST, GetKeyboardState, ToUnicode, MapVirtualKeyA, GetKeyboardLayout, MAPVK_VK_TO_CHAR, MapVirtualKeyW, LoadKeyboardLayoutW, MapVirtualKeyExW, MAPVK_VK_TO_VSC, MAPVK_VSC_TO_VK, WM_INPUT, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, GetRawInputData, RID_INPUT, RIM_TYPEKEYBOARD, RI_KEY_E0, RI_KEY_E1, MAPVK_VSC_TO_VK_EX, VK_RCONTROL, VK_LCONTROL, VK_RMENU, VK_LMENU, VK_PAUSE, VK_SCROLL, GetForegroundWindow, GetWindowThreadProcessId, WM_INPUTLANGCHANGE, WM_INPUTLANGCHANGEREQUEST, ActivateKeyboardLayout, KLF_RESET, KLF_REPLACELANG, KLF_SETFORPROCESS, ToUnicodeEx, VK_DELETE, WM_CREATE, SetWindowTextA};
use winapi::um::sysinfoapi::GetTickCount;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::ffi::CString;
use std::time::{Duration, Instant};
use crate::pattern::MemoryRegion;
use std::ptr::null_mut;
use widestring::WideCStr;

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

#[derive(Debug, Clone)]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent)
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

static mut LAST_LAYOUT: Option<HKL> = None;
static mut LAST_LAYOUT_CHANGE: Option<Instant> = None;

fn push_event(event: InputEvent) {
    unsafe {
        EVENT_POOL.as_mut().unwrap().send(event);
    }
}

#[no_mangle]
pub unsafe extern "stdcall" fn WndProc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
                is_up
            };

            push_event(InputEvent::Keyboard(event));

            if wparam as i32 == VK_DELETE && !is_up {
                push_event(InputEvent::Keyboard(KeyboardEvent::Char('\u{007F}')));
            }
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
            push_event(InputEvent::Mouse(MouseEvent::Click(button, down)))
        },
        WM_MOUSEWHEEL => {
            let scroll = if GET_WHEEL_DELTA_WPARAM(wparam) > 0 { 1.0 } else { -1.0 };
            push_event(InputEvent::Mouse(MouseEvent::Wheel(scroll)))
        },
        WM_MOUSEMOVE => {
            let x = lparam as i16;
            let y = (lparam >> 16) as i16;
            push_event(InputEvent::Mouse(MouseEvent::Move(x, y)))
        },
        WM_CHAR | WM_SYSCHAR => {
            let target_thread = GetWindowThreadProcessId(hwnd, null_mut());
            let layout = LAST_LAYOUT.unwrap_or_else(|| GetKeyboardLayout(target_thread));
            let scan_code = ((lparam >> 16) & 0xFF) as u8;
            let vk = MapVirtualKeyExW(scan_code as u32, MAPVK_VSC_TO_VK, layout);
            let mut key_state = [0u8; 256];
            GetKeyboardState(key_state.as_mut_ptr());
            let mut buf = [0u16; 2];
            let len = ToUnicodeEx(vk, scan_code as u32, key_state.as_mut_ptr(), buf.as_mut_ptr(), 2, 0, layout);
            let chars = WideCStr::from_ptr_with_nul(buf.as_ptr(), len as usize).to_string().expect("chars conversation failed");
            if len == 1 {
                let chr = chars.chars().next().unwrap();
                push_event(InputEvent::Keyboard(KeyboardEvent::Char(chr)))
            }
        },
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

    CallWindowProcW(WND_PROC, hwnd, msg, wparam, lparam)
}

pub struct InputHook {
    receiver: Receiver<InputEvent>
}

impl InputHook {
    pub unsafe fn new(mem: &MemoryRegion) -> Option<InputHook> {
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

    pub fn next_event(&mut self) -> Result<InputEvent, TryRecvError> {
        self.receiver.try_recv()
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