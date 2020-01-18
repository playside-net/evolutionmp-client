use crate::runtime::{Script, ScriptEnv, ScriptContainer, Runtime, ScriptEvent, TaskQueue};
use crate::pattern::MemoryRegion;
use crate::GameState;
use crate::{invoke, game, native};
use crate::game::entity::Entity;
use crate::game::stats::Stat;
use crate::game::ped::Ped;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::game::{streaming, gameplay, dlc, script, clock};
use crate::win::input::{KeyboardEvent, InputEvent};
use crate::info;
use std::time::{Duration, Instant};
use game::controls::{Control, Group as ControlGroup};
use game::ui::{CursorSprite, LoadingPrompt};
use winapi::um::winuser::{VK_NUMPAD5, VK_NUMPAD2, VK_NUMPAD0, VK_RIGHT, VK_LEFT, VK_BACK, ReleaseCapture};
use cgmath::Vector3;
use std::collections::VecDeque;
use crate::game::streaming::{Model, AnimDict};
use crate::game::camera::Camera;
use crate::native::pool::Pool;
use winapi::_core::sync::atomic::Ordering;

pub mod console;
//pub mod network;

pub fn init(runtime: &mut Runtime) {
    info!("Initializing console");
    console::init(runtime);
    info!("Registering scripts");
    //network::init(runtime);

    runtime.register_script("clean_world", ScriptCleanWorld {
        tasks: TaskQueue::new(),
        last_cleanup: Instant::now()
    });
    runtime.register_script("finger_pointing", ScriptFingerPointing {
        active: false,
        camera: None
    });
}

pub struct ScriptCleanWorld {
    tasks: TaskQueue,
    last_cleanup: Instant
}

static AUDIO_FLAGS: [&'static str; 5] = ["LoadMPData", "DisableBarks", "DisableFlightMusic", "PoliceScannerDisabled", "OnlyAllowScriptTriggerPoliceScanner"];

impl Script for ScriptCleanWorld {
    fn prepare(&mut self, mut env: ScriptEnv) {
        let pos = Vector3::new(0.0, 0.0, 73.5);

        streaming::load_scene(pos);

        let player = Player::local();
        player.set_model(&mut env, "mp_m_freemode_01");
        let ped = player.get_ped();
        ped.set_position_no_offset(pos, Vector3::new(false, false, false));
        ped.get_tasks().clear_immediately();

        gameplay::set_freemode_map_behavior(true);
        //game::ui::show_loading_prompt(LoadingPrompt::LoadingRight, "Loading Evolution MP");
        dlc::load_mp_maps();
        script::shutdown_loading_screen();
        game::ui::hide_loading_prompt();

        clock::pause(true);

        self.terminate_script("selector", true);
        self.terminate_script("replay_controller", true);
        self.terminate_all_scripts(true);

        self.cleanup();

        for flag in AUDIO_FLAGS.iter() {
            game::audio::set_flag(flag, true);
        }
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        self.tasks.process(&mut env);

        self.disable_controls();
        streaming::stop_player_switch();

        let player = Player::local();
        let ped = player.get_ped();

        self.cleanup();

        game::ped::set_density_multiplier_this_frame(0.0);
        game::ped::set_scenario_density_multiplier_this_frame(0.0);

        game::vehicle::set_density_multiplier_this_frame(0.0);
        game::vehicle::set_random_density_multiplier_this_frame(0.0);

        game::decision_event::suppress_shocking_events_next_frame();
        game::decision_event::suppress_agitation_events_next_frame();


        /*if let Some(vehicles) = native::pool::get_vehicles() {
            let len = vehicles.len();
            let capacity = vehicles.capacity();
            game::ui::show_loading_prompt(LoadingPrompt::LoadingLeft3, &format!("Vehicles: {}/{}", len, capacity));
        }
        if let Some(veh) = ped.get_using_vehicle() {
            let color = veh.get_colors();

        } else {
            //game::ui::hide_loading_prompt();
        }*/
    }

    fn event(&mut self, event: &ScriptEvent, output: &mut VecDeque<ScriptEvent>) -> bool {
        match event {
            ScriptEvent::UserInput(event) => {
                match event {
                    InputEvent::Keyboard(KeyboardEvent::Key { key, is_up, .. }) => {
                        match *key {
                            VK_NUMPAD0 => {
                                /*if let Some(vehicles) = native::pool::get_vehicles() {
                                    for (i, mut v) in vehicles.iter().enumerate() {
                                        crate::info!("deleting vehicle: {} ({})", v.get_handle(), i);
                                        v.delete();
                                    }
                                }*/
                                self.tasks.push(move |env| {
                                    let player = Player::local();
                                    let ped = player.get_ped();
                                    if let Some(veh) = ped.get_in_vehicle(false) {
                                        veh.repair();
                                    }
                                });
                            },
                            0x46 /*F*/ if !is_up => {
                                self.tasks.push(move |env| {
                                    let player = Player::local();
                                    let ped = player.get_ped();
                                    if !crate::runtime::CONSOLE_VISIBLE.load(Ordering::SeqCst) && ped.exists() {
                                        if !ped.is_in_any_vehicle(true) {
                                            if let Some(vehicle) = ped.get_closest_vehicle(10.0) {
                                                if vehicle.is_seat_free(-1) {
                                                    ped.get_tasks().enter_vehicle(vehicle, 5000, -1, 2.0, 1);
                                                }
                                            }
                                        } else if ped.is_in_any_vehicle(false) {
                                            let vehicle = ped.get_using_vehicle().unwrap();

                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            ScriptEvent::ConsoleInput(input) => {
                let input = input.clone();
                self.tasks.push(move |env| {
                    let player = Player::local();
                    let ped = player.get_ped();
                    let model = Model::new(&input);
                    if model.is_valid() && model.is_in_cd_image() && model.is_vehicle() {
                        if !ped.is_in_any_vehicle(false) {
                            let veh = Vehicle::new(env, model, ped.get_position(), ped.get_heading(), false, false)
                                .expect("Vehicle creation failed");
                            ped.put_into_vehicle(&veh, -1);
                            env.log(format!("~y~Spawned vehicle ~w~{}~y~ at your position", input))
                        } else {
                            env.log("~r~You're already in a vehicle");
                        }
                    } else {
                        env.log(format!("~r~Invalid vehicle model: ~w~{}", input));
                    }
                });
            }
            _ => {}
        }
        false
    }
}

impl ScriptCleanWorld {
    fn cleanup(&self) {
        let player = Player::local();
        player.disable_vehicle_rewards();

        game::player::set_max_wanted_level(0);

        game::vehicle::set_garbage_trucks(false);
        game::vehicle::set_random_boats(false);
        game::vehicle::set_random_trains(false);
        game::vehicle::set_far_draw(false);
        game::vehicle::set_distant_visible(false);
        game::vehicle::delete_all_trains();
        game::vehicle::set_parked_count(-1);
        game::vehicle::set_low_priority_generators_active(false);
        game::vehicle::remove_vehicles_from_generators_in_area(
            Vector3::new(-9999.0, -9999.0, -9999.0),
            Vector3::new(9999.0, 9999.0, 9999.0),
            false
        );

        game::ped::set_non_scenario_cops(false);
        game::ped::set_cops(false);
        game::ped::set_scenario_cops(false);

        gameplay::set_time_scale(1.0);

        game::streaming::set_vehicle_population_budget(0);
        game::streaming::set_ped_population_budget(0);

        game::vehicle::set_distant_lights_visible(false);
        game::vehicle::set_parked_density_multiplier_this_frame(0.0);

        game::ui::set_map_revealed(true);
    }

    fn disable_controls(&self) {
        for control in CONTROLS_TO_DISABLE.iter() {
            game::controls::disable_action(ControlGroup::Move, *control, true);
        }
        game::controls::disable_action(ControlGroup::Wheel, Control::CharacterWheel, true);
    }

    fn terminate_all_scripts(&self, cleanup: bool) {
        for script in SCRIPTS_TO_TERMINATE.iter() {
            self.terminate_script(script, cleanup);
        }
    }

    fn terminate_script(&self, script: &str, cleanup: bool) {
        if cleanup {
            game::script::mark_unused(script);
            game::script::force_cleanup(script, 8);
        }
        game::script::terminate_all(script);
    }
}

pub struct ScriptFingerPointing {
    active: bool,
    camera: Option<Camera>
}

impl Script for ScriptFingerPointing {
    fn prepare(&mut self, mut env: ScriptEnv) {
        self.camera = Some(Camera::new("gameplay", false).expect("Camera creation failed"));
    }

    fn frame(&mut self, mut env: ScriptEnv) {
        let player = Player::local().get_ped();
        let tasks = player.get_tasks().get_network();

        tasks.is_move_active();

        let pitch = (self.get_relative_pitch().min(42.0).max(-70.0) + 70.0) / 112.0;
        let heading = (game::camera::get_gameplay_relative_heading().min(180.0).max(-180.0) + 180.0) / 360.0;

        tasks.set_move_signal("Pitch", pitch);
        tasks.set_move_signal("Heading", heading * -1.0 + 1.0);
        tasks.set_move_signal("isBlocked", false);
        use crate::invoke;
        let first_person = invoke!(u32, 0xEE778F8C7E1142E2, invoke!(u32, 0x19CAFA3C87F7C2FF)) == 4;
        tasks.set_move_signal("isFirstPerson", first_person);

        if game::controls::is_disabled_pressed(ControlGroup::Move, Control::Cover) && !player.is_in_any_vehicle(false) {
            if !self.active {
                self.active = true;
                let dict = AnimDict::new("anim@mp_point");
                env.wait_for_resource(&dict);
                player.set_config_flag(36, true);
                tasks.do_move("task_mp_pointing", 0.5, false, &dict, 24);
                dict.mark_unused();
            }
        } else if self.active {
            player.get_tasks().get_network().request_move_state_transition("Stop");
            player.get_tasks().clear_secondary();
            self.active = false;
        }
    }
}

impl ScriptFingerPointing {
    fn get_relative_pitch(&self) -> f32 {
        let camera_rotation = self.camera.as_ref().expect("missing gameplay camera").get_rotation(2);
        camera_rotation.x - Player::local().get_ped().get_pitch()
    }
}

pub(crate) const CONTROLS_TO_DISABLE: [Control; 15] = [
    Control::Enter,
    Control::Cover,
    Control::EnterCheatCode,
    Control::FrontendSocialClub,
    Control::FrontendSocialClubSecondary,
    Control::SpecialAbilityPC,
    Control::SpecialAbilitySecondary,
    Control::Phone,
    Control::Duck,
    Control::DropWeapon,
    Control::DropAmmo,
    Control::SelectCharacterFranklin, Control::SelectCharacterMichael,
    Control::SelectCharacterTrevor, Control::SelectCharacterMultiplayer
];

pub(crate) const SCRIPTS_TO_TERMINATE: [&str; 761] = [
    "abigail1",
    "abigail2",
    "achievement_controller",
    "act_cinema",
    "af_intro_t_sandy",
    "agency_heist1",
    "agency_heist2",
    "agency_heist3a",
    "agency_heist3b",
    "agency_prep1",
    "agency_prep2amb",
    "aicover_test",
    "ainewengland_test",
    "altruist_cult",
    "am_airstrike",
    "am_ammo_drop",
    "am_armwrestling",
    "am_armybase",
    "am_backup_heli",
    "am_boat_taxi",
    "am_bru_box",
    "am_car_mod_tut",
    "am_challenges",
    "am_contact_requests",
    "am_cp_collection",
    "am_cr_securityvan",
    "am_crate_drop",
    "am_criminal_damage",
    "am_darts",
    "am_dead_drop",
    "am_destroy_veh",
    "am_distract_cops",
    "am_doors",
    "am_ferriswheel",
    "am_ga_pickups",
    "am_gang_call",
    "am_heist_int",
    "am_heli_taxi",
    "am_hold_up",
    "am_hot_property",
    "am_hot_target",
    "am_hunt_the_beast",
    "am_imp_exp",
    "am_joyrider",
    "am_kill_list",
    "am_king_of_the_castle",
    "am_launcher",
    "am_lester_cut",
    "am_lowrider_int",
    "am_mission_launch",
    "am_mp_carwash_launch",
    "am_mp_garage_control",
    "am_mp_property_ext",
    "am_mp_property_int",
    "am_mp_smpl_interior_ext",
    "am_mp_smpl_interior_int",
    "am_mp_warehouse",
    "am_mp_yacht",
    "am_npc_invites",
    "am_pass_the_parcel",
    "am_penned_in",
    "am_pi_menu",
    "am_plane_takedown",
    "am_prison",
    "am_prostitute",
    "am_rollercoaster",
    "am_rontrevor_cut",
    "am_taxi",
    "am_vehicle_spawn",
    "ambient_diving",
    "ambient_mrsphilips",
    "ambient_solomon",
    "ambient_sonar",
    "ambient_tonya",
    "ambient_tonyacall",
    "ambient_tonyacall2",
    "ambient_tonyacall5",
    "ambient_ufos",
    "ambientblimp",
    "animal_controller",
    "appbroadcast",
    "appcamera",
    "appchecklist",
    "appcontacts",
    "appemail",
    "appextraction",
    "apphs_sleep",
    "appinternet",
    "appjipmp",
    "appmedia",
    "appmpbossagency",
    "appmpemail",
    "appmpjoblistnew",
    "apporganiser",
    "apprepeatplay",
    "appsecuroserv",
    "appsettings",
    "appsidetask",
    "apptextmessage",
    "apptrackify",
    "appvlsi",
    "appzit",
    "armenian1",
    "armenian2",
    "armenian3",
    "assassin_bus",
    "assassin_construction",
    "assassin_hooker",
    "assassin_multi",
    "assassin_rankup",
    "assassin_valet",
    "atm_trigger",
    "audiotest",
    "autosave_controller",
    "b757d.projitems",
    "b757d.shproj",
    "b757d.sln",
    "bailbond1",
    "bailbond2",
    "bailbond3",
    "bailbond4",
    "bailbond_launcher",
    "barry1",
    "barry2",
    "barry3",
    "barry3a",
    "barry3c",
    "barry4",
    "benchmark",
    "bigwheel",
    "bj",
    "blimptest",
    "blip_controller",
    "bootycall_debug_controller",
    "bootycallhandler",
    "buddydeathresponse",
    "bugstar_mission_export",
    "building_controller",
    "buildingsiteambience",
    "cablecar",
    "cam_coord_sender",
    "camera_test",
    "candidate_controller",
    "car_roof_test",
    "carmod_shop",
    "carsteal1",
    "carsteal2",
    "carsteal3",
    "carsteal4",
    "carwash1",
    "carwash2",
    "celebration_editor",
    "celebrations",
    "cellphone_controller",
    "cellphone_flashhand",
    "charactergoals",
    "charanimtest",
    "cheat_controller",
    "chinese1",
    "chinese2",
    "chop",
    "clothes_shop_mp",
    "clothes_shop_sp",
    "code_controller",
    "combat_test",
    "comms_controller",
    "completionpercentage_controller",
    "component_checker",
    "context_controller",
    "controller_ambientarea",
    "controller_races",
    "controller_taxi",
    "controller_towing",
    "controller_trafficking",
    "coordinate_recorder",
    "country_race",
    "country_race_controller",
    "creation_startup",
    "creator",
    "custom_config",
    "cutscene_test",
    "cutscenemetrics",
    "cutscenesamples",
    "darts",
    "debug",
    "debug_app_select_screen",
    "debug_launcher",
    "debug_ped_data",
    "density_test",
    "dialogue_handler",
    "director_mode",
    "docks2asubhandler",
    "docks_heista",
    "docks_heistb",
    "docks_prep1",
    "docks_prep2b",
    "docks_setup",
    "dont_cross_the_line",
    "dreyfuss1",
    "drf1",
    "drf2",
    "drf3",
    "drf4",
    "drf5",
    "drunk",
    "drunk_controller",
    "dynamixtest",
    "email_controller",
    "emergencycall",
    "emergencycalllauncher",
    "epscars",
    "epsdesert",
    "epsilon1",
    "epsilon2",
    "epsilon3",
    "epsilon4",
    "epsilon5",
    "epsilon6",
    "epsilon7",
    "epsilon8",
    "epsilontract",
    "epsrobes",
    "event_controller",
    "exile1",
    "exile2",
    "exile3",
    "exile_city_denial",
    "extreme1",
    "extreme2",
    "extreme3",
    "extreme4",
    "fairgroundhub",
    "fake_interiors",
    "fame_or_shame_set",
    "fameorshame_eps",
    "fameorshame_eps_1",
    "family1",
    "family1taxi",
    "family2",
    "family3",
    "family4",
    "family5",
    "family6",
    "family_scene_f0",
    "family_scene_f1",
    "family_scene_m",
    "family_scene_t0",
    "family_scene_t1",
    "fanatic1",
    "fanatic2",
    "fanatic3",
    "fbi1",
    "fbi2",
    "fbi3",
    "fbi4",
    "fbi4_intro",
    "fbi4_prep1",
    "fbi4_prep2",
    "fbi4_prep3",
    "fbi4_prep3amb",
    "fbi4_prep4",
    "fbi4_prep5",
    "fbi5a",
    "finale_choice",
    "finale_credits",
    "finale_endgame",
    "finale_heist1",
    "finale_heist2_intro",
    "finale_heist2a",
    "finale_heist2b",
    "finale_heist_prepa",
    "finale_heist_prepb",
    "finale_heist_prepc",
    "finale_heist_prepd",
    "finale_heist_prepeamb",
    "finale_intro",
    "finalea",
    "finaleb",
    "finalec1",
    "finalec2",
    "floating_help_controller",
    "flow_autoplay",
    "flow_controller",
    "flow_help",
    "flowintrotitle",
    "flowstartaccept",
    "flyunderbridges",
    "fm_bj_race_controler",
    "fm_capture_creator",
    "fm_deathmatch_controler",
    "fm_deathmatch_creator",
    "fm_hideout_controler",
    "fm_hold_up_tut",
    "fm_horde_controler",
    "fm_impromptu_dm_controler",
    "fm_intro",
    "fm_intro_cut_dev",
    "fm_lts_creator",
    "fm_main_menu",
    "fm_maintain_cloud_header_data",
    "fm_maintain_transition_players",
    "fm_mission_controller",
    "fm_mission_creator",
    "fm_race_controler",
    "fm_race_creator",
    "fmmc_launcher",
    "fmmc_playlist_controller",
    "forsalesigns",
    "fps_test",
    "fps_test_mag",
    "franklin0",
    "franklin1",
    "franklin2",
    "freemode",
    "freemode_init",
    "friendactivity",
    "friends_controller",
    "friends_debug_controller",
    "fullmap_test",
    "fullmap_test_flow",
    "game_server_test",
    "gb_airfreight",
    "gb_assault",
    "gb_bellybeast",
    "gb_carjacking",
    "gb_cashing_out",
    "gb_collect_money",
    "gb_contraband_buy",
    "gb_contraband_defend",
    "gb_contraband_sell",
    "gb_deathmatch",
    "gb_finderskeepers",
    "gb_fivestar",
    "gb_fragile_goods",
    "gb_headhunter",
    "gb_hunt_the_boss",
    "gb_point_to_point",
    "gb_rob_shop",
    "gb_salvage",
    "gb_sightseer",
    "gb_terminate",
    "gb_yacht_rob",
    "general_test",
    "golf",
    "golf_ai_foursome",
    "golf_ai_foursome_putting",
    "golf_mp",
    "gpb_andymoon",
    "gpb_baygor",
    "gpb_billbinder",
    "gpb_clinton",
    "gpb_griff",
    "gpb_jane",
    "gpb_jerome",
    "gpb_jesse",
    "gpb_mani",
    "gpb_mime",
    "gpb_pameladrake",
    "gpb_superhero",
    "gpb_tonya",
    "gpb_zombie",
    "gtest_airplane",
    "gtest_avoidance",
    "gtest_boat",
    "gtest_divingfromcar",
    "gtest_divingfromcarwhilefleeing",
    "gtest_helicopter",
    "gtest_nearlymissedbycar",
    "gunclub_shop",
    "gunfighttest",
    "hairdo_shop_mp",
    "hairdo_shop_sp",
    "hao1",
    "headertest",
    "heatmap_test",
    "heatmap_test_flow",
    "heist_ctrl_agency",
    "heist_ctrl_docks",
    "heist_ctrl_finale",
    "heist_ctrl_jewel",
    "heist_ctrl_rural",
    "heli_gun",
    "heli_streaming",
    "hud_creator",
    "hunting1",
    "hunting2",
    "hunting_ambient",
    "idlewarper",
    "ingamehud",
    "initial",
    "jewelry_heist",
    "jewelry_prep1a",
    "jewelry_prep1b",
    "jewelry_prep2a",
    "jewelry_setup1",
    "josh1",
    "josh2",
    "josh3",
    "josh4",
    "lamar1",
    "laptop_trigger",
    "launcher_abigail",
    "launcher_barry",
    "launcher_basejumpheli",
    "launcher_basejumppack",
    "launcher_carwash",
    "launcher_darts",
    "launcher_dreyfuss",
    "launcher_epsilon",
    "launcher_extreme",
    "launcher_fanatic",
    "launcher_golf",
    "launcher_hao",
    "launcher_hunting",
    "launcher_hunting_ambient",
    "launcher_josh",
    "launcher_maude",
    "launcher_minute",
    "launcher_mrsphilips",
    "launcher_nigel",
    "launcher_offroadracing",
    "launcher_omega",
    "launcher_paparazzo",
    "launcher_pilotschool",
    "launcher_racing",
    "launcher_rampage",
    "launcher_range",
    "launcher_stunts",
    "launcher_tennis",
    "launcher_thelastone",
    "launcher_tonya",
    "launcher_triathlon",
    "launcher_yoga",
    "lester1",
    "lesterhandler",
    "letterscraps",
    "line_activation_test",
    "liverecorder",
    "locates_tester",
    "luxe_veh_activity",
    "magdemo",
    "magdemo2",
    "main",
    "main_install",
    "main_persistent",
    "maintransition",
    "martin1",
    "maude1",
    "maude_postbailbond",
    "me_amanda1",
    "me_jimmy1",
    "me_tracey1",
    "mg_race_to_point",
    "michael1",
    "michael2",
    "michael3",
    "michael4",
    "michael4leadout",
    "minigame_ending_stinger",
    "minigame_stats_tracker",
    "minute1",
    "minute2",
    "minute3",
    "mission_race",
    "mission_repeat_controller",
    "mission_stat_alerter",
    "mission_stat_watcher",
    "mission_triggerer_a",
    "mission_triggerer_b",
    "mission_triggerer_c",
    "mission_triggerer_d",
    "mp_awards",
    "mp_fm_registration",
    "mp_menuped",
    "mp_prop_global_block",
    "mp_prop_special_global_block",
    "mp_registration",
    "mp_save_game_global_block",
    "mp_unlocks",
    "mp_weapons",
    "mpstatsinit",
    "mptestbed",
    "mrsphilips1",
    "mrsphilips2",
    "murdermystery",
    "navmeshtest",
    "net_bot_brain",
    "net_bot_simplebrain",
    "net_cloud_mission_loader",
    "net_combat_soaktest",
    "net_jacking_soaktest",
    "net_rank_tunable_loader",
    "net_session_soaktest",
    "net_tunable_check",
    "nigel1",
    "nigel1a",
    "nigel1b",
    "nigel1c",
    "nigel1d",
    "nigel2",
    "nigel3",
    "nodeviewer",
    "ob_abatdoor",
    "ob_abattoircut",
    "ob_airdancer",
    "ob_bong",
    "ob_cashregister",
    "ob_drinking_shots",
    "ob_foundry_cauldron",
    "ob_franklin_beer",
    "ob_franklin_tv",
    "ob_franklin_wine",
    "ob_huffing_gas",
    "ob_mp_bed_high",
    "ob_mp_bed_low",
    "ob_mp_bed_med",
    "ob_mp_shower_med",
    "ob_mp_stripper",
    "ob_mr_raspberry_jam",
    "ob_poledancer",
    "ob_sofa_franklin",
    "ob_sofa_michael",
    "ob_telescope",
    "ob_tv",
    "ob_vend1",
    "ob_vend2",
    "ob_wheatgrass",
    "offroad_races",
    "omega1",
    "omega2",
    "paparazzo1",
    "paparazzo2",
    "paparazzo3",
    "paparazzo3a",
    "paparazzo3b",
    "paparazzo4",
    "paradise",
    "paradise2",
    "pausemenu",
    "pausemenu_example",
    "pausemenu_map",
    "pausemenu_multiplayer",
    "pausemenu_sp_repeat",
    "pb_busker",
    "pb_homeless",
    "pb_preacher",
    "pb_prostitute",
    "photographymonkey",
    "photographywildlife",
    "physics_perf_test",
    "physics_perf_test_launcher",
    "pi_menu",
    "pickup_controller",
    "pickuptest",
    "pickupvehicles",
    "pilot_school",
    "pilot_school_mp",
    "placeholdermission",
    "placementtest",
    "planewarptest",
    "player_controller",
    "player_controller_b",
    "player_scene_f_lamgraff",
    "player_scene_f_lamtaunt",
    "player_scene_f_taxi",
    "player_scene_ft_franklin1",
    "player_scene_m_cinema",
    "player_scene_m_fbi2",
    "player_scene_m_kids",
    "player_scene_m_shopping",
    "player_scene_mf_traffic",
    "player_scene_t_bbfight",
    "player_scene_t_chasecar",
    "player_scene_t_insult",
    "player_scene_t_park",
    "player_scene_t_tie",
    "player_timetable_scene",
    "playthrough_builder",
    "pm_defend",
    "pm_delivery",
    "pm_gang_attack",
    "pm_plane_promotion",
    "pm_recover_stolen",
    "postkilled_bailbond2",
    "postrc_barry1and2",
    "postrc_barry4",
    "postrc_epsilon4",
    "postrc_nigel3",
    "profiler_registration",
    "prologue1",
    "prop_drop",
    "racetest",
    "rampage1",
    "rampage2",
    "rampage3",
    "rampage4",
    "rampage5",
    "rampage_controller",
    "randomchar_controller",
    "range_modern",
    "range_modern_mp",
    "re_abandonedcar",
    "re_accident",
    "re_armybase",
    "re_arrests",
    "re_atmrobbery",
    "re_bikethief",
    "re_border",
    "re_burials",
    "re_bus_tours",
    "re_cartheft",
    "re_chasethieves",
    "re_crashrescue",
    "re_cultshootout",
    "re_dealgonewrong",
    "re_domestic",
    "re_drunkdriver",
    "re_duel",
    "re_gang_intimidation",
    "re_gangfight",
    "re_getaway_driver",
    "re_hitch_lift",
    "re_homeland_security",
    "re_lossantosintl",
    "re_lured",
    "re_monkey",
    "re_mountdance",
    "re_muggings",
    "re_paparazzi",
    "re_prison",
    "re_prisonerlift",
    "re_prisonvanbreak",
    "re_rescuehostage",
    "re_seaplane",
    "re_securityvan",
    "re_shoprobbery",
    "re_snatched",
    "re_stag_do",
    "re_yetarian",
    //"replay_controller",
    "rerecord_recording",
    //"respawn_controller",
    "restrictedareas",
    "rollercoaster",
    "rural_bank_heist",
    "rural_bank_prep1",
    "rural_bank_setup",
    "save_anywhere",
    "savegame_bed",
    "sc_lb_global_block",
    "scaleformgraphictest",
    "scaleformminigametest",
    "scaleformprofiling",
    "scaleformtest",
    "scene_builder",
    "sclub_front_bouncer",
    "script_metrics",
    "scripted_cam_editor",
    "scriptplayground",
    "scripttest1",
    "scripttest2",
    "scripttest3",
    "scripttest4",
    "sctv",
    //"selector",
    "selector_example",
    "selling_short_1",
    "selling_short_2",
    "sh_intro_f_hills",
    "sh_intro_m_home",
    "shooting_camera",
    "shop_controller",
    "shoprobberies",
    "shot_bikejump",
    "shrinkletter",
    "smoketest",
    "social_controller",
    "solomon1",
    "solomon2",
    "solomon3",
    "sp_dlc_registration",
    "sp_editor_mission_instance",
    "sp_menuped",
    "sp_pilotschool_reg",
    "spaceshipparts",
    "spawn_activities",
    "speech_reverb_tracker",
    "spmc_instancer",
    "spmc_preloader",
    "standard_global_init",
    "standard_global_reg",
    "startup",
    "startup_install",
    "startup_locationtest",
    "startup_positioning",
    "startup_smoketest",
    "stats_controller",
    "stock_controller",
    "streaming",
    "stripclub",
    "stripclub_drinking",
    "stripclub_mp",
    "stripperhome",
    "stunt_plane_races",
    "tasklist_1",
    "tattoo_shop",
    "taxi_clowncar",
    "taxi_cutyouin",
    "taxi_deadline",
    "taxi_followcar",
    "taxi_gotyounow",
    "taxi_gotyourback",
    "taxi_needexcitement",
    "taxi_procedural",
    "taxi_takeiteasy",
    "taxi_taketobest",
    "taxilauncher",
    "taxiservice",
    "taxitutorial",
    "tempalpha",
    "temptest",
    "tennis",
    "tennis_ambient",
    "tennis_family",
    "tennis_network_mp",
    "test_startup",
    "thelastone",
    "timershud",
    "title_update_registration",
    "tonya1",
    "tonya2",
    "tonya3",
    "tonya4",
    "tonya5",
    "towing",
    "traffick_air",
    "traffick_ground",
    "traffickingsettings",
    "traffickingteleport",
    "train_create_widget",
    "train_tester",
    "trevor1",
    "trevor2",
    "trevor3",
    "trevor4",
    "triathlonsp",
    "tunables_registration",
    "tuneables_processing",
    "ufo",
    "ugc_global_registration",
    "underwaterpickups",
    "utvc",
    "veh_play_widget",
    "vehicle_ai_test",
    "vehicle_force_widget",
    "vehicle_gen_controller",
    "vehicle_plate",
    "walking_ped",
    "wardrobe_mp",
    "wardrobe_sp",
    "weapon_audio_widget",
    "wp_partyboombox",
    "xml_menus",
    "yoga"
];