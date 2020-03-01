use crate::invoke;
use crate::native::{NativeField, NativeFunction};
use crate::pattern::MemoryRegion;
use crate::game::entity::Entity;
use crate::game::vehicle::Vehicle;
use std::sync::atomic::{AtomicI32, Ordering};
use std::collections::HashMap;
use std::marker::PhantomData;

type VehicleField<T> = NativeField<T>;

pub(crate) static CURRENT_GEAR: VehicleField<i32> = VehicleField::unset();
pub(crate) static HIGH_GEAR: VehicleField<i32> = VehicleField::unset();
pub(crate) static FUEL_LEVEL: VehicleField<f32> = VehicleField::unset();
pub(crate) static WHEEL_SPEED: VehicleField<f32> = VehicleField::unset();
pub(crate) static CURRENT_RPM: VehicleField<f32> = VehicleField::unset();
pub(crate) static ACCELERATION: VehicleField<f32> = VehicleField::unset();
pub(crate) static STEERING_SCALE: VehicleField<f32> = VehicleField::unset();
pub(crate) static STEERING_ANGLE: VehicleField<f32> = VehicleField::unset();
pub(crate) static ENGINE_TEMPERATURE: VehicleField<f32> = VehicleField::predefined(0xA4C);
pub(crate) static ENGINE_POWER: VehicleField<f32> = VehicleField::predefined(0xAC0);
pub(crate) static OIL_LEVEL: VehicleField<f32> = VehicleField::predefined(0x838);
pub(crate) static OIL_VOLUME: VehicleField<f32> = VehicleField::predefined(0x0104);
pub(crate) static PETROL_TANK_VOLUME: VehicleField<f32> = VehicleField::predefined(0x0100);
pub(crate) static GEARS: VehicleField<i32> = VehicleField::predefined(0x870);
pub(crate) static NEXT_GEAR: VehicleField<i32> = VehicleField::predefined(0x870);
pub(crate) static TURBO: VehicleField<f32> = VehicleField::predefined(0x8D8);
pub(crate) static CLUTCH: VehicleField<f32> = VehicleField::predefined(0x8C0);
pub(crate) static THROTTLE: VehicleField<f32> = VehicleField::predefined(0x8C4);
pub(crate) static BRAKE_POWER: VehicleField<f32> = VehicleField::predefined(0x9A0);
pub(crate) static THROTTLE_POWER: VehicleField<f32> = VehicleField::predefined(0x99C);
pub(crate) static HELICOPTER_BLADES_SPEED: VehicleField<f32> = VehicleField::predefined(0x1AA8);
pub(crate) static TRAIN_TRACK_NODE: VehicleField<i32> = VehicleField::predefined(0x14C0);

fn find_nearest_native(natives: &HashMap<u64, *const ()>, address: *const ()) -> Option<(u64, *const ())> {
    natives.iter().min_by(|(_, f1), (_, f2)| {
        let o1 = (**f1 as isize - address as isize).abs();
        let o2 = (**f2 as isize - address as isize).abs();
        o1.cmp(&o2)
    }).map(|(k, v)| (*k, *v))
}

pub unsafe fn init(mem: &MemoryRegion) {
    let address = mem.find("48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24")
        .next().expect("gear offset")
        .add(3);
    crate::info!("VA");

    CURRENT_GEAR.set_offset(*address.get::<i32>() + 2);
    crate::info!("VB");
    HIGH_GEAR.set_offset(*address.get::<i32>() + 6);
    crate::info!("VC");

    let address = mem.find("74 ? 0F 57 C9 0F 2F 8B ? ? ? ? 73 ? F3 0F 10 83 ? ? ? ?")
        .next().expect("fuel level offset")
        .add(8);
    FUEL_LEVEL.set_offset(*address.get::<i32>());
    crate::info!("VD");

    let address = mem.find("F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?")
        .next().expect("wheel speed offset")
        .add(4);

    WHEEL_SPEED.set_offset(*address.get::<i32>());
    crate::info!("VE");

    let address = mem.find("76 03 0F 28 F0 F3 44 0F 10 93")
        .next().expect("rpm offset")
        .add(10);
    CURRENT_RPM.set_offset(*address.get::<i32>());
    crate::info!("VF");
    ACCELERATION.set_offset(*address.get::<i32>() + 16);
    crate::info!("VG");

    let address = mem.find("74 0A F3 0F 11 B3 ? ? ? ? EB 25")
        .next().expect("steering offset")
        .add(6);
    STEERING_SCALE.set_offset(*address.get::<i32>());
    crate::info!("VH");
    STEERING_ANGLE.set_offset(*address.get::<i32>() + 8);
    crate::info!("VI");
}