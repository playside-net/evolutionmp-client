use crate::hash::{Hash, Hashable};
use crate::pattern::MemoryRegion;
use crate::{GameState, GAME_STATE, launcher_dir};
use crate::win::input::{KeyboardEvent, InputEvent, MouseEvent, MouseButton};
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
use crate::native::pool::Pool;

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

pub(crate) fn start(script_candidates: Vec<String>, vm: Arc<JavaVM>) {

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
            let class = match env.find_class($class_name) {
                Err(e) if matches!(e.kind(), ErrorKind::JavaException) => {
                    panic!("Unable to find class: {}", get_last_exception(&$env))
                },
                other => other.expect("Unable to find class")
            };
            $env.register_natives(class, vec![$($native),*]).unwrap();
        }};
    }

    macro_rules! pool {
        ($env:expr,$pool:expr,$class:literal) => {{
            extern "C" fn capacity(_env: &JNIEnv, _class: JClass) -> u32 {
                $pool.capacity()
            }
            extern "C" fn is_valid(_env: &JNIEnv, _class: JClass, index: u32) -> bool {
                $pool.is_valid(index)
            }
            extern "C" fn get_address(_env: &JNIEnv, _class: JClass, index: u32) -> u64 {
                $pool.get_address(index) as u64
            }
            natives!($env, $class,
                NativeMethod::new("capacity", "()I", capacity as _),
                NativeMethod::new("isValid", "(I)Z", is_valid as _),
                NativeMethod::new("getAddress", "(I)J", get_address as _)
            );
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
        //NativeMethod::new("yield", "(J)V", wait as _),
        NativeMethod::new("propagate", "(Lmp/evolution/script/event/ScriptEvent;)V", propagate as _)
    );
    natives!(env, "mp/evolution/script/ScriptPrintStream",
        NativeMethod::new("info", "(Ljava/lang/String;)V", info as _),
        NativeMethod::new("error", "(Ljava/lang/String;)V", error as _)
    );
    natives!(env, "mp/evolution/game/entity/pool/Pool",
        NativeMethod::new("isGlobalFull", "()Z", crate::native::pool::is_global_full as _),
        NativeMethod::new("requestHandle", "(J)I", crate::native::pool::request_handle as _),
        NativeMethod::new("getPosition", "(JJ)J", crate::native::pool::get_entity_pos as _)
    );

    macro_rules! g  {
        ($handle: ty, $ty: ty, $vm_name: literal, $vm_sig: literal, $name: ident) => ({
            extern "C" fn get(_env: &JNIEnv, obj: JObject) -> $ty {
                use $crate::native::pool::Handleable;
                let handle = i32::from_java_field(&attach_thread(), obj, "handle");
                <$handle>::from_handle(handle as u32).unwrap().$name()
            }
            NativeMethod::new($vm_name, $vm_sig, get as _)
        })
    }

    macro_rules! s  {
        ($handle: ty, $ty:ty, $vm_name: literal, $vm_sig: literal, $name: ident) => ({
            extern "C" fn set(_env: &JNIEnv, obj: JObject, value: $ty) {
                use $crate::native::pool::Handleable;
                let handle = i32::from_java_field(&attach_thread(), obj, "handle");
                <$handle>::from_handle(handle as u32).unwrap().$name(value)
            }
            NativeMethod::new($vm_name, $vm_sig, set as _)
        })
    }

    natives!(env, "mp/evolution/game/entity/vehicle/Vehicle",
        g!(Vehicle, bool, "isInteriorLight", "()Z", is_interior_light),
        g!(Vehicle, bool, "isHandbrake", "()Z", is_handbrake),
        g!(Vehicle, u8, "getIndicatorLight", "()B", get_indicator_light),
        g!(Vehicle, i32, "getCurrentGear", "()I", get_current_gear),
        s!(Vehicle, i32, "setCurrentGear", "(I)V", set_current_gear),
        g!(Vehicle, i32, "getHighGear", "()I", get_high_gear),
        s!(Vehicle, i32, "setHighGear", "(I)V", set_high_gear),
        g!(Vehicle, f32, "getCurrentRPM", "()F", get_current_rpm),
        s!(Vehicle, f32, "setCurrentRPM", "(F)V", set_current_rpm),
        g!(Vehicle, f32, "getDashboardSpeed", "()F", get_dashboard_speed),
        g!(Vehicle, f32, "getWheelSpeed", "()F", get_wheel_speed),
        s!(Vehicle, f32, "setWheelSpeed", "(F)V", set_wheel_speed),
        g!(Vehicle, f32, "getAcceleration", "()F", get_acceleration),
        s!(Vehicle, f32, "setAcceleration", "(F)V", set_acceleration),
        g!(Vehicle, f32, "getFuel", "()F", get_fuel),
        s!(Vehicle, f32, "setFuel", "(F)V", set_fuel),
        g!(Vehicle, f32, "getMaxOil", "()F", get_max_oil),
        g!(Vehicle, f32, "getOil", "()F", get_oil),
        s!(Vehicle, f32, "setOil", "(F)V", set_oil),
        g!(Vehicle, f32, "getEngineTemperature", "()F", get_engine_temperature),
        s!(Vehicle, f32, "setEngineTemperature", "(F)V", set_engine_temperature),
        g!(Vehicle, f32, "getEnginePower", "()F", get_engine_power),
        g!(Vehicle, f32, "getBrakePower", "()F", get_brake_power)
    );
    pool!(env, crate::game::vehicle::get_pool(), "mp/evolution/game/entity/vehicle/VehiclePool");
    pool!(env, crate::game::prop::get_pool(), "mp/evolution/game/entity/prop/PropPool");
    pool!(env, crate::game::ped::get_pool(), "mp/evolution/game/entity/ped/PedPool");

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

    std::mem::forget(env);

    let mut java_scripts = JAVA_SCRIPTS.lock().unwrap();
    for id in 0..count {
        java_scripts.push(id);
    }
}

lazy_static::lazy_static! {
    pub static ref JAVA_SCRIPTS: Mutex<Vec<i32>> = Mutex::new(Vec::new());
}

pub struct ScriptJava {}

impl ScriptJava {
    pub fn new() -> ScriptJava {
        ScriptJava {}
    }

    fn get_java_object<'a>(&self, id: i32) -> JObject<'a> {
        let env = attach_thread();
        let main_class = env.find_class("mp/evolution/runtime/Runtime").unwrap();
        env.call_static_method(main_class, "getContainer", "(I)Lmp/evolution/script/ScriptContainer;", args![id])
            .unwrap().l().unwrap()
    }
}

impl Script for ScriptJava {
    fn frame(&mut self) {
        if crate::game::is_loaded() {
            let env = attach_thread();
            let scripts = JAVA_SCRIPTS.lock().unwrap();

            for id in scripts.iter() {
                let script = self.get_java_object(*id);
                env.call_method(script, "frame", "()V", args![]).expect("error calling `frame` on vm script");
            }
        }
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        if crate::game::is_loaded() {
            let env = attach_thread();
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
                                    alt,
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
                _ => return false
            };
            let mut result = false;
            for id in JAVA_SCRIPTS.lock().unwrap().iter() {
                let script = self.get_java_object(*id);
                result |= env.call_method(script, "event", "(Lmp/evolution/script/event/ScriptEvent;)Z", args![event]).unwrap().z().unwrap()
            }
            result
        } else {
            false
        }
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
    crate::game::script::wait(millis);
}

unsafe extern "C" fn propagate(_env: &JNIEnv, _script: JObject, event: JObject<'static>) {
    unimplemented!()
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
        crate::native::CURRENT_NATIVE.store(hash, Ordering::SeqCst);
        handler(&mut context);
        crate::native::CURRENT_NATIVE.store(0, Ordering::SeqCst);
        std::mem::forget(context);
    } else {
        env.throw_new("Ljava/lang/IllegalArgumentException", format!("No such native: 0x{:016}", hash)).unwrap();
    }
}

pub struct TaskQueue {
    tasks: VecDeque<Box<dyn FnMut()>>
}

impl TaskQueue {
    pub fn new() -> TaskQueue {
        TaskQueue {
            tasks: VecDeque::new()
        }
    }

    pub fn push<F>(&mut self, task: F) where F: FnMut() + 'static {
        self.tasks.push_back(Box::new(task))
    }

    pub fn process(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            task();
        }
    }
}

pub trait Script {
    fn frame(&mut self);
    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool;
}

pub struct ScriptEnv {}