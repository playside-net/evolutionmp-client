use crate::invoke;
use crate::pattern::MemoryRegion;
use std::sync::atomic::{AtomicI32, Ordering};

pub(crate) static GEARS_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static HIGH_GEAR_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static FUEL_LEVEL_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static WHEEL_SPEED_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static CURRENT_RPM_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static ACCELERATION_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static STEERING_SCALE_OFFSET: AtomicI32 = AtomicI32::new(0);
pub(crate) static STEERING_ANGLE_OFFSET: AtomicI32 = AtomicI32::new(0);

pub unsafe fn init(mem: &MemoryRegion) {
    let address = mem.find("48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24")
        .next().expect("gear offset")
        .add(3);
    GEARS_OFFSET.store(*address.get::<i32>() + 2, Ordering::SeqCst);
    HIGH_GEAR_OFFSET.store(*address.get::<i32>() + 6, Ordering::SeqCst);

    let address = mem.find("74 ? 0F 57 C9 0F 2F 8B ? ? ? ? 73 ? F3 0F 10 83 ? ? ? ?")
        .next().expect("fuel level offset")
        .add(8);
    FUEL_LEVEL_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);

    let address = mem.find("F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?")
        .next().expect("wheel speed offset")
        .add(4);

    WHEEL_SPEED_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);

    let address = mem.find("76 03 0F 28 F0 F3 44 0F 10 93")
        .next().expect("rpm offset")
        .add(10);
    CURRENT_RPM_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);
    ACCELERATION_OFFSET.store(*address.get::<i32>() + 16, Ordering::SeqCst);

    let address = mem.find("74 0A F3 0F 11 B3 ? ? ? ? EB 25")
        .next().expect("steering offset")
        .add(6);
    STEERING_SCALE_OFFSET.store(*address.get::<i32>(), Ordering::SeqCst);
    STEERING_ANGLE_OFFSET.store(*address.get::<i32>() + 8, Ordering::SeqCst);
}