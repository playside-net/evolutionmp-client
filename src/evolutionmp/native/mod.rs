use crate::pattern::MemoryRegion;
use crate::game::{Rgba, Rgb, Handle};
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use winapi::shared::minwindef::DWORD;
use winapi::shared::basetsd::DWORD64;
use winapi::ctypes::c_void;
use std::time::Duration;
use cgmath::{Vector3, Vector2};
use crate::game::ui::CursorSprite;
use crate::hash::Hash;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::ops::Deref;
use std::ptr::{null_mut, null};

pub mod ui;
pub mod graphics;
pub mod scaleform;
pub mod system;
pub mod entity;
pub mod player;
pub mod vehicle;
pub mod socialclub;
pub mod collection;
pub mod script;
pub mod controls;
pub mod streaming;
pub mod ped;
pub mod audio;
pub mod stats;
pub mod gameplay;
pub mod dlc;
pub mod clock;
pub mod decision_event;
pub mod pool;
pub mod camera;
pub mod worldprobe;

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

pub static NATIVES: ThreadSafe<RefCell<Option<Natives>>> = ThreadSafe::new(RefCell::new(None));
pub static SET_VECTOR_RESULTS: ThreadSafe<Cell<Option<NativeFunction>>> = ThreadSafe::new(Cell::new(None));
pub static EXPANDED_RADAR: AtomicPtr<bool> = AtomicPtr::new(null_mut());
pub static REVEAL_FULL_MAP: AtomicPtr<bool> = AtomicPtr::new(null_mut());
pub static CURSOR_SPRITE: AtomicPtr<CursorSprite> = AtomicPtr::new(null_mut());

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    let natives = Natives::new(mem);
    NATIVES.replace(Some(natives));
    SET_VECTOR_RESULTS.set(Some(std::mem::transmute(
        mem.find_await("83 79 18 ? 48 8B D1 74 4A FF 4A 18", 50, 1000)
            .expect("vector fixer").get_mut::<NativeFunction>()
    )));
    let big_map = mem.find("33 C0 0F 57 C0 ? 0D")
        .next().expect("big map")
        .add(7);
    EXPANDED_RADAR.store(big_map.get_mut(), Ordering::SeqCst);
    REVEAL_FULL_MAP.store(big_map.add(30).get_mut(), Ordering::SeqCst);
    let cursor_sprite = mem.find("74 11 8B D1 48 8D 0D ? ? ? ? 45 33 C0")
        .next().expect("cursor sprite");
    CURSOR_SPRITE.store(cursor_sprite.get_mut(), Ordering::SeqCst);
    pool::init(mem);
    vehicle::init(mem);
}

pub fn get_handler(hash: u64) -> NativeFunction {
    let natives = NATIVES.try_borrow().expect("Natives already borrowed");
    let natives = natives.as_ref().expect("Natives aren't initialized yet");
    natives.get_handler(hash).expect(&format!("Missing native handler for 0x{:016X}", hash))
}

#[repr(C)]
struct PtrMagic {
    prev: u64,
    next: u64
}

impl PtrMagic {
    unsafe fn get(&self) -> u64 {
        let addr = self as *const Self as *const u32;
        let mask = (addr as u64 as u32 ^ *addr.wrapping_add(2)) as u64;
        ((mask << 32) | mask) ^ self.prev
    }
}

#[repr(C)]
struct NativeGroup {
    next_group: PtrMagic,
    handlers: [NativeFunction; 7],
    len_1: u32,
    len_2: u32,
    pad: u32,
    hashes: [PtrMagic; 7]
}

#[repr(C)]
struct NativeTable {
    groups: [Box<NativeGroup>; 256],
    _unknown: u32,
    initialized: bool
}

impl NativeTable {
    pub fn find(&self, hash: u64) -> Option<NativeFunction> {
        let mut group = &self.groups[(hash & 0xFF) as usize];
        unsafe {
            group.find(hash)
        }
    }

    pub unsafe fn dump(&self, target: &mut Vec<u64>) {
        for g in self.groups.iter() {
            let l = g.len();
            for i in 0..l {
                target.push(g.get_hash(i));
            }
        }
    }
}

impl NativeGroup {
    pub unsafe fn find(&self, hash: u64) -> Option<NativeFunction> {
        let e = self.len();

        for i in 0..e {
            let h = self.get_hash(i);
            if hash == h {
                let handler = self.handlers[i];
                return Some(handler);
            }
        }
        let next = self.get_next_group();
        if next.is_null() {
            None
        }  else {
            (*next).find(hash)
        }
    }

    pub unsafe fn get_next_group(&self) -> *mut NativeGroup {
        self.next_group.get() as *mut u64 as *mut _
    }

    pub fn len(&self) -> usize {
        let addr: *const u32 = &self.len_1 as *const _;
        (addr as u64 as u32 ^ self.len_1 ^ self.len_2) as usize
    }

    pub unsafe fn get_hash(&self, index: usize) -> u64 {
        let addr = (&self.pad as *const u32).wrapping_add(1 + 4 * index);
        let mask = (addr as u64 as u32 ^ *addr.wrapping_add(2)) as u64;
        ((mask << 32) | mask) ^ *(addr as *const u64)
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

    pub fn read_u64(&mut self) -> u64 {
        let current_pos = self.pos;
        self.pos += 1;
        self.stack[current_pos]
    }

    pub fn read<T>(&mut self) -> T where T: NativeStackValue {
        T::read_from_stack(self)
    }

    pub fn as_ptr(&mut self) -> *const u64 {
        let old_pos = self.pos;
        self.pos += 1;
        self.stack[old_pos..].as_ptr()
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

    pub fn write_u64(&mut self, raw: u64) {
        self.stack[self.pos] = raw;
        self.pos += 1;
    }

    pub fn write<T>(&mut self, value: T) where T: NativeStackValue {
        value.write_to_stack(self)
    }

    pub fn as_ptr(&mut self) -> *mut u64 {
        let old_pos = self.pos;
        self.pos += 1;
        self.stack[old_pos..].as_mut_ptr()
    }
}

pub trait NativeStackValue {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size <= 8 {
            unsafe {
                stack.as_ptr().cast::<Self>().read()
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
                stack.as_ptr().cast::<Self>().write(self);
            }
        } else {
            panic!(
                "Cannot write value of type `{}` to stack as it exceeds default writer's size limits ({} bytes)",
                std::any::type_name::<Self>(),
                size
            )
        }
    }

    fn get_stack_size(&self) -> usize {
        1
    }
}

#[repr(C)]
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

    pub fn push<A>(&mut self, arg: A) where A: NativeStackValue {
        let i = self.arg_count as usize;
        let size = arg.get_stack_size() as u32;
        arg.write_to_stack(&mut NativeStackWriter::new(&mut self.args[i..]));
        self.arg_count += size;
    }

    pub fn get<R>(&mut self) -> R where R: NativeStackValue {
        (SET_VECTOR_RESULTS.get().unwrap())(self);
        R::read_from_stack(&mut NativeStackReader::new(&*self.returns))
    }
}

pub type NativeFunction = extern "C" fn(*mut NativeCallContext);

pub struct Natives {
    mappings: HashMap<u64, NativeFunction>
}

impl Natives {
    pub unsafe fn new(global_region: &MemoryRegion) -> Natives {
        let table = global_region.find_await("76 32 48 8B 53 40", 50, 1000)
            .expect("native table").add(9).read_ptr(4).get_box::<NativeTable>();
        let len = crate::mappings::MAPPINGS.len();

        let mut mappings = HashMap::with_capacity(len);

        for [old, new] in crate::mappings::MAPPINGS.iter() {
            let handler = table.find(*new)
                .expect(&format!("Missing native handler for hash 0x{:016X} (0x{:016X})", old, new));
            mappings.insert(*old, handler);
        }

        Natives { mappings }
    }

    pub fn get_handler(&self, native: u64) -> Option<NativeFunction> {
        self.mappings.get(&native).cloned()
    }
}

#[macro_export]
macro_rules! invoke {
    ($ret: ty, $hash:literal) => {{
        let hash: u64 = $hash;
        let handler = $crate::native::get_handler(hash);
        let mut context = $crate::native::NativeCallContext::new();
        handler(&mut context);
        context.get::<$ret>()
    }};
    ($ret: ty, $hash:literal, $($arg: expr),*) => {{
        let hash: u64 = $hash;
        let handler = $crate::native::get_handler(hash);
        let mut context = $crate::native::NativeCallContext::new();
        $(context.push($arg);)*
        handler(&mut context);
        context.get::<$ret>()
    }};
}

impl NativeStackValue for &str {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        unsafe { CStr::from_ptr(stack.as_ptr() as *const _ as *mut _) }.to_str()
            .expect("Failed to read C string")
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        let native = CString::new(self).expect("Failed to write C string");
        unsafe {
            stack.write_u64(native.as_ptr() as u64);
        }
        std::mem::forget(native);
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

    fn get_stack_size(&self) -> usize {
        3
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

    fn get_stack_size(&self) -> usize {
        2
    }
}

impl NativeStackValue for Rgba {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        panic!("Reading Rgba color from stack is not possible")
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.r);
        stack.write(self.g);
        stack.write(self.b);
        stack.write(self.a);
    }

    fn get_stack_size(&self) -> usize {
        4
    }
}

impl NativeStackValue for Rgb {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let r = stack.read();
        let g = stack.read();
        let b = stack.read();
        Rgb::new(r, g, b)
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.r);
        stack.write(self.g);
        stack.write(self.b);
    }

    fn get_stack_size(&self) -> usize {
        3
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
impl NativeStackValue for bool {}
impl NativeStackValue for &mut bool {}
impl NativeStackValue for u64 {}
impl NativeStackValue for &mut u64 {}
impl NativeStackValue for () {}
impl NativeStackValue for Hash {}

impl<T> NativeStackValue for &mut Vector3<T> where T: NativeStackValue + Copy + Clone {}
impl<T> NativeStackValue for &mut Vector2<T> where T: NativeStackValue + Copy + Clone {}