use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

use cgmath::{Deg, Euler, Quaternion, Vector2, Vector3};
use detour::{GenericDetour, RawDetour};

use crate::client::native::pool::{CEntity, Native};
use crate::client::pattern::{MemoryRegion, RageBox};
use crate::game::{Handle, Rgb, Rgba};
use crate::game::ui::CursorSprite;
use crate::hash::Hash;
use crate::native::pool::Handleable;

pub mod vehicle;
pub mod pool;
pub mod object_hashes;
pub mod fs;
pub mod alloc;
pub mod script;
pub mod streaming;
pub mod grc;
pub mod assets;
pub mod init_fns;

#[repr(C)]
#[derive(Debug)]
pub struct TypeInfo {
    undecorated: ManuallyDrop<Box<CStr>>,
    decorated: [c_char; 1],
}

#[repr(transparent)]
pub struct ThreadSafe<T> {
    t: T
}

impl<T> ThreadSafe<T> {
    pub const fn new(t: T) -> ThreadSafe<T> {
        ThreadSafe { t }
    }
}

unsafe impl<T> std::marker::Send for ThreadSafe<T> {}

unsafe impl<T> std::marker::Sync for ThreadSafe<T> {}

impl<T> std::ops::Deref for ThreadSafe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T> std::ops::DerefMut for ThreadSafe<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.t
    }
}

static HOOKS: ThreadSafe<RefCell<Option<HashMap<u64, RawDetour>>>> = ThreadSafe::new(RefCell::new(None));

#[macro_export]
macro_rules! bind_fn_detour {
    ($name:ident,$pattern:literal,$offset:literal,$detour:path,($($arg:ty),*) -> $ret:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern fn($($arg),*) -> $ret = unsafe {
                let d = $crate::mem!($pattern)
                .expect(concat!("failed to find call for ", stringify!($name)))
                    .offset($offset).detour($detour as _);
                std::mem::transmute(d)
            };
        }
    };
}

#[macro_export]
macro_rules! bind_fn_detour_ip {
    ($name:ident,$pattern:literal,$offset:literal,$detour:path,($($arg:ty),*) -> $ret:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern fn($($arg),*) -> $ret = unsafe {
                let d = $crate::mem!($pattern)
                .expect(concat!("failed to find call for ", stringify!($name)))
                    .offset($offset).detour_ip($detour as _);
                std::mem::transmute(d)
            };
        }
    };
}

#[macro_export]
macro_rules! mem {
    ($pat:literal) => {{
        let mut cache = crate::native::PATTERN_CACHE.lock().expect("mutex poisoned");
        if let Some(region) = cache.get($pat).cloned() {
            region
        } else {
            let region = $crate::pattern::Pattern::from($pat).find();
            cache.insert($pat, region.clone());
            region
        }
    }};
}

#[macro_export]
macro_rules! bind_fn {
    ($name:ident,$pattern:literal,$offset:literal,($($arg:ty),*) -> $ret:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern fn($($arg),*) -> $ret = unsafe {
                let ptr = $crate::mem!($pattern)
                .expect(concat!("failed to bind call for ", stringify!($name)))
                    .offset($offset).as_ptr();
                std::mem::transmute(ptr)
            };
        }
    };
}

#[macro_export]
macro_rules! bind_fn_ip {
    ($name:ident,$pattern:literal,$offset:expr,($($arg:ty),*) -> $ret:ty) => {
        bind_fn_ip!($name,$pattern,$offset,($($arg),*) -> $ret,4);
    };
    ($name:ident,$pattern:literal,$offset:expr,($($arg:ty),*) -> $ret:ty,$ptr_len:literal) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern fn($($arg),*) -> $ret = unsafe {
                let ptr = $crate::mem!($pattern)
                    .expect(concat!("failed to bind call for ", stringify!($name)))
                    .offset($offset).read_ptr($ptr_len).as_ptr();
                std::mem::transmute(ptr)
            };
        }
    };
}

#[macro_export]
macro_rules! bind_inner_field {
    ($host:ident,$pattern:literal,$(($pat_offset:literal, $name:ident,$ty:ty,$offset:literal)),*) => {
        lazy_static::lazy_static! {
            $(
                pub static ref $name: crate::native::NativeField<$host, $ty> = {
                    let pattern = unsafe {
                        $crate::mem!($pattern)
                            .expect(concat!("failed to find field pattern `", $pattern, "` for ", stringify!($host)))
                            .offset($pat_offset)
                            .get_box::<i32>()
                    };
                    let offset = *pattern + $offset;
                    crate::info!("Got offset for {}.{}: 0x{:X} (0x{:X} + {})", stringify!($host), stringify!($name), offset, *pattern, $offset);
                    NativeField::new(offset)
                };
            )*
        }
    };
}

#[macro_export]
macro_rules! bind_field {
    ($name:ident,$pattern:literal,$offset:literal,$ty:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: crate::pattern::RageBox<$ty> = unsafe {
                $crate::mem!($pattern)
                .expect(concat!("failed to bind field for ", stringify!($name)))
                    .offset($offset).get_box()
            };
        }
    };
}

#[macro_export]
macro_rules! bind_field_ip {
    ($name:ident,$pattern:literal,$offset:expr,$ty:ty) => {
        bind_field_ip!($name,$pattern,$offset,$ty,4);
    };
    ($name:ident,$pattern:literal,$offset:expr,$ty:ty,$ptr_len:literal) => {
        lazy_static::lazy_static! {
            pub static ref $name: crate::pattern::RageBox<$ty> = unsafe {
                $crate::mem!($pattern)
                    .expect(concat!("failed to bind field for ", stringify!($name)))
                    .offset($offset).read_ptr($ptr_len).get_box()
            };
        }
    };
}

#[macro_export]
macro_rules! redirect {
    ($from:ident,$to:ident) => {
        let d = detour::RawDetour::new($from,$to as _).expect(concat!("failed to create redirect from ", stringify!($from), " to ", stringify!($to)));
        d.enable().expect(concat!("error redirecting from ", stringify!($from), " to ", stringify!($to)));
        let t = d.trampoline() as *const ();
        std::mem::forget(d);
        t
    };
}

lazy_static! {
    pub static ref OBJECT_HASHES: HashMap<i32, &'static str> = object_hashes::HASHES.iter().cloned().collect::<_>();
    pub static ref NATIVES: Natives = Natives::new();
    pub static ref PATTERN_CACHE: Mutex<HashMap<&'static str, Option<MemoryRegion>>> = Mutex::new(HashMap::new());
}

bind_fn!(SET_VECTOR_RESULTS, "83 79 18 ? 48 8B D1 74 4A FF 4A 18", 0, (&mut NativeCallContext) -> ());
bind_fn!(GET_SCRIPT_ENTITY, "44 8B C1 49 8B 41 08 41 C1 F8 08 41 38 0C 00", -12, (u32) -> RageBox<CEntity>);

bind_field!(EXPANDED_RADAR, "33 C0 0F 57 C0 ? 0D", 7, bool);
bind_field!(REVEAL_FULL_MAP, "33 C0 0F 57 C0 ? 0D", 30, bool);
bind_field!(CURSOR_SPRITE, "74 11 8B D1 48 8D 0D ? ? ? ? 45 33 C0", 0, CursorSprite);

pub(crate) fn hook() {
    script::hook();
    streaming::hook();
    grc::hook();
    pool::hook();
    vehicle::hook();
    init_fns::hook();

    crate::info!("Hooking generic natives...");
    lazy_static::initialize(&EXPANDED_RADAR);
    lazy_static::initialize(&REVEAL_FULL_MAP);
    lazy_static::initialize(&CURSOR_SPRITE);
    lazy_static::initialize(&SET_VECTOR_RESULTS);
}

pub(crate) fn init() {
    crate::info!("Initializing natives...");
    lazy_static::initialize(&NATIVES);
    vehicle::init();
    HOOKS.replace(Some(HashMap::new()));

    crate::events::init();
}

fn get_trampoline(hash: u64) -> NativeFunction {
    let hooks = HOOKS.try_borrow().expect("unable to borrow hook map");
    let hooks = hooks.as_ref().expect("hook map is not initialized");
    let detour = hooks.get(&hash).expect(&format!("missing native trampoline for 0x{:016X}", hash));
    unsafe { std::mem::transmute(detour.trampoline()) }
}

pub fn call_trampoline(hash: u64, context: *mut NativeCallContext) {
    let trampoline = get_trampoline(hash);
    trampoline(context);
}

pub fn detour(hash: u64, hook: fn(&mut NativeCallContext)) {
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

pub fn get_handler_opt(hash: u64) -> Option<NativeFunction> {
    NATIVES.get_handler(hash)
}

pub fn get_handler(hash: u64) -> NativeFunction {
    get_handler_opt(hash).expect(&format!("Missing native handler for 0x{:016X}", hash))
}

#[repr(C, packed(1))]
struct PtrXorU64 {
    prev: u64,
    next: u64,
}

impl PtrXorU64 {
    fn get(&self) -> u64 {
        let addr = self as *const Self as u64;
        let mask = (addr ^ self.next) as u32 as u64;
        (((mask << 32) | mask) ^ self.prev) as _
    }
}

#[repr(C, packed(1))]
struct PtrXorU32 {
    prev: u32,
    next: u32,
}

impl PtrXorU32 {
    fn get(&self) -> u32 {
        let addr = self as *const Self as u64;
        addr as u32 ^ self.next ^ self.prev
    }
}

#[repr(C, packed(1))]
struct NativeGroup {
    next_group: PtrXorU64,
    handlers: [NativeFunction; 7],
    len: PtrXorU32,
    pad: u32,
    hashes: [PtrXorU64; 7],
}

#[repr(C)]
pub struct NativeTable {
    groups: [Box<NativeGroup>; 256],
    _unknown: u32,
    initialized: bool,
}

impl NativeTable {
    pub fn find(&self, hash: u64) -> Option<NativeFunction> {
        let group = &self.groups[(hash & 0xFF) as usize];
        group.find(hash)
    }
}

impl NativeGroup {
    pub fn find(&self, hash: u64) -> Option<NativeFunction> {
        self.iter().find(|(h, _)| *h == hash).map(|(_, handler)| handler)
    }

    pub fn get_next_group(&self) -> Option<&NativeGroup> {
        unsafe { std::mem::transmute(self.next_group.get()) }
    }

    pub fn len(&self) -> usize {
        self.len.get() as usize
    }

    pub fn get_hash(&self, index: usize) -> u64 {
        self.hashes[index].get()
    }

    pub fn iter(&self) -> NativeGroupIterator {
        NativeGroupIterator {
            group: self,
            index: 0,
        }
    }
}

pub struct NativeGroupIterator<'a> {
    group: &'a NativeGroup,
    index: usize,
}

impl<'a> Iterator for NativeGroupIterator<'a> {
    type Item = (u64, NativeFunction);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index < self.group.len() {
            self.index += 1;
            let hash = self.group.get_hash(index);
            //crate::info!("Native: 0x{:016X}", hash);
            let handler = self.group.handlers[index];
            Some((hash, handler))
        } else {
            if let Some(group) = self.group.get_next_group() {
                self.index = 0;
                self.group = group;
                self.next()
            } else {
                None
            }
        }
    }
}

pub struct NativeStackReader<'a> {
    stack: &'a [u64],
    pos: usize,
}

impl<'a> NativeStackReader<'a> {
    pub fn new(stack: &'a [u64]) -> NativeStackReader<'a> {
        NativeStackReader {
            stack,
            pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn is_null(&self) -> bool {
        self.stack[self.pos] == 0
    }

    pub fn read_u64(&mut self) -> u64 {
        let pos = self.pos;
        self.pos += 1;
        self.stack[pos]
    }

    pub unsafe fn read_ptr<T>(&mut self) -> T where T: Sized {
        let pos = self.pos;
        self.pos += 1;
        self.stack[pos..].as_ptr().cast::<T>().read()
    }

    pub fn read<T>(&mut self) -> T where T: NativeStackValue {
        T::read_from_stack(self)
    }

    pub fn read_option<T>(&mut self) -> Option<T> where T: NativeStackValue {
        if self.is_null() {
            self.pos += 1;
            None
        } else {
            Some(self.read())
        }
    }
}

pub struct NativeStackWriter<'a> {
    stack: &'a mut [u64],
    pos: usize,
}

impl<'a> NativeStackWriter<'a> {
    pub fn new(stack: &'a mut [u64]) -> NativeStackWriter<'a> {
        NativeStackWriter {
            stack,
            pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn write_u64(&mut self, raw: u64) -> usize {
        self.stack[self.pos] = raw;
        self.pos += 1;
        1
    }

    pub unsafe fn write_ptr<T>(&mut self, value: T) -> usize where T: Sized {
        let pos = self.pos;
        self.pos += 1;
        self.stack[pos..].as_mut_ptr().cast::<T>().write(value);
        1
    }

    pub fn write<T>(&mut self, value: T) -> usize where T: NativeStackValue {
        let pos = self.pos;
        value.write_to_stack(self);
        self.pos - pos
    }

    pub fn write_option<T>(&mut self, value: Option<T>) -> usize where T: NativeStackValue {
        if let Some(value) = value {
            self.write(value)
        } else {
            self.write_u64(0)
        }
    }
}

pub trait NativeStackValue {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size <= 8 {
            unsafe {
                stack.read_ptr::<Self>()
            }
        } else {
            panic!(
                "Cannot read value of type `{}` from stack as it exceeds default reader's size limits ({} bytes)",
                std::any::type_name::<Self>(),
                size
            )
        }
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size <= 8 {
            unsafe {
                stack.write_ptr(self);
            }
        } else {
            panic!(
                "Cannot write value of type `{}` to stack as it exceeds default writer's size limits ({} bytes)",
                std::any::type_name::<Self>(),
                size
            )
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct NativeCallContext {
    returns: Box<[u64; 3]>,
    arg_count: u32,
    args: Box<[u64; 32]>,
    data_count: u32,
    data: [u32; 48],
}

impl NativeCallContext {
    pub fn new() -> NativeCallContext {
        Self::new_allocated(Box::new([0; 32]), Box::new([0; 3]), 0)
    }

    pub fn new_allocated(args: Box<[u64; 32]>, returns: Box<[u64; 3]>, arg_count: u32) -> NativeCallContext {
        NativeCallContext {
            returns,
            arg_count,
            args,
            data_count: 0,
            data: [0; 48],
        }
    }

    pub fn get_args(&self) -> NativeStackReader {
        NativeStackReader::new(&*self.args)
    }

    pub fn push_arg<A>(&mut self, arg: A) -> usize where A: NativeStackValue {
        let i = self.arg_count as usize;
        let mut writer = NativeStackWriter::new(&mut self.args[i..]);
        let len = writer.write(arg);
        self.arg_count += len as u32;
        len
    }

    pub fn get_result<R>(&mut self) -> R where R: NativeStackValue {
        SET_VECTOR_RESULTS(self);
        let mut reader = NativeStackReader::new(&*self.returns);
        reader.read()
    }

    pub fn set_result<R>(&mut self, result: R) -> usize where R: NativeStackValue {
        let mut writer = NativeStackWriter::new(&mut *self.returns);
        writer.write(result)
    }
}

pub type NativeFunction = extern fn(*mut NativeCallContext);

pub struct Natives {
    mappings: HashMap<u64, u64>,
    handlers: HashMap<u64, NativeFunction>,
}

impl Natives {
    pub fn new() -> Natives {
        bind_field_ip!(NATIVE_TABLE, "76 32 48 8B 53 40", 9, NativeTable);

        let mappings = crate::mappings::MAPPINGS.iter().cloned().collect::<HashMap<_, _>>();
        let mut handlers = HashMap::with_capacity(mappings.len());

        for group in NATIVE_TABLE.groups.iter() {
            for (hash, handler) in group.iter() {
                handlers.insert(hash, handler);
            }
        }

        Natives { mappings, handlers }
    }

    pub fn get_handler(&self, hash: u64) -> Option<NativeFunction> {
        let hash = self.mappings.get(&hash).cloned().unwrap_or(hash);
        self.handlers.get(&hash).cloned()
    }
}

pub(crate) static CURRENT_NATIVE: AtomicU64 = AtomicU64::new(0);

#[macro_export]
macro_rules! invoke {
    ($ret: ty, $hash: literal $(, $arg: expr)*) => {{
        lazy_static! {
            static ref HANDLER: $crate::native::NativeFunction = $crate::native::get_handler($hash);
        }
        let mut context = $crate::native::NativeCallContext::new();
        $(context.push_arg($arg);)*
        use std::sync::atomic::Ordering;
        $crate::native::CURRENT_NATIVE.store($hash, Ordering::SeqCst);
        HANDLER(&mut context);
        $crate::native::CURRENT_NATIVE.store(0, Ordering::SeqCst);
        context.get_result::<$ret>()
    }};
}

#[macro_export]
macro_rules! invoke_option {
    ($ret: expr, $hash: literal, $($arg: expr),*) => {
        if invoke!(bool, $hash, $($arg),*)  {
            Some($ret)
        } else {
            None
        }
    };
}

impl NativeStackValue for &str {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let c_str = unsafe { CStr::from_ptr(stack.read_u64() as *mut _) };
        c_str.to_str().expect(&format!("Failed to read C string: {:?}", c_str))
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        let native = CString::new(self).expect("Failed to write C string");
        stack.write_u64(native.as_ptr() as u64);
        std::mem::forget(native);
    }
}

impl NativeStackValue for String {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        <&str as NativeStackValue>::read_from_stack(stack).to_owned()
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        self.as_str().write_to_stack(stack);
        std::mem::forget(self);
    }
}

impl NativeStackValue for Quaternion<f32> {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let x = stack.read();
        let y = stack.read();
        let z = stack.read();
        Euler::<Deg<f32>>::new(x, y, z).into()
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        let euler = Euler::from(self);
        stack.write(Deg::from(euler.x));
        stack.write(Deg::from(euler.y));
        stack.write(Deg::from(euler.z));
    }
}

impl<T> NativeStackValue for Vector3<T> where T: NativeStackValue + Copy + Clone {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let x = stack.read();
        let y = stack.read();
        let z = stack.read();
        Vector3::new(x, y, z)
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.x);
        stack.write(self.y);
        stack.write(self.z);
    }
}

impl<T> NativeStackValue for Vector2<T> where T: NativeStackValue + Copy + Clone {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let x = stack.read();
        let y = stack.read();
        Vector2::new(x, y)
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.x);
        stack.write(self.y);
    }
}

impl<H> NativeStackValue for H where H: Handleable + Sized {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        H::from_handle(stack.read::<Handle>()).expect("got zero handle")
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.get_handle());
    }
}

impl<H> NativeStackValue for Option<H> where H: Handleable + Sized {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        H::from_handle(stack.read::<Handle>())
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        let handle = self.expect("cannot pass invalid handle as native arg").get_handle();
        stack.write(handle);
    }
}

impl NativeStackValue for Rgba {
    fn read_from_stack(_stack: &mut NativeStackReader) -> Self {
        panic!("Reading Rgba color from stack is not possible")
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.r as u32);
        stack.write(self.g as u32);
        stack.write(self.b as u32);
        stack.write(self.a as u32);
    }
}

impl NativeStackValue for Rgb {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let r = stack.read::<u32>();
        let g = stack.read::<u32>();
        let b = stack.read::<u32>();
        Rgb::new(r as u8, g as u8, b as u8)
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.r as u32);
        stack.write(self.g as u32);
        stack.write(self.b as u32);
    }
}

impl NativeStackValue for u8 {}

impl NativeStackValue for &mut u8 {}

impl NativeStackValue for i32 {}

impl NativeStackValue for &mut i32 {}

impl NativeStackValue for u32 {}

impl NativeStackValue for &mut u32 {}

impl NativeStackValue for f32 {}

impl NativeStackValue for &mut f32 {}

impl NativeStackValue for Deg<f32> {}

impl NativeStackValue for &mut Deg<f32> {}

impl NativeStackValue for bool {}

impl NativeStackValue for &mut bool {}

impl NativeStackValue for u64 {}

impl NativeStackValue for &mut u64 {}

impl NativeStackValue for () {}

impl NativeStackValue for Hash {}

impl NativeStackValue for &mut Hash {}

pub struct NativeField<A, T> where A: Addressable, T: Sized {
    offset: i32,
    _ty_a: PhantomData<A>,
    _ty_t: PhantomData<T>,
}

pub trait Addressable: Native {
    fn get_address(&self) -> *mut u8;
}

impl<A, T> NativeField<A, T> where A: Addressable, T: Sized {
    pub(crate) const fn new(offset: i32) -> NativeField<A, T> {
        NativeField {
            offset,
            _ty_a: PhantomData,
            _ty_t: PhantomData,
        }
    }

    pub(crate) fn get_offset(&self) -> isize {
        let offset = self.offset as isize;
        assert_ne!(offset, 0, "field uninitialized");
        offset
    }

    pub(crate) fn get_ptr(&self, target: &A) -> *mut T {
        let offset = self.get_offset();
        unsafe {
            target.get_address().offset(offset).cast::<T>()
        }
    }

    pub fn set(&self, target: &A, value: T) {
        unsafe {
            self.get_ptr(target).write(value)
        }
    }

    pub fn get(&self, target: &A) -> T {
        unsafe {
            self.get_ptr(target).read()
        }
    }
}

#[repr(C, packed(1))]
pub struct NativeVector3 {
    pub x: f32,
    _x_pad: u32,
    pub y: f32,
    _y_pad: u32,
    pub z: f32,
    _z_pad: u32,
}

impl NativeVector3 {
    pub fn zero() -> NativeVector3 {
        NativeVector3 {
            x: 0.0,
            _x_pad: 0,
            y: 0.0,
            _y_pad: 0,
            z: 0.0,
            _z_pad: 0,
        }
    }
}

impl From<NativeVector3> for Vector3<f32> {
    fn from(native: NativeVector3) -> Self {
        Vector3::new(native.x, native.y, native.z)
    }
}

impl NativeStackValue for &mut NativeVector3 {}