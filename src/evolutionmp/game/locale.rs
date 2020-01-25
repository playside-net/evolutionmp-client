use crate::invoke;
use crate::pattern::MemoryRegion;
use crate::hash::{Hash, Hashable};
use std::ffi::{CString, CStr};

type GetText = extern "C" fn(text: *mut (), hash: Hash) -> *const u8;
static mut NATIVE_GET_TEXT: *const () = std::ptr::null();

pub unsafe extern "C" fn get_text(text: *mut (), hash: Hash) -> *const u8 {
    let mut table = TRANSLATION_TABLE.lock().expect("translation table lock failed");
    if let Some(translation) = table.get(&hash) {
        return translation.as_bytes_with_nul().as_ptr();
    }
    //crate::info!("getting text for hash 0x{:08X}", hash.0);
    let origin: GetText = std::mem::transmute(NATIVE_GET_TEXT);
    let result = origin(text, hash);
    //crate::info!("got text {} for hash 0x{:08X}", CStr::from_ptr(result as _).to_string_lossy(), hash.0);
    result
}

type PauseMenuTrigger = extern "C" fn(u32, u32, u32);
static mut PAUSE_MENU_TRIGGER: *const () = std::ptr::null();

pub unsafe extern "C" fn pause_menu_trigger(trigger: u32, arg2: u32, arg3: u32) {
    let origin: PauseMenuTrigger = std::mem::transmute(PAUSE_MENU_TRIGGER);
    match trigger {
        1 | 10 | 5 | 42 => {}, //Ignore useless tabs
        other => {
            crate::info!("Pause menu trigger: {}, {}, {}", trigger, arg2, arg3);
            origin(trigger, arg2, arg3);
        }
    }
}

pub unsafe fn init(mem: &MemoryRegion) {
    NATIVE_GET_TEXT = mem.find("48 8B CB 8B D0 E8 ? ? ? ? 48 85 C0 0F 95 C0")
        .next().expect("get_text").add(5).detour(get_text as _);

    PAUSE_MENU_TRIGGER = mem.find("48 8D 8D 18 01 00 00 BE 74 26 B5 9F")
        .next().expect("pause_menu_trigger").offset(-5).detour(pause_menu_trigger as _);

    let _ = mem.find("48 85 C0 75 34 8B 0D")
        .next().expect("get_text 2").offset(-5).detour(get_text as _);

    set_translation("PM_PAUSE_HDR", "Evolution MP");
    set_translation("FE_THDR_GTAO", "Evolution MP");
}

use std::sync::Mutex;
use std::collections::HashMap;

lazy_static! {
    static ref TRANSLATION_TABLE: Mutex<HashMap<Hash, CString>> = Mutex::new(HashMap::new());
}

pub fn get_translation<'a>(label: &str) -> &'a str {
    invoke!(&str, 0x7B5280EBA9840C72, label)
}

pub fn set_translation<H>(label: H, translation: &str) where H: Hashable {
    let mut table = TRANSLATION_TABLE.lock().expect("translation table lock failed");
    let translation = CString::new(translation).expect("c string creation failed");
    table.insert(label.joaat(), translation);
}