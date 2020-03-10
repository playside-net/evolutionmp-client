use crate::invoke;
use crate::game::Handle;
use crate::hash::Hashable;
use crate::game::ped::Ped;
use crate::game::entity::Entity;
use crate::native::pool::Handleable;
use cgmath::Vector3;

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum ExplosionSource {
    Grenade,
    GrenadeLauncher,
    StickyBomb,
    Molotov,
    Rocket,
    TankShell,
    HiOctane,
    Car,
    Plane,
    PetrolPump,
    Bike,
    DirSteam,
    DirFlame,
    DirWaterHydrant,
    DirGasCanister,
    Boat,
    ShipDestroy,
    Truck,
    Bullet,
    SmokeGrenadeLauncher,
    SmokeGrenade,
    BZGas,
    Flare,
    GasCanister,
    Extinguisher,
    ProgrammableAR,
    Train,
    Barrel,
    Propane,
    Blimp,
    DirFlameExplode,
    Tanker,
    PlaneRocket,
    VehicleBullet,
    GasTank,
    BirdCrap
}

pub fn explode(pos: Vector3<f32>, source: ExplosionSource, damage: f32, audible: bool, invisible: bool, shake_camera: bool) {
    invoke!((), 0xE3AD2BDBAEE269AC, pos, source as u32, damage, audible, invisible, shake_camera)
}

pub fn explode_owned(ped: &Ped, pos: Vector3<f32>, source: ExplosionSource, damage: f32, audible: bool, invisible: bool, shake_camera: bool) {
    invoke!((), 0x172AA1B624FA1013, ped.get_handle(), pos, source as u32, damage, audible, invisible, shake_camera)
}

pub fn explode_with_fx<F>(pos: Vector3<f32>, source: ExplosionSource, fx: F, damage: f32, audible: bool, invisible: bool, shake_camera: bool) where F: Hashable {
    invoke!((), 0x36DD3FE58B5E5212, pos, source as u32, fx.joaat(), damage, audible, invisible, shake_camera)
}

#[derive(Debug, PartialEq)]
pub struct Fire {
    handle: Handle
}

impl Fire {
    pub fn new(pos: Vector3<f32>, max_children: u32, gas: bool) -> Option<Self> {
        invoke!(Option<Self>, 0x6B83617E04503888, pos, max_children, gas)
    }

    pub fn new_from_entity(entity: &dyn Entity) -> Option<Self> {
        invoke!(Option<Self>, 0xF6A9D9708F6F23DF, entity.get_handle())
    }

    pub fn extinguish(&self) {
        invoke!((), 0x7FF548385680673F, self.handle)
    }
}

crate::impl_handle!(Fire);

pub fn extinguish(pos: Vector3<f32>, radius: f32) {
    invoke!((), 0x056A8A219B8E829F, pos, radius)
}