use crate::hash::{Hash, Hashable};
use crate::pattern::MemoryRegion;
use crate::{GameState, GAME_STATE, launcher_dir};
use crate::win::input::{KeyboardEvent, InputEvent, MouseEvent, MouseButton, InputHook};
use crate::native::{NativeCallContext, NativeStackValue, ThreadSafe, NativeFunction};
use crate::hash::joaat;
use crate::win::thread::Fiber;
use crate::{args, info, error};
use crate::game::streaming::Resource;
use crate::game::player::Player;
use crate::game::ped::Ped;
use crate::game::vehicle::Vehicle;
use crate::events::{NativeEvent, ScriptEvent, EventPool};
use crate::game::ui::FrontendButtons;
use crate::jni::{JavaObject, JavaValue};
use std::os::raw::c_char;
use std::ffi::CString;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::{VecDeque, HashMap};
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::path::Path;
use detour::{GenericDetour, RawDetour};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::minwindef::{LPVOID, DWORD, TRUE};
use winapi::um::winuser::VK_RETURN;
use std::panic::PanicInfo;
use winapi::ctypes::c_void;
use std::cell::{Cell, RefCell};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicPtr};
use std::sync::atomic::Ordering;
use cgmath::Vector3;
use jni_dynamic::{JavaVM, InitArgs, InitArgsBuilder, JNIVersion, NativeMethod, JNIEnv, AttachGuard};
use jni_dynamic::objects::{JClass, JString, JObject, JByteBuffer, JValue};
use jni_dynamic::strings::JNIStr;
use jni_dynamic::errors::ErrorKind;

static mut ACTIVE_SCRIPT: *mut ScriptContainer = std::ptr::null_mut();

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

    pub(crate) fn frame(&mut self) {
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

static HOOKS: ThreadSafe<RefCell<Option<HashMap<u64, RawDetour>>>> = ThreadSafe::new(RefCell::new(None));

pub(crate) fn get_last_exception(env: &JNIEnv) -> String {
    let exception = env.exception_occurred().unwrap();
    env.exception_clear().unwrap();
    let string_writer = env.new_object("java/io/StringWriter", "()V", &[]).unwrap();
    let print_writer = env.new_object("java/io/PrintWriter", "(Ljava/io/Writer;)V", args![
        string_writer
    ]).unwrap();
    env.call_method(*exception, "printStackTrace", "(Ljava/io/PrintWriter;)V", args![
        print_writer
    ]).unwrap();
    String::from_java_value(env, env.call_method(string_writer, "toString", "()Ljava/lang/String;", &[])
        .unwrap())
}

static mut VM: Option<Arc<JavaVM>> = None;

fn attach_thread() -> AttachGuard<'static> {
    unsafe { VM.as_ref().expect("VM not initialized").attach_current_thread().expect("attach failed") }
}

pub(crate) fn start(input: InputHook, script_candidates: Vec<String>, vm: Arc<JavaVM>) {
    let mut runtime = Runtime::new(input);
    info!("Initializing scripts");
    crate::scripts::init(&mut runtime);

    unsafe { VM = Some(vm) };

    let env = attach_thread();

    let thread_class = env.find_class("java/lang/Thread").unwrap();

    let current_thread = env.call_static_method(thread_class, "currentThread", "()Ljava/lang/Thread;", &[])
        .unwrap().l().unwrap();

    let thread_name = "game".to_java_object(&env);
    env.call_method(current_thread, "setName", "(Ljava/lang/String;)V", args![thread_name])
        .expect("Unable to set game thread name");

    let system_loader = env.call_static_method("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", &[])
        .unwrap().l().unwrap();

    env.call_method(current_thread, "setContextClassLoader", "(Ljava/lang/ClassLoader;)V", args![system_loader]).unwrap();

    let ucp = env.get_field(system_loader, "ucp", "Lsun/misc/URLClassPath;").unwrap().l().unwrap();

    {
        let path = launcher_dir().join("client-rt.jar").to_string_lossy().to_java_object(&env);
        let file = env.new_object("java/io/File", "(Ljava/lang/String;)V", args![path]).unwrap();
        let uri = env.call_method(file, "toURI", "()Ljava/net/URI;", &[]).unwrap().l().unwrap();
        let url = env.call_method(uri, "toURL", "()Ljava/net/URL;", &[]).unwrap().l().unwrap();
        env.call_method(ucp, "addURL", "(Ljava/net/URL;)V", args![url]).unwrap();
    }

    macro_rules! natives {
        ($env:expr,$class_name:literal,$($native:expr),*) => {{
            let class = env.find_class($class_name).expect("Unable to find class");
            $env.register_natives(class, vec![$($native),*]).unwrap();
        }};
    }

    natives!(env, "mp/evolution/invoke/NativeArgs",
        NativeMethod::new("push", "(Ljava/lang/String;)V", put_string as _)
    );
    natives!(env, "mp/evolution/invoke/NativeResult",
        NativeMethod::new("getString", "()Ljava/lang/String;", get_string as _)
    );
    natives!(env, "mp/evolution/invoke/Native",
        NativeMethod::new("invoke", "(JLjava/nio/LongBuffer;Ljava/nio/LongBuffer;)V", invoke as _)
    );
    natives!(env, "mp/evolution/script/Script",
        NativeMethod::new("yield", "(J)V", wait as _),
        NativeMethod::new("propagate", "(Lmp/evolution/script/event/ScriptEvent;)V", propagate as _)
    );
    natives!(env, "mp/evolution/script/ScriptPrintStream",
        NativeMethod::new("info", "(Ljava/lang/String;)V", info as _),
        NativeMethod::new("error", "(Ljava/lang/String;)V", error as _)
    );

    let launcher_dir = launcher_dir().to_string_lossy().to_java_object(&env);
    let arr = script_candidates.to_java_object(&env);

    let main_class = env.find_class("mp/evolution/runtime/Runtime")
        .expect("Unable to find main class");

    let count = match env.call_static_method(main_class, "start", "(Ljava/lang/String;[Ljava/lang/String;)I", args![launcher_dir, arr]) {
        Err(e) if matches!(e.kind(), ErrorKind::JavaException) => {
            panic!("{}", get_last_exception(&env));
        },
        other => other.expect("Error invoking main function").i().unwrap(),
    };

    std::mem::forget(env); //Do not detach current thread

    for id in 0..count {
        runtime.register_script(&format!("vm:{}", id), ScriptJava { id })
    }

    crate::native::script::init(runtime);

    /*HOOKS.replace(Some(HashMap::new()));

    info!("Hooking natives");

    hook_native(0x7B5280EBA9840C72, |context| {
        let label = context.get_args().read::<&str>();
        let hash = label.joaat();
        crate::info!("Called GET_LABEL_TEXT for {} (0x{:08X})", label, hash.0);
        call_native_trampoline(0x7B5280EBA9840C72, context);
    });*/

    //crate::events::init(mem);
}

pub struct ScriptJava {
    id: i32
}

impl ScriptJava {
    fn get_java_object<'a>(&self) -> JObject<'a> {
        let env = attach_thread();
        let main_class = env.find_class("mp/evolution/runtime/Runtime").unwrap();
        env.call_static_method(main_class, "getContainer", "(I)Lmp/evolution/script/ScriptContainer;", args![self.id])
            .unwrap().l().unwrap()
    }
}

impl Script for ScriptJava {
    fn prepare(&mut self, env: ScriptEnv) {
        let env = attach_thread();
        let script = self.get_java_object();
        env.call_method(script, "prepare", "()V", args![]).expect("error calling `prepare` on vm script");
    }

    fn frame(&mut self, env: ScriptEnv, game_state: GameState) {
        let env = attach_thread();
        let script = self.get_java_object();
        match env.call_method(script, "frame", "()V", args![]) {
            Err(e) if matches!(e.kind(), ErrorKind::JavaException) => {
                panic!("{}", get_last_exception(&env));
            },
            other => other.expect("error calling `frame` on vm script")
        };
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        let env = attach_thread();
        let script = self.get_java_object();
        let event = match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(event) => {
                        match event {
                            KeyboardEvent::Key {
                                key,
                                repeats,
                                scan_code,
                                is_extended,
                                alt ,
                                shift,
                                control,
                                was_down_before,
                                is_up
                            } => {
                                env.new_object("mp/evolution/script/event/ScriptEventKeyboardKey", "(ISBZZZZZZ)V", args![
                                    *key, *repeats as i16, *scan_code as i8, *is_extended, *alt,
                                    *shift, *control, *was_down_before, *is_up
                                ]).unwrap()
                            },
                            KeyboardEvent::Char(c) => {
                                env.new_object("mp/evolution/script/event/ScriptEventKeyboardChar", "(C)V", args![
                                    *c as u16
                                ]).unwrap()
                            },
                        }
                    },/*
                    InputEvent::Mouse(event) => {

                    },*/
                    _ => return false
                }
            },
            ScriptEvent::JavaEvent(event) => *event,
            _ => return false
        };
        env.call_method(script, "event", "(Lmp/evolution/script/event/ScriptEvent;)Z", args![event]).unwrap().z().unwrap()
    }
}

unsafe extern "C" fn put_string(_env: &JNIEnv, args: JObject, value: JString) {
    let env = attach_thread();
    let buffer = env.get_field(args, "buffer", "Ljava/nio/ByteBuffer;").unwrap().l().unwrap();
    let ptr = env.get_string_utf_chars(value).unwrap();
    env.call_method(buffer, "putLong", "(J)Ljava/nio/ByteBuffer;", args![ptr as i64]).unwrap();
}

unsafe extern "C" fn get_string<'a>(_env: &'a JNIEnv, args: JObject) -> JString<'a> {
    let env = attach_thread();
    let buffer = env.get_field(args, "buffer", "Ljava/nio/ByteBuffer;").unwrap().l().unwrap();
    let ptr = env.call_method(buffer, "getLong", "()J", args![]).unwrap().j().unwrap() as u64 as *const i8;
    env.new_string(JNIStr::from_ptr(ptr).to_owned()).unwrap()
}

unsafe extern "C" fn wait(_env: &JNIEnv, _script: JObject, millis: u64) {
    if !ACTIVE_SCRIPT.is_null() {
        (&mut *ACTIVE_SCRIPT).wait(millis);
    }
}

unsafe extern "C" fn propagate(_env: &JNIEnv, _script: JObject, event: JObject<'static>) {
    if !ACTIVE_SCRIPT.is_null() {
        (&mut *ACTIVE_SCRIPT).propagate(ScriptEvent::JavaEvent(event));
    }
}

unsafe extern "C" fn info(_env: &JNIEnv, _class: JClass, line: JObject) {
    let env = attach_thread();
    let line = String::from_java_object(&env, line);
    crate::info!(target: "script", "{}", line);
}

unsafe extern "C" fn error(_env: &JNIEnv, _class: JClass, line: JObject) {
    let env = attach_thread();
    let line = String::from_java_object(&env, line);
    crate::error!(target: "script", "{}", line);
}

unsafe extern "C" fn invoke(_env: &JNIEnv, _class: JClass, hash: u64, args: JObject, result: JObject) {
    let env = attach_thread();
    if let Some(handler) = crate::native::get_handler_opt(hash) {
        let arg_count = env.call_method(args, "limit", "()I", &[]).unwrap().i().unwrap() as u32;
        let args = env.call_method(args, "address", "()J", &[]).unwrap().j().unwrap() as *mut u64;
        let result = env.call_method(result, "address", "()J", &[]).unwrap().j().unwrap() as *mut u64;
        let mut context = NativeCallContext::new_allocated(
            Box::from_raw(args as _),
            Box::from_raw(result as _),
            arg_count
        );
        handler(&mut context);
        std::mem::forget(context);
    } else {
        env.throw_new("Ljava/lang/IllegalArgumentException", format!("No such native: 0x{:016}", hash)).unwrap();
    }
}

fn get_trampoline(hash: u64) -> NativeFunction {
    let hooks = HOOKS.try_borrow().expect("unable to borrow hook map");
    let hooks = hooks.as_ref().expect("hook map is not initialized");
    let detour = hooks.get(&hash).expect(&format!("missing native trampoline for 0x{:016X}", hash));
    unsafe { std::mem::transmute(detour.trampoline()) }
}

pub fn call_native_trampoline(hash: u64, context: *mut NativeCallContext) {
    let trampoline = get_trampoline(hash);
    trampoline(context);
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
    terminated: bool,
    event_pool: Option<EventPool>
}

impl ScriptContainer {
    pub fn new<N, S>(name: N, script: S) -> ScriptContainer where N: Into<String>, S: Script + 'static {
        ScriptContainer {
            name: name.into(),
            fiber: None,
            main_fiber: None,
            wake_at: Instant::now(),
            terminated: false,
            script: Some(Box::new(script)),
            event_pool: None
        }
    }

    extern "system" fn fiber_loop(&mut self) {
        /*while **GAME_STATE != GameState::Playing {
            self.wait(0)
        }*/
        let mut script = self.script.take().unwrap();
        script.prepare(ScriptEnv::new(self));
        while !self.terminated {
            self.process_input(&mut script);
            script.frame(ScriptEnv::new(self), *GAME_STATE.as_ref());
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
            unsafe { ACTIVE_SCRIPT = self as *mut Self; }
            if let Some(fiber) = &self.fiber {
                fiber.make_current();
            } else {
                self.fiber = Some(Fiber::new(0, self, ScriptContainer::fiber_loop));
            }
            unsafe { ACTIVE_SCRIPT = std::ptr::null_mut(); }
        }
    }

    pub fn wait(&mut self, millis: u64) {
        unsafe { ACTIVE_SCRIPT = std::ptr::null_mut(); }
        self.wake_at = Instant::now() + Duration::from_millis(millis);
        self.main_fiber.as_mut().expect("missing main fiber").make_current();
    }

    pub fn propagate(&mut self, event: ScriptEvent) {
        let event_pool = self.event_pool.as_mut().expect("missing script event pool");
        event_pool.output.push_back(event);
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
    fn frame(&mut self, env: ScriptEnv, game_state: GameState);
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

    pub fn error<L>(&mut self, line: L) where L: Into<String> {
        self.event(ScriptEvent::ConsoleOutput(format!("~r~{}", line.into())));
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