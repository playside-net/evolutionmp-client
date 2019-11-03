use crate::hash::Hash;
use crate::pattern::MemoryRegion;
use crate::native::collection::PtrCollection;
use crate::GameState;
use crate::win::input::{KeyboardEvent, InputEvent, MouseEvent, MouseButton};
use crate::native::{NativeCallContext, NativeStackValue};
use crate::hash::joaat;
use crate::win::thread::Fiber;
use std::os::raw::c_char;
use std::ffi::CString;
use std::time::{Instant, Duration};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::minwindef::{LPVOID, DWORD, TRUE};
use winapi::um::winuser::VK_RETURN;
use detour::static_detour;
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::VecDeque;
use winapi::_core::panic::PanicInfo;
use std::panic::AssertUnwindSafe;

const ACTIVE_THREAD_TLS_OFFSET: isize = 0x830;

pub(crate) static mut MAIN_FIBER: Option<Fiber> = None;
pub(crate) static mut SCRIPTS: Vec<ScriptContainer> = Vec::new();
pub(crate) static mut CONSOLE: Option<Console> = None;

static_detour! {
    static GetFrameCountHook: extern "C" fn(*mut NativeCallContext);
}

fn get_frame_count_native(context: *mut NativeCallContext) {
    unsafe {
        if MAIN_FIBER.is_none() {
            MAIN_FIBER = Fiber::convert_thread();
        }
        for s in &mut SCRIPTS {
            s.try_resume();
        }
    }

    GetFrameCountHook.call(context)
}

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    let natives = crate::native::NATIVES.as_mut().expect("Natives aren't initialized yet");
    let get_frame_count = natives.get_handler(0xFC8202EFC642E6F2)
        .expect("Unable to get native handler for GET_FRAME_COUNT");
    GetFrameCountHook
        .initialize(get_frame_count, get_frame_count_native).expect("GET_FRAME_COUNT hook initialization failed")
        .enable().expect("GET_FRAME_COUNT hook enabling failed");

    CONSOLE.replace(Console {
        lines: Mutex::new(VecDeque::new()),
        is_open: Mutex::new(false),
        last_closed: Mutex::new(Instant::now())
    });
}

pub(crate) unsafe fn register_script<N, S>(name: N, script: S) where N: Into<String>, S: Script + 'static {
    SCRIPTS.push(ScriptContainer::new(name, script));
}

pub struct ScriptContainer {
    name: String,
    fiber: Option<Fiber>,
    wake_at: Instant,
    script: Option<Box<dyn Script>>,
    input_events: Vec<(InputEvent, Instant)>
}

impl ScriptContainer {
    pub fn new<N, S>(name: N, script: S) -> ScriptContainer where N: Into<String>, S: Script + 'static {
        ScriptContainer {
            name: name.into(),
            fiber: None,
            wake_at: Instant::now(),
            script: Some(Box::new(script)),
            input_events: Vec::new()
        }
    }

    extern "system" fn fiber_loop(&mut self) {
        let name = self.name.clone();
        let mut this = AssertUnwindSafe(self);
        let result = std::panic::catch_unwind(move || {
            let mut script = this.script.take().unwrap();
            script.prepare(ScriptEnv {
                wait: &mut |d| this.wait(d)
            });
            loop {
                while let Some((event, time_caught)) = this.input_events.pop() {
                    script.input(ScriptEnv {
                        wait: &mut |d| this.wait(d)
                    }, event, time_caught);
                }
                script.frame(ScriptEnv {
                    wait: &mut |d| this.wait(d)
                });
                this.wait(Duration::from_millis(0))
            }
        });
        if let Err(err) = result {
            crate::error!("EvolutionMP Error", "script `{}` panicked at: {}", name, crate::downcast_str(&err));
        }
    }

    fn try_resume(&mut self) {
        if Instant::now() >= self.wake_at {
            if let Some(fiber) = &self.fiber {
                fiber.make_current();
            } else {
                self.fiber = Some(Fiber::new(0, self, ScriptContainer::fiber_loop));
            }
        }
    }

    pub fn input(&mut self, event: InputEvent) {
        self.input_events.push((event, Instant::now()))
    }

    pub fn wait(&mut self, duration: Duration) {
        self.wake_at = Instant::now() + duration;
        unsafe {
            let fiber = MAIN_FIBER.as_ref().expect("Missing main fiber");
            fiber.make_current();
        }
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

pub trait Script {
    fn prepare(&mut self, mut env: ScriptEnv) {}
    fn frame(&mut self, mut env: ScriptEnv) {}
    fn input(&mut self, mut env: ScriptEnv, event: InputEvent, time_caught: Instant) {}
}

pub struct ScriptEnv<'a> {
    wait: &'a mut dyn FnMut(Duration)
}

impl<'a> ScriptEnv<'a> {
    pub fn wait(&mut self, duration: Duration) {
        (self.wait)(duration)
    }
}

pub struct Console {
    lines: Mutex<VecDeque<String>>,
    is_open: Mutex<bool>,
    last_closed: Mutex<Instant>
}

impl Console {
    pub fn is_open(&self) -> bool {
        *self.is_open.lock().expect("Console lock failed")
    }

    pub fn set_open(&mut self, open: bool) {
        *self.is_open.lock().expect("Console lock failed") = open;
        if !open {
            *self.last_closed.lock().expect("Console lock failed") = Instant::now() + Duration::from_millis(200)
        }
    }

    pub fn take_lines(&self) -> Vec<String> {
        let mut lines = self.lines.lock().expect("Console lock failed");
        let mut result = Vec::new();
        while let Some(line) = lines.pop_front() {
            result.push(line);
        }
        result
    }

    pub fn add_line<L>(&self, line: L) where L: Into<String> {
        self.lines.lock().expect("Console lock failed").push_back(line.into())
    }

    pub fn get_last_closed(&self) -> Instant {
        *self.last_closed.lock().expect("Console lock failed")
    }
}