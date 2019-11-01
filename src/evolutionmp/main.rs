use crate::win::user::{MessageBoxButtons, MessageBoxIcon, message_box};
use crate::win::ps::get_current_process;
use crate::pattern::{MemoryRegion, RegionIterator};
use crate::game::vehicle::Vehicle;
use crate::hash::joaat;
use crate::game::player::Player;
use crate::game::entity::Entity;
use crate::game::ui::{CursorSprite, LoadingPrompt};
use crate::win::input::{InputHook, KeyEvent};
use crate::game::scaleform::Scaleform;
use crate::script::{Script, Wait};
use crate::game::GameState;
use std::ptr::null_mut;
use std::time::Duration;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;
use std::env::current_dir;
use std::collections::HashMap;
use std::time::Instant;
use winapi::shared::minwindef::{HINSTANCE, DWORD, LPVOID, BOOL, TRUE, HMODULE};
use winapi::um::libloaderapi::{DisableThreadLibraryCalls, FreeLibraryAndExitThread};
use winapi::ctypes::c_void;
use winapi::um::winuser::{VK_BACK, VK_NUMPAD5};
use std::panic::PanicInfo;
use dirs::desktop_dir;

pub mod win;
pub mod hash;
pub mod native;
pub mod script;
pub mod mappings;
pub mod game;
pub mod pattern;
pub mod process;
pub mod registry;
pub mod mp;

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_THREAD_ATTACH: u32 = 2;
const DLL_THREAD_DETACH: u32 = 3;
const DLL_PROCESS_DETACH: u32 = 0;

fn attach(instance: HMODULE) {
    unsafe {
        DisableThreadLibraryCalls(instance);
        let mem = MemoryRegion::image();

        //global_region.find("E8 ? ? ? ? 84 C0 75 0C B2 01 B9 2F").next().expect("launcher").nop(21); //Disable launcher check
        //global_region.find("72 1F E8 ? ? ? ? 8B 0D").next().expect("legals").nop(2); //Disable legals
        mem.find_first_await("70 6C 61 74 66 6F 72 6D 3A 2F 6D 6F 76", 50, 1000)
            .expect("movie").nop(13); //Disable movie
        /*mem.find("72 6F 63 6B 73 74 61 72 5F 6C 6F 67 6F 73 00 62 69 6B")
            .next().expect("news").replace("32 73 65 63 6F 6E 64 73 62 6C 61 63 6B 2E 62 69 6B 00");*/

        native::init(&mem);

        std::thread::spawn(move || {
            let mut input = win::input::InputHook::new().expect("Input hooking failed");

            while game::get_state() != GameState::MainMenu {
                std::thread::sleep(Duration::from_millis(50));
            }

            script::init(&mem);

            mp::init(&mem);

            loop {
                while let Some(event) = input.next_event() {
                    for s in &mut script::SCRIPTS {
                        s.key(event);
                    }
                }
            }
        });
    }
}

fn detach() {

}

#[macro_export]
macro_rules! error {
    ($caption:expr,$($arg:tt)*) => {
        use crate::win::user::*;
        unsafe { message_box(None, format!($($arg)*), $caption, MessageBoxButtons::Ok, Some(MessageBoxIcon::Error)) };
    };
}

#[macro_export]
macro_rules! info {
    ($caption:expr,$($arg:tt)*) => {
        use crate::win::user::*;
        unsafe { message_box(None, format!($($arg)*), $caption, MessageBoxButtons::Ok, Some(MessageBoxIcon::Information)) };
    };
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HINSTANCE, reason: DWORD, reserved: LPVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::panic::set_hook(Box::new(|info: &PanicInfo| {
                //let backtrace = Backtrace::new();

                let thread = std::thread::current();
                let thread = thread.name().unwrap_or("unnamed");

                let reason = self::downcast_str(info.payload());

                let location = match info.location() {
                    Some(location) => format!(": {}:{}", location.file(), location.line()).replace("\\", "/"),
                    None => String::from("")
                };

                error!("EvolutionMP Error", "thread '{}' panicked at '{}'{}", thread, reason, location);

                /*let s = format!("{:?}", backtrace);

                for line in s.lines(){
                    debug!(target: LOG_PANIC, "{}", line);
                }*/
            }));
            attach(instance)
        }
        DLL_PROCESS_DETACH => {
            detach();
        }
        _ => {},
    }
    TRUE
}

pub fn downcast_str(string: &(dyn std::any::Any + Send)) -> &str  {
    match string.downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => {
            match string.downcast_ref::<String>() {
                Some(s) => &**s,
                None => {
                    "Box<Any>"
                }
            }
        }
    }
}