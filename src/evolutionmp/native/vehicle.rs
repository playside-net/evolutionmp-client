use crate::{invoke, bind_field};
use crate::native::{EntityField, NativeFunction};
use crate::pattern::MemoryRegion;
use crate::game::entity::Entity;
use crate::game::vehicle::Vehicle;
use std::sync::atomic::{AtomicI32, Ordering};
use std::collections::HashMap;
use std::marker::PhantomData;

type VehicleField<T> = EntityField<T>;

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

bind_field!(GEAR_OFFSET, "48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24", 3, i32);
bind_field!(FUEL_OFFSET, "74 ? 0F 57 C9 0F 2F 8B ? ? ? ? 73 ? F3 0F 10 83 ? ? ? ?", 8, i32);
bind_field!(WHEEL_OFFSET, "F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?", 4, i32);
bind_field!(SPEED_OFFSET, "76 03 0F 28 F0 F3 44 0F 10 93", 10, i32);
bind_field!(STEERING_OFFSET, "74 0A F3 0F 11 B3 ? ? ? ? EB 25", 6, i32);

pub fn pre_init() {
    lazy_static::initialize(&GEAR_OFFSET);
    lazy_static::initialize(&FUEL_OFFSET);
    lazy_static::initialize(&WHEEL_OFFSET);
    lazy_static::initialize(&SPEED_OFFSET);
    lazy_static::initialize(&STEERING_OFFSET);
}

pub fn init() {
    CURRENT_GEAR.set_offset(**GEAR_OFFSET + 2);
    HIGH_GEAR.set_offset(**GEAR_OFFSET + 6);
    FUEL_LEVEL.set_offset(**FUEL_OFFSET);
    WHEEL_SPEED.set_offset(**WHEEL_OFFSET);
    CURRENT_RPM.set_offset(**SPEED_OFFSET);
    ACCELERATION.set_offset(**SPEED_OFFSET + 16);
    STEERING_SCALE.set_offset(**STEERING_OFFSET);
    STEERING_ANGLE.set_offset(**STEERING_OFFSET + 8);
}