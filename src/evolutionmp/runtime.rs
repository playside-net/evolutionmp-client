use crate::hash::Hash;
use crate::pattern::MemoryRegion;
use crate::native::collection::PtrCollection;
use crate::GameState;
use crate::win::input::{KeyEvent, InputEvent, MouseEvent, MouseButton};
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

const ACTIVE_THREAD_TLS_OFFSET: isize = 0x830;

pub(crate) static mut MAIN_FIBER: Option<Fiber> = None;
pub(crate) static mut SCRIPTS: Vec<ScriptContainer> = Vec::new();

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
}

pub(crate) unsafe fn register_script<S>(script: S) where S: Script + 'static {
    SCRIPTS.push(ScriptContainer::new(script));
}

pub struct ScriptContainer {
    fiber: Option<Fiber>,
    wake_at: Instant,
    script: Option<Box<dyn Script>>,
    input_events: Vec<(InputEvent, Instant)>
}

impl ScriptContainer {
    pub fn new<S>(script: S) -> ScriptContainer where S: Script + 'static {
        ScriptContainer {
            fiber: None,
            wake_at: Instant::now(),
            script: Some(Box::new(script)),
            input_events: Vec::new()
        }
    }

    extern "system" fn fiber_loop(&mut self) {
        let mut script = self.script.take().unwrap();
        script.prepare(ScriptEnv {
            container: self
        });
        loop {
            while let Some((event, time_caught)) = self.input_events.pop() {
                script.input(ScriptEnv {
                    container: self
                }, event, time_caught);
            }
            script.frame(ScriptEnv {
                container: self
            });
            self.wait(Duration::from_millis(0))
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
    container: &'a mut ScriptContainer
}

impl<'a> ScriptEnv<'a> {
    pub fn wait(&mut self, duration: Duration) {
        self.container.wait(duration)
    }
}