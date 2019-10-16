use crate::pattern::Region;
use crate::{info, error};
use std::cell::UnsafeCell;
use winapi::shared::minwindef::DWORD;
use winapi::shared::basetsd::DWORD64;
use std::collections::HashMap;
use winapi::ctypes::c_void;
use std::ffi::CString;
use std::mem::ManuallyDrop;
use widestring::WideCString;
use crate::game::Vector3;

pub mod ui;
pub mod system;
pub mod entity;
pub mod player;
pub mod vehicle;
pub mod socialclub;

pub static mut NATIVES: Option<Natives> = None;

pub(crate) unsafe fn init(global_region: &Region) {
    NATIVES = Some(Natives::new(global_region));
}

pub static mut ARG: UnsafeCell<NativeArgStack> = UnsafeCell::new(NativeArgStack {
    stack: [0; 32]
});

pub static mut RETURN: UnsafeCell<NativeReturnStack> = UnsafeCell::new(NativeReturnStack {
    stack: [0; 3]
});

pub static mut CONTEXT: UnsafeCell<NativeCallContext> = UnsafeCell::new(NativeCallContext {
    returns: unsafe { RETURN.get() },
    arg_count: 0,
    args: unsafe { ARG.get() },
    data_count: 0,
    data: [0; 48]
});

#[repr(C)]
struct NativeRegistration {
    next_registration_1: u64,
    next_registration_2: u64,
    handlers: [NativeHandler; 7],
    num_entries_1: u32,
    num_entries_2: u32,
    hashes: u64
}

#[repr(C)]
struct NativeRegistrationTable {
    entries: [*mut NativeRegistration; 0xFF],
    _unknown: u32,
    initialized: bool
}

impl NativeRegistration {
    pub unsafe fn get_next_registration(&self) -> *mut NativeRegistration {
        let mut addr: *mut u32 = std::mem::transmute(&self.next_registration_1);
        let mask = (addr as u64 as u32 ^ *addr.offset(2)) as u64;
        std::mem::transmute((mask << 32 | mask) ^ *(addr as *mut u64))
    }

    pub unsafe fn get_num_entries(&self) -> usize {
        let mut addr: *mut u32 = std::mem::transmute(&self.num_entries_1);
        (addr as u64 as u32 ^ self.num_entries_1 ^ self.num_entries_2) as usize
    }

    pub unsafe fn get_hash(&self, index: usize) -> u64 {
        let addr: *mut u32 = std::mem::transmute(&self.next_registration_1);
        let addr = addr.add(4 * index + 21);
        let mask = (addr as u64 as u32 ^ *addr.offset(2)) as u64;
        (mask << 32 | mask) ^ *(addr as *mut u64)
    }
}

pub trait NativeStackValue {
    fn read_from_stack(stack: &NativeReturnStack) -> Self where Self: Sized;
    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack);
}

#[repr(C)]
pub struct NativeReturnStack {
    pub stack: [u64; 3]
}

impl NativeReturnStack {
    pub fn get<T>(&self) -> T where T: NativeStackValue {
        T::read_from_stack(&self)
    }
}

#[repr(C)]
pub struct NativeArgStack {
    pub stack: [u64; 32]
}

impl NativeArgStack {
    pub fn set<T>(&mut self, index: &mut usize, value: T) where T: NativeStackValue {
        value.write_to_stack(index, self)
    }
}

#[repr(C)]
pub struct NativeCallContext {
    pub returns: *mut NativeReturnStack,
    pub arg_count: u32,
    pub args: *mut NativeArgStack,
    pub data_count: u32,
    pub data: [u32; 48],
}

type NativeHandler = extern "C" fn(*mut NativeCallContext) -> *mut ();

pub struct Natives {
    mappings: HashMap<u64, u64>,
    table: *mut NativeRegistrationTable
}

unsafe impl Sync for Natives {}

impl Natives {
    pub unsafe fn new(global_region: &Region) -> Natives {
        let table = global_region.find("76 32 48 8B 53 40")
            .next().expect("native table")
            .add(9).rip(4).get_mut::<NativeRegistrationTable>();

        let mappings = crate::mappings::MAPPINGS.iter().map(|a| (a[0], a[1])).collect::<HashMap<_, _>>();

        Natives { mappings, table }
    }

    pub unsafe fn get_handler(&self, native: u64) -> Option<NativeHandler> {
        let native = *self.mappings.get(&native)
            .expect(&format!("Missing mapping for native 0x{:016X}", native));

        let mut table = (*self.table).entries[(native & 0xFF) as usize];

        loop {
            let e = (*table).get_num_entries();

            for i in 0..e {
                let h = (*table).get_hash(i);
                if native == h {
                    return Some((*table).handlers[i as usize]);
                }
            }
            table = (*table).get_next_registration();
            if table.is_null() {
                return None;
            }
        }

        None
    }
}

#[macro_export]
macro_rules! invoke {
    ($ret: ty, $hash:literal) => {{
        let hash: u64 = $hash;

        let natives = $crate::natives::NATIVES.as_mut().expect("Natives aren't initialized yet");
        let handler = natives.get_handler(hash).expect(&format!("Missing native handler for 0x{:016X}", hash));
        {
            let mut ctx = $crate::natives::CONTEXT.get();
            (*ctx).arg_count = 0;
            (*ctx).data_count = 0;
            handler(ctx);
        }
        (*$crate::natives::RETURN.get()).get::<$ret>()
    }};
    ($ret: ty, $hash:literal, $($arg: expr),*) => {{
        let hash: u64 = $hash;

        let natives = $crate::natives::NATIVES.as_mut().expect("Natives aren't initialized yet");
        let handler = natives.get_handler(hash).expect(&format!("Missing native handler for 0x{:016X}", hash));
        let mut i = 0usize;
        $(
            let arg = $arg;
            (*$crate::natives::ARG.get()).set(&mut i, arg);
            i += 1;
        )*
        {
            let mut ctx = $crate::natives::CONTEXT.get();
            (*ctx).arg_count = i as u32;
            (*ctx).data_count = 0;
            handler(ctx);
        }
        (*$crate::natives::RETURN.get()).get::<$ret>()
    }};
}

impl NativeStackValue for CString {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        ManuallyDrop::into_inner(ManuallyDrop::<Self>::read_from_stack(stack).clone())
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = self.into_raw() as u64
    }
}

impl NativeStackValue for ManuallyDrop<CString> {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        unsafe {
            ManuallyDrop::new(CString::from_raw(std::mem::transmute(stack.stack[0])))
        }
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = ManuallyDrop::into_inner(self).into_raw() as u64
    }
}

impl NativeStackValue for WideCString {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        ManuallyDrop::into_inner(ManuallyDrop::<Self>::read_from_stack(stack).clone())
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = self.into_raw() as u64
    }
}

impl NativeStackValue for ManuallyDrop<WideCString> {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        unsafe {
            ManuallyDrop::new(WideCString::from_raw(std::mem::transmute(stack.stack[0])))
        }
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = ManuallyDrop::into_inner(self).into_raw() as u64
    }
}

impl NativeStackValue for u32 {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        stack.stack[0] as u32
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = self as u64
    }
}

impl NativeStackValue for u64 {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        stack.stack[0]
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = self
    }
}

impl NativeStackValue for f32 {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        f32::from_bits(stack.stack[0] as u32)
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = self.to_bits() as u64
    }
}

impl NativeStackValue for bool {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        stack.stack[0] == 1
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {
        stack.stack[0] = self as u64
    }
}

impl NativeStackValue for () {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        ()
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {}
}

impl NativeStackValue for Vector3 {
    fn read_from_stack(stack: &NativeReturnStack) -> Self {
        let x = f32::from_bits(stack.stack[0] as u32);
        let y = f32::from_bits(stack.stack[1] as u32);
        let z = f32::from_bits(stack.stack[2] as u32);
        Vector3::new(x, y, z)
    }

    fn write_to_stack(self, index: &mut usize, stack: &mut NativeArgStack) {}
}