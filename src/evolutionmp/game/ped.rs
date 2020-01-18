use super::Handle;
use crate::native;
use crate::game::entity::Entity;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::invoke;
use crate::native::pool::{Handleable, Pool};
use crate::hash::Hashable;
use cgmath::{Vector3, MetricSpace};
use crate::game::streaming::AnimDict;

#[derive(Debug)]
pub struct Ped {
    handle: Handle
}

pub fn set_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x95E3D6257B166CF2, multiplier)
}

pub fn set_scenario_density_multiplier_this_frame(multiplier: f32) {
    invoke!((), 0x7A556143A1C03898, multiplier)
}

pub fn set_non_scenario_cops(enabled: bool) {
    invoke!((), 0x8A4986851C4EF6E7, enabled)
}

pub fn set_scenario_cops(enabled: bool) {
    invoke!((), 0x444CB7D7DBE6973D, enabled)
}

pub fn set_cops(enabled: bool) {
    invoke!((), 0x102E68B2024D536D, enabled)
}

impl Ped {
    pub fn new<H>(ty: u32, model: H, pos: Vector3<f32>, heading: f32, network: bool, this_script_check: bool) -> Option<Ped> where H: Hashable {
        invoke!(Option<Ped>, 0xD49F9B0955C367DE, ty, model.joaat(), pos, heading, network, this_script_check)
    }

    pub fn from_player(player: &Player) -> Ped {
        invoke!(Ped, 0x43A66C31C68491C0, player.get_handle())
    }

    pub fn local() -> Ped {
        invoke!(Ped, 0xD80958FC74E988A6)
    }

    pub fn is_in_any_vehicle(&self, at_get_in: bool) -> bool {
        invoke!(bool, 0x997ABD671D25CA0B, self.handle, at_get_in)
    }

    pub fn get_in_vehicle(&self, last: bool) -> Option<Vehicle> {
        invoke!(Option<Vehicle>, 0x9A9112A0FE9A4713, self.handle, last)
    }

    pub fn get_using_vehicle(&self) -> Option<Vehicle> {
        invoke!(Option<Vehicle>, 0x6094AD011A2EA87D, self.handle)
    }

    pub fn get_entering_vehicle(&self) -> Option<Vehicle> {
        invoke!(Option<Vehicle>, 0xF92691AED837A5FC, self.handle)
    }

    pub fn put_into_vehicle(&self, vehicle: &Vehicle, seat: i32) {
        invoke!((), 0xF75B0D629E1C063D, self.handle, vehicle.get_handle(), seat)
    }

    pub fn set_current_weapon_visible(&self, visible: bool, deselect: bool, p3: bool, p4: bool) {
        invoke!((), 0x0725A4CCFDED9A70, self.handle, visible, deselect, p3, p4)
    }

    pub fn set_config_flag(&self, flag: u32, value: bool) {
        invoke!((), 0x1913FE4CBF41C463, self.handle, flag, value)
    }

    pub fn set_default_component_variation(&self) {
        invoke!((), 0x45EEE61580806D63, self.handle)
    }

    pub fn get_closest_vehicle(&self, max_distance: f32) -> Option<Vehicle> {
        let pos = self.get_position_by_offset(Vector3::new(0.0, 0.0, -1.0));
        let mut result = None;
        let mut last_max_distance = max_distance;
        if let Some(vehicles) = native::pool::get_vehicles() {
            for vehicle in vehicles.iter() {
                if vehicle.exists() {
                    let v_pos = vehicle.get_position_by_offset(Vector3::new(0.0, 0.0, 0.0));
                    let distance = v_pos.distance(pos);
                    if distance < last_max_distance {
                        last_max_distance = distance;
                        result = Some(vehicle);
                    }
                }
            }
        }
        result
    }

    pub fn get_tasks(&self) -> PedTasks {
        PedTasks {
            ped: self
        }
    }
}

impl Entity for Ped {
    fn get_handle(&self) -> Handle {
        self.handle
    }

    fn delete(&mut self) {
        self.set_persistent(false);
        native::entity::delete(&mut self.handle)
    }
}

impl Handleable for Ped {
    fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }

    fn get_handle(&self) -> u32 {
        self.handle
    }
}

pub trait NetworkSignalValue {
    fn set(&self, ped: &Ped, name: &str);
}

impl NetworkSignalValue for f32 {
    fn set(&self, ped: &Ped, name: &str) {
        invoke!((), 0xD5BB4025AE449A4E, ped.get_handle(), name, *self)
    }
}

impl NetworkSignalValue for bool {
    fn set(&self, ped: &Ped, name: &str) {
        invoke!((), 0xB0A6CFD2C69C1088, ped.get_handle(), name, *self)
    }
}

pub struct PedTasks<'a> {
    ped: &'a Ped
}

impl<'a> PedTasks<'a> {
    pub fn get_network(self) -> PedNetworkTasks<'a> {
        PedNetworkTasks {
            ped: self.ped
        }
    }

    pub fn clear_immediately(&self) {
        invoke!((), 0xAAA34F8A7CB32098, self.ped.handle)
    }

    pub fn clear_secondary(&self) {
        invoke!((), 0x176CECF6F920D707, self.ped.handle)
    }

    pub fn enter_vehicle(&self, vehicle: Vehicle, timeout: u32, seat: i32, speed: f32, flag: i32) {
        invoke!((), 0xC20E50AA46D09CA8, self.ped.handle, vehicle.get_handle(), timeout, seat, speed, flag, 0u32)
    }
}

pub struct PedNetworkTasks<'a> {
    ped: &'a Ped
}

impl<'a> PedNetworkTasks<'a> {
    pub fn do_move(&self, name: &str, multiplier: f32, p3: bool, dict: &AnimDict, flags: u32) {
        invoke!((), 0x2D537BA194896636, self.ped.handle, name, multiplier, p3, dict.get_name(), flags)
    }

    pub fn is_move_active(&self) -> bool {
        invoke!(bool, 0x921CE12C489C4C41, self.ped.handle)
    }

    pub fn set_move_signal<S>(&self, name: &str, value: S) where S: NetworkSignalValue {
        value.set(self.ped, name)
    }

    pub fn request_move_state_transition(&self, name: &str) -> bool {
        invoke!(bool, 0xD01015C7316AE176, self.ped.handle, name)
    }
}