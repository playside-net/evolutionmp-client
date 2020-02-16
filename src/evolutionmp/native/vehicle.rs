use crate::invoke;
use crate::pattern::MemoryRegion;
use std::sync::atomic::{AtomicI32, Ordering};
use crate::game::entity::Entity;
use winapi::_core::marker::PhantomData;
use crate::native::NativeEntityField;
use crate::game::vehicle::Vehicle;

type VehicleField<T> = NativeEntityField<T>;

pub(crate) static GEAR: VehicleField<i32> = VehicleField::new();
pub(crate) static HIGH_GEAR: VehicleField<i32> = VehicleField::new();
pub(crate) static FUEL_LEVEL: VehicleField<f32> = VehicleField::new();
pub(crate) static WHEEL_SPEED: VehicleField<f32> = VehicleField::new();
pub(crate) static CURRENT_RPM: VehicleField<f32> = VehicleField::new();
pub(crate) static ACCELERATION: VehicleField<f32> = VehicleField::new();
pub(crate) static STEERING_SCALE: VehicleField<f32> = VehicleField::new();
pub(crate) static STEERING_ANGLE: VehicleField<f32> = VehicleField::new();

pub unsafe fn init(mem: &MemoryRegion) {
    let address = mem.find("48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24")
        .next().expect("gear offset")
        .add(3);
    GEAR.set_offset(*address.get::<i32>() + 2);
    HIGH_GEAR.set_offset(*address.get::<i32>() + 6);

    let address = mem.find("74 ? 0F 57 C9 0F 2F 8B ? ? ? ? 73 ? F3 0F 10 83 ? ? ? ?")
        .next().expect("fuel level offset")
        .add(8);
    FUEL_LEVEL.set_offset(*address.get::<i32>());

    let address = mem.find("F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?")
        .next().expect("wheel speed offset")
        .add(4);

    WHEEL_SPEED.set_offset(*address.get::<i32>());

    let address = mem.find("76 03 0F 28 F0 F3 44 0F 10 93")
        .next().expect("rpm offset")
        .add(10);
    CURRENT_RPM.set_offset(*address.get::<i32>());
    ACCELERATION.set_offset(*address.get::<i32>() + 16);

    let address = mem.find("74 0A F3 0F 11 B3 ? ? ? ? EB 25")
        .next().expect("steering offset")
        .add(6);
    STEERING_SCALE.set_offset(*address.get::<i32>());
    STEERING_ANGLE.set_offset(*address.get::<i32>() + 8);
}