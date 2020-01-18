use crate::pattern::MemoryRegion;
use crate::game::Handle;
use crate::game::vehicle::Vehicle;
use crate::game::ped::Ped;
use crate::game::entity::Entity;
use crate::game::object::Object;
use crate::game::pickup::Pickup;
use crate::game::checkpoint::Checkpoint;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use crate::native::ThreadSafe;
use std::cell::Cell;

pub static PARTICLE_ADDRESS: ThreadSafe<Cell<Option<GetHandleAddress>>> = ThreadSafe::new(Cell::new(None));
pub static ENTITY_ADDRESS: ThreadSafe<Cell<Option<GetHandleAddress>>> = ThreadSafe::new(Cell::new(None));
pub static PLAYER_ADDRESS: ThreadSafe<Cell<Option<GetHandleAddress>>> = ThreadSafe::new(Cell::new(None));
static ADDRESS_TO_HANDLE: ThreadSafe<Cell<Option<GetAddressHandle>>> = ThreadSafe::new(Cell::new(None));

pub type RefPool<T> = *mut Option<ManuallyDrop<Box<T>>>;

static mut PED_POOL: RefPool<GenericPool<Ped>> = std::ptr::null_mut();
static mut GLOBAL_POOL: RefPool<GlobalPool> = std::ptr::null_mut();
static mut OBJECT_POOL: RefPool<GenericPool<Object>> = std::ptr::null_mut();
static mut PICKUP_POOL: RefPool<GenericPool<Pickup>> = std::ptr::null_mut();
static mut VEHICLE_POOL: RefPool<Box<VehiclePool>> = std::ptr::null_mut();
static mut CHECKPOINT_POOL: RefPool<GenericPool<Checkpoint>> = std::ptr::null_mut();

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    PARTICLE_ADDRESS.set(Some(std::mem::transmute(mem.find("74 21 48 8B 48 20 48 85 C9 74 18 48 8B D6 E8")
        .next().expect("particle address")
        .offset(-10).read_ptr(4).as_mut_ptr())));
    ENTITY_ADDRESS.set(Some(std::mem::transmute(mem.find("E8 ? ? ? ? 48 8B D8 48 85 C0 74 2E 48 83 3D")
        .next().expect("entity address")
        .add(1).read_ptr(4).as_mut_ptr())));
    PLAYER_ADDRESS.set(Some(std::mem::transmute(mem.find("B2 01 E8 ? ? ? ? 48 85 C0 74 1C 8A 88")
        .next().expect("entity address")
        .add(3).read_ptr(4).as_mut_ptr())));

    ADDRESS_TO_HANDLE.set(Some(std::mem::transmute(mem.find("48 F7 F9 49 8B 48 08 48 63 D0 C1 E0 08 0F B6 1C 11 03 D8")
        .next().expect("address to handle")
        .offset(-0x68).as_mut_ptr())));

    PED_POOL = mem.find("48 8B 05 ? ? ? ? 41 0F BF C8 0F BF 40 10")
        .next().expect("ped pool")
        .add(3).read_ptr(4).get_mut();
    OBJECT_POOL = mem.find("48 8B 05 ? ? ? ? 8B 78 10 85 FF")
        .next().expect("object pool")
        .add(3).read_ptr(4).get_mut();
    GLOBAL_POOL = mem.find("4C 8B 0D ? ? ? ? 44 8B C1 49 8B 41 08")
        .next().expect("entity pool")
        .add(3).read_ptr(4).get_mut();
    VEHICLE_POOL = mem.find("48 8B 05 ? ? ? ? F3 0F 59 F6 48 8B 08")
        .next().expect("vehicle pool")
        .add(3).read_ptr(4).get_mut();
    PICKUP_POOL = mem.find("4C 8B 05 ? ? ? ? 40 8A F2 8B E9")
        .next().expect("pickup pool")
        .add(3).read_ptr(4).get_mut();
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

    fn get_handle(&self, index: u32) -> Option<Handle> {
        if self.is_valid(index) {
            let address = self.get_address(index);
            let handle = unsafe { (ADDRESS_TO_HANDLE.get().unwrap())(address) };
            Some(handle)
        } else {
            None
        }
    }

    fn get(&self, index: u32) -> Option<T> {
        self.get_handle(index).and_then(T::from_handle)
    }

    fn len(&self) -> u32;
    fn capacity(&self) -> u32;

    fn iter(&self) -> PoolIterator<T> where Self: Sized {
        PoolIterator {
            pool: self,
            index: 0
        }
    }
}

#[repr(C)]
pub struct VehiclePool {
    pool_address: *mut u64,
    capacity: u32,
    pad1: [u32; 9],
    bit_array: *mut u32,
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
    byte_array: *mut u8,
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

pub fn get_peds() -> Option<ManuallyDrop<Box<GenericPool<Ped>>>> {
    unsafe { PED_POOL.read() }
}

pub fn get_global() -> Option<ManuallyDrop<Box<GlobalPool>>> {
    unsafe { GLOBAL_POOL.read() }
}

pub fn get_objects() -> Option<ManuallyDrop<Box<GenericPool<Object>>>> {
    unsafe { OBJECT_POOL.read() }
}

pub fn get_pickups() -> Option<ManuallyDrop<Box<GenericPool<Pickup>>>> {
    unsafe { PICKUP_POOL.read() }
}

pub fn get_vehicles() -> Option<ManuallyDrop<Box<Box<VehiclePool>>>> {
    unsafe { VEHICLE_POOL.read() }
}

pub fn get_checkpoints() -> Option<ManuallyDrop<Box<GenericPool<Checkpoint>>>> {
    unsafe { CHECKPOINT_POOL.read() }
}

pub struct PoolIterator<'a, T: Handleable> {
    pool: &'a dyn Pool<T>,
    index: u32
}

impl<'a, T> Iterator for PoolIterator<'a, T> where T: Handleable {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(global) = get_global() {
            if global.is_full() {
                return None;
            }
        } else {
            return None;
        }
        let capacity = self.pool.capacity();
        while self.index < capacity {
            let index = self.index;
            self.index += 1;
            if let Some(result) = self.pool.get(index) {
                return Some(result);
            }
        }
        None
    }
}

pub trait Handleable {
    fn from_handle(handle: Handle) -> Option<Self> where Self: Sized;
    fn get_handle(&self) -> Handle;
}