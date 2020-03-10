use crate::{bind_fn, bind_field, bind_field_ip};
use crate::hash::Hash;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use crate::native::TypeInfo;

bind_fn!(INIT_MANIFEST_CHUNK, "48 8D 4F 10 B2 01 48 89 2F", -0x2E, "C", fn(*const ()) -> ());
bind_fn!(LOAD_MANIFEST_CHUNK, "45 38 AE C0 00 00 00 0F 95 C3 E8", -5, "C", fn(*const ()) -> ());
bind_fn!(CLEAR_MANIFEST_CHUNK, "33 FF 48 8D 4B 10 B2 01", -0x15, "C", fn(*const ()) -> ());
bind_fn!(ADD_PACK_FILE, "EB 15 48 8B 0B 40 38 7B 0C 74 07 E8", 11, "C", fn(*const DataFileEntry) -> ());
bind_fn!(REMOVE_PACK_FILE, "EB 15 48 8B 0B 40 38 7B 0C 74 07 E8", 18, "C", fn(*const DataFileEntry) -> ());

bind_field!(MANIFEST_CHUNK, "83 F9 08 75 43 48 8D 0D", 8, ());
bind_field_ip!(MOUNTERS, "48 63 82 90 00 00 00 49 8B 8C C0 ? ? ? ? 48", 11, [PackFileMounter; 255]);
bind_field_ip!(DATA_TYPES, "61 44 DF 04 00 00 00 00", 0, *const DataFileType);
/*lazy_static! {
    pub static ref DATA_TYPES_BY_HASH: HashMap<Hash, u32> = {
        DATA_TYPES.iter().map(|t| (t.hash, t.index)).collect::<_>()
    };
}*/

pub(crate) fn pre_init() {
    lazy_static::initialize(&MANIFEST_CHUNK);
    lazy_static::initialize(&MOUNTERS);
    lazy_static::initialize(&DATA_TYPES);
}

#[repr(C)]
pub struct DataFileType {
    hash: Hash,
    index: u32
}

impl DataFileType {
    /*pub fn find<T>(ty: T) -> Option<DataFileType> where T: AsRef<str> {
        let hash = crate::hash::joaat_cs(ty);
        let index = DATA_TYPES_BY_HASH.get(&hash).cloned()?;
        Some(DataFileType { hash, index })
    }

    pub fn get_mounter(&self) -> Option<&'static PackFileMounter> {
        MOUNTERS.get(self.index as usize)
    }*/
}

#[repr(C)]
pub struct DataFileEntry {
    name: [u8; 128],
    pad: [u8; 16],
    ty: u32,
    index: u32,
    locked: bool,
    flag2: bool,
    flag3: bool,
    disabled: bool,
    persistent: bool,
    overlay: bool,
    pad0: [u8; 10]
}

impl DataFileEntry {
}

#[repr(C)]
pub struct PackFileMounterVTable {
    type_info: ManuallyDrop<Box<TypeInfo>>,
    drop: extern "C" fn(this: *mut PackFileMounter),
    mount: extern "C" fn(this: *mut PackFileMounter, entry: *mut DataFileEntry),
    unmount: extern "C" fn(this: *mut PackFileMounter, entry: *mut DataFileEntry)
}

#[repr(C)]
pub struct PackFileMounter {
    v_table: ManuallyDrop<Box<PackFileMounterVTable>>
}

impl PackFileMounter {
    /*pub fn find<T>(ty: T) -> Option<&'static PackFileMounter> where T: AsRef<str> {
        DataFileType::find(ty)?.get_mounter()
    }*/

    pub fn mount(&mut self, entry: &mut DataFileEntry) {
        (self.v_table.mount)(self, entry)
    }

    pub fn unmount(&mut self, entry: &mut DataFileEntry) {
        (self.v_table.unmount)(self, entry)
    }
}

#[repr(C)]
pub struct CustomPackFileMounter {
    parent: PackFileMounter
}

impl CustomPackFileMounter {
    /*pub fn new() -> CustomPackFileMounter {
        unsafe {
            CustomPackFileMounter {
                parent: PackFileMounter {
                    v_table: ManuallyDrop::new(Box::new(PackFileMounterVTable {
                        drop: std::mem::transmute(Self::drop as *const ()),
                        mount: std::mem::transmute(Self::mount as *const ()),
                        unmount: std::mem::transmute(Self::unmount as *const ()),
                    }))
                }
            }
        }
    }*/

    fn drop(&mut self) {}

    pub fn mount(&mut self, entry: &mut DataFileEntry) {
        entry.disabled = true;
        //self.persistent = true;
        //self.locked = true;
        //self.overlay = true;
        INIT_MANIFEST_CHUNK(MANIFEST_CHUNK.as_ref());
        ADD_PACK_FILE(entry);
        LOAD_MANIFEST_CHUNK(MANIFEST_CHUNK.as_ref());
        CLEAR_MANIFEST_CHUNK(MANIFEST_CHUNK.as_ref());
    }

    pub fn unmount(&mut self, entry: &mut DataFileEntry) {
        REMOVE_PACK_FILE(entry);
    }
}