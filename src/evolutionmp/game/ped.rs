use super::Handle;
use crate::native;
use crate::game::entity::Entity;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::game::controls::Control::VehicleAccelerate;

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
}

impl Entity for Ped {
    fn get_handle(&self) -> Handle {
        self.handle
    }
}