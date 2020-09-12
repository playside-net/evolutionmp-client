use std::collections::HashMap;

use crate::{bind_fn_detour, bind_fn_detour_ip};
use crate::hash::{Hash, Hashable};
use crate::win::thread::seh;
use crate::pattern::RageBox;
use crate::native::alloc::{RageVec, ChainedBox};

bitflags! {
    #[repr(C)]
    pub struct InitFnMask: u32 {
        const UNKNOWN = 0;
        const CORE = 1;
        const BEFORE_MAP_LOADED = 2;
        const AFTER_MAP_LOADED = 4;
        const SESSION = 8;
    }
}

#[repr(C)]
struct InitFnData {
    init: extern fn(InitFnMask),
    shutdown: extern fn(InitFnMask),
    init_order: u32,
    shutdown_order: u32,
    init_mask: InitFnMask,
    shutdown_mask: InitFnMask,
    hash: Hash
}

impl InitFnData {
    fn get_name(&self) -> String {
        if let Some(name) = FN_MAP.get(&self.hash) {
            String::from(*name)
        } else {
            format!("0x{:08X}", self.hash.0)
        }
    }

    unsafe fn try_init(&self, mask: InitFnMask) {
        let name = self.get_name();
        seh(|| (self.init)(mask), move |rec| {
            if (rec.ExceptionCode & 0x80000000) != 0 {
                error!(
                    "An exception occurred (0x{:08X} at {:p}) during execution of {:?} function for {}. The game will be terminated.",
                    rec.ExceptionCode,
                    rec.ExceptionAddress,
                    mask, name
                );
            }
            0 //EXCEPTION_CONTINUE_SEARCH
        });
    }
}

#[repr(C)]
struct InitFn {
    order: u32,
    fns: RageVec<u32>
}

#[repr(C)]
struct InitFnGroup {
    mask: InitFnMask,
    entries: ChainedBox<InitFn>
}

#[repr(C)]
struct UpdateFnVTable {
    destructor: extern fn(Box<UpdateFn>),
    run: extern fn(&UpdateFn)
}

#[repr(C)]
pub struct UpdateFn {
    v_table: RageBox<UpdateFnVTable>,
    flag: bool,
    float: f32,
    hash: Hash,
    next: Option<ChainedBox<UpdateFn>>,
    child: Option<ChainedBox<UpdateFn>>
}

impl UpdateFn {
    fn run(&self) {
        let name = self.get_name();
        info!("Running update on {}", name);
        (self.v_table.run)(self);
        info!("Done update");
    }

    fn get_name(&self) -> String {
        if let Some(name) = FN_MAP.get(&self.hash) {
            String::from(*name)
        } else {
            format!("0x{:08X}", self.hash.0)
        }
    }
    extern fn run_group(&mut self) {
        let name = self.get_name();
        info!("Running group update on {}", name);
        if let Some(ref child) = self.child {
            for child in child.iter() {
                child.run();
            }
        }
        info!("Done group update");
    }
}

#[repr(C)]
struct UpdateFnGroup {
    ty: u32,
    entries: ChainedBox<UpdateFn>
}

#[repr(C)]
struct GameSkeletonVTable {
    destructor: extern fn(Box<GameSkeleton>)
}

#[repr(C)]
pub struct GameSkeleton {
    v_table: RageBox<GameSkeletonVTable>,
    fn_order: u32,
    fn_mask: InitFnMask,
    pad: u32,
    update_ty: u32,
    init_fns: RageVec<InitFnData>,
    pad2: *mut u8,
    pad3: [u8; 256],
    init_groups: ChainedBox<InitFnGroup>,
    pad4: *mut u8,
    update_groups: ChainedBox<UpdateFnGroup>
}

impl GameSkeleton {
    extern fn init(&mut self, mask: InitFnMask) {
        trace!("Running {:?} init functions", mask);
        for group in self.init_groups.iter() {
            if group.mask == mask {
                for entry in group.entries.iter() {
                    let total_fns = entry.fns.len();
                    trace!("Running functions init functions of order {} ({} total)", entry.order, total_fns);
                    for index in entry.fns.iter().cloned() {
                        let index = index as usize;
                        let func = &self.init_fns[index];
                        trace!("Invoking {} {:?} init ({} out of {}) init functions", func.get_name(), mask, index + 1, total_fns);
                        unsafe {
                            func.try_init(mask);
                        }
                        trace!("Done");
                    }
                }
            }
        }
        trace!("Done running {:?} init functions!", mask);
    }

    extern fn update(&mut self, ty: u32) {
        for group in self.update_groups.iter() {
            if group.ty == ty {
                for entry in group.entries.iter() {
                    entry.run();
                    if let Some(ref next) = entry.next {
                        for item in next.iter() {
                            item.run();
                        }
                    }
                }
            }
        }
    }
}

lazy_static! {
    static ref FN_MAP: HashMap<Hash, &'static str> = KNOWN_INIT_FNS.iter().map(|f| (f.joaat(), *f)).collect();
}

bind_fn_detour_ip!(RUN_INIT, "BA 04 00 00 00 E8 ? ? ? ? E8 ? ? ? ? E8", 5, GameSkeleton::init, (&mut GameSkeleton, InitFnMask) -> ());
bind_fn_detour_ip!(RUN_UPDATE, "48 8D 0D ? ? ? ? BA 01 00 00 00 E8 ? ? ? ? E8 ? ? ? ?", 12, GameSkeleton::update, (&mut GameSkeleton, u32) -> ());
bind_fn_detour!(RUN_UPDATE_GROUP, "40 53 48 83 EC 20 48 8B 59 20 EB 0D 48 8B 03 48", 0, UpdateFn::run_group, (&mut UpdateFn) -> ());

pub fn hook() {
    info!("Hooking init functions...");
    lazy_static::initialize(&FN_MAP);
    lazy_static::initialize(&RUN_INIT);
    /*lazy_static::initialize(&RUN_UPDATE);
    lazy_static::initialize(&RUN_UPDATE_GROUP);*/
}

static KNOWN_INIT_FNS: [&'static str; 289] = [
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
    "CAnimSceneManager",
    "CTextInputBox",
    "CMultiplayerChat",
    "CCreditsText",
    "CReplayMgr",
    "CReplayCoordinator",
    "CMousePointer",
    "CVideoEditorUI",
    "CVideoEditorInterface",
    "VideoRecording",
    "WatermarkRenderer",
];