use crate::hash::Hash;
use crate::pattern::MemoryRegion;
use crate::native::collection::PtrCollection;
use crate::GameState;
use crate::win::input::{KeyboardEvent, InputEvent, MouseEvent, MouseButton, InputHook};
use crate::native::{NativeCallContext, NativeStackValue};
use crate::hash::joaat;
use crate::win::thread::Fiber;
use crate::{info, error};
use std::os::raw::c_char;
use std::ffi::CString;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::path::Path;
use detour::static_detour;
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::minwindef::{LPVOID, DWORD, TRUE};
use winapi::um::winuser::VK_RETURN;
use winapi::_core::panic::PanicInfo;
use jni_dynamic::{InitArgsBuilder, JNIVersion, JavaVM, NativeMethod};
use jni_dynamic::objects::{JValue, JObject};
use jni_dynamic::sys::{jlong, jobject, jobjectArray, JNINativeInterface_};
use winapi::ctypes::c_void;

const ACTIVE_THREAD_TLS_OFFSET: isize = 0x830;

pub(crate) static mut RUNTIME: Option<Runtime> = None;
pub(crate) static mut VM: Option<Arc<JavaVM>> = None;
pub(crate) static mut CONSOLE_VISIBLE: bool = false;

static_detour! {
    static GetFrameCountHook: extern "C" fn(*mut NativeCallContext);
    static ReturnTrueFromScriptHook: extern "C" fn(*mut c_void, *mut c_void) -> bool;
}

fn get_frame_count_native(context: *mut NativeCallContext) {
    unsafe {
        loop {
            if let Some(runtime) = RUNTIME.as_mut() {
                runtime.frame();
                break;
            }
        }

    }
    GetFrameCountHook.call(context)
}

fn return_true_from_script(arg1: *mut c_void, arg2: *mut c_void) -> bool {
    true
}

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

    pub fn iterate<F>(&mut self, mut handler: F) where F: FnMut(&mut ScriptEvent) -> bool {
        for i in 0..self.input.len() {
            if handler(&mut self.input[i]) {
                self.input.remove(i);
            }
        }
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

pub(crate) unsafe fn start(mem: &MemoryRegion, input: InputHook) {
    let natives = crate::native::NATIVES.as_mut().expect("Natives aren't initialized yet");

    /*let mut dump = Vec::new();
    unsafe { natives.dump(&mut dump) };
    info!("Dumping natives: {:#?}", dump);*/

    let get_frame_count = natives.get_handler(0xFC8202EFC642E6F2)
        .expect("Unable to get native handler for GET_FRAME_COUNT");
    let return_true = mem.find("74 3C 48 8B 01 FF 50 10 84 C0")
        .next()
        .expect("Unable to find return_true_from_script").get::<u8>();
    GetFrameCountHook
        .initialize(get_frame_count, get_frame_count_native).expect("GET_FRAME_COUNT hook initialization failed")
        .enable().expect("GET_FRAME_COUNT hook enabling failed");
    ReturnTrueFromScriptHook
        .initialize(std::mem::transmute(return_true), return_true_from_script).expect("return_true_from_script hook initialization failed")
        .enable().expect("return_true_from_script hook enabling failed");

    let mut runtime = Runtime::new(input);
    info!("Initializing multiplayer");
    crate::multiplayer::init(&mut runtime);
    RUNTIME = Some(runtime);

    let launcher_path = Path::new("C:/Users/Виктор/Desktop/Проекты/Rust/evolutionmp-client");
    //start_vm(launcher_path);
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
    fn event(&mut self, event: &mut ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool { false }
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
}

pub enum ScriptEvent {
    ConsoleInput(String),
    ConsoleOutput(String),
    UserInput(InputEvent)
}

extern "C" fn invoke(itf: *mut *const JNINativeInterface_, hash: jlong, args: jobjectArray) -> jlong {
    crate::info_message!("Info", "Hello from java!");
    0
}

unsafe fn start_vm(launcher_path: &Path) {
    let java_exe = launcher_path.join("java").join("bin").join("server").join("jvm.dll");
    let jar_path = launcher_path.join("client-rt.jar");
    crate::info_message!("Info", "Jar path is {:?}", jar_path);
    let mut java_args = InitArgsBuilder::new()
        .version(JNIVersion::V8)
        .option(&format!("-Djava.class.path={}", jar_path.to_str().unwrap()));

    let java_args = java_args.build().expect("Error building JVM args");

    let vm = Arc::new(
        JavaVM::new(java_exe, java_args).expect("Error creating JVM")
    );

    VM = Some(vm.clone());

    std::thread::Builder::new().name(String::from("vm")).spawn(move || {
        let env = vm.attach_current_thread().expect("Thread attach failed");

        let string_class = env.find_class("java/lang/String").unwrap();
        let thread_class = env.find_class("java/lang/Thread").unwrap();
        let file_class = env.find_class("java/io/File").unwrap();

        let current_thread = env.call_static_method(thread_class, "currentThread", "()Ljava/lang/Thread;", &[]).unwrap();

        let thread_name = env.new_string("vm").unwrap();

        /*{
            let s = env.new_string(".").unwrap();
            let file = env.new_object(file_class, "(Ljava/lang/String;)V", &[JValue::Object(s.into())]).unwrap();
            let dir = env.call_method(file, "getAbsolutePath", "()Ljava/lang/String;", &[]).unwrap();
            let dir = env.get_string(JString::from(dir.l().unwrap())).unwrap();
            info!("JVM working dir is: {}", dir.to_str().unwrap());
        }*/

        env.call_method(current_thread.l().unwrap(), "setName", "(Ljava/lang/String;)V", &[JValue::Object(thread_name.into())])
            .expect("Unable to set main jvm thread name");

        let script_class = env.find_class("mp/evolution/script/Script")
            .expect("Unable to find script class");

        env.register_natives(script_class, vec![
            NativeMethod::new("invoke", "(JLjava/lang/Object;)J", invoke as *mut ())
        ]).unwrap();

        let arr = env.new_object_array(0, string_class, JObject::null()).unwrap();
        let main_class = env.find_class("mp/evolution/Main")
            .expect("Unable to find main class");
        env.call_static_method(main_class, "main", "([Ljava/lang/String;)V", &[JValue::Object(arr.into())])
            .expect("Error invoking main function");
    }).unwrap();
}