use crate::pattern::MemoryRegion;
use crate::game::Handle;

pub static mut PARTICLE_ADDRESS: Option<GetHandleAddress> = None;
pub static mut ENTITY_ADDRESS: Option<GetHandleAddress> = None;
pub static mut PLAYER_ADDRESS: Option<GetHandleAddress> = None;

pub static mut PED_POOL: *mut GenericPool = std::ptr::null_mut();
pub static mut ENTITY_POOL: *mut EntityPool = std::ptr::null_mut();
pub static mut OBJECT_POOL: *mut GenericPool = std::ptr::null_mut();
pub static mut PICKUP_POOL: *mut GenericPool = std::ptr::null_mut();
pub static mut VEHICLE_POOL: *mut VehiclePool = std::ptr::null_mut();
pub static mut CHECKPOINT_POOL: *mut GenericPool = std::ptr::null_mut();

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    PARTICLE_ADDRESS = Some(std::mem::transmute(mem.find("74 21 48 8B 48 20 48 85 C9 74 18 48 8B D6 E8")
        .next().expect("particle address")
        .offset(-10).read_ptr(4).as_mut_ptr()));
    ENTITY_ADDRESS = Some(std::mem::transmute(mem.find("E8 ? ? ? ? 48 8B D8 48 85 C0 74 2E 48 83 3D")
        .next().expect("entity address")
        .add(1).read_ptr(4).as_mut_ptr()));
    PLAYER_ADDRESS = Some(std::mem::transmute(mem.find("B2 01 E8 ? ? ? ? 48 85 C0 74 1C 8A 88")
        .next().expect("entity address")
        .add(3).read_ptr(4).as_mut_ptr()));

    PED_POOL = mem.find("48 8B 05 ? ? ? ? 41 0F BF C8 0F BF 40 10")
        .next().expect("ped pool")
        .add(3).read_ptr(4).get_mut();
    OBJECT_POOL = mem.find("48 8B 05 ? ? ? ? 8B 78 10 85 FF")
        .next().expect("object pool")
        .add(3).read_ptr(4).get_mut();
    ENTITY_POOL = mem.find("4C 8B 0D ? ? ? ? 44 8B C1 49 8B 41 08")
        .next().expect("entity pool")
        .add(3).read_ptr(4).get_mut();
    VEHICLE_POOL = mem.find("48 8B 05 ? ? ? ? F3 0F 59 F6 48 8B 08")
        .next().expect("vehicle pool")
        .add(3).read_ptr(4).get_mut();
    PICKUP_POOL = mem.find("4C 8B 05 ? ? ? ? 40 8A F2 8B E9")
        .next().expect("pickup pool")
        .add(3).read_ptr(4).get_mut();
}

pub type GetHandleAddress = unsafe extern "C" fn(Handle) -> *mut u8;

#[repr(C)]
pub struct EntityPool {
    pad1: u8,
    pad2: u8,
    num1: u32,
    pad3: u8,
    num2: u32
}

impl EntityPool {
    pub fn is_full(&self) -> bool {
        self.num1 - (self.num2 & 0x3FFFFFFF) <= 256
    }
}

#[repr(C)]
pub struct VehiclePool {
    pool_address: *mut u64,
    size: u32,
    pad1: [u8; 5],
    bit_array: *mut u32,
    pad2: [u8; 5],
    count: u32
}

impl VehiclePool {
    pub fn is_valid(&self, index: u32) -> bool {
        unsafe {
            ((self.bit_array.add((index >> 5) as usize).read() >> (index as i32 & 0x1F) as u32) & 1) != 0
        }
    }

    pub fn get_address(&self, index: u32) -> u64 {
        unsafe { self.pool_address.add(index as usize).read() }
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}

#[repr(C)]
pub struct GenericPool {
    start_address: u64,
    byte_array: *mut u8,
    size: u32,
    item_size: u32
}

impl GenericPool {
    pub fn is_valid(&self, index: u32) -> bool {
        self.mask(index) != 0
    }

    pub fn get_address(&self, index: u32) -> u64 {
        (self.mask(index) & (self.start_address + index as u64 * self.item_size as u64))
    }

    pub fn mask(&self, index: u32) -> u64 {
        let num1 = unsafe { (self.byte_array.add(index as usize).read() & 0x80) as i64 };
        !((num1 | -num1) >> 63) as u64
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }
}