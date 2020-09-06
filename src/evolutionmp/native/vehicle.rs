use std::collections::HashMap;

use crate::bind_field;
use crate::native::EntityField;

pub(crate) static CURRENT_GEAR: EntityField<i32> = EntityField::unset();
pub(crate) static HIGH_GEAR: EntityField<i32> = EntityField::unset();
pub(crate) static FUEL_LEVEL: EntityField<f32> = EntityField::unset();
pub(crate) static OIL_LEVEL: EntityField<f32> = EntityField::unset();
pub(crate) static WHEEL_SPEED: EntityField<f32> = EntityField::unset();
pub(crate) static CURRENT_RPM: EntityField<f32> = EntityField::unset();
pub(crate) static ACCELERATION: EntityField<f32> = EntityField::unset();
pub(crate) static DASHBOARD_SPEED: EntityField<f32> = EntityField::unset();
pub(crate) static STEERING_SCALE: EntityField<f32> = EntityField::unset();
pub(crate) static STEERING_ANGLE: EntityField<f32> = EntityField::unset();
pub(crate) static HANDBRAKE: EntityField<bool> = EntityField::unset();
pub(crate) static ENGINE_TEMPERATURE: EntityField<f32> = EntityField::unset();
pub(crate) static LIGHTS: EntityField<i32> = EntityField::unset();
pub(crate) static ENGINE_POWER: EntityField<f32> = EntityField::predefined(0xAC0);
pub(crate) static BRAKE_POWER: EntityField<f32> = EntityField::predefined(0x9A0);
pub(crate) static OIL_VOLUME: EntityField<f32> = EntityField::predefined(0x0104);
pub(crate) static PETROL_TANK_VOLUME: EntityField<f32> = EntityField::predefined(0x0100);
pub(crate) static GEARS: EntityField<i32> = EntityField::predefined(0x870);
pub(crate) static NEXT_GEAR: EntityField<i32> = EntityField::predefined(0x870);
pub(crate) static TURBO: EntityField<f32> = EntityField::predefined(0x8D8);
pub(crate) static CLUTCH: EntityField<f32> = EntityField::predefined(0x8C0);
pub(crate) static THROTTLE: EntityField<f32> = EntityField::predefined(0x8C4);
pub(crate) static THROTTLE_POWER: EntityField<f32> = EntityField::predefined(0x99C);
pub(crate) static HELICOPTER_BLADES_SPEED: EntityField<f32> = EntityField::predefined(0x1AA8);
pub(crate) static TRAIN_TRACK_NODE: EntityField<i32> = EntityField::predefined(0x14C0);

fn find_nearest_native(natives: &HashMap<u64, *const ()>, address: *const ()) -> Option<(u64, *const ())> {
    natives.iter().min_by(|(_, f1), (_, f2)| {
        let o1 = (**f1 as isize - address as isize).abs();
        let o2 = (**f2 as isize - address as isize).abs();
        o1.cmp(&o2)
    }).map(|(k, v)| (*k, *v))
}

bind_field!(LIGHTS_OFFSET, "FD 02 DB 08 98 ? ? ? ? 48 8B 5C 24 30", -4, i32);
bind_field!(GEAR_OFFSET, "48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24", 3, i32);
bind_field!(FUEL_OFFSET, "48 3B CA 0F 84 ? ? ? ? 8B 81", 49, i32);
bind_field!(OIL_OFFSET, "48 3B CA 0F 84 ? ? ? ? 8B 81", 61, i32);
bind_field!(WHEEL_OFFSET, "F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?", 4, i32);
bind_field!(SPEED_OFFSET, "76 03 0F 28 F0 F3 44 0F 10 93", 10, i32);
bind_field!(DASHBOARD_SPEED_OFFSET, "0F 84 ? ? ? ? 44 89 AE ? ? ? ? 44 84 F3", 9, i32);
bind_field!(STEERING_OFFSET, "74 0A F3 0F 11 B3 ? ? ? ? EB 25", 6, i32);
bind_field!(HANDBRAKE_OFFSET, "8A C2 24 01 C0 E0 04 08 81", 19, i32);
bind_field!(ENGINE_TEMPERATURE_OFFSET, "48 8D 8F ? ? ? ? 45 32 FF", -4, i32);

pub fn pre_init() {
    lazy_static::initialize(&LIGHTS_OFFSET);
    lazy_static::initialize(&GEAR_OFFSET);
    lazy_static::initialize(&FUEL_OFFSET);
    lazy_static::initialize(&OIL_OFFSET);
    lazy_static::initialize(&WHEEL_OFFSET);
    lazy_static::initialize(&SPEED_OFFSET);
    lazy_static::initialize(&DASHBOARD_SPEED_OFFSET);
    lazy_static::initialize(&STEERING_OFFSET);
    lazy_static::initialize(&HANDBRAKE_OFFSET);
    lazy_static::initialize(&ENGINE_TEMPERATURE_OFFSET);
}

pub fn init() {
    LIGHTS.set_offset(**LIGHTS_OFFSET);
    CURRENT_GEAR.set_offset(**GEAR_OFFSET + 2);
    HIGH_GEAR.set_offset(**GEAR_OFFSET + 6);
    FUEL_LEVEL.set_offset(**FUEL_OFFSET);
    OIL_LEVEL.set_offset(**OIL_OFFSET);
    WHEEL_SPEED.set_offset(**WHEEL_OFFSET);
    CURRENT_RPM.set_offset(**SPEED_OFFSET);
    ACCELERATION.set_offset(**SPEED_OFFSET + 16);
    DASHBOARD_SPEED.set_offset(**DASHBOARD_SPEED_OFFSET);
    STEERING_SCALE.set_offset(**STEERING_OFFSET);
    STEERING_ANGLE.set_offset(**STEERING_OFFSET + 8);
    HANDBRAKE.set_offset(**HANDBRAKE_OFFSET);
    ENGINE_TEMPERATURE.set_offset(**ENGINE_TEMPERATURE_OFFSET);
}