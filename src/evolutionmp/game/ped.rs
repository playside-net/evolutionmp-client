use super::Handle;
use crate::native;
use crate::game::entity::Entity;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::native::pool::FromHandle;
use crate::hash::Hashable;
use cgmath::Vector3;

#[derive(Debug)]
pub struct Ped {
    handle: Handle
}

impl Ped {
    pub fn new<H>(ty: u32, model: H, pos: Vector3<f32>, heading: f32, network: bool, this_script_check: bool) -> Option<Ped> where H: Hashable {
        Self::from_handle(native::ped::new(ty, model, pos, heading, network, this_script_check))
    }

    pub fn from_player(player: &Player) -> Ped {
        Ped {
            handle: native::player::get_ped(player.get_handle())
        }
    }

    pub fn local() -> Ped {
        Ped {
            handle: native::player::get_local_ped()
        }
    }

    pub fn is_in_any_vehicle(&self, at_get_in: bool) -> bool {
        native::ped::is_in_any_vehicle(self.handle, at_get_in)
    }

    pub fn get_in_vehicle(&self, last: bool) -> Option<Vehicle> {
        let handle = native::ped::get_in_vehicle(self.handle, last);
        Vehicle::from_handle(handle)
    }

    pub fn get_using_vehicle(&self) -> Option<Vehicle> {
        let handle = native::ped::get_using_vehicle(self.handle);
        Vehicle::from_handle(handle)
    }

    pub fn get_entering_vehicle(&self) -> Option<Vehicle> {
        let handle = native::ped::get_entering_vehicle(self.handle);
        Vehicle::from_handle(handle)
    }

    pub fn put_into_vehicle(&self, vehicle: &Vehicle, seat: i32) {
        native::ped::put_into_vehicle(self.handle, vehicle.get_handle(), seat)
    }

    pub fn set_current_weapon_visible(&self, visible: bool, deselect: bool, p3: bool, p4: bool) {
        native::ped::set_current_weapon_visible(self.handle, visible, deselect, p3, p4)
    }

    pub fn set_config_flag(&self, flag: u32, value: bool) {
        native::ped::set_config_flag(self.handle, flag, value)
    }

    pub fn set_default_component_variation(&self) {
        native::ped::set_default_component_variation(self.handle)
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

impl FromHandle for Ped {
    fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }
}

pub trait NetworkSignalValue {
    fn set(&self, ped: &Ped, name: &str);
}

impl NetworkSignalValue for f32 {
    fn set(&self, ped: &Ped, name: &str) {
        native::ped::task::set_network_move_signal_float(ped.get_handle(), name, *self)
    }
}

impl NetworkSignalValue for bool {
    fn set(&self, ped: &Ped, name: &str) {
        native::ped::task::set_network_move_signal_bool(ped.get_handle(), name, *self)
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
        native::ped::task::clear_immediately(self.ped.handle)
    }

    pub fn clear_secondary(&self) {
        native::ped::task::clear_secondary(self.ped.handle)
    }

    pub fn enter_vehicle(&self, vehicle: Vehicle, timeout: u32, seat: i32, speed: f32, flag: i32) {
        native::ped::task::enter_vehicle(self.ped.handle, vehicle.get_handle(), timeout, seat, speed, flag)
    }
}

pub struct PedNetworkTasks<'a> {
    ped: &'a Ped
}

impl<'a> PedNetworkTasks<'a> {
    pub fn do_move(&self, name: &str, multiplier: f32, p3: bool, dict: &str, flags: u32) {
        native::ped::task::network_move(self.ped.handle, name, multiplier, p3, dict, flags)
    }

    pub fn is_move_active(&self) -> bool {
        native::ped::task::is_network_move_active(self.ped.handle)
    }

    pub fn set_move_signal<S>(&self, name: &str, value: S) where S: NetworkSignalValue {
        value.set(self.ped, name)
    }

    pub fn request_move_state_transition(&self, name: &str) -> bool {
        native::ped::task::request_network_move_state_transition(self.ped.handle, name)
    }
}