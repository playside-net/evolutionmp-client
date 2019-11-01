use crate::hash::Hash;
use crate::pattern::MemoryRegion;
use crate::native::collection::PtrCollection;
use crate::{GameState, GAME_STATE};
use crate::win::input::KeyEvent;
use crate::native::NativeCallContext;
use crate::hash::joaat;
use crate::win::thread::Fiber;
use std::os::raw::c_char;
use std::ffi::CString;
use std::time::{Instant, Duration};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::minwindef::{LPVOID, DWORD, TRUE};
use winapi::um::winuser::VK_RETURN;
use detour::static_detour;

const ACTIVE_THREAD_TLS_OFFSET: isize = 0x830;

pub(crate) static mut MAIN_FIBER: Option<Fiber> = None;
pub(crate) static mut SCRIPTS: Vec<ScriptContainer> = Vec::new();

static_detour! {
    static SystemWaitHook: extern "C" fn(*mut NativeCallContext);
}

fn wait_native(context: *mut NativeCallContext) {
    unsafe {
        if crate::game::get_state() == GameState::Playing {
            MAIN_FIBER = Fiber::current_or_convert_thread();
            if MAIN_FIBER.is_some() {
                let thread = get_active_thread();
                if (*thread).context.state == ThreadState::Running {
                    if (*thread).context.script_hash == joaat("main_persistent") {
                        for s in &mut SCRIPTS {
                            s.try_resume();
                        }
                    }
                }
            }
        }
    }

    SystemWaitHook.call(context)
}

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    let natives = crate::native::NATIVES.as_mut().expect("Natives aren't initialized yet");
    let system_wait = loop {
        if let Some(handler) = natives.get_handler(0x4EDE34FBADD967A6) {
            if handler as u64 != 0 {
                break handler;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    SystemWaitHook
        .initialize(system_wait, wait_native).expect("wait_native hook initialization failed")
        .enable().expect("wait_native hook enabling failed");
}

pub(crate) unsafe fn register<S>(script: S) where S: Script + 'static {
    SCRIPTS.push(ScriptContainer::new(script));
}

pub(crate) unsafe fn get_module_tls() -> *mut *mut u8 {
    std::mem::transmute(ntapi::winapi_local::um::winnt::__readgsqword(88))
}

pub(crate) unsafe fn get_active_thread() -> *mut ScriptThread {
    let mut tls = *get_module_tls();
    tls.offset(ACTIVE_THREAD_TLS_OFFSET).cast::<*mut ScriptThread>().read()
}

pub(crate) unsafe fn set_active_thread(thread: *mut ScriptThread) {
    let mut tls = *get_module_tls();
    tls.offset(ACTIVE_THREAD_TLS_OFFSET).cast::<*mut ScriptThread>().write(thread)
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ThreadState {
    Idle,
    Running,
    Killed,
    Unknown3,
    Unknown4
}

#[repr(C)]
pub struct ThreadContext {
    thread_id: u32,
    script_hash: u32,
    state: ThreadState,
    ip: u32,
    frame_sp: u32,
    sp: u32,
    timer_a: f32,
    timer_b: f32,
    wait_timer: f32,
    unknown1: u32,
    unknown2: u32,
    _f2c: u32,
    _f30: u32,
    _f34: u32,
    _f38: u32,
    _f3c: u32,
    _f40: u32,
    _f44: u32,
    _f48: u32,
    _f4c: u32,
    stack_size: u32,
    catch_ip: u32,
    catch_frame: u32,
    catch_sp: u32,
    _set1: u32,
    function_depth: u32,
    function_returns: [u32; 16]
}

pub struct ScriptContainer {
    fiber: Option<Fiber>,
    wake_at: Instant,
    script: Box<dyn Script>,
    key_events: Vec<(KeyEvent, Instant)>
}

impl ScriptContainer {
    pub fn new<S>(script: S) -> ScriptContainer where S: Script + 'static {
        ScriptContainer {
            fiber: None,
            wake_at: Instant::now(),
            script: Box::new(script),
            key_events: Vec::new()
        }
    }

    unsafe extern "system" fn fiber_loop(&mut self) {
        if !(self as *mut Self).is_null() {
            let ptr = self as *mut Self;
            let mut wait = move |d| (*ptr).wait(d);
            self.script.load(&mut wait);
        }

        while !(self as *mut Self).is_null() {
            let game_state = *crate::GAME_STATE;
            while let Some((event, time_caught)) = self.key_events.pop() {
                self.script.on_key(event, time_caught);
            }
            let ptr = self as *mut Self;
            let mut wait = move |d| (*ptr).wait(d);
            self.script.frame(&mut wait, game_state);
            self.wait(Duration::from_millis(0))
        }
    }

    fn try_resume(&mut self) {
        if Instant::now() < self.wake_at {
            unsafe {
                let fiber = MAIN_FIBER.as_mut().expect("Missing main fiber");
                if !fiber.is_current() {
                    fiber.make_current();
                }
            }
        } else {
            if let Some(fiber) = &self.fiber {
                fiber.make_current();
            } else {
                self.fiber = Some(Fiber::new(0, self, ScriptContainer::fiber_loop));
            }
            unsafe { MAIN_FIBER.as_mut().expect("Missing main fiber").make_current() };
        }
    }

    pub fn key(&mut self, key: KeyEvent) {
        self.key_events.push((key, Instant::now()))
    }

    fn wait(&mut self, duration: Duration) {
        unsafe {
            let fiber = MAIN_FIBER.as_mut().expect("Missing main fiber");
            if !fiber.is_current() {
                fiber.make_current();
            }
        }
        self.wake_at = Instant::now() + duration;
    }
}

impl std::ops::Drop for ScriptContainer {
    fn drop(&mut self) {
        if let Some(fiber) = self.fiber.as_mut() {
            fiber.delete();
        }
    }
}

type TY = unsafe extern "system" fn(LPVOID) -> DWORD;

pub type Wait = dyn FnMut(Duration);

pub trait Script {
    fn load(&mut self, wait: &mut Wait) {}
    fn frame(&mut self, wait: &mut Wait, game_state: GameState) {}
    fn on_key(&mut self, key: KeyEvent, time_caught: Instant) {}
}

#[repr(C)]
pub struct ScriptThread {
    v_table: *mut u8,
    context: ThreadContext,
    stack: *mut u8,
    pad0: i32,
    parameter_size: i32,
    statics_size: i32,
    pad3: i32,
    exit_message: *const c_char,
    name: [u8; 64]
}