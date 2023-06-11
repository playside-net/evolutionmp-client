use std::sync::Arc;
use std::sync::atomic::Ordering;

use jni_dynamic::{JavaVM, JNIEnv, NativeMethod};
use jni_dynamic::errors::ErrorKind;
use jni_dynamic::objects::{JClass, JObject, JString, JMethodID, JValue, JStaticFieldID, GlobalRef};
use jni_dynamic::strings::JNIStr;

use crate::{args, args_v, call, java_static_method};
use crate::events::ScriptEvent;
use crate::game::vehicle::Vehicle;
use crate::jni::{JavaObject, JavaValue};
use crate::jni::attach_thread;
use crate::launcher_dir;
use crate::native::{NativeCallContext, ThreadSafe};
use crate::native::pool::Pool;
use crate::win::input::{InputEvent, KeyboardEvent};
use jni_dynamic::sys::jint;
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

static mut LOADER: Option<GlobalRef> = None;

fn load_class<'a>(env: &'a JNIEnv, name: &str) -> JClass<'a> {
    let name = name.to_java_value(&env);
    let loader = unsafe { LOADER.as_ref().unwrap() };
    call!(env, env.call_method(loader.as_obj(), "loadClass", "(Ljava/lang/String;Z)Ljava/lang/Class;", args![name, true]))
        .l().unwrap().into()
}

lazy_static! {
    static ref RUNTIME: GlobalRef = {
        let env = attach_thread();
        let cls = load_class(&env, "mp.evolution.runtime.Runtime");
        let runtime = env.get_static_field_unchecked_fast(cls, **INSTANCE, JavaType::Object(String::new()))
            .unwrap().l().unwrap();
        env.new_global_ref(runtime).unwrap()
    };
}

macro_rules! class_id {
    ($name: ident, $cls:literal) => {
        lazy_static! {
            static ref $name: GlobalRef = {
                let env = attach_thread();
                let cls = load_class(&env, $cls);
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
                let cls = load_class(&env, $cls);
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
                let cls = load_class(&env, $cls);
                ThreadSafe::new(env.get_method_id(cls, $fn_name, $sig).unwrap())
            };
        }
    };
}

class_id!(KEY_EVENT, "mp.evolution.script.event.ScriptEventKeyboardKey");
class_id!(CHAR_EVENT, "mp.evolution.script.event.ScriptEventKeyboardChar");

static_field_id!(INSTANCE, "mp.evolution.runtime.Runtime", "INSTANCE", "Lmp/evolution/runtime/Runtime;");

method_id!(SCRIPT_FRAME, "mp.evolution.runtime.Runtime", "frame", "()V");
method_id!(SCRIPT_EVENT, "mp.evolution.runtime.Runtime", "event", "(Lmp/evolution/script/event/ScriptEvent;)V");
method_id!(HANDLED_HANDLE, "mp.evolution.invoke.Handled", "handle", "()I");
method_id!(NEW_KEY_EVENT, "mp.evolution.script.event.ScriptEventKeyboardKey", "<init>", "(ISBZZZZZZ)V");
method_id!(NEW_CHAR_EVENT, "mp.evolution.script.event.ScriptEventKeyboardChar", "<init>", "(Ljava/lang/String;)V");

pub(crate) fn start(vm: Arc<JavaVM>) {
    unsafe { crate::jni::set_vm(vm); };

    let env = attach_thread();

    info!("setting user.dir");

    set_system_property("user.dir", &launcher_dir().display().to_string());
    set_system_property("java.class.path", &LIBS.join(";"));

    call!(env, env.call_static_method("jdk/internal/util/StaticProperty", "<clinit>", "()V", &[]));
    call!(env, env.call_static_method("jdk/internal/loader/NativeLibraries$LibraryPaths", "<clinit>", "()V", &[]));

    let dir = launcher_dir().display().to_string().to_java_value(&env);

    let nio_fs_class = call!(env, env.find_class("java/nio/file/FileSystems$DefaultFileSystemHolder"));
    let def_nio_fs = call!(env, env.get_static_field(nio_fs_class, "defaultFileSystem", "Ljava/nio/file/FileSystem;")).l().unwrap();
    call!(env, env.set_field(def_nio_fs, "defaultDirectory", "Ljava/lang/String;", dir));
    let file_class = call!(env, env.find_class("java/io/File"));
    let def_io_fs = call!(env, env.get_static_field(file_class, "fs", "Ljava/io/FileSystem;")).l().unwrap();
    let normalized_dir = call!(env, env.call_method(def_io_fs, "normalize", "(Ljava/lang/String;)Ljava/lang/String;", &[dir]));
    call!(env, env.set_field(def_io_fs, "userDir", "Ljava/lang/String;", normalized_dir));
    call!(env, env.call_static_method("java/io/FilePermission", "<clinit>", "()V", &[]));

    let thread_class = env.find_class("java/lang/Thread").unwrap();

    let current_thread = call!(env, env.call_static_method(thread_class, "currentThread", "()Ljava/lang/Thread;", &[])).l().unwrap();

    let thread_name = "game".to_java_object(&env);
    env.call_method(current_thread, "setName", "(Ljava/lang/String;)V", args![thread_name])
        .expect("Unable to set game thread name");

    let system_loader = call!(env, env.call_static_method("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", &[])).l().unwrap();
    let urls = call!(env, env.new_object_array(0, "java/net/URL", JObject::null()));
    let loader = call!(env, env.new_object("java/net/URLClassLoader", "([Ljava/net/URL;Ljava/lang/ClassLoader;)V", args![JObject::from(urls), system_loader]));

    unsafe {
        LOADER = Some(env.new_global_ref(loader).unwrap());
    }

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
        env.call_method(loader, "addURL", "(Ljava/net/URL;)V", args![url]).unwrap();
    }

    macro_rules! natives {
        ($env:expr,$class_name:literal,$($native:expr),*) => {{
            let class = load_class(&env, $class_name);
            /*let class = match env.find_class($class_name) {
                Err(e) if matches!(e.kind(), ErrorKind::JavaException) => {
                    panic!("Unable to find class: {}", get_last_exception(&$env))
                },
                other => other.expect("Unable to find class")
            };*/
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

    natives!(env, "mp.evolution.invoke.NativeArgs",
        NativeMethod::new("getStringUTFChars", "(Ljava/lang/String;)J", get_string_utf_chars as _)
    );
    natives!(env, "mp.evolution.invoke.NativeResult",
        NativeMethod::new("getStringFromUTFChars", "(J)Ljava/lang/String;", get_string_from_utf_chars as _)
    );
    natives!(env, "mp.evolution.invoke.Native",
        NativeMethod::new("invoke", "(JJIJ)V", invoke as _)
    );
    natives!(env, "mp.evolution.script.Script",
        //NativeMethod::new("yield", "(J)V", wait as _),
        NativeMethod::new("propagate", "(Lmp/evolution/script/event/ScriptEvent;)V", propagate as _)
    );
    natives!(env, "mp.evolution.script.ScriptPrintStream",
        NativeMethod::new("info", "(Ljava/lang/String;)V", info as _),
        NativeMethod::new("error", "(Ljava/lang/String;)V", error as _)
    );
    natives!(env, "mp.evolution.game.entity.pool.Pool",
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
            extern fn get(_env: &JNIEnv, _class: JClass, handle: u32) -> $ty {
                use $crate::native::pool::Handleable;
                <$handle>::from_handle(handle as u32).unwrap().$name()
            }
            NativeMethod::new($vm_name, $vm_sig, get as _)
        })
    }

    macro_rules! s {
        ($handle: ty, $ty:ty, $vm_name: literal, $vm_sig: literal, $name: ident) => ({
            extern fn set(_env: &JNIEnv, _class: JClass, handle: u32, value: $ty) {
                use $crate::native::pool::Handleable;
                <$handle>::from_handle(handle as u32).unwrap().$name(value)
            }
            NativeMethod::new($vm_name, $vm_sig, set as _)
        })
    }

    natives!(env, "mp.evolution.game.entity.vehicle.Vehicle",
        g!(Vehicle, u32, "getLightFlags", "(I)I", get_light_flags),
        s!(Vehicle, u32, "setLightFlags", "(II)V", set_light_flags),
        g!(Vehicle, bool, "isEngineStarting", "(I)Z", is_engine_starting),
        g!(Vehicle, bool, "isInteriorLight", "(I)Z", is_interior_light),
        g!(Vehicle, bool, "isHandbrake", "(I)Z", is_handbrake),
        g!(Vehicle, u8, "getNextGear", "(I)B", get_next_gear),
        s!(Vehicle, u8, "setNextGear", "(IB)V", set_next_gear),
        g!(Vehicle, u8, "getCurrentGear", "(I)B", get_current_gear),
        s!(Vehicle, u8, "setCurrentGear", "(IB)V", set_current_gear),
        g!(Vehicle, u8, "getHighGear", "(I)B", get_high_gear),
        s!(Vehicle, u8, "setHighGear", "(IB)V", set_high_gear),
        g!(Vehicle, f32, "getCurrentRPM", "(I)F", get_rpm),
        s!(Vehicle, f32, "setCurrentRPM", "(IF)V", set_rpm),
        g!(Vehicle, f32, "getTurbo", "(I)F", get_turbo),
        s!(Vehicle, f32, "setTurbo", "(IF)V", set_turbo),
        g!(Vehicle, f32, "getDashboardSpeed", "(I)F", get_dashboard_speed),
        g!(Vehicle, f32, "getWheelSpeed", "(I)F", get_wheel_speed),
        s!(Vehicle, f32, "setWheelSpeed", "(IF)V", set_wheel_speed),
        g!(Vehicle, f32, "getThrottle", "(I)F", get_throttle),
        s!(Vehicle, f32, "setThrottle", "(IF)V", set_throttle),
        g!(Vehicle, f32, "getThrottlePower", "(I)F", get_throttle_power),
        s!(Vehicle, f32, "setThrottlePower", "(IF)V", set_throttle_power),
        g!(Vehicle, f32, "getFuel", "(I)F", get_fuel),
        s!(Vehicle, f32, "setFuel", "(IF)V", set_fuel),
        g!(Vehicle, f32, "getMaxOil", "(I)F", get_max_oil),
        g!(Vehicle, f32, "getOil", "(I)F", get_oil),
        s!(Vehicle, f32, "setOil", "(IF)V", set_oil),
        g!(Vehicle, f32, "getClutch", "(I)F", get_clutch),
        s!(Vehicle, f32, "setClutch", "(IF)V", set_clutch),
        g!(Vehicle, f32, "getEngineTemperature", "(I)F", get_engine_temperature),
        s!(Vehicle, f32, "setEngineTemperature", "(IF)V", set_engine_temperature),
        g!(Vehicle, u32, "getAlarmTime", "(I)I", get_alarm_time),
        s!(Vehicle, u32, "setAlarmTime", "(II)V", set_alarm_time),
        g!(Vehicle, f32, "getEnginePower", "(I)F", get_engine_power),
        g!(Vehicle, f32, "getBrakePower", "(I)F", get_brake_power),
        g!(Vehicle, f32, "getSteeringAngle", "(I)F", get_steering_angle),
        s!(Vehicle, f32, "setSteeringAngle", "(IF)V", set_steering_angle),
        g!(Vehicle, f32, "getSteeringScale", "(I)F", get_steering_scale),
        s!(Vehicle, f32, "setSteeringScale", "(IF)V", set_steering_scale)
    );
    pool!(env, crate::game::vehicle::get_pool(), "mp.evolution.game.entity.vehicle.VehiclePool");
    pool!(env, crate::game::prop::get_pool(), "mp.evolution.game.entity.prop.PropPool");
    pool!(env, crate::game::ped::get_pool(), "mp.evolution.game.entity.ped.PedPool");

    natives!(env, "mp.evolution.runtime.Runtime",
        NativeMethod::new("restart", "()V", restart as _),
        NativeMethod::new("pid", "()I", pid as _),
        NativeMethod::new("unlockModule", "(Ljava/lang/Module;Ljava/lang/String;)V", unlock_module as _)
    );

    lazy_static::initialize(&RUNTIME);

    warn!("Runtime thread exited!");
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
            env.call_method_unchecked_fast(RUNTIME.as_obj(), **SCRIPT_FRAME, JavaType::Primitive(Void), &[])
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
            env.call_method_unchecked_fast(RUNTIME.as_obj(), **SCRIPT_EVENT, JavaType::Primitive(Void), &[JValue::Object(event).to_jni()])
                .expect("error calling `event`");
        }
    }
}

extern "C" fn unlock_module(_: &JNIEnv, _class: JClass, module: JObject, package: JString) {
    let env = attach_thread();
    call!(env, env.call_method(module, "implAddExportsToAllUnnamed", "(Ljava/lang/String;)V", args![*package]));
    call!(env, env.call_method(module, "implAddOpensToAllUnnamed", "(Ljava/lang/String;)V", args![*package]));
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

unsafe extern fn invoke(_env: &JNIEnv, _class: JClass, hash: u64, args: &mut [u64; 32], arg_count: u32, result: &mut [u64; 3]) {
    if let Some(handler) = crate::native::get_handler_opt(hash) {
        let mut context = NativeCallContext::new(args, result, arg_count);
        crate::native::CURRENT_NATIVE.store(hash, Ordering::SeqCst);
        handler(&mut context);
        crate::native::SET_VECTOR_RESULTS(&mut context); //Flush all &mut NativeVector3 args
        crate::native::CURRENT_NATIVE.store(0, Ordering::SeqCst);
    } else {
        let env = attach_thread();
        env.throw_new("java/lang/IllegalArgumentException", format!("No such native: 0x{:016X}", hash)).unwrap();
    }
}

pub trait Script {
    fn frame(&mut self);
    fn event(&mut self, event: ScriptEvent);
}

pub struct ScriptEnv {}