use crate::bind_inner_field;
use crate::client::game::vehicle::Vehicle;
use crate::native::NativeField;

type VehicleField<T> = NativeField<Vehicle, T>;

bind_inner_field!(Vehicle, "48 8D 8F ? ? ? ? 4C 8B C3 F3 0F 11 7C 24", (3, NEXT_GEAR, u8, 0), (3, CURRENT_GEAR, u8, 2), (3, HIGH_GEAR, u8, 6));
bind_inner_field!(Vehicle, "48 3B CA 0F 84 ? ? ? ? 8B 81", (49, FUEL_LEVEL, f32, 0), (61, OIL_LEVEL, f32, 0));
bind_inner_field!(Vehicle, "FD 02 DB 08 98 ? ? ? ? 48 8B 5C 24 30", (-4, LIGHTS, u32, 0));
bind_inner_field!(Vehicle, "F3 0F 10 8F ? ? ? ? F3 0F 59 05 ? ? ? ?", (4, WHEEL_SPEED, f32, 0));
bind_inner_field!(Vehicle, "76 03 0F 28 F0 F3 44 0F 10 93", (10, RPM, f32, 0), (10, CLUTCH, f32, 12), (10, THROTTLE, f32, 16));
bind_inner_field!(Vehicle, "0F 84 ? ? ? ? 44 89 AE ? ? ? ? 44 84 F3", (9, DASHBOARD_SPEED, f32, 0));
bind_inner_field!(Vehicle, "74 0A F3 0F 11 B3 ? ? ? ? EB 25", (6, STEERING_SCALE, f32, 0), (6, STEERING_ANGLE, f32, 8), (6, THROTTLE_POWER, f32, 16), (6, BRAKE_POWER, f32, 20));
bind_inner_field!(Vehicle, "8A C2 24 01 C0 E0 04 08 81", (19, HANDBRAKE, bool, 0));
bind_inner_field!(Vehicle, "48 8D 8F ? ? ? ? 45 32 FF", (-4, ENGINE_TEMPERATURE, f32, 0));
bind_inner_field!(Vehicle, "E8 ? ? ? ? 40 8A F8 84 C0 75 ? 48 8B CB E8", (-4, TRAIN_TRACK_NODE, i32, 0));
bind_inner_field!(Vehicle, "24 07 3C 03 74 ? E8", (52, ALARM_TIME, u16, 0));
bind_inner_field!(Vehicle, "F3 0F 10 9F ? ? ? ? 0F 2F DF 73 0A", (4, TURBO, f32, 0));

pub(crate) static ENGINE_POWER: VehicleField<f32> = NativeField::new(0xAE8);
pub(crate) static OIL_VOLUME: VehicleField<f32> = NativeField::new(0x0104);
pub(crate) static HELICOPTER_BLADES_SPEED: VehicleField<f32> = NativeField::new(0x1AE4);

pub fn hook() {
    info!("Hooking vehicle offsets...");
    lazy_static::initialize(&NEXT_GEAR);
    lazy_static::initialize(&CURRENT_GEAR);
    lazy_static::initialize(&HIGH_GEAR);
    //lazy_static::initialize(&FUEL_LEVEL);
    //lazy_static::initialize(&OIL_LEVEL);
    lazy_static::initialize(&LIGHTS);
    lazy_static::initialize(&WHEEL_SPEED);
    lazy_static::initialize(&RPM);
    lazy_static::initialize(&CLUTCH);
    lazy_static::initialize(&THROTTLE);
    lazy_static::initialize(&DASHBOARD_SPEED);
    lazy_static::initialize(&STEERING_SCALE);
    lazy_static::initialize(&STEERING_ANGLE);
    lazy_static::initialize(&THROTTLE_POWER);
    lazy_static::initialize(&BRAKE_POWER);
    lazy_static::initialize(&HANDBRAKE);
    lazy_static::initialize(&ENGINE_TEMPERATURE);
    lazy_static::initialize(&TRAIN_TRACK_NODE);
    lazy_static::initialize(&ALARM_TIME);
    lazy_static::initialize(&TURBO);
}

pub fn init() {}