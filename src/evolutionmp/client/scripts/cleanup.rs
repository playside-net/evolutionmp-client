use std::time::Instant;

use cgmath::{Array, Vector3};

use crate::events::ScriptEvent;
use crate::game;
use crate::game::controls::{Control, Group as ControlGroup};
use crate::game::ped::Ped;
use crate::game::player::Player;
use crate::game::vehicle::Dispatch;
use crate::native::pool::Handleable;
use crate::runtime::Script;
use crate::client::win::input::{InputEvent, KeyboardEvent};
use winapi::um::winuser::VK_NUMPAD9;

use crate::hash::Hashable;
use std::collections::HashMap;

static AUDIO_FLAGS: [(&'static str, bool); 7] = [
    ("LoadMPData", true),
    ("DisableBarks", true),
    ("DisableFlightMusic", true),
    ("PoliceScannerDisabled", true),
    ("OnlyAllowScriptTriggerPoliceScanner", true),
    ("PlayMenuMusic", false),
    ("ActivateSwitchWheelAudio", false)
];

pub struct ScriptCleanWorld {
    last_cleanup: Instant,
    loaded: bool
}

impl ScriptCleanWorld {
    pub fn new() -> ScriptCleanWorld {
        ScriptCleanWorld {
            last_cleanup: Instant::now(),
            loaded: false
        }
    }
}

impl Script for ScriptCleanWorld {
    fn frame(&mut self) {
        if !self.loaded {
            self.loaded = true;

            crate::invoke!((), 0x77B5F9A36BF96710, false);

            let ped = Ped::local();
            let pos = Vector3::new(-1030.0, -2730.0, 13.46);
            game::streaming::load_scene(pos);
            crate::invoke!((), 0x621873ECE1178967, ped.get_handle(), pos);

            game::misc::set_stunt_jumps_can_trigger(false);
            //game::gameplay::lower_map_prop_density(true);
            game::clock::pause(true);

            for (flag, enabled) in AUDIO_FLAGS.iter() {
                game::audio::set_flag(flag, *enabled);
            }

            /*while !game::is_loaded() {
                wait(1);
            }

            ped.set_model("mp_m_freemode_01");

            let pf = ped.get_parental_features();
            pf.set(ParentalFeatures {
                face_shape: Vector3::new(14, 17, 0),
                skin_tone: Vector3::new(14, 17, 0),
                mix: Vector3::new(0.85, 0.84, 0.0)
            });
            let appearance = ped.get_appearance();
            appearance.get_components().set(AppearanceComponent::Hair, AppearanceVariation {
                drawable: 2,
                texture: 0,
                palette: 2
            });*/
        }

        self.disable_controls();
        game::streaming::stop_player_switch();
        game::gameplay::set_time_scale(1.0);

        self.cleanup();

        game::ped::set_density_multiplier_this_frame(0.0);
        game::ped::set_scenario_density_multiplier_this_frame(0.0);

        game::vehicle::set_density_multiplier_this_frame(0.0);
        game::vehicle::set_random_density_multiplier_this_frame(0.0);

        game::decision_event::suppress_shocking_events_next_frame();
        game::decision_event::suppress_agitation_events_next_frame();

        if game::misc::is_stunt_jump_in_progress() {
            game::misc::cancel_stunt_jump();
        }

        if game::misc::get_mission_flag() {
            game::misc::set_mission_flag(false);
        }

        if game::misc::get_random_event_flag() {
            game::misc::set_random_event_flag(false);
        }

        if game::misc::is_cutscene_active() {
            game::misc::cancel_cutscene();
        }

        let ped = Ped::local();

        let has_special_ability = if let Some(vehicle) = ped.get_in_vehicle(false) {
            vehicle.has_jumping_ability() || vehicle.has_kers_boost() || vehicle.has_rocket_boost()
        } else {
            false
        };
        game::ui::set_ability_bar_visible(has_special_ability);

        if game::controls::is_disabled_just_pressed(ControlGroup::Wheel, Control::FrontendPauseAlternate) && game::controls::is_enabled(ControlGroup::Wheel, Control::FrontendSelect) {
            game::ui::activate_frontend_menu("FE_MENU_VERSION_SP_PAUSE", false, -1);
        }

        if game::ui::is_pause_menu_active() {
            game::controls::disable_action(ControlGroup::VehicleMoveAll, Control::VehicleAccelerate, true);
            game::controls::disable_action(ControlGroup::VehicleMoveAll, Control::VehicleBrake, true);
        }
    }

    fn event(&mut self, _event: ScriptEvent) {}
}

impl ScriptCleanWorld {
    fn cleanup(&self) {
        //let player = Player::local();
        //player.disable_vehicle_rewards();

        game::player::set_max_wanted_level(0);

        game::vehicle::set_dispatch_service(Dispatch::AmbulanceDepartment, false);
        game::vehicle::set_dispatch_service(Dispatch::FireDepartment, false);
        game::vehicle::set_dispatch_service(Dispatch::BikerBackup, false);
        game::vehicle::set_garbage_trucks(false);
        game::vehicle::set_random_boats(false);
        game::vehicle::set_random_trains(false);
        game::vehicle::set_far_draw(false);
        game::vehicle::set_distant_visible(false);
        game::vehicle::delete_all_trains();
        game::vehicle::set_parked_count(-1);
        game::vehicle::set_low_priority_generators_active(false);
        let range = Vector3::from_value(9999.0);
        game::vehicle::remove_vehicles_from_generators_in_area(-range, range, false);

        //game::ped::set_non_scenario_cops(false);
        game::ped::set_cops(false);
        game::ped::set_scenario_cops(false);

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
}

pub(crate) const CONTROLS_TO_DISABLE: [Control; 20] = [
    Control::Cover,
    Control::EnterCheatCode,
    Control::FrontendPause,
    Control::FrontendPauseAlternate,
    Control::FrontendSocialClub,
    Control::FrontendSocialClubSecondary,
    Control::SpecialAbilityPC,
    Control::SpecialAbilitySecondary,
    Control::Phone,
    Control::Duck,
    Control::DropWeapon,
    Control::DropAmmo,
    Control::SelectCharacterFranklin, Control::SelectCharacterMichael,
    Control::SelectCharacterTrevor, Control::SelectCharacterMultiplayer,
    Control::CinematicSlowMo,
    Control::VehicleSlowMoDownOnly,
    Control::VehicleSlowMoUpOnly,
    Control::VehicleSlowMoUpDown
];