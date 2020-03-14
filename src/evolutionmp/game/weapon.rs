use crate::{invoke, invoke_option};
use crate::hash::{Hash, Hashable};
use cgmath::Vector3;
use crate::native::NativeStackValue;
use crate::game::streaming::Model;

#[repr(u32)]
#[derive(Debug)]
pub enum DamageType {
    Invalid,
    NoDamage, //Flare/Snowball/Pretrol can
    Melee,
    Bullet,
    ForceRagdollFall,
    Explosive, //RPC/Railgun/Grenade,
    Fire, //Molotov
    Fall, //(WEAPON_HELI_CRASH)
    Unknown,
    Electric,
    BarbedWire,
    Extinguisher,
    Gas,
    WaterCannon //(WEAPON_HIT_BY_WATER_CANNON)
}

#[repr(align(8))]
#[derive(Debug, Default)]
pub struct WeaponHudStats {
    damage: u8,
    speed: u8,
    capacity: u8,
    accuracy: u8,
    range: u8
}

impl NativeStackValue for &mut WeaponHudStats {}

pub struct Weapon {
    hash: Hash
}

impl<H> From<H> for Weapon where H: Hashable {
    fn from(hash: H) -> Self {
        Self { hash: hash.joaat() }
    }
}

impl Weapon {
    pub fn can_use_on_parachute(&self) -> bool {
        invoke!(bool, 0xBC7BE5ABC0879F74, self.hash)
    }

    pub fn can_take_component<C>(&self, component: C) where C: Hashable {
        invoke!((), 0x5CEE3DF569CECAB0, self.hash, component.joaat())
    }

    pub fn create_prop(&self, ammo: u32, pos: Vector3<f32>, show_world_model: bool, heading: f32) {
        invoke!((), 0x9541D3CF0D398F36, self.hash, ammo, pos, show_world_model, heading)
    }

    pub fn get_clip_size(&self) -> u32 {
        invoke!(u32, 0x583BE370B1EC6EB4, self.hash)
    }

    pub fn get_damage<C>(&self, component: C) -> f32 where C: Hashable {
        invoke!(f32, 0x3133B907D8B32053, self.hash, component.joaat())
    }

    pub fn set_damage_modifier(&self, modifier: f32) {
        invoke!((), 0x4757F00BC6323CFE, self.hash, modifier)
    }

    pub fn get_damage_type(&self) -> DamageType {
        unsafe {
            std::mem::transmute(invoke!(u32, 0x3BE0BB12D25FB305, self.hash))
        }
    }

    pub fn get_hud_stats(&self) -> Option<WeaponHudStats> {
        let mut stats = WeaponHudStats::default();
        invoke_option!(stats, 0xD92C739EE34C9EBA, self.hash, &mut stats)
    }

    pub fn get_shot_timeout(&self) -> f32 {
        invoke!(f32, 0x065D2AACAD8CF7A4, self.hash)
    }

    pub fn get_tint_count(&self) -> u32 {
        invoke!(u32, 0x5DCF6C5CAB2E9BF7, self.hash)
    }

    pub fn get_group(&self) -> Hash {
        invoke!(Hash, 0xC3287EE3050FB74C, self.hash)
    }

    pub fn get_model(&self) -> Model {
        Model::from(invoke!(Hash, 0xF46CDC33180FDA94, self.hash))
    }

    pub fn get_slot(&self) -> Hash {
        invoke!(Hash, 0x4215460B9B8B7FA0, self.hash)
    }

    pub fn is_valid(&self) -> bool {
        invoke!(bool, 0x937C71165CF334B3, self.hash)
    }

    pub fn is_asset_loaded(&self) -> bool {
        invoke!(bool, 0x36E353271F0E90EE, self.hash)
    }

    pub fn remove_asset(&self) {
        invoke!((), 0xAA08EF13F341C8FC, self.hash)
    }
}