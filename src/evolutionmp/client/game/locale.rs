use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::sync::Mutex;

use crate::{bind_fn_detour_ip, bind_field_ip, bind_fn, invoke};
use crate::hash::{Hash, Hashable, joaat};

bind_fn_detour_ip!(GET_TEXT, "48 8B CB 8B D0 E8 ? ? ? ? 48 85 C0 0F 95 C0", 5, TranslationTable::get_text, (&TranslationTable, Hash) -> *const u8);
bind_fn_detour_ip!(GET_TEXT2, "48 85 C0 75 34 8B 0D", -5, TranslationTable::get_text, (&TranslationTable, Hash) -> *const u8);

pub enum TranslationTable {}

impl TranslationTable {
    extern fn get_text(&self, hash: Hash) -> *const u8 {
        let table = TRANSLATION_TABLE.lock().expect("mutex poisoned");
        if let Some(translation) = table.get(&hash) {
            return translation.as_bytes_with_nul().as_ptr();
        }
        //info!("getting text for hash 0x{:08X}", hash.0);
        let result = GET_TEXT(self, hash);
        //info!("got text {} for hash 0x{:08X}", CStr::from_ptr(result as _).to_string_lossy(), hash.0);
        result
    }
}

pub fn hook() {
    info!("Hooking locales...");
    lazy_static::initialize(&GET_TEXT);
    lazy_static::initialize(&GET_TEXT2);
}

pub fn init() {
    set_translation("PM_PAUSE_HDR", "Evolution MP");
    let title = "Загрузка сетевой игры";
    set_translation("LOADING_SPLAYER_L", title);
    set_translation("LOADING_MPLAYER_L", title);
}

lazy_static! {
    static ref TRANSLATION_TABLE: Mutex<HashMap<Hash, CString>> = Mutex::new(HashMap::new());
}

pub fn get_translation<'a>(label: &str) -> &'a str {
    invoke!(&str, 0x7B5280EBA9840C72, label)
}

pub fn set_translation<H>(label: H, translation: &str) where H: Hashable {
    let mut table = TRANSLATION_TABLE.lock().expect("mutex poisoned");
    let hash = label.joaat();
    if hash == joaat("LOADING_SPLAYER_L") {

    }
    if let Ok(translation) = CString::new(translation) {
        table.insert(hash, translation);
    }
}