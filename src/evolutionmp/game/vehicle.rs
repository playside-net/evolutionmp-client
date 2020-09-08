use super::Handle;
use crate::{invoke, impl_handle};
use crate::game::entity::Entity;
use crate::game::ped::Ped;
use crate::game::streaming::{Model, Resource};
use crate::game::radio::RadioStation;
use crate::game::worldprobe::ProbeEntity;
use crate::hash::{Hashable, Hash};
use crate::native::vehicle::{CURRENT_GEAR, CURRENT_RPM, HIGH_GEAR, WHEEL_SPEED, ACCELERATION, STEERING_SCALE, STEERING_ANGLE, GEARS, CLUTCH, TURBO, BRAKE_POWER, THROTTLE, THROTTLE_POWER, TRAIN_TRACK_NODE, LIGHTS, FUEL_LEVEL, ENGINE_TEMPERATURE, OIL_LEVEL, OIL_VOLUME, DASHBOARD_SPEED, HANDBRAKE, ENGINE_POWER};
use crate::native::pool::{Handleable, VehiclePool};
use cgmath::{Vector3, Rad, Vector2};

pub fn get_pool() -> &'static Box<VehiclePool> {
    crate::native::pool::VEHICLE.as_ref().as_ref().expect("vehicle pool is not initialized")
}

pub fn set_parked_count(count: i32) {
    invoke!((), 0xCAA15F13EBD417FF, count)
}

pub fn set_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x245A6883D966D537, multiplier)
}

pub fn set_random_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0xB3B3359379FE77D3, multiplier)
}

pub fn set_parked_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0xEAE6DCC7EEE3DB1D, multiplier)
}

pub fn set_garbage_trucks(enabled: bool) {
    invoke!((), 0x2AFD795EEAC8D30D, enabled)
}

pub fn set_random_boats(enabled: bool) {
    invoke!((), 0x84436EC293B1415F, enabled)
}

pub fn set_random_trains(enabled: bool) {
    invoke!((), 0x80D9F74197EA47D9, enabled)
}

pub fn set_far_draw(enabled: bool) {
    invoke!((), 0x26324F33423F3CC3, enabled)
}

pub fn set_distant_visible(visible: bool) {
    invoke!((), 0xF796359A959DF65D, visible)
}

pub fn set_distant_lights_visible(visible: bool) {
    invoke!((), 0xC9F98AC1884E73A2, !visible)
}

pub fn delete_all_trains() {
    invoke!((), 0x736A718577F39C7D)
}

pub fn set_low_priority_generators_active(active: bool) {
    invoke!((), 0x608207E7A8FB787C, active)
}

pub fn remove_vehicles_from_generators_in_area(start: Vector3<f32>, end: Vector3<f32>, unknown: bool) {
    invoke!((), 0x46A1E1A299EC4BBA, start, end, unknown)
}

#[repr(u32)]
pub enum Dispatch {
    PoliceAutomobile,
    PoliceHelicopter,
    FireDepartment,
    SwatAutomobile,
    AmbulanceDepartment,
    PoliceRiders,
    PoliceVehicleRequest,
    PoliceRoadBlock,
    PoliceAutomobileWaitPulledOver,
    PoliceAutomobileWaitCruising,
    Gangs,
    SwatHelicopter,
    PoliceBoat,
    ArmyVehicle,
    BikerBackup
}

pub fn set_dispatch_service(dispatch: Dispatch, enabled: bool) {
    invoke!((), 0xDC0F817884CDD856, dispatch as u32, enabled)
}

#[derive(Debug)]
pub struct DispatchSpawnBlockingArea {
    handle: Handle
}

impl DispatchSpawnBlockingArea {
    pub fn new(start: Vector2<f32>, end: Vector2<f32>) -> Option<Self> {
        invoke!(Option<Self>, 0x2D4259F1FEB81DA9, start, end)
    }

    pub fn delete(self) {
        invoke!((), 0x264AC28B01B353A5, self.handle)
    }
}

crate::impl_handle!(DispatchSpawnBlockingArea);

pub struct MissionTrain {
    pub vehicle: Vehicle
}

impl MissionTrain {
    pub fn new(model: u8, pos: Vector3<f32>, direction: bool) -> Option<MissionTrain> {
        let models = vec![
            Model::from("freight"),
            Model::from("freightcar"),
            Model::from("freightgrain"),
            Model::from("freightcont1"),
            Model::from("freightcont2"),
            Model::from("freighttrailer"),
            Model::from("tankercar"),
            Model::from("metrotrain")
        ];
        for model in models.iter() {
            model.request_and_wait();
        }
        let vehicle = invoke!(Option<Vehicle>, 0x63C6CCA8E68AE8C8, model as u32, pos, direction)?;
        Some(MissionTrain { vehicle })
    }

    pub fn set_position(&self, pos: Vector3<f32>) {
        invoke!((), 0x591CA673AA6AB736, self.vehicle.handle, pos)
    }

    pub fn get_carriage(&self, trailer: u32) -> Option<ProbeEntity> {
        invoke!(Option<ProbeEntity>, 0x08AAFD0814722BC3, self.vehicle.handle, trailer)
    }

    pub fn set_speed(&self, speed: f32) {
        invoke!((), 0xAA0BC91BE0B796E3, self.vehicle.handle, speed)
    }

    pub fn set_cruise_speed(&self, speed: f32) {
        invoke!((), 0x16469284DB8C62B5, self.vehicle.handle, speed)
    }

    pub fn get_track_node(&self) -> i32 {
        TRAIN_TRACK_NODE.get(&self.vehicle)
    }

    pub fn as_vehicle(&self) -> &Vehicle {
        &self.vehicle
    }

    pub fn delete(&mut self) {
        invoke!((), 0x5B76B14AE875C795, &mut self.vehicle.handle)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct Vehicle {
    handle: Handle
}

impl Vehicle {
    pub fn new<H>(model: H, pos: Vector3<f32>, heading: f32, is_network: bool, this_script_check: bool) -> Option<Vehicle> where H: Hashable {
        let model = Model::from(model);
        if model.is_in_cd_image() && model.is_valid() && model.is_vehicle() {
            model.request_and_wait();
            invoke!(Option<Vehicle>, 0xAF35D0D2583051B0, model.joaat(), pos, heading, is_network, this_script_check)
        } else {
            None
        }
    }

    pub fn get_colors(&self) -> VehicleColors {
        let mut primary = 0;
        let mut secondary = 0;
        invoke!((), 0xA19435F193E081AC, self.handle, &mut primary, &mut secondary);
        VehicleColors { primary, secondary }
    }

    pub fn set_colors(&self, primary: u32, secondary: u32) {
        invoke!((), 0x4F1D4BE3A7F24601, self.handle, primary, secondary)
    }

    pub fn get_current_gear(&self) -> i32 {
        CURRENT_GEAR.get(self)
    }

    pub fn set_current_gear(&self, gear: i32) {
        CURRENT_GEAR.set(self, gear)
    }

    pub fn get_high_gear(&self) -> i32 {
        HIGH_GEAR.get(self)
    }

    pub fn set_high_gear(&self, high_gear: i32) {
        HIGH_GEAR.set(self, high_gear)
    }

    pub fn get_current_rpm(&self) -> f32 {
        CURRENT_RPM.get(self)
    }

    pub fn set_current_rpm(&self, rpm: f32) {
        CURRENT_RPM.set(self, rpm)
    }

    pub fn get_dashboard_speed(&self) -> f32 {
        DASHBOARD_SPEED.get(self)
    }

    pub fn get_wheel_speed(&self) -> f32 {
        WHEEL_SPEED.get(self)
    }

    pub fn set_wheel_speed(&self, speed: f32) {
        WHEEL_SPEED.set(self, speed)
    }

    pub fn get_acceleration(&self) -> f32 {
        ACCELERATION.get(self)
    }

    pub fn set_acceleration(&self, acceleration: f32) {
        ACCELERATION.set(self, acceleration)
    }

    pub fn get_fuel(&self) -> f32 {
        FUEL_LEVEL.get(self)
    }

    pub fn set_fuel(&self, fuel: f32) {
        FUEL_LEVEL.set(self, fuel)
    }

    pub fn get_oil(&self) -> f32 {
        OIL_LEVEL.get(self)
    }

    pub fn set_oil(&self, oil: f32) {
        OIL_LEVEL.set(self, oil)
    }

    pub fn get_max_oil(&self) -> f32 {
        OIL_VOLUME.get(self)
    }

    pub fn get_engine_temperature(&self) -> f32 {
        ENGINE_TEMPERATURE.get(self)
    }

    pub fn set_engine_temperature(&self, temperature: f32) {
        ENGINE_TEMPERATURE.set(self, temperature)
    }

    pub fn get_engine_power(&self) -> f32 {
        ENGINE_POWER.get(self)
    }

    pub fn get_brake_power(&self) -> f32 {
        BRAKE_POWER.get(self)
    }

    pub fn get_steering_scale(&self) -> f32 {
        STEERING_SCALE.get(self)
    }

    pub fn set_steering_scale(&self, scale: f32) {
        STEERING_SCALE.set(self, scale)
    }

    pub fn get_steering_angle(&self) -> Rad<f32> {
        Rad(STEERING_ANGLE.get(self))
    }

    pub fn set_steering_angle(&self, angle: Rad<f32>) {
        STEERING_ANGLE.set(self, angle.0)
    }

    pub fn get_gears(&self) -> i32 {
        GEARS.get(self)
    }

    pub fn set_gears(&self, gears: i32) {
        GEARS.set(self, gears)
    }

    pub fn get_clutch(&self) -> f32 {
        CLUTCH.get(self)
    }

    pub fn set_clutch(&self, clutch: f32) {
        CLUTCH.set(self, clutch)
    }

    pub fn get_turbo(&self) -> f32 {
        TURBO.get(self)
    }

    pub fn set_turbo(&self, turbo: f32) {
        TURBO.set(self, turbo)
    }

    pub fn get_throttle(&self) -> f32 {
        THROTTLE.get(self)
    }

    pub fn set_throttle(&self, throttle: f32) {
        THROTTLE.set(self, throttle)
    }

    pub fn get_throttle_power(&self) -> f32 {
        THROTTLE_POWER.get(self)
    }

    pub fn set_throttle_power(&self, power: f32) {
        THROTTLE_POWER.set(self, power)
    }

    pub fn is_handbrake(&self) -> bool {
        HANDBRAKE.get(self)
    }

    pub fn get_passenger(&self, seat: i32) -> Option<Ped> {
        invoke!(Option<Ped>, 0xBB40DD2270B65366, self.handle, seat)
    }

    pub fn get_max_passengers(&self) -> u32 {
        invoke!(u32, 0xA7C4F2C6E744A550, self.handle)
    }

    pub fn set_taxi_lights(&self, lights: bool) {
        invoke!((), 0x598803E85E8448D9, self.handle, lights)
    }

    pub fn get_class(&self) -> VehicleClass {
        invoke!(VehicleClass, 0x29439776AAA00A62, self.handle)
    }

    pub fn set_mod(&self, id: u32, value: i32, custom_tires: bool) {
        invoke!((), 0x6AF0636DDEDCB6DD, self.handle, id, value, custom_tires)
    }

    pub fn set_mod_kit(&self, id: u32) {
        invoke!((), 0x1F2AA07F00B3217A, self.handle, id)
    }

    pub fn get_mod_kit(&self) -> u32 {
        invoke!(u32, 0x6325D1A044AE510D, self.handle)
    }

    pub fn get_mod_kit_type(&self) -> u32 {
        invoke!(u32, 0xFC058F5121E54C32, self.handle)
    }

    pub fn set_livery(&self, livery: u32) {
        invoke!((), 0x60BF608F1B8CD1B6, self.handle, livery)
    }

    pub fn is_seat_free(&self, seat: i32) -> bool {
        invoke!(bool, 0x22AC59A870E6A669, self.handle, seat)
    }

    pub fn repair(&self) {
        invoke!((), 0x115722B1B9C14C1C, self.handle)
    }

    pub fn repair_deformation(&self) {
        invoke!((), 0x953DA1E1B12C0491, self.handle)
    }

    pub fn place_on_ground(&self) {
        invoke!((), 0x49733E92263139D1, self.handle)
    }

    pub fn start_horn<H>(&self, duration: u32, hash: H, forever: bool) where H: Hashable {
        invoke!((), 0x9C8C6504B5B63D2C, self.handle, duration, hash.joaat(), forever)
    }

    pub fn get_waypoint_progress(&self) -> f32 {
        invoke!(f32, 0x9824CFF8FC66E159, self.handle)
    }

    pub fn get_waypoint_target_point(&self) -> Handle {
        invoke!(Handle, 0x416B62AC8B9E5BBD, self.handle)
    }

    pub fn get_radio(&self) -> VehicleRadio {
        VehicleRadio {
            vehicle: self
        }
    }

    pub fn is_model<H>(&self, model: H) -> bool where H: Hashable {
        invoke!(bool, 0x423E8DE37D934D89, self.handle, model.joaat())
    }

    pub fn is_any_model<H>(&self, models: &[H]) -> bool where H: Hashable {
        models.iter().any(|m|self.is_model(m.joaat()))
    }

    pub fn is_engine_on(&self) -> bool {
        invoke!(bool, 0xAE31E7DF9B5B132E, self.handle)
    }

    pub fn set_engine_on(&self, on: bool, instant: bool, disable_auto_start: bool) {
        invoke!((), 0x2497C4717C8B881E, self.handle, on, instant, disable_auto_start)
    }

    pub fn get_engine_health(&self) -> f32 {
        invoke!(f32, 0xC45D23BAF168AAB8, self.handle)
    }

    pub fn copy_damage_to(&self, target: &Vehicle) {
        invoke!((), 0xE44A982368A4AF23, self.handle, target.get_handle())
    }

    pub fn has_door(&self, door: u32) -> bool {
        invoke!(bool, 0x645F4B6E8499F632, self.handle, door)
    }

    pub fn has_roof(&self) -> bool {
        invoke!(bool, 0x8AC862B0B32C5B80, self.handle)
    }

    pub fn has_weapon(&self) -> bool {
        invoke!(bool, 0x25ECB9F8017D98E0, self.handle)
    }

    pub fn has_kers_boost(&self) -> bool {
        invoke!(bool, 0x50634E348C8D44EF, self.handle)
    }

    pub fn has_rocket_boost(&self) -> bool {
        invoke!(bool, 0x36D782F68B309BDA, self.handle)
    }

    pub fn has_jumping_ability(&self) -> bool {
        invoke!(bool, 0x9078C0C5EF8C19E9, self.handle)
    }

    pub fn get_last_ped_in_seat(&self, seat: i32) -> Option<Ped> {
        invoke!(Option<Ped>, 0x83F969AA1EE2A664, self.handle, seat)
    }

    pub fn is_interior_light(&self) -> bool {
        let lights = LIGHTS.get(self);
        (lights & 0b01000000) > 0
    }

    pub fn set_interior_light(&self, enabled: bool) {
        invoke!((), 0xBC2042F090AF6AD3, self.handle, enabled)
    }

    pub fn get_indicator_light(&self) -> u8 {
        let lights = LIGHTS.get(self);
        lights.to_ne_bytes()[0]
    }

    pub fn as_cargobob(&self) -> Option<VehicleCargobob> {
        if self.is_any_model(&["cargobob", "cargobob1", "cargobob2", "cargobob3"]) {
            Some(VehicleCargobob {
                vehicle: self
            })
        } else {
            None
        }
    }

    pub fn as_towtruck(&self) -> Option<VehicleTowtruck> {
        if self.is_any_model(&["towtruck", "towtruck2"]) {
            Some(VehicleTowtruck {
                vehicle: self
            })
        } else {
            None
        }
    }

    pub fn get_remote_id(&self) -> Option<u32> {
        crate::scripts::network::get_vehicle_remote_id(self)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum VehicleClass {
    Compact,
    Sedan,
    SUV,
    Coupe,
    Muscle,
    SportClassic,
    Sport,
    Super,
    Motorcycle,
    OffRoad,
    Industrial,
    Utility,
    Van,
    Cycle,
    Boat,
    Helicopter,
    Plane,
    Service,
    Emergency,
    Military,
    Commercial,
    Train
}

impl Handleable for VehicleClass {
    fn from_handle(handle: u32) -> Option<Self> where Self: Sized {
        if handle > 21 {
            None
        } else {
            Some(unsafe { std::mem::transmute(handle) })
        }
    }

    fn get_handle(&self) -> u32 {
        *self as u32
    }
}

impl VehicleClass {
    pub fn from<H>(hash: H) -> Option<VehicleClass> where H: Hashable {
        invoke!(Option<Self>, 0xDEDF1C8BD47C2200, hash.joaat())
    }

    pub fn get_estimated_max_speed(&self) -> f32 {
        invoke!(f32, 0x00C09F246ABEDD82, self.get_handle())
    }

    pub fn get_max_acceleration(&self) -> f32 {
        invoke!(f32, 0x2F83E7E45D9EA7AE, self.get_handle())
    }

    pub fn get_max_agility(&self) -> f32 {
        invoke!(f32, 0x4F930AD022D6DE3B, self.get_handle())
    }

    pub fn get_max_braking(&self) -> f32 {
        invoke!(f32, 0x4BF54C16EC8FEC03, self.get_handle())
    }

    pub fn get_max_traction(&self) -> f32 {
        invoke!(f32, 0xDBC86D85C5059461, self.get_handle())
    }

    pub fn has_custom_horns(&self) -> bool {
        match self {
            VehicleClass::Emergency
            | VehicleClass::Service
            | VehicleClass::Helicopter
            | VehicleClass::Plane
            | VehicleClass::Commercial
            | VehicleClass::Boat
            | VehicleClass::Train => false,
            _ => true
        }
    }
}

pub struct VehicleColors {
    pub primary: u32,
    pub secondary: u32
}

impl Entity for Vehicle {
    fn delete(&mut self) {
        self.set_persistent(false);
        invoke!((), 0xEA386986E786A54F, &mut self.handle)
    }
}

impl_handle!(Vehicle);

pub struct VehicleRadio<'a> {
    vehicle: &'a Vehicle
}

impl<'a> VehicleRadio<'a> {
    pub fn is_loud(&self) -> bool {
        invoke!(bool, 0x032A116663A4D5AC, self.vehicle.handle)
    }

    pub fn set_loud(&self, loud: bool) {
        invoke!((), 0xBB6F1CAEC68B0BCE, self.vehicle.handle, loud)
    }

    pub fn set_enabled(&self, enabled: bool) {
        invoke!((), 0x3B988190C0AA6C0B, self.vehicle.handle, enabled)
    }

    pub fn set_station(&self, station: &RadioStation) {
        invoke!((), 0x1B9C0099CB942AC6, self.vehicle.handle, station.get_name())
    }

    pub fn skip_track(&self) {
        invoke!((), 0x6DDBBDD98E2E9C25, self.vehicle.handle)
    }
}

pub struct VehicleModel {
    hash: Hash
}

impl VehicleModel {
    pub fn from_vehicle(veh: &Vehicle) -> VehicleModel {
        VehicleModel {
            hash: veh.get_model()
        }
    }

    pub fn get_display_name(&self) -> &str {
        invoke!(&str, 0xB215AAC32D25D019, self.hash)
    }

    pub fn get_acceleration(&self) -> f32 {
        invoke!(f32, 0x8C044C5C84505B6A, self.hash)
    }

    pub fn get_down_force(&self) -> f32 {
        invoke!(f32, 0x53409B5163D5B846, self.hash)
    }

    pub fn get_estimated_max_speed(&self) -> f32 {
        invoke!(f32, 0xF417C2502FFFED43, self.hash)
    }

    pub fn get_max_braking(&self) -> f32 {
        invoke!(f32, 0xDC53FD41B4ED944C, self.hash)
    }

    pub fn get_max_braking_max_mods(&self) -> f32 {
        invoke!(f32, 0xBFBA3BA79CFF7EBF, self.hash)
    }

    pub fn get_max_knots(&self) -> f32 {
        invoke!(f32, 0xC6AD107DDC9054CC, self.hash)
    }

    pub fn get_max_traction(&self) -> f32 {
        invoke!(f32, 0x539DE94D44FDFD0D, self.hash)
    }

    pub fn get_move_resistance(&self) -> f32 {
        invoke!(f32, 0x5AA3F878A178C4FC, self.hash)
    }

    pub fn get_seats(&self) -> u32 {
        invoke!(u32, 0x2AD93716F184EDA4, self.hash)
    }

    pub fn is_bicycle(&self) -> bool {
        invoke!(bool, 0xBF94DD42F63BDED2, self.hash)
    }

    pub fn is_bike(&self) -> bool {
        invoke!(bool, 0xB50C0B0CEDC6CE84, self.hash)
    }

    pub fn is_boat(&self) -> bool {
        invoke!(bool, 0x45A9187928F4B9E3, self.hash)
    }

    pub fn is_car(&self) -> bool {
        invoke!(bool, 0x7F6DB52EEFC96DF8, self.hash)
    }

    pub fn is_helicopter(&self) -> bool {
        invoke!(bool, 0xDCE4334788AF94EA, self.hash)
    }

    pub fn is_jet_ski(&self) -> bool {
        invoke!(bool, 0x9537097412CF75FE, self.hash)
    }

    pub fn is_plane(&self) -> bool {
        invoke!(bool, 0xA0948AB42D7BA0DE, self.hash)
    }

    pub fn is_quad_bike(&self) -> bool {
        invoke!(bool, 0x39DAC362EE65FA28, self.hash)
    }

    pub fn is_train(&self) -> bool {
        invoke!(bool, 0xAB935175B22E822B, self.hash)
    }

    pub fn is_amphibious_car(&self) -> bool {
        invoke!(bool, 0x633F6F44A537EBB6, self.hash)
    }

    pub fn is_amphibious_quad_bike(&self) -> bool {
        invoke!(bool, 0xA1A9FC1C76A6730D, self.hash)
    }
}

impl Hashable for VehicleModel {
    fn joaat(&self) -> Hash {
        self.hash
    }
}

pub struct VehicleCargobob<'a> {
    vehicle: &'a Vehicle
}

impl<'a> VehicleCargobob<'a> {
    pub fn attach_entity(&self, entity: &dyn Entity, p2: i32, hook_offset: Vector3<f32>) {
        invoke!((), 0xA1DD82F3CCF9A01E, self.vehicle.handle, entity.get_handle(), p2, hook_offset)
    }

    pub fn get_attached_entity(&self) -> Option<ProbeEntity> {
        invoke!(Option<ProbeEntity>, 0x99093F60746708CA, self.vehicle.handle)
    }

    pub fn attach_vehicle(&self, vehicle: &Vehicle, p2: i32, hook_offset: Vector3<f32>) {
        invoke!((), 0x4127F1D84E347769, self.vehicle.handle, vehicle.get_handle(), p2, hook_offset)
    }

    pub fn get_attached_vehicle(&self) -> Option<Vehicle> {
        invoke!(Option<Vehicle>, 0x873B82D42AC2B9E5, self.vehicle.handle)
    }

    pub fn is_vehicle_attached(&self, vehicle: &Vehicle) -> bool {
        invoke!(bool, 0xD40148F22E81A1D9, self.vehicle.handle, vehicle.get_handle())
    }

    pub fn create_rope(&self, ty: u32) {
        invoke!((), 0x7BEB0C7A235F6F3B, self.vehicle.handle, ty)
    }

    pub fn has_rope(&self) -> bool {
        invoke!(bool, 0x1821D91AD4B56108, self.vehicle.handle)
    }

    pub fn remove_rope(&self) {
        invoke!((), 0x9768CF648F54C804, self.vehicle.handle)
    }

    pub fn set_rope_type(&self, ty: u32) {
        invoke!((), 0x0D5F65A8F4EBDAB5, self.vehicle.handle, ty)
    }

    pub fn has_magnet(&self) -> bool {
        invoke!(bool, 0x6E08BF5B3722BAC9, self.vehicle.handle)
    }

    pub fn detach_entity(&self, entity: &dyn Entity) {
        invoke!((), 0xAF03011701811146, self.vehicle.handle, entity.get_handle())
    }

    pub fn detach_vehicle(&self, vehicle: &Vehicle) {
        invoke!((), 0x0E21D3DF1051399D, vehicle.get_handle(), self.vehicle.handle)
    }

    pub fn get_hook_position(&self) -> Vector3<f32> {
        invoke!(Vector3<f32>, 0xCBDB9B923CACC92D, self.vehicle.handle)
    }

    pub fn set_hook_position(&self, pos: Vector3<f32>, ty: u32) {
        invoke!((), 0x877C1EAEAC531023, self.vehicle.handle, pos, ty)
    }

    pub fn set_magnet_active(&self, active: bool) {
        invoke!((), 0x9A665550F8DA349B, self.vehicle.handle, active)
    }

    pub fn set_magnet_effect_radius(&self, radius: f32) {
        invoke!((), 0xA17BAD153B51547E, self.vehicle.handle, radius)
    }

    pub fn set_magnet_falloff(&self, falloff: f32) {
        invoke!((), 0x685D5561680D088B, self.vehicle.handle, falloff)
    }

    pub fn set_magnet_pull_rope_length(&self, length: f32) {
        invoke!((), 0x6D8EAC07506291FB, self.vehicle.handle, length)
    }

    pub fn set_magnet_pull_strength(&self, strength: f32) {
        invoke!((), 0xED8286F71A819BAA, self.vehicle.handle, strength)
    }

    pub fn set_magnet_strength(&self, strength: f32) {
        invoke!((), 0xBCBFCD9D1DAC19E2, self.vehicle.handle, strength)
    }

    pub fn set_magnet_reduced_falloff(&self, falloff: f32) {
        invoke!((), 0x66979ACF5102FD2F, self.vehicle.handle, falloff)
    }
}

pub struct VehicleTowtruck<'a> {
    vehicle: &'a Vehicle
}

impl<'a> VehicleTowtruck<'a> {
    pub fn attach_vehicle(&self, vehicle: &Vehicle, near: bool, hook_offset: Vector3<f32>) {
        invoke!((), 0x29A16F8D621C4508, self.vehicle.handle, vehicle.get_handle(), near, hook_offset)
    }

    pub fn detach_vehicle(&self, vehicle: &Vehicle) {
        invoke!((), 0xC2DB6B6708350ED8, self.vehicle.handle, vehicle.get_handle())
    }

    pub fn get_attached_entity(&self) -> Option<ProbeEntity> {
        invoke!(Option<ProbeEntity>, 0xEFEA18DCF10F8F75, self.vehicle.handle)
    }

    pub fn is_vehicle_attached(&self, vehicle: &Vehicle) -> bool {
        invoke!(bool, 0x146DF9EC4C4B9FD4, self.vehicle.handle, vehicle.get_handle())
    }

    pub fn set_crane_uplift(&self, uplift: f32) {
        invoke!((), 0xFE54B92A344583CA, self.vehicle.handle, uplift)
    }
}