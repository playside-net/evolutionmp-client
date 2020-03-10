use crate::pattern::MemoryRegion;
use crate::game::{Rgba, Rgb, Handle};
use crate::game::ui::CursorSprite;
use crate::hash::Hash;
use crate::native::pool::Handleable;
use crate::game::entity::Entity;
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ops::Deref;
use std::ptr::null_mut;
use std::marker::PhantomData;
use std::sync::atomic::AtomicI32;
use cgmath::{Vector3, Vector2, Euler, Deg, Quaternion};
use byteorder::WriteBytesExt;
use std::os::raw::c_char;
use std::mem::ManuallyDrop;
use detour::RawDetour;

pub mod vehicle;
pub mod pool;
pub mod object_hashes;
pub mod fs;
pub mod alloc;
pub mod script;
pub mod streaming;

#[repr(C)]
#[derive(Debug)]
pub struct TypeInfo {
    undecorated: ManuallyDrop<Box<CStr>>,
    decorated: [c_char; 1]
}

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

lazy_static! {
    pub static ref MEM: MemoryRegion = MemoryRegion::image();
}

#[macro_export]
macro_rules! bind_fn_detour {
    ($name:ident,$pattern:literal,$offset:literal,$detour:ident,$abi:literal,fn($($arg:ty),*) -> $ret:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern $abi fn($($arg),*) -> $ret = unsafe {
                let d = crate::native::MEM.find($pattern)
                    .next().expect(concat!("failed to find call for ", stringify!($name)))
                    .offset($offset).detour($detour as _);
                std::mem::transmute(d)
            };
        }
    };
}

#[macro_export]
macro_rules! bind_fn {
    ($name:ident,$pattern:literal,$offset:literal,$abi:literal,fn($($arg:ty),*) -> $ret:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern $abi fn($($arg),*) -> $ret = unsafe {
                let ptr = crate::native::MEM.find($pattern)
                    .next().expect(concat!("failed to bind call for ", stringify!($name)))
                    .offset($offset).as_ptr();
                std::mem::transmute(ptr)
            };
        }
    };
}

#[macro_export]
macro_rules! bind_fn_ip {
    ($name:ident,$pattern:literal,$offset:literal,$abi:literal,fn($($arg:ty),*) -> $ret:ty) => {
        bind_fn_ip!($name,$pattern,$offset,$abi,fn($($arg),*) -> $ret,4);
    };
    ($name:ident,$pattern:literal,$offset:literal,$abi:literal,fn($($arg:ty),*) -> $ret:ty,$ptr_len:literal) => {
        lazy_static::lazy_static! {
            pub static ref $name: extern $abi fn($($arg),*) -> $ret = unsafe {
                let ptr = crate::native::MEM.find($pattern)
                    .next().expect(concat!("failed to bind call for ", stringify!($name)))
                    .offset($offset).read_ptr($ptr_len).as_ptr();
                std::mem::transmute(ptr)
            };
        }
    };
}

#[macro_export]
macro_rules! bind_field {
    ($name:ident,$pattern:literal,$offset:literal,$ty:ty) => {
        lazy_static::lazy_static! {
            pub static ref $name: crate::pattern::RageBox<$ty> = unsafe {
                crate::native::MEM.find($pattern)
                    .next().expect(concat!("failed to bind field for ", stringify!($name)))
                    .offset($offset).get_box()
            };
        }
    };
}

#[macro_export]
macro_rules! bind_field_ip {
    ($name:ident,$pattern:literal,$offset:literal,$ty:ty) => {
        bind_field_ip!($name,$pattern,$offset,$ty,4);
    };
    ($name:ident,$pattern:literal,$offset:literal,$ty:ty,$ptr_len:literal) => {
        lazy_static::lazy_static! {
            pub static ref $name: crate::pattern::RageBox<$ty> = unsafe {
                crate::native::MEM.find($pattern)
                    .next().expect(concat!("failed to bind field for ", stringify!($name)))
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
}

bind_fn!(SET_VECTOR_RESULTS, "83 79 18 ? 48 8B D1 74 4A FF 4A 18", 0, "C", fn(*mut NativeCallContext) -> ());

bind_field!(EXPANDED_RADAR, "33 C0 0F 57 C0 ? 0D", 7, bool);
bind_field!(REVEAL_FULL_MAP, "33 C0 0F 57 C0 ? 0D", 30, bool);
bind_field!(CURSOR_SPRITE, "74 11 8B D1 48 8D 0D ? ? ? ? 45 33 C0", 0, CursorSprite);

pub(crate) fn pre_init() {
    script::pre_init();
    streaming::pre_init();
    pool::pre_init();
    vehicle::pre_init();
    lazy_static::initialize(&EXPANDED_RADAR);
    lazy_static::initialize(&REVEAL_FULL_MAP);
    lazy_static::initialize(&CURSOR_SPRITE);
}

pub(crate) fn init() {
    lazy_static::initialize(&NATIVES);
    lazy_static::initialize(&SET_VECTOR_RESULTS);
    vehicle::init();
}

pub fn get_handler(hash: u64) -> NativeFunction {
    NATIVES.get_handler(hash).expect(&format!("Missing native handler for 0x{:016X}", hash))
}

#[repr(C, packed(1))]
struct PtrXorU64 {
    prev: u64,
    next: u64
}

impl PtrXorU64 {
    fn get(&self) -> u64 {
        let addr = self as *const Self as u64;
        let mask = (addr ^ self.next) as u32 as u64;
        //crate::info!("xoring:  {:016X} ^ {:016X} = {:08X} ; result: {:016X}", addr, self.next, mask, ((mask << 32) | mask) ^ self.prev);
        (((mask << 32) | mask) ^ self.prev) as _
    }
}

#[repr(C, packed(1))]
struct PtrXorU32 {
    prev: u32,
    next: u32
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
    hashes: [PtrXorU64; 7]
}

#[repr(C)]
pub struct NativeTable {
    groups: [Box<NativeGroup>; 256],
    _unknown: u32,
    initialized: bool
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
            index: 0
        }
    }
}

pub struct NativeGroupIterator<'a> {
    group: &'a NativeGroup,
    index: usize
}

impl<'a> Iterator for NativeGroupIterator<'a> {
    type Item = (u64, NativeFunction);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index < self.group.len() {
            self.index += 1;
            let hash = self.group.get_hash(index);
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
    stack: &'a[u64],
    pos: usize
}

impl<'a> NativeStackReader<'a> {
    pub fn new(stack: &'a[u64]) -> NativeStackReader<'a> {
        NativeStackReader {
            stack, pos: 0
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
    stack: &'a mut[u64],
    pos: usize
}

impl<'a> NativeStackWriter<'a> {
    pub fn new(stack: &'a mut [u64]) -> NativeStackWriter<'a> {
        NativeStackWriter {
            stack, pos: 0
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
        NativeCallContext {
            returns: Box::new([0; 3]),
            arg_count: 0,
            args: Box::new([0; 32]),
            data_count: 0,
            data: [0; 48]
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

pub type NativeFunction = extern "C" fn(*mut NativeCallContext);

pub struct Natives {
    mappings: HashMap<u64, u64>,
    handlers: HashMap<u64, NativeFunction>
}

impl Natives {
    pub fn new() -> Natives {
        bind_field_ip!(NATIVE_TABLE, "76 32 48 8B 53 40", 9, NativeTable);

        let mut mappings = crate::mappings::MAPPINGS.iter().cloned().collect::<HashMap<_, _>>();
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

#[macro_export]
macro_rules! invoke {
    ($ret: ty, $hash: literal) => {{
        lazy_static! {
            static ref HANDLER: $crate::native::NativeFunction = $crate::native::get_handler($hash);
        }
        let mut context = $crate::native::NativeCallContext::new();
        HANDLER(&mut context);
        context.get_result::<$ret>()
    }};
    ($ret: ty, $hash: literal, $($arg: expr),*) => {{
        lazy_static! {
            static ref HANDLER: $crate::native::NativeFunction = $crate::native::get_handler($hash);
        }
        let mut context = $crate::native::NativeCallContext::new();
        $(context.push_arg($arg);)*
        HANDLER(&mut context);
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
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
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

pub struct EntityField<T> where T: Sized {
    offset: AtomicI32,
    _ty: PhantomData<T>
}

pub trait Addressable {
    fn get_address(&self) -> *mut u8;
}

impl<T> EntityField<T> where T: Sized {
    pub const fn unset() -> EntityField<T> {
        Self::predefined(0)
    }

    pub const fn predefined(offset: i32) -> EntityField<T> {
        EntityField {
            offset: AtomicI32::new(offset),
            _ty: PhantomData
        }
    }

    pub(crate) fn set_offset(&self, offset: i32) {
        self.offset.store(offset, Ordering::SeqCst)
    }

    pub(crate) fn get_offset(&self) -> isize {
        let offset = self.offset.load(Ordering::SeqCst) as isize;
        assert_ne!(offset, 0, "field uninitialized");
        offset
    }

    pub(crate) fn get_ptr(&self, target: &dyn Addressable) -> *mut T {
        let offset = self.get_offset();
        unsafe {
            target.get_address().offset(offset).cast::<T>()
        }
    }

    pub fn set(&self, target: &dyn Addressable, value: T) {
        unsafe {
            self.get_ptr(target).write(value)
        }
    }

    pub fn get(&self, target: &dyn Addressable) -> T {
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
    _z_pad: u32
}

impl NativeVector3 {
    pub fn zero() -> NativeVector3 {
        NativeVector3 {
            x: 0.0, _x_pad: 0,
            y: 0.0, _y_pad: 0,
            z: 0.0, _z_pad: 0
        }
    }
}

impl From<NativeVector3> for Vector3<f32> {
    fn from(native: NativeVector3) -> Self {
        Vector3::new(native.x, native.y, native.z)
    }
}

impl NativeStackValue for &mut NativeVector3 {}