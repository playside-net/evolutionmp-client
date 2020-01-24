use crate::invoke;
use crate::native;
use crate::hash::{Hash, Hashable};
use crate::runtime::ScriptEnv;
use std::time::Duration;
use cgmath::Vector3;
use crate::native::{NativeStackValue, NativeStackReader, NativeStackWriter};
use crate::game::Handle;
use crate::game::ped::Ped;
use crate::native::pool::Handleable;

pub trait Resource {
    fn is_loaded(&self) -> bool;
    fn request(&self);
    fn mark_unused(&self);
}

#[derive(Clone)]
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

pub fn load_scene(pos: Vector3<f32>) {
    invoke!((), 0x4448EB75B4904BDB, pos)
}

pub fn request_menu_ped_model(model: Hash) {
    invoke!((), 0xA0261AEF7ACFC51E, model)
}

pub fn request_models_in_room(interior: u32, room: &str) {
    invoke!((), 0x8A7A40100EDFEC58, interior, room)
}

pub fn is_player_switch_in_progress() -> bool {
    invoke!(bool, 0xD9D2CFFF49FAB35F)
}

pub fn stop_player_switch() {
    invoke!((), 0x95C0A5BBDC189AA1)
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
}

impl Resource for Model {
    fn is_loaded(&self) -> bool {
        invoke!(bool, 0x98A4EB5D89A0C952, self.hash)
    }

    fn request(&self) {
        invoke!((), 0x963D27A58DF860AC, self.hash)
    }

    fn mark_unused(&self) {
        invoke!((), 0xE532F5D78798DAAB, self.hash)
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.mark_unused()
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
}

impl Resource for AnimDict {
    fn is_loaded(&self) -> bool {
        invoke!(bool, 0xD031A9162D01088C, self.get_name())
    }

    fn request(&self) {
        invoke!((), 0xD3BD40951412FEF6, self.get_name())
    }

    fn mark_unused(&self) {
        invoke!((), 0xF66A602F829E2A06, self.get_name())
    }
}

impl Drop for AnimDict {
    fn drop(&mut self) {
        self.mark_unused()
    }
}

pub struct PedPhoto {
    handle: Handle
}

impl PedPhoto {
    pub fn new_transparent(ped: &Ped) -> PedPhoto {
        PedPhoto {
            handle: invoke!(Handle, 0x953563CE563143AF, ped.get_handle())
        }
    }

    pub fn new(ped: &Ped) -> PedPhoto {
        PedPhoto {
            handle: invoke!(Handle, 0x4462658788425076, ped.get_handle())
        }
    }

    pub fn is_valid(&self) -> bool {
        invoke!(bool, 0xA0A9668F158129A2, self.handle)
    }

    pub fn get_txd<'a>(&self) -> &'a str {
        invoke!(&str, 0xDB4EACD4AD0A5D6B, self.handle)
    }

    pub fn get_texture(&self) -> Texture {
        let txd = self.get_txd();
        Texture::new(txd, txd)
    }
}

impl Resource for PedPhoto {
    fn is_loaded(&self) -> bool {
        invoke!(bool, 0x7085228842B13A67, self.handle) && self.is_valid()
    }

    fn request(&self) {}

    fn mark_unused(&self) {
        invoke!((), 0x96B1361D9B24C2FF, self.handle)
    }
}

impl Drop for PedPhoto {
    fn drop(&mut self) {
        self.mark_unused()
    }
}

pub struct Texture {
    dict: String,
    name: String
}

impl Texture {
    pub fn new<D, N>(dict: D, name: N) -> Texture where D: Into<String>, N: Into<String> {
        Texture {
            dict: dict.into(),
            name: name.into()
        }
    }
}

impl NativeStackValue for Texture {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let dict = stack.read::<&str>();
        let name = stack.read::<&str>();
        Self::new(dict, name)
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        stack.write(self.dict.as_str());
        stack.write(self.name.as_str());
        std::mem::forget(self)
    }
}

impl NativeStackValue for Option<Texture> {
    fn read_from_stack(stack: &mut NativeStackReader) -> Self {
        let dict = stack.read_option::<&str>();
        let name = stack.read_option::<&str>();
        if let Some(dict) = dict {
            if let Some(name) = name {
                return Some(Texture::new(dict, name));
            }
        }
        None
    }

    fn write_to_stack(self, stack: &mut NativeStackWriter) {
        let (dict, name) = self.map_or((None, None), |s| (Some(s.dict), Some(s.name)));
        stack.write_option(dict);
        stack.write_option(name);
    }
}