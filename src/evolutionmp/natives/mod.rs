use crate::pattern::Region;
use crate::{info, error};
use crate::game::Vector3;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use winapi::shared::minwindef::DWORD;
use winapi::shared::basetsd::DWORD64;
use winapi::ctypes::c_void;
use widestring::WideCString;

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
    stack: [0u64; 32]
});

pub static mut RETURN: UnsafeCell<NativeReturnStack> = UnsafeCell::new(NativeReturnStack {
    stack: [0u64; 3]
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

pub type SetVectorResults = unsafe extern "stdcall" fn(*mut NativeCallContext);

impl NativeRegistration {
    pub unsafe fn get_next_registration(&self) -> *mut NativeRegistration {
        let addr: *mut u32 = std::mem::transmute(&self.next_registration_1);
        let mask = (addr as u64 as u32 ^ *addr.offset(2)) as u64;
        std::mem::transmute((mask << 32 | mask) ^ *(addr as *mut u64))
    }

    pub unsafe fn get_num_entries(&self) -> usize {
        let addr: *mut u32 = std::mem::transmute(&self.num_entries_1);
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
    unsafe fn read_from_stack(stack: *const u64) -> Self where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size <= 8 {
            ((stack as *const u8).add(8 - size) as *const Self).read()
        } else {
            panic!(
                "Cannot read value of type `{}` from stack as it exceeds default reader's size limits ({} bytes)",
                std::any::type_name::<Self>(),
                size
            )
        }
    }

    unsafe fn write_to_stack(self, stack: *mut u64) where Self: Sized {
        let size = std::mem::size_of::<Self>();
        if size <= 8 {
            ((stack as *mut u8).add(8 - size) as *mut Self).write(self)
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
pub struct NativeReturnStack {
    pub stack: [u64; 3]
}

impl NativeReturnStack {
    pub fn get<T>(&self) -> T where T: NativeStackValue {
        unsafe { T::read_from_stack(self.stack.as_ptr()) }
    }
}

#[repr(C)]
pub struct NativeArgStack {
    pub stack: [u64; 32]
}

impl NativeArgStack {
    pub fn set<T>(&mut self, index: usize, value: T) where T: NativeStackValue {
        unsafe { value.write_to_stack(self.stack.as_mut_ptr().add(index)) }
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
    table: *mut NativeRegistrationTable,
    vector_fixer: SetVectorResults
}

unsafe impl Sync for Natives {}

impl Natives {
    pub unsafe fn new(global_region: &Region) -> Natives {
        let table = global_region.find("76 32 48 8B 53 40")
            .next().expect("native table")
            .add(9).rip(4).get_mut::<NativeRegistrationTable>();
        let vector_fixer: SetVectorResults = std::mem::transmute(global_region.find("83 79 18 ? 48 8B D1 74 4A FF 4A 18")
            .next().expect("vector fixer").as_mut_ptr());

        let mappings = crate::mappings::MAPPINGS.iter().map(|a| (a[0], a[1])).collect::<HashMap<_, _>>();

        Natives { mappings, table, vector_fixer }
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

    pub unsafe fn set_vector_result(&self, context: *mut NativeCallContext) {
        (self.vector_fixer)(context)
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
        use $crate::natives::NativeStackValue;

        let hash: u64 = $hash;

        let natives = $crate::natives::NATIVES.as_mut().expect("Natives aren't initialized yet");
        let handler = natives.get_handler(hash).expect(&format!("Missing native handler for 0x{:016X}", hash));
        let mut i = 0usize;
        $(
            let arg = $arg;
            let s = arg.get_stack_size();
            (*$crate::natives::ARG.get()).set(i, arg);
            i += s;
        )*
        {
            let mut ctx = $crate::natives::CONTEXT.get();
            (*ctx).arg_count = i as u32;
            (*ctx).data_count = 0;
            handler(ctx);
            natives.set_vector_result(ctx);
        }
        (*$crate::natives::RETURN.get()).get::<$ret>()
    }};
}

impl NativeStackValue for &str {
    unsafe fn read_from_stack(stack: *const u64) -> Self {
        CStr::from_ptr(stack.read() as *const c_char as *mut _).to_str()
            .expect("Failed to read C string")
    }

    unsafe fn write_to_stack(self, stack: *mut u64) {
        let native = CString::new(self).expect("Failed to write C string");
        stack.write(native.as_ptr() as u64);
        std::mem::forget(native);
    }
}

impl NativeStackValue for i32 {}
impl NativeStackValue for u32 {}
impl NativeStackValue for f32 {}
impl NativeStackValue for bool {}
impl NativeStackValue for () {}

impl NativeStackValue for Vector3 {
    unsafe fn read_from_stack(stack: *const u64) -> Self {
        let stack = stack as *const f32;
        let x = stack.offset(1).read();
        let y = stack.offset(3).read();
        let z = stack.offset(5).read();
        Vector3::new(x, y, z)
    }

    unsafe fn write_to_stack(self, stack: *mut u64) {
        let stack = stack as *mut f32;
        stack.add(1).write(self.x);
        stack.add(3).write(self.y);
        stack.add(5).write(self.z);
    }

    fn get_stack_size(&self) -> usize {
        3
    }
}