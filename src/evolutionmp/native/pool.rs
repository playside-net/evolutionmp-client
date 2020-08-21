use crate::pattern::{MemoryRegion, RageBox};
use crate::game::Handle;
use crate::game::vehicle::Vehicle;
use crate::game::ped::Ped;
use crate::game::entity::Entity;
use crate::game::pickup::Pickup;
use crate::game::checkpoint::Checkpoint;
use crate::native::ThreadSafe;
use crate::game::prop::Prop;
use crate::game::camera::Camera;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::cell::Cell;

use crate::{bind_fn, bind_fn_ip, bind_field_ip};
use cgmath::{Vector3, MetricSpace, Zero, Array};
use jni_dynamic::JNIEnv;
use jni_dynamic::objects::JClass;

bind_fn_ip!(PARTICLE_ADDRESS, "74 21 48 8B 48 20 48 85 C9 74 18 48 8B D6 E8", -10, "C", fn(Handle) -> *mut u8);
bind_fn_ip!(ENTITY_ADDRESS, "E8 ? ? ? ? 48 8B D8 48 85 C0 74 2E 48 83 3D", 1, "C", fn(Handle) -> *mut u8);
bind_fn_ip!(PLAYER_ADDRESS, "B2 01 E8 ? ? ? ? 48 85 C0 74 1C 8A 88", 3, "C", fn(Handle) -> *mut u8);
bind_fn!(ENTITY_ADD_TO_POOL, "48 89 5C 24 ? 48 89 74 24 ? 57 48 83 EC 20 8B 15 ? ? ? ? 48 8B F9 48 83 C1 10 33 DB", 0, "C", fn(*mut u8) -> Handle);
//bind_fn!(ENTITY_ADD_TO_POOL, "48 F7 F9 49 8B 48 08 48 63 D0 C1 E0 08 0F B6 1C 11 03 D8", -0x68, "C", fn(*mut u8) -> Handle);
bind_fn!(ENTITY_POS, "48 8B DA E8 ? ? ? ? F3 0F 10 44 24", -6, "C", fn(*mut u8, *mut f32) -> u64);

bind_field_ip!(PED, "48 8B 05 ? ? ? ? 41 0F BF C8 0F BF 40 10", 3, Option<Box<GenericPool<Ped>>>);
bind_field_ip!(PROP, "48 8B 05 ? ? ? ? 8B 78 10 85 FF", 3, Option<Box<GenericPool<Prop>>>);
bind_field_ip!(GLOBAL, "4C 8B 0D ? ? ? ? 44 8B C1 49 8B 41 08", 3, Option<Box<GlobalPool>>);
bind_field_ip!(VEHICLE, "48 8B 05 ? ? ? ? F3 0F 59 F6 48 8B 08", 3, Option<Box<Box<VehiclePool>>>);
bind_field_ip!(PICKUP, "4C 8B 05 ? ? ? ? 40 8A F2 8B E9", 3, Option<Box<GenericPool<Pickup>>>);

pub(crate) fn pre_init() {
    lazy_static::initialize(&PARTICLE_ADDRESS);
    lazy_static::initialize(&ENTITY_ADDRESS);
    lazy_static::initialize(&PLAYER_ADDRESS);
    lazy_static::initialize(&ENTITY_ADD_TO_POOL);

    lazy_static::initialize(&PED);
    lazy_static::initialize(&PROP);
    lazy_static::initialize(&GLOBAL);
    lazy_static::initialize(&VEHICLE);
    lazy_static::initialize(&PICKUP);
}

pub extern "C" fn is_global_full(_env: &JNIEnv, _class: JClass) -> bool {
    let global = GLOBAL.as_ref().as_ref().expect("global pool is not initialized");
    global.is_full()
}

pub extern "C" fn request_handle(_env: &JNIEnv, _class: JClass, address: u64) -> u32 {
    ENTITY_ADD_TO_POOL(address as _)
}

pub extern "C" fn get_entity_pos(_env: &JNIEnv, _class: JClass, address: u64, buffer: u64) {
    ENTITY_POS(address as _, buffer as _);
}

pub type GetHandleAddress = extern "C" fn(Handle) -> *mut u8;
pub type GetAddressHandle = extern "C" fn(*mut u8) -> Handle;

#[repr(C)]
pub struct GlobalPool {
    pad1: [u32; 4],
    num1: u32,
    pad2: [u32; 3],
    num2: u32
}

impl GlobalPool {
    pub fn is_full(&self) -> bool {
        self.num1 - (self.num2 & 0x3FFFFFFF) <= 256
    }
}

pub trait Pool<T: Handleable> {
    fn is_valid(&self, index: u32) -> bool;

    fn get_address(&self, index: u32) -> *mut u8;

    fn len(&self) -> u32;

    fn capacity(&self) -> u32;

    fn iter(&self) -> PoolIterator<T> where Self: Sized {
        PoolIterator::new(self)
    }
}

#[repr(C)]
pub struct VehiclePool {
    pool_address: ThreadSafe<*mut u64>,
    capacity: u32,
    pad1: [u32; 9],
    bit_array: ThreadSafe<*mut u32>,
    pad2: [u32; 10],
    len: u32
}

impl Pool<Vehicle> for VehiclePool {
    fn is_valid(&self, index: u32) -> bool {
        let block = unsafe { self.bit_array.wrapping_add(index as usize >> 5).read() };
        let offset = (index & 0x1F) as u32;
        ((block >> offset) & 1) != 0
    }

    fn get_address(&self, index: u32) -> *mut u8 {
        unsafe { self.pool_address.wrapping_add(index as usize).read() as *mut u8 }
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn capacity(&self) -> u32 {
        self.capacity
    }
}

#[repr(C)]
pub struct GenericPool<T: Handleable> {
    start_address: u64,
    byte_array: ThreadSafe<*mut u8>,
    capacity: u32,
    len: u32,
    _ty: PhantomData<T>
}

impl<T> GenericPool<T> where T: Handleable {
    pub fn mask(&self, index: u32) -> u64 {
        let num1 = unsafe { (self.byte_array.add(index as usize).read() & 0x80) as i64 };
        !((num1 | -num1) >> 63) as u64
    }
}

impl<T> Pool<T> for GenericPool<T> where T: Handleable {
    fn is_valid(&self, index: u32) -> bool {
        self.mask(index) != 0
    }

    fn get_address(&self, index: u32) -> *mut u8 {
        (self.mask(index) & (self.start_address + index as u64 * self.len as u64)) as _
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn capacity(&self) -> u32 {
        self.capacity
    }
}

#[repr(C)]
pub struct CameraPool {
    start_address: u64,
    byte_array: ThreadSafe<*mut u8>,
    capacity: u32,
    len: u32
}

impl Pool<Camera> for CameraPool {
    fn is_valid(&self, index: u32) -> bool {
        unsafe { self.byte_array.add(index as usize).read() == (index & 0xFF) as u8 }
    }

    fn get_address(&self, index: u32) -> *mut u8 {
        (self.start_address + index as u64 * self.len as u64) as _
    }

    fn len(&self) -> u32 {
        self.len
    }

    fn capacity(&self) -> u32 {
        self.capacity
    }
}

pub struct PoolEntry<T: Handleable> {
    address: *mut u8,
    _ty: PhantomData<T>
}

impl<E> PoolEntry<E> where E: Entity {
    pub fn get_position(&self) -> Vector3<f32> {
        let mut pos = Vector3::zero();
        ENTITY_POS(self.address, pos.as_mut_ptr());
        pos
    }

    pub fn pooled(self) -> Option<E> {
        let global = GLOBAL.as_ref().as_ref().expect("global pool is not initialized");
        if global.is_full() {
            None
        } else {
            E::from_handle(ENTITY_ADD_TO_POOL(self.address))
        }
    }
}

pub struct PoolIterator<'a, T: Handleable> {
    pool: &'a dyn Pool<T>,
    index: u32
}

impl<'a, T> Iterator for PoolIterator<'a, T> where T: Handleable {
    type Item = PoolEntry<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let capacity = self.pool.capacity();
        while self.index < capacity {
            let index = self.index;
            self.index += 1;
            if self.pool.is_valid(index) {
                let address = self.pool.get_address(index);
                return Some(PoolEntry {
                    address,
                    _ty: PhantomData
                });
            }
        }
        None
    }
}

impl<'a, T> PoolIterator<'a, T> where T: Handleable {
    pub fn new(pool: &dyn Pool<T>) -> PoolIterator<T> {
        PoolIterator {
            pool,
            index: 0
        }
    }
}

pub trait Handleable {
    fn from_handle(handle: Handle) -> Option<Self> where Self: Sized;
    fn get_handle(&self) -> Handle;
}

#[macro_export]
macro_rules! impl_handle {
    ($ty:ident) => {
        impl crate::native::pool::Handleable for $ty {
            fn from_handle(handle: crate::game::Handle) -> Option<Self> where Self: Sized {
                if handle == 0 || handle == std::u32::MAX {
                    None
                } else {
                    Some($ty { handle })
                }
            }

            fn get_handle(&self) -> crate::game::Handle {
                self.handle
            }
        }
    };
}