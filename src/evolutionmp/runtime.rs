use crate::hash::{Hash, Hashable};
use crate::pattern::MemoryRegion;
use crate::GameState;
use crate::win::input::{KeyboardEvent, InputEvent, MouseEvent, MouseButton, InputHook};
use crate::native::{NativeCallContext, NativeStackValue, ThreadSafe, NativeFunction};
use crate::hash::joaat;
use crate::win::thread::Fiber;
use crate::{info, error};
use std::os::raw::c_char;
use std::ffi::CString;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::{VecDeque, HashMap};
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::path::Path;
use detour::{static_detour, GenericDetour, RawDetour};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::minwindef::{LPVOID, DWORD, TRUE};
use winapi::um::winuser::VK_RETURN;
use std::panic::PanicInfo;
use winapi::ctypes::c_void;
use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::atomic::Ordering;
use crate::game::streaming::Resource;
use crate::game::player::Player;
use cgmath::Vector3;
use crate::game::ped::Ped;
use crate::game::vehicle::Vehicle;
use crate::events::{NativeEvent, ScriptEvent, EventPool};
use crate::game::ui::FrontendButtons;

const ACTIVE_THREAD_TLS_OFFSET: isize = 0x830;

pub struct Runtime {
    user_input: InputHook,
    main_fiber: Option<Fiber>,
    scripts: Vec<ScriptContainer>,
    event_pool: Option<EventPool>
}

impl Runtime {
    fn new(input: InputHook) -> Runtime {
        Runtime {
            user_input: input,
            scripts: Vec::new(),
            main_fiber: None,
            event_pool: Some(EventPool::new())
        }
    }

    fn frame(&mut self) {
        if self.main_fiber.is_none() {
            self.main_fiber = Some(Fiber::convert_thread().expect("cannot convert frame thread to fiber"));
        }
        while let Some(event) = self.user_input.next_event().ok() {
            let mut event_pool = self.event_pool.as_mut().expect("missing runtime event pool");
            event_pool.push_input(ScriptEvent::UserInput(event));
        }
        if let Ok(mut native_events) = crate::events::EVENTS.try_borrow_mut() {
            if let Some(native_events) = native_events.as_mut() {
                let event_pool = self.event_pool.as_mut().expect("missing runtime event pool");
                while let Some(event) = native_events.pop_front() {
                    event_pool.push_input(ScriptEvent::NativeEvent(event));
                }
            }
        }
        for s in &mut self.scripts {
            s.main_fiber = self.main_fiber.take();
            s.event_pool = self.event_pool.take();
            s.try_resume();
            self.main_fiber = s.main_fiber.take();
            self.event_pool = s.event_pool.take();
        }
        let event_pool = self.event_pool.as_mut().expect("missing runtime event pool");
        event_pool.swap();
    }

    pub(crate) fn register_script<N, S>(&mut self, name: N, script: S) where N: Into<String>, S: Script + 'static {
        self.scripts.push(ScriptContainer::new(name, script));
    }
}

static RUNTIME: ThreadSafe<RefCell<Option<Runtime>>> = ThreadSafe::new(RefCell::new(None));
static HOOKS: ThreadSafe<RefCell<Option<HashMap<u64, RawDetour>>>> = ThreadSafe::new(RefCell::new(None));

pub(crate) unsafe fn start(mem: &MemoryRegion, input: InputHook) {
    let mut runtime = Runtime::new(input);
    info!("Initializing scripts");
    crate::scripts::init(&mut runtime);

    RUNTIME.replace(Some(runtime));
    HOOKS.replace(Some(HashMap::new()));

    info!("Hooking natives");

    hook_native(0xFC8202EFC642E6F2, |context| {
        crate::game::ui::MOUSE_VISIBLE.store(false, Ordering::SeqCst);
        if let Ok(mut runtime) = RUNTIME.try_borrow_mut() {
            if let Some(runtime) = runtime.as_mut() {
                runtime.frame();
            }
        }
        call_native_trampoline(0xFC8202EFC642E6F2, context)
    });
    hook_native(0x7B5280EBA9840C72, |context| {
        let label = context.get_args().read::<&str>();
        let hash = label.joaat();
        crate::info!("Called GET_LABEL_TEXT for {} (0x{:08X})", label, hash.0);
        call_native_trampoline(0x7B5280EBA9840C72, context);
    });
    hook_native(0xAAE7CE1D63167423, |context| {
        crate::game::ui::MOUSE_VISIBLE.store(true, Ordering::SeqCst);
        call_native_trampoline(0xAAE7CE1D63167423, context)
    });

    crate::events::init(mem);
}

fn get_trampoline(hash: u64) -> NativeFunction {
    let hooks = HOOKS.try_borrow().expect("unable to borrow hook map");
    let hooks = hooks.as_ref().expect("hook map is not initialized");
    let detour = hooks.get(&hash).expect(&format!("missing native trampoline for 0x{:016X}", hash));
    unsafe { std::mem::transmute(detour.trampoline()) }
}

pub fn call_native_trampoline(hash: u64, context: *mut NativeCallContext) {
    let trampoline = get_trampoline(hash);
    unsafe {
        trampoline(context);
    }
}

pub fn hook_native(hash: u64, hook: fn(&mut NativeCallContext)) {
    let original = crate::native::get_handler(hash);
    unsafe {
        let detour = GenericDetour::new(original, std::mem::transmute(hook))
            .expect(&format!("native hook creation failed for 0x{:016X}", hash));
        detour.enable().expect(&format!("native hook enabling failed for 0x{:016X}", hash));
        let mut hooks = HOOKS.try_borrow_mut().expect("unable to mutably borrow hook map");
        let detour = std::mem::transmute::<GenericDetour<_>, RawDetour>(detour);
        hooks.as_mut().expect("hook map is not initialized").insert(hash, detour);
    }
}

unsafe impl std::marker::Send for ScriptContainer {}

pub struct TaskQueue {
    tasks: VecDeque<Box<dyn FnMut(&mut ScriptEnv)>>
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue {
            tasks: VecDeque::new()
        }
    }

    pub fn push<F>(&mut self, task: F) where F: FnMut(&mut ScriptEnv) + 'static {
        self.tasks.push_back(Box::new(task))
    }

    pub fn process(&mut self, env: &mut ScriptEnv) {
        while let Some(mut task) = self.tasks.pop_front() {
            task(env);
        }
    }
}

#[repr(C)]
pub struct ScriptContainer {
    name: String,
    fiber: Option<Fiber>,
    main_fiber: Option<Fiber>,
    script: Option<Box<dyn Script>>,
    wake_at: Instant,
    event_pool: Option<EventPool>
}

impl ScriptContainer {
    pub fn new<N, S>(name: N, script: S) -> ScriptContainer where N: Into<String>, S: Script + 'static {
        ScriptContainer {
            name: name.into(),
            fiber: None,
            main_fiber: None,
            wake_at: Instant::now(),
            script: Some(Box::new(script)),
            event_pool: None
        }
    }

    extern "system" fn fiber_loop(&mut self) {
        let mut script = self.script.take().unwrap();
        script.prepare(ScriptEnv::new(self));
        loop {
            self.process_input(&mut script);
            script.frame(ScriptEnv::new(self));
            self.wait(0)
        }
        self.script = Some(script);
    }

    fn process_input(&mut self, script: &mut Box<dyn Script>) {
        let event_pool = self.event_pool.as_mut().expect("missing script event pool");
        let mut output = VecDeque::new();
        event_pool.iterate(|e| script.event(e, &mut output));
        event_pool.output.extend(output.into_iter());
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

    pub fn wait(&mut self, millis: u64) {
        self.wake_at = Instant::now() + Duration::from_millis(millis);
        self.main_fiber.as_mut().expect("missing main fiber").make_current();
    }
}

impl std::ops::Drop for ScriptContainer {
    fn drop(&mut self) {
        if let Some(fiber) = self.fiber.as_mut() {
            fiber.delete();
        }
    }
}

pub trait Script {
    fn prepare(&mut self, env: ScriptEnv);
    fn frame(&mut self, env: ScriptEnv);
    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool;
}

pub struct ScriptEnv<'a> {
    container: &'a mut ScriptContainer
}

impl<'a> ScriptEnv<'a> {
    fn new(container: &'a mut ScriptContainer) -> ScriptEnv<'a> {
        ScriptEnv { container }
    }

    pub fn wait(&mut self, millis: u64) {
        self.container.wait(millis)
    }

    pub fn event(&mut self, event: ScriptEvent) {
        let mut event_pool = self.container.event_pool.as_mut().expect("missing env event pool");
        event_pool.push_output(event);
    }

    pub fn log<L>(&mut self, line: L) where L: Into<String> {
        self.event(ScriptEvent::ConsoleOutput(line.into()));
    }

    pub fn prompt(&mut self, title: &str, placeholder: &str, max_length: u32) -> Option<String> {
        crate::game::ui::prompt(self, title, placeholder, max_length)
    }

    pub fn warn(&mut self, title: &str, line1: &str, line2: &str, buttons: FrontendButtons, background: bool) -> FrontendButtons {
        crate::game::ui::warn(self, title, line1, line2, buttons, background)
    }

    pub fn wait_for_resource(&mut self, resource: &dyn Resource) {
        resource.request();
        while !resource.is_loaded() {
            self.wait(0);
        }
    }
}