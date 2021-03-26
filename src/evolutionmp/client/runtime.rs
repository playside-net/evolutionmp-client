use std::sync::Arc;
use std::sync::atomic::Ordering;

use jni_dynamic::{JavaVM, JNIEnv, NativeMethod};
use jni_dynamic::errors::ErrorKind;
use jni_dynamic::objects::{JClass, JObject, JString, JMethodID, JValue, JFieldID, JStaticFieldID, GlobalRef};
use jni_dynamic::strings::JNIStr;

use crate::{args, args_v, java_static_method};
use crate::events::ScriptEvent;
use crate::game::vehicle::Vehicle;
use crate::jni::{JavaObject, JavaValue};
use crate::jni::attach_thread;
use crate::launcher_dir;
use crate::native::{NativeCallContext, ThreadSafe};
use crate::native::pool::Pool;
use crate::win::input::{InputEvent, KeyboardEvent};
use jni_dynamic::sys::jint;
use crate::client::game::ped::Ped;
use crate::client::game::interior::Interior;
use crate::client::game::entity::Entity;
use crate::client::native::NativeVector3;
use crate::client::native::pool::Handleable;
use jni_dynamic::signature::JavaType;
use jni_dynamic::signature::Primitive::{Void, Int};

java_static_method!(set_system_property, "java/lang/System", "setProperty", fn(property: &str, value: &str) -> Option<String>);

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

macro_rules! class_id {
    ($name: ident, $cls:literal) => {
        lazy_static! {
            static ref $name: GlobalRef = {
                let env = attach_thread();
                let cls = env.find_class($cls).unwrap();
                env.new_global_ref(*cls).unwrap()
            };
        }
    };
}

macro_rules! static_field_id {
    ($name: ident, $cls:literal, $f_name: literal, $sig: literal) => {
        lazy_static! {
            static ref $name: ThreadSafe<JStaticFieldID<'static>> = {
                let env = attach_thread();
                let cls = env.find_class($cls).unwrap();
                ThreadSafe::new(env.get_static_field_id(cls, $f_name, $sig).unwrap())
            };
        }
    };
}

macro_rules! method_id {
    ($name: ident, $cls:literal, $fn_name: literal, $sig: literal) => {
        lazy_static! {
            static ref $name: ThreadSafe<JMethodID<'static>> = {
                let env = attach_thread();
                let cls = env.find_class($cls).unwrap();
                ThreadSafe::new(env.get_method_id(cls, $fn_name, $sig).unwrap())
            };
        }
    };
}

class_id!(RUNTIME, "mp/evolution/runtime/Runtime");
class_id!(KEY_EVENT, "mp/evolution/script/event/ScriptEventKeyboardKey");
class_id!(CHAR_EVENT, "mp/evolution/script/event/ScriptEventKeyboardChar");

static_field_id!(INSTANCE, "mp/evolution/runtime/Runtime", "INSTANCE", "Lmp/evolution/runtime/Runtime;");

method_id!(SCRIPT_FRAME, "mp/evolution/runtime/Runtime", "frame", "()V");
method_id!(SCRIPT_EVENT, "mp/evolution/runtime/Runtime", "event", "(Lmp/evolution/script/event/ScriptEvent;)V");
method_id!(HANDLED_HANDLE, "mp/evolution/invoke/Handled", "handle", "()I");
method_id!(NEW_KEY_EVENT, "mp/evolution/script/event/ScriptEventKeyboardKey", "<init>", "(ISBZZZZZZ)V");
method_id!(NEW_CHAR_EVENT, "mp/evolution/script/event/ScriptEventKeyboardChar", "<init>", "(Ljava/lang/String;)V");

pub(crate) fn start(vm: Arc<JavaVM>) {
    unsafe { crate::jni::set_vm(vm); };

    let env = attach_thread();

    info!("setting user.dir");

    set_system_property("user.dir", &launcher_dir().display().to_string());

    env.set_static_field("java/lang/ClassLoader", "sys_paths", "[Ljava/lang/String;", JObject::null()).unwrap();
    let fs_holder_class = env.find_class("java/nio/file/FileSystems$DefaultFileSystemHolder").unwrap();
    let fs = env.call_static_method(fs_holder_class, "defaultFileSystem", "()Ljava/nio/file/FileSystem;", &[])
        .unwrap().l().unwrap();
    env.set_static_field("java/nio/file/FileSystems$DefaultFileSystemHolder", "defaultFileSystem", "Ljava/nio/file/FileSystem;", fs)
        .unwrap();

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

    let bin_dir = launcher_dir().join("bin");

    const LIBS: [&'static str; 5] = [
        "commons-io-2.5.jar",
        "json-simple-1.1.1.jar",
        "netty-all-4.1.30.Final.jar",
        "shared.jar",
        "csl.jar"
    ];

    for lib in &LIBS {
        let path = bin_dir.join(lib).to_string_lossy().to_java_object(&env);
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
            extern fn capacity(_env: &JNIEnv, _class: JClass) -> u32 {
                $pool.capacity()
            }
            extern fn is_valid(_env: &JNIEnv, _class: JClass, index: u32) -> bool {
                $pool.is_valid(index)
            }
            extern fn get_address(_env: &JNIEnv, _class: JClass, index: u32) -> u64 {
                $pool.get_address(index) as u64
            }
            natives!($env, $class,
                NativeMethod::new("capacity", "()I", capacity as _),
                NativeMethod::new("isValid", "(I)Z", is_valid as _),
                NativeMethod::new("getAddress", "(I)J", get_address as _)
            );
        }};
    }

    extern fn restart(_env: &JNIEnv, _obj: JObject) {
        info!("restart requested");
        crate::game::restart();
    }

    extern fn pid(_env: &JNIEnv, _obj: JObject) -> jint {
        std::process::id() as _
    }

    natives!(env, "mp/evolution/invoke/NativeArgs",
        NativeMethod::new("getStringUTFChars", "(Ljava/lang/String;)J", get_string_utf_chars as _)
    );
    natives!(env, "mp/evolution/invoke/NativeResult",
        NativeMethod::new("getStringFromUTFChars", "(J)Ljava/lang/String;", get_string_from_utf_chars as _)
    );
    natives!(env, "mp/evolution/invoke/Native",
        NativeMethod::new("invoke", "(JJIJ)V", invoke as _)
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

    fn get_handle(env: &JNIEnv, obj: JObject) -> u32 {
        env.call_method_unchecked_fast(obj, **HANDLED_HANDLE, JavaType::Primitive(Int), &[])
            .unwrap().i().unwrap() as u32
    }

    macro_rules! g {
        ($handle: ty, $ty: ty, $vm_name: literal, $vm_sig: literal, $name: ident) => ({
            extern fn get(_env: &JNIEnv, obj: JObject) -> $ty {
                use $crate::native::pool::Handleable;
                let handle = get_handle(&attach_thread(), obj);
                <$handle>::from_handle(handle as u32).unwrap().$name()
            }
            NativeMethod::new($vm_name, $vm_sig, get as _)
        })
    }

    macro_rules! s {
        ($handle: ty, $ty:ty, $vm_name: literal, $vm_sig: literal, $name: ident) => ({
            extern fn set(_env: &JNIEnv, obj: JObject, value: $ty) {
                use $crate::native::pool::Handleable;
                let handle = get_handle(&attach_thread(), obj);
                <$handle>::from_handle(handle as u32).unwrap().$name(value)
            }
            NativeMethod::new($vm_name, $vm_sig, set as _)
        })
    }

    natives!(env, "mp/evolution/game/entity/vehicle/Vehicle",
        g!(Vehicle, u32, "getLightFlags", "()I", get_light_flags),
        g!(Vehicle, bool, "isEngineStarting", "()Z", is_engine_starting),
        g!(Vehicle, bool, "isInteriorLight", "()Z", is_interior_light),
        g!(Vehicle, bool, "isHandbrake", "()Z", is_handbrake),
        g!(Vehicle, u8, "getIndicatorLight", "()B", get_indicator_light),
        g!(Vehicle, u8, "getNextGear", "()B", get_next_gear),
        s!(Vehicle, u8, "setNextGear", "(B)V", set_next_gear),
        g!(Vehicle, u8, "getCurrentGear", "()B", get_current_gear),
        s!(Vehicle, u8, "setCurrentGear", "(B)V", set_current_gear),
        g!(Vehicle, u8, "getHighGear", "()B", get_high_gear),
        s!(Vehicle, u8, "setHighGear", "(B)V", set_high_gear),
        g!(Vehicle, f32, "getCurrentRPM", "()F", get_rpm),
        s!(Vehicle, f32, "setCurrentRPM", "(F)V", set_rpm),
        g!(Vehicle, f32, "getTurbo", "()F", get_turbo),
        s!(Vehicle, f32, "setTurbo", "(F)V", set_turbo),
        g!(Vehicle, f32, "getDashboardSpeed", "()F", get_dashboard_speed),
        g!(Vehicle, f32, "getWheelSpeed", "()F", get_wheel_speed),
        s!(Vehicle, f32, "setWheelSpeed", "(F)V", set_wheel_speed),
        g!(Vehicle, f32, "getThrottle", "()F", get_throttle),
        s!(Vehicle, f32, "setThrottle", "(F)V", set_throttle),
        g!(Vehicle, f32, "getThrottlePower", "()F", get_throttle_power),
        s!(Vehicle, f32, "setThrottlePower", "(F)V", set_throttle_power),
        g!(Vehicle, f32, "getFuel", "()F", get_fuel),
        s!(Vehicle, f32, "setFuel", "(F)V", set_fuel),
        g!(Vehicle, f32, "getMaxOil", "()F", get_max_oil),
        g!(Vehicle, f32, "getOil", "()F", get_oil),
        s!(Vehicle, f32, "setOil", "(F)V", set_oil),
        g!(Vehicle, f32, "getClutch", "()F", get_clutch),
        s!(Vehicle, f32, "setClutch", "(F)V", set_clutch),
        g!(Vehicle, f32, "getEngineTemperature", "()F", get_engine_temperature),
        s!(Vehicle, f32, "setEngineTemperature", "(F)V", set_engine_temperature),
        g!(Vehicle, f32, "getEnginePower", "()F", get_engine_power),
        g!(Vehicle, f32, "getBrakePower", "()F", get_brake_power),
        g!(Vehicle, f32, "getSteeringAngle", "()F", get_steering_angle),
        s!(Vehicle, f32, "setSteeringAngle", "(F)V", set_steering_angle),
        g!(Vehicle, f32, "getSteeringScale", "()F", get_steering_scale),
        s!(Vehicle, f32, "setSteeringScale", "(F)V", set_steering_scale)
    );
    pool!(env, crate::game::vehicle::get_pool(), "mp/evolution/game/entity/vehicle/VehiclePool");
    pool!(env, crate::game::prop::get_pool(), "mp/evolution/game/entity/prop/PropPool");
    pool!(env, crate::game::ped::get_pool(), "mp/evolution/game/entity/ped/PedPool");

    natives!(env, "mp/evolution/runtime/Runtime",
        NativeMethod::new("restart", "()V", restart as _),
        NativeMethod::new("pid", "()I", pid as _)
    );

    let _ = get_runtime(&env);
}

fn get_runtime<'a>(env: &'a JNIEnv) -> JObject<'a> {
    env.get_static_field_unchecked_fast(RUNTIME.as_obj().into(), **INSTANCE, JavaType::Object(String::new()))
        .unwrap().l().unwrap()
}

pub struct ScriptJava {}

impl ScriptJava {
    pub fn new() -> ScriptJava {
        ScriptJava {}
    }
}

impl Script for ScriptJava {
    fn frame(&mut self) {
        if crate::game::is_loaded() {
            let env = attach_thread();
            let runtime = get_runtime(&env);
            env.call_method_unchecked_fast(runtime, **SCRIPT_FRAME, JavaType::Primitive(Void), &[])
                .expect("error calling `frame`");
        }
    }

    fn event(&mut self, event: ScriptEvent) {
        if crate::game::is_loaded() {
            let env = attach_thread();
            let event = match event {
                ScriptEvent::UserInput(event) => {
                    match event {
                        InputEvent::Keyboard(event) => {
                            match event {
                                KeyboardEvent::Key {
                                    key, repeats, scan_code, is_extended,
                                    alt, shift, control, was_down_before,
                                    is_up
                                } => {
                                    env.new_object_unchecked_fast(KEY_EVENT.as_obj().into(), **NEW_KEY_EVENT, args_v![
                                        key, repeats as i16, scan_code as i8, is_extended, alt,
                                        shift, control, was_down_before, is_up
                                    ]).unwrap()
                                }
                                KeyboardEvent::Char(c) => {
                                    let c = c.to_java_object(&env);
                                    env.new_object_unchecked_fast(CHAR_EVENT.as_obj().into(), **NEW_CHAR_EVENT, args_v![
                                        c
                                    ]).unwrap()
                                }
                            }
                        }/*
                        InputEvent::Mouse(event) => {

                        },*/
                        _ => return
                    }
                }
                _ => return
            };
            let runtime = get_runtime(&env);
            env.call_method_unchecked_fast(runtime, **SCRIPT_EVENT, JavaType::Primitive(Void), &[JValue::Object(event).to_jni()])
                .expect("error calling `event`");
        }
    }
}

unsafe extern fn get_string_utf_chars(_env: &JNIEnv, _class: JClass, value: JString) -> *const i8 {
    let env = attach_thread();
    env.get_string_utf_chars(value).unwrap()
}

unsafe extern fn get_string_from_utf_chars<'a>(_env: &'a JNIEnv, _class: JClass, ptr: *const i8) -> JString<'a> {
    let env = attach_thread();
    env.new_string(JNIStr::from_ptr(ptr).to_owned()).unwrap()
}

unsafe extern fn propagate(_env: &JNIEnv, _script: JObject, _event: JObject<'static>) {
    unimplemented!()
}

unsafe extern fn info(_env: &JNIEnv, _class: JClass, line: JObject) {
    let env = attach_thread();
    let line = String::from_java_object(&env, line);
    info!(target: "script", "{}", line);
}

unsafe extern fn error(_env: &JNIEnv, _class: JClass, line: JObject) {
    let env = attach_thread();
    let line = String::from_java_object(&env, line);
    error!(target: "script", "{}", line);
}

unsafe extern fn invoke(_env: &JNIEnv, _class: JClass, hash: u64, args: Box<[u64; 32]>, arg_count: u32, result: Box<[u64; 3]>) {
    if let Some(handler) = crate::native::get_handler_opt(hash) {
        let mut context = NativeCallContext::new_allocated(args, result, arg_count);
        crate::native::CURRENT_NATIVE.store(hash, Ordering::SeqCst);
        handler(&mut context);
        crate::native::SET_VECTOR_RESULTS(&mut context); //Flush all &mut NativeVector3 args
        crate::native::CURRENT_NATIVE.store(0, Ordering::SeqCst);
        std::mem::forget(context); //Do not free java nio buffers
    } else {
        let env = attach_thread();
        env.throw_new("java/lang/IllegalArgumentException", format!("No such native: 0x{:016}", hash)).unwrap();
    }
}

pub trait Script {
    fn frame(&mut self);
    fn event(&mut self, event: ScriptEvent);
}

pub struct ScriptEnv {}