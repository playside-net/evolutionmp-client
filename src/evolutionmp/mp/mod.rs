use crate::script::{Script, Wait};
use crate::pattern::MemoryRegion;
use crate::GameState;
use crate::{invoke, game, native};
use crate::game::entity::Entity;
use crate::game::stats::Stat;
use crate::game::ped::Ped;
use crate::game::player::Player;
use crate::game::vehicle::Vehicle;
use crate::win::input::KeyEvent;
use crate::hash::joaat;
use std::time::{Duration, Instant};
use game::controls::{Control, Group as ControlGroup};
use game::ui::{CursorSprite, LoadingPrompt};
use winapi::um::winuser::{VK_NUMPAD5, VK_NUMPAD2, VK_NUMPAD0, VK_RIGHT, VK_LEFT, VK_BACK};

pub mod disabled_scripts;

pub unsafe fn init(mem: &MemoryRegion) {
    crate::script::register(ScriptCleanWorld {
        money: Stat::new("SP1_TOTAL_CASH")
    });
}

pub struct ScriptCleanWorld {
    money: Stat<i32>
}

impl Script for ScriptCleanWorld {
    fn load(&mut self, wait: &mut Wait) {
        unsafe {
            while crate::native::ui::is_loading_screen_active() {
                wait(Duration::from_millis(0));
            }

            native::audio::set_flag("LoadMPData", true);
            native::audio::set_flag("DisableBarks", true);
            native::audio::set_flag("DisableFlightMusic", true);
            native::audio::set_flag("PoliceScannerDisabled", true);
            native::audio::set_flag("OnlyAllowScriptTriggerPoliceScanner", true);
        }
    }

    fn frame(&mut self, wait: &mut Wait, game_state: GameState) {
        /*for script in disabled_scripts::DISABLED_SCRIPTS.iter() {
            unsafe { native::script::terminate_all(script) };
        }*/
        unsafe {
            native::streaming::set_vehicle_population_budget(0);
            native::streaming::set_ped_population_budget(0);

            native::vehicle::set_parked_count(std::u32::MAX);
            native::vehicle::set_distant_visible(false);
            native::vehicle::set_distant_lights_visible(false);
            native::vehicle::set_density_multiplier_this_frame(0.0);
            native::vehicle::set_random_density_multiplier_this_frame(0.0);
            native::vehicle::set_parked_density_multiplier_this_frame(0.0);

            native::ped::set_density_multiplier_this_frame(0.0);
            native::ped::set_scenario_density_multiplier_this_frame(0.0);

            native::ui::set_map_revealed(true);
        }
        game::ui::set_cursor_sprite(CursorSprite::MiddleFinger);
        let disabled_controls = [
            Control::EnterCheatCode, Control::FrontendSocialClub, Control::FrontendSocialClubSecondary,
            Control::SpecialAbilityPC, Control::SpecialAbilitySecondary, Control::Phone,
            Control::Duck
        ];
        for control in disabled_controls.iter() {
            game::controls::disable_action(ControlGroup::Move, *control, true);
        }
        game::controls::disable_action(ControlGroup::Wheel, Control::CharacterWheel, true);
        let player = Player::local();
        let ped = player.get_ped();
        if let Some(veh) = ped.get_using_vehicle() {
            let color = veh.get_colors();
            game::ui::show_loading_prompt(LoadingPrompt::LoadingLeft3, &format!("Primary: {}, Secondary: {}", color.primary, color.secondary));
        } else {
            game::ui::hide_loading_prompt();
        }
    }

    fn on_key(&mut self, key: KeyEvent, time_caught: Instant) {
        if key.key == VK_NUMPAD5 {
            self.money.set(99999, true);
        } else if key.key == VK_RIGHT {
            let player = Player::local();
            let ped = player.get_ped();
            if let Some(veh) = ped.get_in_vehicle(false) {
                let colors = veh.get_colors();
                if colors.primary < 159 {
                    veh.set_colors(colors.primary + 1, colors.secondary);
                }
            }
        } else if key.key == VK_LEFT {
            let player = Player::local();
            let ped = player.get_ped();
            if let Some(veh) = ped.get_in_vehicle(false) {
                let colors = veh.get_colors();
                if colors.primary > 0 {
                    veh.set_colors(colors.primary - 1, colors.secondary);
                }
            }
        } else if key.key == VK_NUMPAD0 {
            let player = Player::local();
            let ped = player.get_ped();
            if let Some(veh) = ped.get_in_vehicle(false) {
                veh.repair();
            }
        } else if key.key == VK_BACK {

        }
    }
}