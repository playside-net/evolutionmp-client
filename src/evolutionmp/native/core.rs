use crate::native::alloc::{RageVec, Chained, ChainedBox};
use crate::hash::{Hash, Hashable};
use crate::pattern::MemoryRegion;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use winapi::_core::ops::Deref;

static mut PERFORMING_ASYNC_INIT: *mut bool = std::ptr::null_mut();

pub(crate) unsafe fn init(mem: &MemoryRegion) {
    crate::info!("run_init_fns");
    let l = mem.find("BA 04 00 00 00 48 8D 0D ? ? ? ? E8 ? ? ? ? E8 ? ? ? ? E8")
        .next().expect("run_init_fns")
        .add(12)
        .jump(GameCore::run_init_fns as _);
    crate::info!("run_update_fns");
    mem.find("E8 ? ? ? ? BA 01 00 00 00 48 8D 0D ? ? ? ? E8 ? ? ? ? E8")
        .next().expect("run_update_fns")
        .add(12)
        .jump(GameCore::run_update_fns as _);
    crate::info!("run_update_group_fns");
    mem.find("40 53 48 83 EC 20 48 8B 59 20 EB 0D 48 8B 03 48")
        .next().expect("run_update_group_fns")
        .jump(UpdateFnsEntry::run_update_group_fns as _);
    crate::info!("performing_async_init");
    PERFORMING_ASYNC_INIT = mem.find("42 F6 04 13 01 75 10 80 3D")
        .next().expect("performing_async_init")
        .add(9).get_mut::<bool>().add(1);
    crate::info!("core initialized");
}

#[repr(i32)]
#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum InitFnType {
    Unknown = 0,
    Core = 1,
    BeforeMapLoaded = 2,
    AfterMapLoaded = 4,
    Session = 8
}

#[repr(C)]
pub struct InitFnData {
    init: extern "C" fn(i32),
    shutdown: extern "C" fn(i32),
    init_mask: InitFnType,
    async_init_mask: InitFnType,
    shutdown_mask: InitFnType,
    fn_hash: Hash,
    pad: [u8; 16]
}

impl InitFnData {
    pub fn get_name(&self) -> String {
        if let Some(name) = KNOWN_FNS.get(&self.fn_hash).cloned() {
            name.to_owned()
        } else {
            format!("0x{:08X}", self.fn_hash.0)
        }
    }

    pub fn init(&self, ty: InitFnType) {
        (self.init)(ty as i32)
    }

    pub fn shutdown(&self, ty: InitFnType) {
        (self.shutdown)(ty as i32)
    }
}

#[repr(C)]
pub struct InitFnsEntry {
    order: i32,
    fns: RageVec<i32>
}

#[repr(C)]
pub struct Fns<E> {
    ty: InitFnType,
    entries: ChainedBox<Chained<E>>
}

#[repr(C)]
pub struct UpdateFnsEntryVTable {
    destructor: extern "C" fn(this: *mut UpdateFnsEntryInner),
    run:        extern "C" fn(this: *const UpdateFnsEntryInner)
}

#[repr(C)]
pub struct UpdateFnsEntryInner {
    v_table: ManuallyDrop<Box<UpdateFnsEntryVTable>>,
    flag: bool,
    unk: f32,
    hash: Hash
}

impl UpdateFnsEntryInner {
    pub fn run(&self) {
        (self.v_table.run)(self)
    }
}

#[repr(C)]
pub struct UpdateFnsEntry {
    inner: ChainedBox<Chained<UpdateFnsEntry>>,
    group: ChainedBox<Chained<UpdateFnsEntry>>
}

impl Deref for UpdateFnsEntry {
    type Target = UpdateFnsEntryInner;

    fn deref(&self) -> &Self::Target {
        &**self.inner
    }
}

impl UpdateFnsEntry {
    pub unsafe extern "C" fn run_update_group_fns(&self) {
        Self::run_entries(&self.group)
    }

    fn run_entries(entries: &ChainedBox<Chained<UpdateFnsEntry>>) {
        for e in entries.iter() {
            if e.inner.hash == Hash(0x73AA6F9E) {
                return;
            }
            e.run();
        }
    }
}

#[repr(C)]
pub struct GameCoreVTable {
    destructor: extern "C" fn(this: *mut GameCore)
}

#[repr(C)]
pub struct GameCore {
    v_table: ManuallyDrop<Box<GameCoreVTable>>,
    fn_order: i32,
    fn_type: InitFnType,
    pad1: i32,
    update_type: i32,
    init_fns_data: RageVec<InitFnData>,
    pad2: *const (),
    pad3: [u8; 256],
    init_fns: ChainedBox<Chained<Fns<InitFnsEntry>>>,
    pad4: *const (),
    update_fns: ChainedBox<Chained<Fns<UpdateFnsEntry>>>
}

impl GameCore {
    pub unsafe extern "C" fn run_init_fns(&self, ty: InitFnType) {
        for f in self.init_fns.iter() {
            if f.ty == ty {
                for e in f.entries.iter() {
                    let mut i = 0;

                    *PERFORMING_ASYNC_INIT = true;

                    for is_async in [true, false].iter() {
                        for index in e.fns.iter() {
                            let func = self.init_fns_data.get(*index as usize).unwrap();
                            let fn_async = (func.async_init_mask as i32 & ty as i32) != 0;
                            if fn_async != *is_async {
                                continue;
                            }
                            crate::info!("Invoking fn {} (async: {}; type: {:?})", func.get_name(), is_async, ty);
                            func.init(ty);
                            i += 1;
                        }
                    }

                    *PERFORMING_ASYNC_INIT = false;
                }
            }
        }
    }

    pub unsafe extern "C" fn run_update_fns(&self, ty: InitFnType) {
        for f in self.update_fns.iter() {
            if f.ty == ty {
                UpdateFnsEntry::run_entries(&f.entries)
            }
        }
    }
}

lazy_static! {
    static ref KNOWN_FNS: HashMap<Hash, &'static str> = {
        KNOWN_FN_NAMES.iter().map(|name| (name.joaat(), *name)).collect::<_>()
    };
}

const KNOWN_FN_NAMES: [&'static str; 278] = [
    "AmbientLights",
    "AnimBlackboard",
    "Audio",
    "BackgroundScripts",
    "CActionManager",
    "CAgitatedManager",
    "CAmbientAnimationManager",
    "CAmbientAudioManager",
    "CAmbientModelSetManager",
    "CAnimBlackboard",
    "CAppDataMgr",
    "CAssistedMovementRouteStore",
    "CBoatChaseDirector",
    "CBuses",
    "CBusySpinner",
    "CCheat",
    "CCheckCRCs",
    "CClipDictionaryStoreInterface",
    "CClock",
    "CCombatDirector",
    "CCombatInfoMgr",
    "CCompEntity",
    "CConditionalAnimManager",
    "CContentExport",
    "CContentSearch",
    "CControl",
    "CControlMgr",
    "CControllerLabelMgr",
    "CCover",
    "CCoverFinder",
    "CCredits",
    "CCrimeInformationManager",
    "CCullZones",
    "CDLCScript",
    "CDecoratorInterface",
    "CDispatchData",
    "CEventDataManager",
    "CExpensiveProcessDistributer",
    "CExplosionManager",
    "CExtraContent",
    "CExtraContentWrapper",
    "CExtraContentWrapper::Shutdown",
    "CExtraContentWrapper::ShutdownStart",
    "CExtraMetadataMgr",
    "CExtraMetadataMgr::ClassInit",
    "CExtraMetadataMgr::ClassShutdown",
    "CExtraMetadataMgr::ShutdownDLCMetaFiles",
    "CFlyingVehicleAvoidanceManager",
    "CFocusEntityMgr",
    "CFrontendStatsMgr",
    "CGameLogic",
    "CGameSituation",
    "CGameStreamMgr",
    "CGameWorld",
    "CGameWorldHeightMap",
    "CGameWorldWaterHeight",
    "CGarages",
    "CGenericGameStorage",
    "CGestureManager",
    "CGps",
    "CGtaAnimManager",
    "CHandlingDataMgr",
    "CInstanceListAssetLoader::Init",
    "CInstanceListAssetLoader::Shutdown",
    "CIplCullBox",
    "CJunctions",
    "CLODLightManager",
    "CLODLights",
    "CLadderMetadataManager",
    "CLoadingScreens",
    "CMapAreas",
    "CMapZoneManager",
    "CMessages",
    "CMiniMap",
    "CModelInfo",
    "CModelInfo::Init",
    "CMovieMeshManager",
    "CMultiplayerGamerTagHud",
    "CNetRespawnMgr",
    "CNetwork",
    "CNetworkTelemetry",
    "CNewHud",
    "CObjectPopulationNY",
    "COcclusion",
    "CParaboloidShadow",
    "CPathFind",
    "CPathServer::InitBeforeMapLoaded",
    "CPathServer::InitSession",
    "CPathServer::ShutdownSession",
    "CPathZoneManager",
    "CPatrolRoutes",
    "CPauseMenu",
    "CPed",
    "CPedAILodManager",
    "CPedGeometryAnalyser",
    "CPedModelInfo",
    "CPedPopulation",
    "CPedPopulation::ResetPerFrameScriptedMu",
    "CPedPopulation::ResetPerFrameScriptedMultipiers",
    "CPedPropsMgr",
    "CPedVariationPack",
    "CPedVariationStream",
    "CPerformance",
    "CPhoneMgr",
    "CPhotoManager",
    "CPhysics",
    "CPickupDataManager",
    "CPickupManager",
    "CPlantMgr",
    "CPlayStats",
    "CPlayerSwitch",
    "CPopCycle",
    "CPopZones",
    "CPopulationStreaming",
    "CPopulationStreamingWrapper",
    "CPortal",
    "CPortalTracker",
    "CPostScan",
    "CPrecincts",
    "CPrioritizedClipSetRequestManager",
    "CPrioritizedClipSetStreamer",
    "CProcObjectMan",
    "CProceduralInfo",
    "CProfileSettings",
    "CRandomEventManager",
    "CRecentlyPilotedAircraft",
    "CRenderPhaseCascadeShadowsInterface",
    "CRenderTargetMgr",
    "CRenderThreadInterface",
    "CRenderer",
    "CReportMenu",
    "CRestart",
    "CRiots",
    "CRoadBlock",
    "CScaleformMgr",
    "CScenarioActionManager",
    "CScenarioManager",
    "CScenarioManager::ResetExclusiveScenari",
    "CScenarioManager::ResetExclusiveScenarioGroup",
    "CScenarioPointManager",
    "CScenarioPointManagerInitSession",
    "CScene",
    "CSceneStreamerMgr::PreScanUpdate",
    "CScriptAreas",
    "CScriptCars",
    "CScriptDebug",
    "CScriptEntities",
    "CScriptHud",
    "CScriptPedAIBlips",
    "CScriptPeds",
    "CScriptedGunTaskMetadataMgr",
    "CShaderHairSort",
    "CShaderLib",
    "CSituationalClipSetStreamer",
    "CSky",
    "CSlownessZonesManager",
    "CSprite2d",
    "CStaticBoundsStore",
    "CStatsMgr",
    "CStreaming",
    "CStreamingRequestList",
    "CStuntJumpManager",
    "CTVPlaylistManager",
    "CTacticalAnalysis",
    "CTask",
    "CTaskClassInfoManager",
    "CTaskRecover",
    "CTexLod",
    "CText",
    "CThePopMultiplierAreas",
    "CTheScripts",
    "CTimeCycle",
    "CTrafficLights",
    "CTrain",
    "CTuningManager",
    "CUserDisplay",
    "CVehicleAILodManager",
    "CVehicleChaseDirector",
    "CVehicleCombatAvoidanceArea",
    "CVehicleDeformation",
    "CVehicleMetadataMgr",
    "CVehicleModelInfo",
    "CVehiclePopulation",
    "CVehiclePopulation::ResetPerFrameScript",
    "CVehiclePopulation::ResetPerFrameScriptedMultipiers",
    "CVehicleRecordingMgr",
    "CVehicleVariationInstance",
    "CVisualEffects",
    "CWarpManager",
    "CWaypointRecording",
    "CWeaponManager",
    "CWitnessInformationManager",
    "CWorldPoints",
    "CZonedAssetManager",
    "Common",
    "CreateFinalScreenRenderPhaseList",
    "Credits",
    "CutSceneManager",
    "CutSceneManagerWrapper",
    "FacialClipSetGroupManager",
    "FireManager",
    "FirstPersonProp",
    "FirstPersonPropCam",
    "Game",
    "GenericGameStoragePhotoGallery",
    "INSTANCESTORE",
    "ImposedTxdCleanup",
    "InitSystem",
    "Kick",
    "LightEntityMgr",
    "Lights",
    "MeshBlendManager",
    "Misc",
    "NewHud",
    "Occlusion",
    "PauseMenu",
    "Ped",
    "PedHeadShotManager",
    "PedModelInfo",
    "PedPopulation",
    "PlantsMgr::UpdateBegin",
    "PlantsMgr::UpdateEnd",
    "Population",
    "PostFX",
    "PostFx",
    "Pre-vis",
    "Prioritized",
    "Proc",
    "ProcessAfterCameraUpdate",
    "ProcessAfterMovement",
    "ProcessPedsEarlyAfterCameraUpdate",
    "Render",
    "ResetSceneLights",
    "Run",
    "Script",
    "ScriptHud",
    "ShaderLib::Update",
    "Situational",
    "SocialClubMenu",
    "Streaming",
    "UI3DDrawManager",
    "UIWorldIconManager",
    "Update",
    "VehPopulation",
    "VideoPlayback",
    "VideoPlaybackThumbnailManager",
    "VideoPlaybackThumbnails",
    "Viewport",
    "ViewportSystemInit",
    "ViewportSystemInitLevel",
    "ViewportSystemShutdown",
    "ViewportSystemShutdownLevel",
    "Visibility",
    "Visual",
    "WarningScreen",
    "Water",
    "WaterHeightSim",
    "World",
    "audNorthAudioEngine",
    "audNorthAudioEngineDLC",
    "cStoreScreenMgr",
    "camManager",
    "decorators",
    "fwAnimDirector",
    "fwClipSetManager",
    "fwClothStore",
    "fwDrawawableStoreWrapper",
    "fwDwdStore",
    "fwDwdStoreWrapper",
    "fwExpressionSetManager",
    "fwFacialClipSetGroupManager",
    "fwFragmentStoreWrapper",
    "fwMapTypesStore",
    "fwMetaDataStore",
    "fwTimer",
    "fwTxdStore",
    "perfClearingHouse",
    "strStreamingEngine::SubmitDeferredAsyncPlacementRequests",
];