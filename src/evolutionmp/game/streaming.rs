use crate::invoke;
use crate::native;
use crate::hash::{Hash, Hashable};
use crate::runtime::ScriptEnv;
use std::time::Duration;
use cgmath::Vector3;

pub trait Resource {
    fn is_loaded(&self) -> bool;
    fn request(&self);
}

#[derive(Copy, Clone)]
pub struct Model {
    hash: Hash
}

pub fn request_collision_at(pos: Vector3<f32>) {
    invoke!((), 0x07503F7948F491A7, pos)
}

pub fn set_vehicle_population_budget(budget: u32) {
    invoke!((), 0xCB9E1EB3BE2AF4E9, budget)
}

pub fn set_ped_population_budget(budget: u32) {
    invoke!((), 0x8C95333CFC3340F3, budget)
}

pub fn stop_player_switch() {
    invoke!((), 0x95C0A5BBDC189AA1)
}

pub fn load_scene(pos: Vector3<f32>) {
    invoke!((), 0x4448EB75B4904BDB, pos)
}

pub fn request_menu_ped_model(model: Hash) {
    invoke!((), 0xA0261AEF7ACFC51E, model)
}

pub fn request_models_in_room(interior: u32, room: &str) {
    invoke!((), 0x8A7A40100EDFEC58, interior, room)
}

impl Model {
    pub fn new<H>(hash: H) -> Model where H: Hashable {
        Model {
            hash: hash.joaat()
        }
    }

    pub fn is_collision_loaded(&self) -> bool {
        invoke!(bool, 0x22CCA434E368F03A, self.hash)
    }

    pub fn is_valid(&self) -> bool {
        invoke!(bool, 0xC0296A2EDF545E92, self.hash)
    }

    pub fn is_in_cd_image(&self) -> bool {
        invoke!(bool, 0x35B9E0803292B641, self.hash)
    }

    pub fn is_vehicle(&self) -> bool {
        invoke!(bool, 0x19AAC8F07BFEC53E, self.hash)
    }

    pub fn request_collision(&self) {
        invoke!((), 0x923CB32A3B874FCB, self.hash)
    }

    pub fn mark_unused(&self) {
        invoke!((), 0xE532F5D78798DAAB, self.hash)
    }
}

impl Resource for Model {
    fn is_loaded(&self) -> bool {
        invoke!(bool, 0x98A4EB5D89A0C952, self.hash)
    }

    fn request(&self) {
        invoke!((), 0x963D27A58DF860AC, self.hash)
    }
}

impl Hashable for Model {
    fn joaat(&self) -> Hash {
        self.hash
    }
}

pub struct AnimDict {
    name: String
}

impl AnimDict {
    pub fn new<N>(name: N) -> AnimDict where N: Into<String> {
        AnimDict {
            name: name.into()
        }
    }
}

impl AnimDict {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_valid(&self) -> bool {
        invoke!(bool, 0x2DA49C3B79856961, self.get_name())
    }

    pub fn mark_unused(&self) {
        invoke!((), 0xF66A602F829E2A06, self.get_name())
    }
}

impl Resource for AnimDict {
    fn is_loaded(&self) -> bool {
        invoke!(bool, 0xD031A9162D01088C, self.get_name())
    }

    fn request(&self) {
        invoke!((), 0xD3BD40951412FEF6, self.get_name())
    }
}