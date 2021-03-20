use std::ffi::{CStr, OsStr};
use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Seek, SeekFrom, Write};
use std::iter::once;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::os::raw::c_char;
use std::path::Path;

use alignas::AlignAs;
use cgmath::Zero;
use winapi::shared::minwindef::{DWORD, FILETIME};
use winapi::um::fileapi::INVALID_FILE_ATTRIBUTES;
use winapi::um::winbase::{FILE_BEGIN, FILE_CURRENT, FILE_END};
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;

use crate::{bind_field_ip, bind_fn, bind_fn_detour, bind_fn_detour_ip};
use crate::pattern::RageBox;

bind_fn_detour!(OPEN_PACK_FILES, "41 B0 01 BA 1B E6 DA 93 E8", -12, open_pack_files, () -> ());
bind_fn_detour!(ADD_COLLISION, "48 8B FA 89 44 24 30 48 8B D9 E8 ? ? ? ? 0F", 10, add_collision, (*mut u8, &mut u32, &u32) -> ());
bind_fn_detour!(SOME_FN, "66 39 79 38 74 06 4C 8B 41 30 EB 07 4C 8D", 19, some_fn, (*mut u8, *mut u8, RagePath) -> u32);

bind_fn!(GET_DEVICE, "41 B8 07 00 00 00 48 8B F1 E8", -0x1F, (RagePath, bool) -> Option<ManuallyDrop<Box<Device>>>);
bind_fn!(MOUNT_GLOBAL, "41 8A F0 48 8B F9 E8 ? ? ? ? 33 DB 85 C0", -0x28, (RagePath, &Device, bool) -> bool);
bind_fn!(UNMOUNT, "E8 ? ? ? ? 85 C0 75 23 48 83", -0x22, (RagePath) -> ());
bind_fn!(PACK_FILE_INIT, "44 89 41 28 4C 89 41 38 4C 89 41 50 48 8D", -0x1E, (&mut PackFile) -> ());
bind_fn!(PACK_FILE_OPEN, "48 8D 68 98 48 81 EC 40 01 00 00 41 8B F9", -0x18, (&mut PackFile, RagePath, bool, i32, u64) -> bool);
bind_fn!(PACK_FILE_MOUNT, "84 C0 74 1D 48 85 DB 74 0F 48", -0x1E, (&mut PackFile, RagePath) -> ());
bind_fn!(RELATIVE_DEVICE_SET_PATH, "49 8B F9 48 8B D9 4C 8B CA", -0x17, (&mut RelativeDevice, RagePath, bool, Option<&Device>) -> ());
bind_fn!(RELATIVE_DEVICE_MOUNT, "44 8A 81 14 01 00 00 48 8B DA 48 8B F9 48 8B D1", -0xD, (&mut RelativeDevice, RagePath, bool) -> ());
bind_fn!(KEY_STATE_INIT, "45 33 F6 48 89 85 30 02 00 00 48 8D 45 30 48", -12, (&mut KeyState, *const u8) -> ());

bind_fn_detour_ip!(INITIAL_MOUNT, "0F B7 05 ? ? ? ? 48 03 C3 44 88 34 38 66", 0x15, initial_mount, () -> ());

bind_field_ip!(DEVICE_VTABLE, "48 21 35 ? ? ? ? 48 8B 74 24 38 48 8D 05", 15, DeviceVTable);
bind_field_ip!(PACK_FILE_VTABLE, "44 89 41 28 4C 89 41 38 4C 89 41 50 48 8D 05", 15, DeviceVTable);
bind_field_ip!(RELATIVE_DEVICE_VTABLE, "48 85 C0 74 11 48 83 63 08 00 48", 13, DeviceVTable);
bind_field_ip!(ENCRYPTING_DEVICE_VTABLE, "45 33 F6 48 89 85 30 02 00 00 48 8D 45 30 48", -4, DeviceVTable);

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct RagePath {
    inner: *const c_char
}

impl AsRef<Path> for RagePath {
    fn as_ref(&self) -> &Path {
        let os: &OsStr = unsafe { std::mem::transmute(CStr::from_ptr(self.inner as _).to_bytes()) };
        Path::new(os)
    }
}

impl std::fmt::Display for RagePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = <Self as AsRef<Path>>::as_ref(self);
        f.write_fmt(format_args!("{}", path.display()))
    }
}

impl<P> From<P> for RagePath where P: AsRef<OsStr> {
    fn from(other: P) -> Self {
        let path: Vec<i8> = unsafe { std::mem::transmute::<_, &[i8]>(other.as_ref()) }
            .iter()
            .cloned()
            .chain(once(0))
            .collect::<_>();
        RagePath {
            inner: ManuallyDrop::new(path.into_boxed_slice()).as_ptr()
        }
    }
}

pub(crate) fn hook() {
    info!("Hooking filesystem...");
    //lazy_static::initialize(&OPEN_PACK_FILES);
    //lazy_static::initialize(&ADD_COLLISION);

    //lazy_static::initialize(&SOME_FN);

    lazy_static::initialize(&INITIAL_MOUNT);
    lazy_static::initialize(&GET_DEVICE);
    lazy_static::initialize(&MOUNT_GLOBAL);
    lazy_static::initialize(&UNMOUNT);
    lazy_static::initialize(&PACK_FILE_INIT);
    lazy_static::initialize(&PACK_FILE_OPEN);
    lazy_static::initialize(&PACK_FILE_MOUNT);
    lazy_static::initialize(&RELATIVE_DEVICE_SET_PATH);
    lazy_static::initialize(&RELATIVE_DEVICE_MOUNT);
    lazy_static::initialize(&KEY_STATE_INIT);

    lazy_static::initialize(&DEVICE_VTABLE);
    lazy_static::initialize(&PACK_FILE_VTABLE);
    lazy_static::initialize(&RELATIVE_DEVICE_VTABLE);
    lazy_static::initialize(&ENCRYPTING_DEVICE_VTABLE);
}

extern fn open_pack_files() {
    OPEN_PACK_FILES()
}

extern fn add_collision(module: *mut u8, index: &mut u32, hash: &u32) {
    ADD_COLLISION(module, index, hash)
}

extern "cdecl" fn some_fn(extra_content_manager: *mut u8, unk: *mut u8, device_name: RagePath) -> u32 {
    let result = SOME_FN(extra_content_manager, unk, device_name);
    info!("called some_fn on {}", device_name.as_ref().display());

    info!("got {}", result);

    result
}

pub(crate) fn init() {
    info!("Initializing FS");

    let mut dev = RelativeDevice::new();
    info!("created relative device");
    dev.set_path("C:/dlcpacks", true, None);
    info!("set path");
    let mnt = dev.mount("kek:/", true);
    walk(&*mnt, Path::new("kek:/"));
    info!("mounted");

    fn walk(device: &Device, path: &Path) {
        for f in device.entries(path) {
            let path = path.join(f.get_name());
            if f.is_directory() {
                let name = f.get_name();
                if name != "." && name != ".." {
                    info!("found dir: {} (attr: {})", path.display(), f.get_attributes());
                    walk(device, &path);
                }
            } else {
                info!("found file: {} ({} bytes)", path.display(), f.get_size());
            }
        }
    }

    /*if let Some(device) = Device::get("platform:/", true) {
        info!("platform walking...");
        walk(&device, Path::new("platform:/.."));
    }*/

    /*let mut d = RelativeDevice::new();
    d.set_path("C:/dlc.rpf", true, None);
    info!("dlc pack: {}", d.get_name());*/
    //walk(&d, Path::new("/"));
    //d.mount("kek:/", true).unmount();
}

extern fn initial_mount() {
    warn!("Initial mount");

    INITIAL_MOUNT();
}

#[repr(C)]
pub struct DeviceEntry {
    name: [u8; 256],
    size: u64,
    last_write_time: FILETIME,
    attributes: DWORD,
}

impl DeviceEntry {
    pub fn get_name(&self) -> &OsStr {
        let len = self.name.iter().position(u8::is_zero).unwrap_or_default();
        unsafe { std::mem::transmute(&self.name[0..len]) }
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

    pub fn is_directory(&self) -> bool {
        (self.attributes & FILE_ATTRIBUTE_DIRECTORY) != 0
    }

    pub fn get_last_write_time(&self) -> FILETIME {
        self.last_write_time
    }

    pub fn get_attributes(&self) -> DWORD {
        self.attributes
    }
}


#[repr(C)]
struct ResourceFlags {
    flag1: u32,
    flag2: u32,
}

#[repr(C)]
pub struct FileEntry {
    header: u64,
    virtual_flags: u32,
    physical_flags: u32,
}

impl FileEntry {
    pub fn get_name_offset(&self) -> u64 {
        (self.header >> 48) & 0xFF
    }

    pub fn get_size(&self) -> u64 {
        (self.header >> 24) & 0xFFF
    }

    pub fn get_offset(&self) -> u64 {
        self.header & 0xFFF
    }

    pub fn get_virtual_flags(&self) -> u32 {
        self.virtual_flags
    }

    pub fn get_physical_flags(&self) -> u32 {
        self.physical_flags
    }
}

#[repr(C)]
pub struct CollectionVTable {
    close: extern fn(*mut Collection),
    open_entry: extern fn(*mut Collection, index: u16, ptr: *mut u8),
    get_entry: extern fn(*mut Collection, index: u16) -> RageBox<FileEntry>,
    unk1: extern fn(*mut Collection, index: u16, flag: bool) -> u64,
    get_entry_name: extern fn(*mut Collection, index: u16) -> RagePath,
    get_entry_name_to_buffer: extern fn(*mut Collection, index: u16, buf: *mut u8, len: u32),
    get_entry_by_name: extern fn(*mut Collection, name: *const i8) -> u16,
    get_unk1: extern fn(*mut Collection) -> u32,
    get_unk_a: extern fn(*mut Collection, index: u16) -> bool,
    close_base_pack_file: extern fn(*mut Collection) -> bool,
    unk_c: extern fn(*mut Collection) -> u8,
    unk_d: extern fn(*mut Collection, u8) -> bool,
    unk_e: extern fn(*mut Collection, name: *const i8) -> *mut u8,
    unk_f: extern fn(*mut Collection, *mut (), bool) -> u64,

}

#[repr(C)]
pub struct Collection {
    device: Device,
    v_table: RageBox<CollectionVTable>,
}

pub struct CollectionExt {
    collection: Collection,
    m_pad: [u8; 184],
    child_pack_file: [u8; 192],
    child_pack_file_const: [u8; 192],
    parent: Option<Box<Collection>>,
}

#[repr(C)]
pub struct DeviceVTable {
    destructor: extern fn(this: *mut Device),
    open: extern fn(this: *mut Device, file_name: RagePath, read_only: bool) -> u64,
    open_bulk: extern fn(this: *mut Device, file_name: RagePath, base_offset: &mut usize) -> u64,
    open_bulk_wrap: extern fn(this: *mut Device, file_name: RagePath, ptr: *const u64, *const ()) -> u64,
    create_local: extern fn(this: *mut Device, file_name: RagePath) -> u64,
    create: extern fn(this: *mut Device, file_name: RagePath) -> u64,
    read: extern fn(this: *mut Device, handle: u64, buffer: *mut u8, to_read: u32) -> u32,
    read_bulk: extern fn(this: *mut Device, handle: u64, offset: usize, buffer: *mut u8, to_read: u32) -> u32,
    write_bulk: extern fn(this: *mut Device, handle: u64, i32, i32, i32, i32) -> u32,
    write: extern fn(this: *mut Device, handle: u64, buffer: *const u8, to_write: u32) -> u32,
    seek: extern fn(this: *mut Device, handle: u64, distance: i32, method: u32) -> u32,
    seek_long: extern fn(this: *mut Device, handle: u64, distance: i64, method: u32) -> u64,
    close: extern fn(this: *mut Device, handle: u64) -> i32,
    close_bulk: extern fn(this: *mut Device, handle: u64) -> i32,
    get_file_len: extern fn(this: *const Device, handle: u64) -> i32,
    get_file_len_u: extern fn(this: *const Device, handle: u64) -> u64,
    m_40: extern fn(this: *mut Device, i32) -> i32,
    remove_file: extern fn(this: *mut Device, file_name: RagePath) -> bool,
    rename_file: extern fn(this: *mut Device, from: RagePath, to: RagePath) -> i32,
    create_dir: extern fn(this: *mut Device, dir_name: RagePath) -> i32,
    remove_dir: extern fn(this: *mut Device, dir_name: RagePath) -> i32,
    m_xx: extern fn(this: *mut Device),
    get_file_len_l: extern fn(this: *const Device, file_name: RagePath) -> u64,
    get_file_time: extern fn(this: *const Device, file_name: RagePath) -> u64,
    set_file_time: extern fn(this: *mut Device, file_name: RagePath, time: FILETIME),
    find_first: extern fn(this: *const Device, path: RagePath, data: *mut DeviceEntry) -> u64,
    find_next: extern fn(this: *const Device, handle: u64, data: *mut DeviceEntry) -> bool,
    find_close: extern fn(this: *const Device, handle: u64),
    get_unk_device: extern fn(this: *mut Device) -> *const Device,
    m_xy: extern fn(this: *mut Device, *const (), i32, *const ()) -> *const (),
    truncate: extern fn(this: *mut Device, handle: u64) -> bool,
    get_file_attr: extern fn(this: *const Device, path: RagePath) -> u32,
    m_xz: extern fn(this: *mut Device) -> bool,
    set_file_attr: extern fn(this: *mut Device, attributes: u32) -> bool,
    m_yx: extern fn(this: *mut Device) -> i32,
    read_full: extern fn(this: *mut Device, handle: u64, buffer: *const (), len: u32) -> bool,
    write_full: extern fn(this: *mut Device, handle: u64, buffer: *const (), len: u32) -> bool,
    get_res_ver: extern fn(this: *mut Device, file_name: RagePath, flags: *const ResourceFlags) -> i32,
    m_yy: extern fn(this: *mut Device) -> i32,
    m_yz: extern fn(this: *mut Device, *const ()) -> i32,
    m_zx: extern fn(this: *mut Device, *const ()) -> i32,
    is_collection: extern fn(this: *mut Device) -> bool,
    m_added_in_1290: extern fn(this: *mut Device) -> bool,
    get_collection: extern fn(this: *mut Device) -> *const Device,
    m_ax: extern fn(this: *mut Device) -> bool,
    get_collection_id: extern fn(this: *mut Device) -> i32,
    get_name: extern fn(this: *const Device) -> RagePath,
}

#[repr(C)]
pub struct Device {
    v_table: RageBox<DeviceVTable>
}

impl Device {
    pub fn get<P>(path: P, allow_root: bool) -> Option<ManuallyDrop<Box<Device>>> where P: AsRef<Path> {
        GET_DEVICE(path.as_ref().into(), allow_root)
    }

    pub fn mount_global<P>(&self, mount_point: P, allow_root: bool) -> bool where P: AsRef<Path> {
        MOUNT_GLOBAL(mount_point.as_ref().into(), self, allow_root)
    }

    pub fn open<P>(&mut self, file_name: P, read_only: bool) -> Option<DeviceOpenGuard> where P: AsRef<Path> {
        let handle = (self.v_table.open)(self, file_name.as_ref().into(), read_only);
        if handle != u64::MAX {
            Some(DeviceOpenGuard {
                device: self,
                handle,
            })
        } else {
            None
        }
    }

    pub fn open_bulk<P>(&mut self, file_name: P) -> Option<DeviceOpenBulkGuard> where P: AsRef<Path> {
        let file_name: RagePath = file_name.as_ref().into();
        let mut base_offset = 0;
        let len = self.len(&file_name);
        let handle = (self.v_table.open_bulk)(self, file_name, &mut base_offset);
        if handle != u64::MAX {
            Some(DeviceOpenBulkGuard {
                device: self,
                len,
                handle,
                base_offset,
                offset: 0,
            })
        } else {
            None
        }
    }

    fn close(&mut self, handle: u64) -> i32 {
        (self.v_table.close)(self, handle)
    }

    fn close_bulk(&mut self, handle: u64) -> i32 {
        (self.v_table.close_bulk)(self, handle)
    }

    fn read(&mut self, handle: u64, buffer: &mut [u8]) -> IoResult<usize> {
        let read = (self.v_table.read)(self, handle, buffer.as_mut_ptr(), buffer.len() as _);
        if read == u32::MAX {
            Err(IoError::new(ErrorKind::UnexpectedEof, "unable to read"))
        } else {
            Ok(read as _)
        }
    }

    fn read_bulk(&mut self, handle: u64, offset: usize, buffer: &mut [u8]) -> IoResult<usize> {
        let read = (self.v_table.read_bulk)(self, handle, offset, buffer.as_mut_ptr(), buffer.len() as _);
        if read == u32::MAX {
            Err(IoError::new(ErrorKind::UnexpectedEof, "unable to read bulk"))
        } else {
            Ok(read as _)
        }
    }

    fn write(&mut self, handle: u64, buffer: &[u8]) -> IoResult<usize> {
        let written = (self.v_table.write)(self, handle, buffer.as_ptr(), buffer.len() as _);
        if written == u32::MAX {
            Err(IoError::new(ErrorKind::WriteZero, "unable to write"))
        } else {
            Ok(written as usize)
        }
    }

    /*fn write_bulk(&mut self, handle: u64, ptr: u64, buffer: &[u8]) -> IoResult<usize> {
        let written = (self.v_table.write_bulk)(self, handle, ptr, buffer.as_ptr(), buffer.len() as _);
        if written == u32::MAX {
            Err(IoError::new(ErrorKind::WriteZero, "unable to write"))
        } else {
            Ok(written as usize)
        }
    }*/

    fn seek(&mut self, handle: u64, from: SeekFrom) -> IoResult<u64> {
        let (method, distance) = match from {
            SeekFrom::Start(offset) => (FILE_BEGIN, offset as i64),
            SeekFrom::End(offset) => (FILE_END, offset),
            SeekFrom::Current(offset) => (FILE_CURRENT, offset)
        };
        let seek = (self.v_table.seek_long)(self, handle, distance, method);
        if seek == u64::MAX {
            Err(IoError::new(ErrorKind::UnexpectedEof, "unable to seek"))
        } else {
            Ok(seek)
        }
    }

    pub fn get_attributes<P>(&self, path: P) -> u32 where P: AsRef<Path> {
        (self.v_table.get_file_attr)(self, path.as_ref().into())
    }

    pub fn exists<P>(&self, path: P) -> bool where P: AsRef<Path> {
        self.get_attributes(path) != INVALID_FILE_ATTRIBUTES
    }

    pub fn is_directory<P>(&self, path: P) -> bool where P: AsRef<Path> {
        self.get_attributes(path) & FILE_ATTRIBUTE_DIRECTORY != 0
    }

    pub fn entries<P>(&self, path: P) -> DeviceEntries where P: AsRef<Path> {
        DeviceEntries {
            device: self,
            path: path.as_ref().into(),
            handle: None,
        }
    }

    fn entry_first(&self, path: RagePath, data: &mut DeviceEntry) -> u64 {
        (self.v_table.find_first)(self, path, data)
    }

    fn entry_next(&self, handle: u64, data: &mut DeviceEntry) -> bool {
        (self.v_table.find_next)(self, handle, data)
    }

    fn entry_close(&self, handle: u64) {
        (self.v_table.find_close)(self, handle)
    }

    fn create_local<P>(&mut self, file_name: P) -> u64 where P: AsRef<Path> {
        (self.v_table.create_local)(self, file_name.as_ref().into())
    }

    fn create<P>(&mut self, file_name: P) -> u64 where P: AsRef<Path> {
        (self.v_table.create)(self, file_name.as_ref().into())
    }

    pub fn len<P>(&self, file_name: P) -> u64 where P: AsRef<Path> {
        (self.v_table.get_file_len_l)(self, file_name.as_ref().into())
    }

    fn handle_len(&self, handle: u64) -> u64 {
        (self.v_table.get_file_len_u)(self, handle)
    }

    pub fn get_name(&self) -> RagePath {
        (self.v_table.get_name)(self)
    }
}

pub struct DeviceOpenGuard<'a> {
    device: &'a mut Device,
    handle: u64,
}

impl<'a> Drop for DeviceOpenGuard<'a> {
    fn drop(&mut self) {
        self.device.close(self.handle);
    }
}

impl<'a> Read for DeviceOpenGuard<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.device.read(self.handle, buf)
    }
}

impl<'a> Write for DeviceOpenGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.device.write(self.handle, buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

impl<'a> Seek for DeviceOpenGuard<'a> {
    fn seek(&mut self, from: SeekFrom) -> IoResult<u64> {
        self.device.seek(self.handle, from)
    }
}

pub struct DeviceEntries<'a> {
    device: &'a Device,
    path: RagePath,
    handle: Option<u64>,
}

impl<'a> Iterator for DeviceEntries<'a> {
    type Item = DeviceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut file = unsafe { std::mem::zeroed() };
        if let Some(handle) = self.handle {
            if self.device.entry_next(handle, &mut file) {
                Some(file)
            } else {
                None
            }
        } else {
            let handle = self.device.entry_first(self.path, &mut file);
            if handle != u64::MAX {
                self.handle = Some(handle);
                Some(file)
            } else {
                None
            }
        }
    }
}

impl<'a> Drop for DeviceEntries<'a> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.device.entry_close(handle)
        }
    }
}

pub struct DeviceOpenBulkGuard<'a> {
    device: &'a mut Device,
    len: u64,
    handle: u64,
    base_offset: usize,
    offset: isize,
}

impl<'a> DeviceOpenBulkGuard<'a> {
    pub fn len(&self) -> u64 {
        self.len
    }
}

impl<'a> Drop for DeviceOpenBulkGuard<'a> {
    fn drop(&mut self) {
        self.device.close_bulk(self.handle);
    }
}

impl<'a> Read for DeviceOpenBulkGuard<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let offset = (self.base_offset as isize + self.offset) as usize;
        let remaining = (self.len as isize - self.offset) as usize;
        let len = remaining.min(buf.len());
        if len == 0 {
            Ok(0)
        } else {
            let read = self.device.read_bulk(self.handle, offset, &mut buf[0..len])?;
            self.offset += read as isize;
            Ok(read)
        }
    }
}

/*impl<'a> Write for DeviceOpenBulkGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.device.write_bulk(self.handle, self.ptr, buf)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}*/

impl<'a> Seek for DeviceOpenBulkGuard<'a> {
    fn seek(&mut self, from: SeekFrom) -> IoResult<u64> {
        match from {
            SeekFrom::Start(offset) => {
                self.offset = offset as isize;
            }
            SeekFrom::End(offset) => {
                let len = self.len();
                self.offset = (len as i64 - offset) as isize
            }
            SeekFrom::Current(offset) => {
                self.offset = offset as isize;
            }
        };
        Ok(self.offset as _)
    }
}

const PACK_FILE_SIZE: usize = 368 + (0x650 - 0x590);

#[repr(C)]
pub struct PackFile {
    device: Device,
    inner: [u8; PACK_FILE_SIZE],
}

impl PackFile {
    pub fn open<P>(archive: P, ty: i32) -> Option<PackFile> where P: AsRef<Path> {
        let mut pack_file = PackFile {
            device: Device {
                v_table: PACK_FILE_VTABLE.cloned()
            },
            inner: [0; PACK_FILE_SIZE],
        };
        PACK_FILE_INIT(&mut pack_file);
        if PACK_FILE_OPEN(&mut pack_file, archive.as_ref().into(), true, ty, 0) {
            Some(pack_file)
        } else {
            None
        }
    }

    pub fn mount<'a, P>(mut self, mount_point: P) -> MountLock<Self> where P: AsRef<Path> {
        let mount_point = mount_point.as_ref().into();
        PACK_FILE_MOUNT(&mut self, mount_point);
        MountLock {
            device: ManuallyDrop::new(self),
            mount_point,
        }
    }
}

impl Deref for PackFile {
    type Target = Device;

    fn deref(&self) -> &Device {
        &self.device
    }
}

impl DerefMut for PackFile {
    fn deref_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}

const RELATIVE_DEVICE_SIZE: usize = 272;

#[repr(C)]
pub struct RelativeDevice {
    device: Device,
    inner: [u8; RELATIVE_DEVICE_SIZE],
}

#[repr(C)]
struct PackFileHeader {
    magic: u32,
    toc_size: u32,
    num_entries: u32,
    unk_flag: u32,
    crypto_flag: u32,
}

impl RelativeDevice {
    pub fn new() -> RelativeDevice {
        assert!(!RELATIVE_DEVICE_VTABLE.is_null(), "RELATIVE_DEVICE_VTABLE is null");
        RelativeDevice {
            device: Device {
                v_table: RELATIVE_DEVICE_VTABLE.cloned()
            },
            inner: [0; RELATIVE_DEVICE_SIZE],
        }
    }

    pub fn set_path<P>(&mut self, relative_to: P, allow_root: bool, base_device: Option<&Device>) where P: AsRef<Path> {
        RELATIVE_DEVICE_SET_PATH(self, relative_to.as_ref().into(), allow_root, base_device)
    }

    pub fn mount<'a, P>(mut self, mount_point: P, allow_root: bool) -> MountLock<Self> where P: AsRef<Path> {
        let mount_point = mount_point.as_ref().into();
        RELATIVE_DEVICE_MOUNT(&mut self, mount_point, allow_root);
        MountLock {
            device: ManuallyDrop::new(self),
            mount_point,
        }
    }
}

pub struct MountLock<D> where D: Deref<Target=Device> {
    device: ManuallyDrop<D>,
    mount_point: RagePath,
}

impl<D> MountLock<D> where D: Deref<Target=Device> {
    pub fn unmount(self) -> D {
        UNMOUNT(self.mount_point);
        ManuallyDrop::into_inner(self.device)
    }
}

impl<D> Deref for MountLock<D> where D: Deref<Target=Device> {
    type Target = D;

    fn deref(&self) -> &D {
        &*self.device
    }
}

impl<D> DerefMut for MountLock<D> where D: Deref<Target=Device> {
    fn deref_mut(&mut self) -> &mut D {
        &mut *self.device
    }
}

impl Deref for RelativeDevice {
    type Target = Device;

    fn deref(&self) -> &Device {
        &self.device
    }
}

impl DerefMut for RelativeDevice {
    fn deref_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}

const KEY_STATE_SIZE: usize = 1024;

#[repr(C)]
pub struct KeyState {
    state: Box<[u8; KEY_STATE_SIZE]>
}

impl KeyState {
    pub fn new(key: [u8; 32]) -> KeyState {
        let mut state = KeyState {
            state: Box::new([0; KEY_STATE_SIZE])
        };
        KEY_STATE_INIT(&mut state, key.as_ptr());
        state
    }
}

#[repr(C)]
pub struct EncryptingDevice {
    device: Device,
    key_state: KeyState,
    m_0010: *const (),
    buffer: [u8; 4096],
    m_1018: bool,
    pad: AlignAs<[u8; 64], i32>,
}

impl EncryptingDevice {
    pub fn new(key: [u8; 32]) -> EncryptingDevice {
        let device = Device {
            v_table: ENCRYPTING_DEVICE_VTABLE.cloned()
        };
        EncryptingDevice {
            device,
            key_state: KeyState::new(key),
            m_0010: std::ptr::null(),
            buffer: [0; 4096],
            m_1018: false,
            pad: AlignAs::new([0; 64]),
        }
    }
}

impl Deref for EncryptingDevice {
    type Target = Device;

    fn deref(&self) -> &Device {
        &self.device
    }
}

impl DerefMut for EncryptingDevice {
    fn deref_mut(&mut self) -> &mut Device {
        &mut self.device
    }
}