use crate::{invoke, bind_fn_detour_ip};
use crate::pattern::MemoryRegion;
use crate::hash::{Hash, Hashable};
use std::ffi::{CString, CStr};

bind_fn_detour_ip!(GET_TEXT, "48 8B CB 8B D0 E8 ? ? ? ? 48 85 C0 0F 95 C0", 5, get_text, "C", fn(*mut (), Hash) -> *const u8);
bind_fn_detour_ip!(GET_TEXT2, "48 85 C0 75 34 8B 0D", -5, get_text, "C", fn(*mut (), Hash) -> *const u8);

pub extern "C" fn get_text(text: *mut (), hash: Hash) -> *const u8 {
    let mut table = TRANSLATION_TABLE.lock().expect("translation table lock failed");
    if let Some(translation) = table.get(&hash) {
        return translation.as_bytes_with_nul().as_ptr();
    }
    //crate::info!("getting text for hash 0x{:08X}", hash.0);
    let result = GET_TEXT(text, hash);
    //crate::info!("got text {} for hash 0x{:08X}", CStr::from_ptr(result as _).to_string_lossy(), hash.0);
    result
}

pub fn pre_init() {
    lazy_static::initialize(&GET_TEXT);
    lazy_static::initialize(&GET_TEXT2);
}

pub fn init() {
    set_translation("PM_PAUSE_HDR", "Evolution MP");
    set_translation("LOADING_SPLAYER_L", "Loading Evolution MP");
    set_translation("LOADING_MPLAYER_L", "Loading Evolution MP");
}

use std::sync::Mutex;
use std::collections::HashMap;
use backtrace::SymbolName;
use std::path::PathBuf;

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