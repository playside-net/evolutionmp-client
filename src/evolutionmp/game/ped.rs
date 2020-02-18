use super::Handle;
use crate::native;
use crate::game::entity::{Entity, Bone};
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::invoke;
use crate::native::pool::{Handleable, Pool, GenericPool};
use crate::hash::Hashable;
use crate::game::streaming::{AnimDict, PedPhoto};
use cgmath::{Vector3, MetricSpace};
use winapi::_core::mem::ManuallyDrop;

pub fn get_pool() -> ManuallyDrop<Box<GenericPool<Ped>>> {
    crate::native::pool::get_peds().expect("ped pool not initialized")
}

#[derive(Debug, PartialEq)]
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

    pub fn set_position_keep_vehicle(&self, pos: Vector3<f32>) {
        invoke!((), 0x9AFEFF481A85AB2E, self.handle, pos)
    }

    pub fn get_waypoint_distance(&self) -> f32 {
        invoke!(f32, 0xE6A877C64CAF1BC5, self.handle)
    }

    pub fn get_waypoint_progress(&self) -> f32 {
        invoke!(f32, 0x2720AAA75001E094, self.handle)
    }

    pub fn get_seat_is_trying_to_enter(&self) -> i32 {
        invoke!(i32, 0x6F4C85ACD641BCD2, self.handle)
    }

    pub fn get_closest_vehicle<F>(&self, max_distance: f32, filter: F) -> Option<Vehicle>
        where F: Fn(&Vehicle) -> bool {

        let pos = self.get_position_by_offset(Vector3::new(0.0, 0.0, -1.0));
        let mut result = None;
        let mut last_max_distance = max_distance;
        for vehicle in super::vehicle::get_pool().iter() {
            if vehicle.exists() && filter(&vehicle) {
                let v_pos = vehicle.get_position_by_offset(Vector3::new(0.0, 0.0, 0.0));
                let distance = v_pos.distance(pos);
                if distance < last_max_distance {
                    last_max_distance = distance;
                    result = Some(vehicle);
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

    pub fn get_photo(&self) -> PedPhoto {
        PedPhoto::new(self)
    }

    pub fn get_photo_transparent(&self) -> PedPhoto {
        PedPhoto::new_transparent(self)
    }

    pub fn get_bone(&self, bone: PedBone) -> Option<Bone<Self>> {
        let index = invoke!(i32, 0x3F428D08BE5AAE31, self.handle, bone as u32);
        if index != -1 {
            Some(Bone { entity: self, index })
        } else {
            None
        }
    }
}

impl Entity for Ped {
    fn delete(&mut self) {
        self.set_persistent(false);
        invoke!((), 0x9614299DCB53E54B, &mut self.handle)
    }
}

crate::impl_handle!(Ped);

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

    pub fn is_active(&self, task: u32) -> bool {
        invoke!(bool, 0xB0760331C7AA4155, self.ped.handle, task)
    }

    pub fn clear(&self) {
        invoke!((), 0xE1EF3C1216AFF2CD, self.ped.handle)
    }

    pub fn clear_immediately(&self) {
        invoke!((), 0xAAA34F8A7CB32098, self.ped.handle)
    }

    pub fn clear_secondary(&self) {
        invoke!((), 0x176CECF6F920D707, self.ped.handle)
    }

    pub fn enter_vehicle(&self, vehicle: &Vehicle, timeout: u32, seat: i32, speed: f32, flag: u32) {
        invoke!((), 0xC20E50AA46D09CA8, self.ped.handle, vehicle.get_handle(), timeout, seat, speed, flag, 0u32)
    }

    pub fn leave_vehicle(&self, vehicle: &Vehicle, flag: u32) {
        invoke!((), 0xD3DBCE61A490BE02, self.ped.handle, vehicle.get_handle(), flag)
    }

    pub fn play_animation(&self, dict: &AnimDict, name: &str, blend_in_speed: f32, blend_out_speed: f32,
                          duration: i32, flag: i32, playback_rate: f32) {

        invoke!((), 0xEA47FE3719165B94, self.ped.handle, dict.get_name(), name, blend_in_speed, blend_out_speed, duration, flag, playback_rate, 0, 0, 0)
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

pub enum PedBone {
    SkelRoot = 0x0,
    SkelPelvis = 0x2E28,
    SkelLThigh = 0xE39F,
    SkelLCalf = 0xF9BB,
    SkelLFoot = 0x3779,
    SkelLToe0 = 0x83C,
    EoLFoot = 0x84C5,
    EoLToe = 0x68BD,
    IkLFoot = 0xFEDD,
    PhLFoot = 0xE175,
    MhLKnee = 0xB3FE,
    SkelRThigh = 0xCA72,
    SkelRCalf = 0x9000,
    SkelRFoot = 0xCC4D,
    SkelRToe0 = 0x512D,
    EoRFoot = 0x1096,
    EoRToe = 0x7163,
    IkRFoot = 0x8AAE,
    PhRFoot = 0x60E6,
    MhRKnee = 0x3FCF,
    RbLThighRoll = 0x5C57,
    RbRThighRoll = 0x192A,
    SkelSpineRoot = 0xE0FD,
    SkelSpine0 = 0x5C01,
    SkelSpine1 = 0x60F0,
    SkelSpine2 = 0x60F1,
    SkelSpine3 = 0x60F2,
    SkelLClavicle = 0xFCD9,
    SkelLUpperArm = 0xB1C5,
    SkelLForearm = 0xEEEB,
    SkelLHand = 0x49D9,
    SkelLFinger00 = 0x67F2,
    SkelLFinger01 = 0xFF9,
    SkelLFinger02 = 0xFFA,
    SkelLFinger10 = 0x67F3,
    SkelLFinger11 = 0x1049,
    SkelLFinger12 = 0x104A,
    SkelLFinger20 = 0x67F4,
    SkelLFinger21 = 0x1059,
    SkelLFinger22 = 0x105A,
    SkelLFinger30 = 0x67F5,
    SkelLFinger31 = 0x1029,
    SkelLFinger32 = 0x102A,
    SkelLFinger40 = 0x67F6,
    SkelLFinger41 = 0x1039,
    SkelLFinger42 = 0x103A,
    PhLHand = 0xEB95,
    IkLHand = 0x8CBD,
    RbLForeArmRoll = 0xEE4F,
    RbLArmRoll = 0x1470,
    MhLElbow = 0x58B7,
    SkelRClavicle = 0x29D2,
    SkelRUpperArm = 0x9D4D,
    SkelRForearm = 0x6E5C,
    SkelRHand = 0xDEAD,
    SkelRFinger00 = 0xE5F2,
    SkelRFinger01 = 0xFA10,
    SkelRFinger02 = 0xFA11,
    SkelRFinger10 = 0xE5F3,
    SkelRFinger11 = 0xFA60,
    SkelRFinger12 = 0xFA61,
    SkelRFinger20 = 0xE5F4,
    SkelRFinger21 = 0xFA70,
    SkelRFinger22 = 0xFA71,
    SkelRFinger30 = 0xE5F5,
    SkelRFinger31 = 0xFA40,
    SkelRFinger32 = 0xFA41,
    SkelRFinger40 = 0xE5F6,
    SkelRFinger41 = 0xFA50,
    SkelRFinger42 = 0xFA51,
    PhRHand = 0x6F06,
    IkRHand = 0x188E,
    RbRForeArmRoll = 0xAB22,
    RbRArmRoll = 0x90FF,
    MhRElbow = 0xBB0,
    SkelNeck1 = 0x9995,
    SkelHead = 0x796E,
    IkHead = 0x322C,
    FacialFacialRoot = 0xFE2C,
    FbLBrowOut = 0xE3DB,
    FbLLidUpper = 0xB2B6,
    FbLEye = 0x62AC,
    FbLCheekBone = 0x542E,
    FbLLipCorner = 0x74AC,
    FbRLidUpper = 0xAA10,
    FbREye = 0x6B52,
    FbRCheekBone = 0x4B88,
    FbRBrowOut = 0x54C,
    FbRLipCorner = 0x2BA6,
    FbBrowCentre = 0x9149,
    FbUpperLipRoot = 0x4ED2,
    FbUpperLip = 0xF18F,
    FbLLipTop = 0x4F37,
    FbRLipTop = 0x4537,
    FbJaw = 0xB4A0,
    FbLowerLipRoot = 0x4324,
    FbLowerLip = 0x508F,
    FbLLipBot = 0xB93B,
    FbRLipBot = 0xC33B,
    FbTongue = 0xB987,
    RbNeck1 = 0x8B93,
    SprLBreast = 0xFC8E,
    SprRBreast = 0x885F,
    IkRoot = 0xDD1C,
    SkelNeck2 = 0x5FD4,
    SkelPelvis1 = 0xD003,
    SkelPelvisRoot = 0x45FC,
    SkelSaddle = 0x9524,
    MhLCalfBack = 0x1013,
    MhLThighBack = 0x600D,
    SmLSkirt = 0xC419,
    MhRCalfBack = 0xB013,
    MhRThighBack = 0x51A3,
    SmRSkirt = 0x7712,
    SmMBackSkirtRoll = 0xDBB,
    SmLBackSkirtRoll = 0x40B2,
    SmRBackSkirtRoll = 0xC141,
    SmMFrontSkirtRoll = 0xCDBB,
    SmLFrontSkirtRoll = 0x9B69,
    SmRFrontSkirtRoll = 0x86F1,
    SmCockNBallsRoot = 0xC67D,
    SmCockNBalls = 0x9D34,
    MhLFinger00 = 0x8C63,
    MhLFingerBulge00 = 0x5FB8,
    MhLFinger10 = 0x8C53,
    MhLFingerTop00 = 0xA244,
    MhLHandSide = 0xC78A,
    MhWatch = 0x2738,
    MhLSleeve = 0x933C,
    MhRFinger00 = 0x2C63,
    MhRFingerBulge00 = 0x69B8,
    MhRFinger10 = 0x2C53,
    MhRFingerTop00 = 0xEF4B,
    MhRHandSide = 0x68FB,
    MhRSleeve = 0x92DC,
    FacialJaw = 0xB21,
    FacialUnderChin = 0x8A95,
    FacialLUnderChin = 0x234E,
    FacialChin = 0xB578,
    FacialChinSkinBottom = 0x98BC,
    FacialLChinSkinBottom = 0x3E8F,
    FacialRChinSkinBottom = 0x9E8F,
    FacialTongueA = 0x4A7C,
    FacialTongueB = 0x4A7D,
    FacialTongueC = 0x4A7E,
    FacialTongueD = 0x4A7F,
    FacialTongueE = 0x4A80,
    FacialLTongueE = 0x35F2,
    FacialRTongueE = 0x2FF2,
    FacialLTongueD = 0x35F1,
    FacialRTongueD = 0x2FF1,
    FacialLTongueC = 0x35F0,
    FacialRTongueC = 0x2FF0,
    FacialLTongueB = 0x35EF,
    FacialRTongueB = 0x2FEF,
    FacialLTongueA = 0x35EE,
    FacialRTongueA = 0x2FEE,
    FacialChinSkinTop = 0x7226,
    FacialLChinSkinTop = 0x3EB3,
    FacialChinSkinMid = 0x899A,
    FacialLChinSkinMid = 0x4427,
    FacialLChinSide = 0x4A5E,
    FacialRChinSkinMid = 0xF5AF,
    FacialRChinSkinTop = 0xF03B,
    FacialRChinSide = 0xAA5E,
    FacialRUnderChin = 0x2BF4,
    FacialLLipLowerSdk = 0xB9E1,
    FacialLLipLowerAnalog = 0x244A,
    FacialLLipLowerThicknessV = 0xC749,
    FacialLLipLowerThicknessH = 0xC67B,
    FacialLipLowerSdk = 0x7285,
    FacialLipLowerAnalog = 0xD97B,
    FacialLipLowerThicknessV = 0xC5BB,
    FacialLipLowerThicknessH = 0xC5ED,
    FacialRLipLowerSdk = 0xA034,
    FacialRLipLowerAnalog = 0xC2D9,
    FacialRLipLowerThicknessV = 0xC6E9,
    FacialRLipLowerThicknessH = 0xC6DB,
    FacialNose = 0x20F1,
    FacialLNostril = 0x7322,
    FacialLNostrilThickness = 0xC15F,
    FacialNoseLower = 0xE05A,
    FacialLNoseLowerThickness = 0x79D5,
    FacialRNoseLowerThickness = 0x7975,
    FacialNoseTip = 0x6A60,
    FacialRNostril = 0x7922,
    FacialRNostrilThickness = 0x36FF,
    FacialNoseUpper = 0xA04F,
    FacialLNoseUpper = 0x1FB8,
    FacialNoseBridge = 0x9BA3,
    FacialLNasolabialFurrow = 0x5ACA,
    FacialLNasolabialBulge = 0xCD78,
    FacialLCheekLower = 0x6907,
    FacialLCheekLowerBulge1 = 0xE3FB,
    FacialLCheekLowerBulge2 = 0xE3FC,
    FacialLCheekInner = 0xE7AB,
    FacialLCheekOuter = 0x8161,
    FacialLEyesackLower = 0x771B,
    FacialLEyeball = 0x1744,
    FacialLEyelidLower = 0x998C,
    FacialLEyelidLowerOuterSdk = 0xFE4C,
    FacialLEyelidLowerOuterAnalog = 0xB9AA,
    FacialLEyelashLowerOuter = 0xD7F6,
    FacialLEyelidLowerInnerSdk = 0xF151,
    FacialLEyelidLowerInnerAnalog = 0x8242,
    FacialLEyelashLowerInner = 0x4CCF,
    FacialLEyelidUpper = 0x97C1,
    FacialLEyelidUpperOuterSdk = 0xAF15,
    FacialLEyelidUpperOuterAnalog = 0x67FA,
    FacialLEyelashUpperOuter = 0x27B7,
    FacialLEyelidUpperInnerSdk = 0xD341,
    FacialLEyelidUpperInnerAnalog = 0xF092,
    FacialLEyelashUpperInner = 0x9B1F,
    FacialLEyesackUpperOuterBulge = 0xA559,
    FacialLEyesackUpperInnerBulge = 0x2F2A,
    FacialLEyesackUpperOuterFurrow = 0xC597,
    FacialLEyesackUpperInnerFurrow = 0x52A7,
    FacialForehead = 0x9218,
    FacialLForeheadInner = 0x843,
    FacialLForeheadInnerBulge = 0x767C,
    FacialLForeheadOuter = 0x8DCB,
    FacialSkull = 0x4221,
    FacialForeheadUpper = 0xF7D6,
    FacialLForeheadUpperInner = 0xCF13,
    FacialLForeheadUpperOuter = 0x509B,
    FacialRForeheadUpperInner = 0xCEF3,
    FacialRForeheadUpperOuter = 0x507B,
    FacialLTemple = 0xAF79,
    FacialLEar = 0x19DD,
    FacialLEarLower = 0x6031,
    FacialLMasseter = 0x2810,
    FacialLJawRecess = 0x9C7A,
    FacialLCheekOuterSkin = 0x14A5,
    FacialRCheekLower = 0xF367,
    FacialRCheekLowerBulge1 = 0x599B,
    FacialRCheekLowerBulge2 = 0x599C,
    FacialRMasseter = 0x810,
    FacialRJawRecess = 0x93D4,
    FacialREar = 0x1137,
    FacialREarLower = 0x8031,
    FacialREyesackLower = 0x777B,
    FacialRNasolabialBulge = 0xD61E,
    FacialRCheekOuter = 0xD32,
    FacialRCheekInner = 0x737C,
    FacialRNoseUpper = 0x1CD6,
    FacialRForeheadInner = 0xE43,
    FacialRForeheadInnerBulge = 0x769C,
    FacialRForeheadOuter = 0x8FCB,
    FacialRCheekOuterSkin = 0xB334,
    FacialREyesackUpperInnerFurrow = 0x9FAE,
    FacialREyesackUpperOuterFurrow = 0x140F,
    FacialREyesackUpperInnerBulge = 0xA359,
    FacialREyesackUpperOuterBulge = 0x1AF9,
    FacialRNasolabialFurrow = 0x2CAA,
    FacialRTemple = 0xAF19,
    FacialREyeball = 0x1944,
    FacialREyelidUpper = 0x7E14,
    FacialREyelidUpperOuterSdk = 0xB115,
    FacialREyelidUpperOuterAnalog = 0xF25A,
    FacialREyelashUpperOuter = 0xE0A,
    FacialREyelidUpperInnerSdk = 0xD541,
    FacialREyelidUpperInnerAnalog = 0x7C63,
    FacialREyelashUpperInner = 0x8172,
    FacialREyelidLower = 0x7FDF,
    FacialREyelidLowerOuterSdk = 0x1BD,
    FacialREyelidLowerOuterAnalog = 0x457B,
    FacialREyelashLowerOuter = 0xBE49,
    FacialREyelidLowerInnerSdk = 0xF351,
    FacialREyelidLowerInnerAnalog = 0xE13,
    FacialREyelashLowerInner = 0x3322,
    FacialLLipUpperSdk = 0x8F30,
    FacialLLipUpperAnalog = 0xB1CF,
    FacialLLipUpperThicknessH = 0x37CE,
    FacialLLipUpperThicknessV = 0x38BC,
    FacialLipUpperSdk = 0x1774,
    FacialLipUpperAnalog = 0xE064,
    FacialLipUpperThicknessH = 0x7993,
    FacialLipUpperThicknessV = 0x7981,
    FacialLLipCornerSdk = 0xB1C,
    FacialLLipCornerAnalog = 0xE568,
    FacialLLipCornerThicknessUpper = 0x7BC,
    FacialLLipCornerThicknessLower = 0xDD42,
    FacialRLipUpperSdk = 0x7583,
    FacialRLipUpperAnalog = 0x51CF,
    FacialRLipUpperThicknessH = 0x382E,
    FacialRLipUpperThicknessV = 0x385C,
    FacialRLipCornerSdk = 0xB3C,
    FacialRLipCornerAnalog = 0xEE0E,
    FacialRLipCornerThicknessUpper = 0x54C3,
    FacialRLipCornerThicknessLower = 0x2BBA,
    MhMulletRoot = 0x3E73,
    MhMulletScaler = 0xA1C2,
    MhHairScale = 0xC664,
    MhHairCrown = 0x1675,
    SmTorch = 0x8D6,
    FxLight = 0x8959,
    FxLightScale = 0x5038,
    FxLightSwitch = 0xE18E,
    BagRoot = 0xAD09,
    BagPivotROOT = 0xB836,
    BagPivot = 0x4D11,
    BagBody = 0xAB6D,
    BagBoneR = 0x937,
    BagBoneL = 0x991,
    SmLifeSaverFront = 0x9420,
    SmRPouchesRoot = 0x2962,
    SmRPouches = 0x4141,
    SmLPouchesRoot = 0x2A02,
    SmLPouches = 0x4B41,
    SmSuitBackFlapper = 0xDA2D,
    SprCopRadio = 0x8245,
    SmLifeSaverBack = 0x2127,
    MhBlushSlider = 0xA0CE,
    SkelTail01 = 0x347,
    SkelTail02 = 0x348,
    MhLConcertinaB = 0xC988,
    MhLConcertinaA = 0xC987,
    MhRConcertinaB = 0xC8E8,
    MhRConcertinaA = 0xC8E7,
    MhLShoulderBladeRoot = 0x8711,
    MhLShoulderBlade = 0x4EAF,
    MhRShoulderBladeRoot = 0x3A0A,
    MhRShoulderBlade = 0x54AF,
    FbREar = 0x6CDF,
    SprREar = 0x63B6,
    FbLEar = 0x6439,
    SprLEar = 0x5B10,
    FbTongueA = 0x4206,
    FbTongueB = 0x4207,
    FbTongueC = 0x4208,
    SkelLToe1 = 0x1D6B,
    SkelRToe1 = 0xB23F,
    SkelTail03 = 0x349,
    SkelTail04 = 0x34A,
    SkelTail05 = 0x34B,
    SprGonadsRoot = 0xBFDE,
    SprGonads = 0x1C00
}