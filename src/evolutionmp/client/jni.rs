use jni_dynamic::{JNIEnv, JavaVM, AttachGuard};
use jni_dynamic::objects::{JObject, JValue, JString};
use std::sync::Arc;
use std::marker::PhantomData;

#[macro_export]
macro_rules! args {
    [$($arg:expr),*] => {
        &[$(jni_dynamic::objects::JValue::from($arg)),*]
    };
}

static mut VM: Option<Arc<JavaVM>> = None;

pub(crate) unsafe fn set_vm(vm: Arc<JavaVM>) {
    VM = Some(vm);
}

pub(crate) fn attach_thread() -> AttachGuard<'static> {
    unsafe {
        VM.as_ref().expect("VM not initialized").attach_current_thread().expect("attach failed")
    }
}

#[macro_export]
macro_rules! java_static_method {
    ($name:ident, $class:literal, $java_name:literal, fn($($arg:ident: $arg_ty:ty),*)) => {
        java_static_method!($name, $class, $java_name, fn($($arg: $arg_ty),*) -> ());
    };
    ($name:ident, $class:literal, $java_name:literal, fn($($arg:ident: $arg_ty:ty),*) -> $ret:ty) => {
        pub fn $name($($arg: $arg_ty),*) -> $ret {
            let env = $crate::jni::attach_thread();
            let class = env.find_class($class).expect(concat!("Unable to find class ", $class));
            let args = &[
                $($arg.to_java_value(&env)),*
            ];
            lazy_static::lazy_static! {
                static ref SIGNATURE: String = {
                    let mut signature = String::from("(");
                    $(
                        signature.push_str(&<$arg_ty>::get_signature());
                    )*
                    signature.push(')');
                    signature.push_str(&<$ret>::get_signature());
                    signature
                };
            }
            let result = match env.call_static_method(class, $java_name, &**SIGNATURE, args) {
                Err(e) if matches!(*e.kind(), ErrorKind::JavaException) => {
                    let reason = crate::runtime::get_last_exception(&env);
                    panic!(concat!("Error calling ", $class, ".", $java_name, "{}: {}"), &**SIGNATURE, reason);
                },
                other => other.expect(concat!("error calling ", $class, ".", $java_name))
            };
            <$ret>::from_java_value(&env, result)
        }
    };
}

#[repr(transparent)]
pub struct J<'a, T, R = T> where T: JavaObject<R> {
    inner: JObject<'a>,
    _ty: PhantomData<T>,
    _result: PhantomData<R>
}

impl<'a, T, R> J<'a, T, R> where T: JavaObject<R> {
    pub fn get(&self, env: &JNIEnv) -> R {
        T::from_java_object(&env, self.inner)
    }
}

pub trait JavaValue<R> where R: Sized {
    fn get_signature() -> String;
    fn from_java_value<'a>(env: &'a JNIEnv<'a>, value: JValue<'a>) -> R;
    #[inline]
    fn from_java_field<'a>(env: &'a JNIEnv<'a>, obj: JObject<'a>, field: &str) -> R {
        let field = env.get_field(obj, field, Self::get_signature()).expect(&format!("error getting field {}.{}", Self::get_signature(), field));
        Self::from_java_value(env, field)
    }
    fn to_java_value<'a>(&self, env: &'a JNIEnv<'a>) -> JValue<'a>;
}

pub trait JavaObject<R>: JavaValue<R> where R: Sized {
    fn get_class_name<'a>() -> &'a str;
    fn from_java_object<'a>(env: &'a JNIEnv<'a>, obj: JObject<'a>) -> R;
    fn to_java_object<'a>(&self, env: &'a JNIEnv<'a>) -> JObject<'a>;
}

pub fn to_string_java(env: &JNIEnv, obj: JObject) -> String {
    String::from_java_value(env, env.call_method(obj, "toString", "()Ljava/lang/String;", &[]).unwrap())
}

impl<T, R> JavaValue<R> for T where T: JavaObject<R> {
    fn get_signature() -> String {
        format!("L{};", Self::get_class_name())
    }

    #[inline]
    fn from_java_value<'a>(env: &'a JNIEnv<'a>, value: JValue<'a>) -> R {
        let obj = value.l().expect("java value is not an object");
        Self::from_java_object(env, obj)
    }

    fn to_java_value<'a>(&self, env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Object(self.to_java_object(env))
    }
}

impl<S> JavaObject<String> for S where S: AsRef<str> {
    fn get_class_name<'a>() -> &'a str {
        "java/lang/String"
    }

    fn from_java_object<'a>(env: &'a JNIEnv, obj: JObject<'a>) -> String {
        env.get_string(JString::from(obj)).expect("string reading failed").to_string_lossy().to_string()
    }

    fn to_java_object<'a>(&self, env: &'a JNIEnv<'a>) -> JObject<'a> {
        *env.new_string(self.as_ref()).expect("string writing failed")
    }
}

impl JavaValue<i32> for i32 {
    fn get_signature() -> String {
        String::from("I")
    }

    fn from_java_value<'a>(_env: &'a JNIEnv<'a>, value: JValue<'a>) -> i32 {
        value.i().expect("java value is not an integer") as _
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Int(*self)
    }
}

impl JavaValue<u32> for u32 {
    fn get_signature() -> String {
        String::from("I")
    }

    fn from_java_value<'a>(_env: &'a JNIEnv<'a>, value: JValue<'a>) -> u32 {
        value.i().expect("java value is not an integer") as _
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Int(*self as i32)
    }
}

impl JavaValue<usize> for usize {
    fn get_signature() -> String {
        String::from("I")
    }

    fn from_java_value<'a>(_env: &'a JNIEnv<'a>, value: JValue<'a>) -> usize {
        value.i().expect("java value is not an integer") as _
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Int(*self as i64 as i32)
    }
}

impl JavaValue<f32> for f32 {
    fn get_signature() -> String {
        String::from("F")
    }

    fn from_java_value<'a>(_env: &'a JNIEnv<'a>, value: JValue<'a>) -> f32 {
        value.f().expect("java value is not a float") as _
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Float(*self)
    }
}

impl JavaValue<f64> for f64 {
    fn get_signature() -> String {
        String::from("D")
    }

    fn from_java_value<'a>(_env: &'a JNIEnv<'a>, value: JValue<'a>) -> f64 {
        value.d().expect("java value is not a double") as _
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Double(*self)
    }
}

impl JavaValue<bool> for bool {
    fn get_signature() -> String {
        String::from("Z")
    }

    fn from_java_value<'a>(_env: &'a JNIEnv<'a>, value: JValue<'a>) -> bool {
        value.z().expect("java value is not a boolean") as _
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Bool(*self as u8)
    }
}

impl JavaValue<()> for () {
    fn get_signature() -> String {
        String::from("V")
    }

    fn from_java_value(_env: &JNIEnv, value: JValue) -> () {
        value.v().expect("java value is not a void")
    }

    fn to_java_value<'a>(&self, _env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Void
    }
}

impl<T, R> JavaObject<Option<R>> for Option<T> where T: JavaObject<R> {
    fn get_class_name<'a>() -> &'a str {
        T::get_class_name()
    }

    fn from_java_object<'a>(env: &'a JNIEnv<'a>, obj: JObject<'a>) -> Option<R> {
        if obj.is_null() {
            None
        } else {
            Some(T::from_java_object(env, obj))
        }
    }

    fn to_java_object<'a>(&self, env: &'a JNIEnv<'a>) -> JObject<'a> {
        if let Some(value) = self.as_ref() {
            T::to_java_object(value, env)
        } else {
            JObject::null()
        }
    }
}

impl<T, R> JavaObject<Vec<R>> for [T] where T: JavaObject<R> {
    fn get_class_name<'a>() -> &'a str {
        T::get_class_name() //TODO: Fixme [ + Class name
    }

    fn from_java_object<'a>(env: &'a JNIEnv<'a>, obj: JObject<'a>) -> Vec<R> {
        let len = env.get_array_length(*obj).expect("failed to get array length");
        let mut result = Vec::<R>::with_capacity(len as usize);
        for i in 0..len {
            result.push(T::from_java_object(env, env.get_object_array_element(*obj, i)
                .expect(&format!("unable to get array element at index {}", i))));
        }
        result
    }

    fn to_java_object<'a>(&self, env: &JNIEnv<'a>) -> JObject<'a> {
        let arr = env.new_object_array(self.len() as _, T::get_class_name(), JObject::null())
            .expect("java array creation failed");
        for (i, e) in self.iter().enumerate() {
            env.set_object_array_element(arr, i as _, e.to_java_object(env))
                .expect(&format!("unable to set array element at index {}", i));
        }
        JObject::from(arr)
    }
}

impl<T, R> JavaValue<Vec<R>> for [T] where T: JavaObject<R> {
    fn get_signature() -> String {
        format!("[{}", T::get_signature())
    }

    fn from_java_value<'a>(env: &'a JNIEnv, value: JValue<'a>) -> Vec<R> {
        let obj = value.l().expect("unable to convert java value to object array");
        Self::from_java_object(env, obj)
    }

    fn to_java_value<'a>(&self, env: &'a JNIEnv<'a>) -> JValue<'a> {
        JValue::Object(self.to_java_object(env))
    }
}

impl<T> JavaValue<Vec<T>> for Vec<T> where T: JavaObject<T> {
    fn get_signature() -> String {
        format!("[{}", T::get_signature())
    }

    fn from_java_value<'a>(env: &'a JNIEnv<'a>, value: JValue<'a>) -> Vec<T> {
        <[T]>::from_java_value(env, value)
    }

    fn to_java_value<'a>(&self, env: &'a JNIEnv<'a>) -> JValue<'a> {
        <[T]>::to_java_value(self, env)
    }
}

#[derive(Debug)]
pub struct URL {
    inner: String
}

impl JavaObject<URL> for URL {
    fn get_class_name<'a>() -> &'a str {
        "java/net/URL"
    }

    fn from_java_object<'a>(env: &'a JNIEnv<'a>, obj: JObject<'a>) -> URL {
        let inner = String::from_java_value(env, env.call_method(obj, "toString", "()Ljava/lang/String;", &[]).unwrap());
        URL {
            inner
        }
    }

    fn to_java_object<'a>(&self, env: &JNIEnv<'a>) -> JObject<'a> {
        let inner = self.inner.to_java_object(env);
        env.new_object(Self::get_class_name(), "(Ljava/lang/String;)V", args![inner]).unwrap()
    }
}

impl URL {
    pub fn new(inner: String) -> URL {
        URL { inner }
    }
}