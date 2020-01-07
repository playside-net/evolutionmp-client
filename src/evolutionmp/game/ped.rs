use super::Handle;
use crate::native;
use crate::game::entity::Entity;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::game::controls::Control::VehicleAccelerate;
use crate::native::pool::FromHandle;

pub struct Ped {
    handle: Handle
}

impl Ped {
    pub fn from_player(player: &Player) -> Ped {
        Ped {
            handle: unsafe { native::player::get_ped(player.get_handle()) }
        }
    }

    pub fn local() -> Ped {
        Ped {
            handle: unsafe { native::player::get_local_ped() }
        }
    }

    pub fn is_in_any_vehicle(&self, at_get_in: bool) -> bool {
        unsafe { native::ped::is_in_any_vehicle(self.handle, at_get_in) }
    }

    pub fn get_in_vehicle(&self, last: bool) -> Option<Vehicle> {
        unsafe {
            let handle = native::ped::get_in_vehicle(self.handle, last);
            Vehicle::from_handle(handle)
        }
    }

    pub fn get_using_vehicle(&self) -> Option<Vehicle> {
        unsafe {
            let handle = native::ped::get_using_vehicle(self.handle);
            Vehicle::from_handle(handle)
        }
    }

    pub fn get_entering_vehicle(&self) -> Option<Vehicle> {
        unsafe {
            let handle = native::ped::get_entering_vehicle(self.handle);
            Vehicle::from_handle(handle)
        }
    }

    pub fn clear_tasks_immediately(&self) {
        unsafe { native::ped::clear_tasks_immediately(self.handle) }
    }

    pub fn put_into_vehicle(&self, vehicle: &Vehicle, seat: i32) {
        unsafe { native::ped::put_into_vehicle(self.handle, vehicle.get_handle(), seat) }
    }

    pub fn set_current_weapon_visible(&self, visible: bool, deselect: bool, p3: bool, p4: bool) {
        unsafe { native::ped::set_current_weapon_visible(self.handle, visible, deselect, p3, p4) }
    }

    pub fn set_config_flag(&self, flag: u32, value: bool) {
        unsafe { native::ped::set_config_flag(self.handle, flag, value) }
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
        unsafe { native::entity::delete(&mut self.handle) }
    }
}

impl FromHandle for Ped {
    unsafe fn from_handle(handle: Handle) -> Option<Self> {
        if handle == 0 {
            None
        } else {
            Some(Self { handle })
        }
    }
}

pub trait NetworkSignalValue {
    unsafe fn set(&self, ped: &Ped, name: &str);
}

impl NetworkSignalValue for f32 {
    unsafe fn set(&self, ped: &Ped, name: &str) {
        native::ped::task::set_network_move_signal_float(ped.get_handle(), name, *self)
    }
}

impl NetworkSignalValue for bool {
    unsafe fn set(&self, ped: &Ped, name: &str) {
        native::ped::task::set_network_move_signal_bool(ped.get_handle(), name, *self)
    }
}

pub struct PedTasks<'a> {
    ped: &'a Ped
}

impl<'a> PedTasks<'a> {
    pub fn clear_secondary(&self) {
        unsafe { native::ped::task::clear_secondary(self.ped.handle) }
    }

    pub fn network_move(&self, name: &str, multiplier: f32, p3: bool, dict: &str, flags: u32) {
        unsafe { native::ped::task::network_move(self.ped.handle, name, multiplier, p3, dict, flags) }
    }

    pub fn is_network_move_active(&self) -> bool {
        unsafe { native::ped::task::is_network_move_active(self.ped.handle) }
    }

    pub fn set_network_move_signal<S>(&self, name: &str, value: S) where S: NetworkSignalValue {
        unsafe { value.set(self.ped, name) }
    }

    pub fn request_network_move_state_transition(&self, name: &str) -> bool {
        unsafe { native::ped::task::request_network_move_state_transition(self.ped.handle, name) }
    }
}