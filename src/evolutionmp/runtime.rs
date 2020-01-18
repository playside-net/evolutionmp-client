use crate::hash::Hash;
use crate::pattern::MemoryRegion;
use crate::native::collection::PtrCollection;
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
use crate::game::streaming::Resource;
use crate::game::player::Player;
use cgmath::Vector3;
use crate::game::ped::Ped;
use crate::game::vehicle::Vehicle;

const ACTIVE_THREAD_TLS_OFFSET: isize = 0x830;

pub(crate) static CONSOLE_VISIBLE: AtomicBool = AtomicBool::new(false);

pub struct EventPool {
    input: Vec<ScriptEvent>,
    output: VecDeque<ScriptEvent>
}

impl EventPool {
    pub fn new() -> EventPool {
        EventPool {
            input: Vec::new(),
            output: VecDeque::new()
        }
    }

    pub fn push_input(&mut self, event: ScriptEvent) {
        self.input.push(event)
    }

    pub fn push_output(&mut self, event: ScriptEvent) {
        self.output.push_back(event)
    }

    pub fn swap(&mut self) {
        self.input.clear();
        while let Some(event) = self.output.pop_front() {
            self.input.push(event)
        }
    }

    pub fn iterate<F>(&mut self, mut handler: F) where F: FnMut(&ScriptEvent) -> bool {
        self.input.retain(|i| !handler(i));
    }
}

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
        if let Ok(mut native_events) = EVENTS.try_borrow_mut() {
            if let Some(native_events) = native_events.as_mut() {
                let mut event_pool = self.event_pool.as_mut().expect("missing runtime event pool");
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
        let mut event_pool = self.event_pool.as_mut().expect("missing runtime event pool");
        event_pool.swap();
    }

    pub(crate) fn register_script<N, S>(&mut self, name: N, script: S) where N: Into<String>, S: Script + 'static {
        self.scripts.push(ScriptContainer::new(name, script));
    }
}

static EVENTS: ThreadSafe<RefCell<Option<VecDeque<NativeEvent>>>> = ThreadSafe::new(RefCell::new(None));
static RUNTIME: ThreadSafe<RefCell<Option<Runtime>>> = ThreadSafe::new(RefCell::new(None));
static HOOKS: ThreadSafe<RefCell<Option<HashMap<u64, RawDetour>>>> = ThreadSafe::new(RefCell::new(None));

#[derive(Debug)]
pub enum NativeEvent {
    NewVehicle {
        model: Hash,
        pos: Vector3<f32>,
        heading: f32,
        is_network: bool,
        this_script_check: bool
    },
    TaskEnterVehicle {
        ped: Ped,
        vehicle: Vehicle,
        timeout: u32,
        seat: i32,
        speed: f32,
        flag: i32,
        unknown: u32
    }
}

impl NativeEvent {
    pub fn vehicle(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::NewVehicle {
            model: args.read(),
            pos: args.read(),
            heading: args.read(),
            is_network: args.read(),
            this_script_check: args.read(),
        }
    }

    pub fn task_enter_vehicle(context: &mut NativeCallContext) -> NativeEvent {
        let mut args = context.get_args();
        NativeEvent::TaskEnterVehicle {
            ped: args.read(),
            vehicle: args.read(),
            timeout: args.read(),
            seat: args.read(),
            speed: args.read(),
            flag: args.read(),
            unknown: args.read()
        }
    }
}

macro_rules! native_event {
    ($hash:literal, $constructor:ident) => {
        hook_native($hash, |context| {
            push_native_event(NativeEvent::$constructor(context));
            call_native_trampoline($hash, context);
        });
    };
}

pub(crate) unsafe fn start(mem: &MemoryRegion, input: InputHook) {
    let mut runtime = Runtime::new(input);
    info!("Initializing multiplayer");
    crate::multiplayer::init(&mut runtime);

    EVENTS.replace(Some(VecDeque::new()));
    RUNTIME.replace(Some(runtime));
    HOOKS.replace(Some(HashMap::new()));

    info!("Hooking natives");

    hook_native(0xFC8202EFC642E6F2, |context| {
        if let Ok(mut runtime) = RUNTIME.try_borrow_mut() {
            if let Some(mut runtime) = runtime.as_mut() {
                runtime.frame();
            }
        }
        call_native_trampoline(0xFC8202EFC642E6F2, context)
    });
    native_event!(0xAF35D0D2583051B0, vehicle);
    native_event!(0xC20E50AA46D09CA8, task_enter_vehicle);
}

fn push_native_event(event: NativeEvent) {
    if let Ok(mut events) = EVENTS.try_borrow_mut() {
        if let Some(mut events) = events.as_mut() {
            events.push_back(event);
        }
    }
}

fn call_native_trampoline(hash: u64, context: *mut NativeCallContext) {
    let hooks = HOOKS.try_borrow().expect("unable to borrow hook map");
    let hooks = hooks.as_ref().expect("hook map is not initialized");
    let detour = hooks.get(&hash).expect(&format!("missing native trampoline for 0x{:016X}", hash));
    unsafe {
        let trampoline: NativeFunction = std::mem::transmute(detour.trampoline());
        trampoline(context);
    }
}

fn hook_native(hash: u64, hook: fn(&mut NativeCallContext)) {
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
            self.wait(Duration::from_millis(0))
        }
        self.script = Some(script);
    }

    fn process_input(&mut self, script: &mut Box<dyn Script>) {
        let mut event_pool = self.event_pool.as_mut().expect("missing script event pool");
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

    pub fn wait(&mut self, duration: Duration) {
        self.wake_at = Instant::now() + duration;
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
    fn prepare(&mut self, mut env: ScriptEnv) {}
    fn frame(&mut self, mut env: ScriptEnv) {}
    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool { false }
}

pub struct ScriptEnv<'a> {
    container: &'a mut ScriptContainer
}

impl<'a> ScriptEnv<'a> {
    fn new(container: &'a mut ScriptContainer) -> ScriptEnv<'a> {
        ScriptEnv { container }
    }

    pub fn wait(&mut self, duration: Duration) {
        self.container.wait(duration)
    }

    pub fn event(&mut self, event: ScriptEvent) {
        let mut event_pool = self.container.event_pool.as_mut().expect("missing env event pool");
        event_pool.push_output(event);
    }

    pub fn log<L>(&mut self, line: L) where L: Into<String> {
        self.event(ScriptEvent::ConsoleOutput(line.into()));
    }

    pub fn wait_for_resource(&mut self, resource: &dyn Resource) {
        resource.request();
        while !resource.is_loaded() {
            self.wait(Duration::from_millis(0));
        }
    }
}

pub enum ScriptEvent {
    ConsoleInput(String),
    ConsoleOutput(String),
    NativeEvent(NativeEvent),
    UserInput(InputEvent)
}