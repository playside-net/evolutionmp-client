use crate::pattern::MemoryRegion;
use std::os::raw::c_char;
use std::os::windows::ffi::OsStrExt;
use std::mem::ManuallyDrop;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::io::{Read, Error as IoError, Write, ErrorKind, Result as IoResult, Seek, SeekFrom, Error};
use std::path::{PathBuf, Path};
use std::ops::{Deref, DerefMut};
use winapi::shared::minwindef::{FILETIME, DWORD};
use winapi::um::winbase::{FILE_BEGIN, FILE_END, FILE_CURRENT};
use winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY;
use alignas::AlignAs;
use detour::RawDetour;
use std::fmt::Formatter;
use winapi::um::fileapi::INVALID_FILE_ATTRIBUTES;
use crate::launcher_dir;

#[repr(C)]
#[derive(Clone)]
pub struct RagePath {
    inner: *const u8
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
        f.pad(&format!("{}", path.display()))
    }
}

impl<P> From<P> for RagePath where P: AsRef<OsStr> {
    fn from(other: P) -> Self {
        let path: Vec<u8> = unsafe { std::mem::transmute::<_, &[u8]>(other.as_ref()) }
            .iter()
            .cloned()
            .chain(once(b'\0'))
            .collect::<_>();
        RagePath {
            inner: ManuallyDrop::new(path.into_boxed_slice()).as_ptr()
        }
    }
}

use crate::{bind_fn, bind_field, bind_fn_detour_ip};
use std::fs::File;
use std::iter::once;

bind_fn!(GET_DEVICE, "41 B8 07 00 00 00 48 8B F1 E8", -0x1F, "C", fn(RagePath, bool) -> Option<ManuallyDrop<Box<Device>>>);
bind_fn!(MOUNT_GLOBAL, "41 8A F0 48 8B F9 E8 ? ? ? ? 33 DB 85 C0", -0x28, "C", fn(RagePath, *const Device, bool) -> bool);
bind_fn!(UNMOUNT, "E8 ? ? ? ? 85 C0 75 23 48 83", -0x22, "C", fn(RagePath) -> ());
bind_fn!(PACK_FILE_INIT, "44 89 41 28 4C 89 41 38 4C 89 41 50 48 8D", -0x1E, "thiscall", fn(*mut PackFile) -> ());
bind_fn!(PACK_FILE_OPEN, "48 8D 68 98 48 81 EC 40 01 00 00 41 8B F9", -0x18, "thiscall", fn(*mut PackFile, RagePath, bool, i32, bool) -> bool);
bind_fn!(PACK_FILE_MOUNT, "84 C0 74 1D 48 85 DB 74 0F 48", -0x1E, "thiscall", fn(*mut PackFile, RagePath) -> ());
bind_fn!(RELATIVE_DEVICE_SET_PATH, "49 8B F9 48 8B D9 4C 8B CA", -0x17, "thiscall", fn(*mut RelativeDevice, RagePath, bool, Option<&Device>) -> ());
bind_fn!(RELATIVE_DEVICE_MOUNT, "44 8A 81 14 01 00 00 48 8B DA 48 8B F9 48 8B D1", -0xD, "thiscall", fn(*mut RelativeDevice, RagePath, bool) -> ());
bind_fn!(KEY_STATE_INIT, "45 33 F6 48 89 85 30 02 00 00 48 8D 45 30 48", -12, "thiscall", fn(*mut KeyState, *const u8) -> ());

bind_fn_detour_ip!(INITIAL_MOUNT, "0F B7 05 ? ? ? ? 48 03 C3 44 88 34 38 66", 0x15, initial_mount, "C", fn() -> ());

static mut DEVICE_VTABLE: *const u8 = std::ptr::null();
static mut PACK_FILE_VTABLE: *const u8 = std::ptr::null();
static mut RELATIVE_DEVICE_VTABLE: *const u8 = std::ptr::null();
static mut ENCRYPTING_DEVICE_VTABLE: *const u8 = std::ptr::null();

macro_rules! vtable {
    ($field:ident,$name:literal,$pattern:literal,$offset:literal) => {
        $field = crate::native::MEM.find($pattern)
            .next().expect(concat!($name, "vtable"))
            .offset($offset).read_ptr(4).as_ptr();
    };
}

pub(crate) unsafe fn pre_init(mem: &MemoryRegion) {
    bind_field!(DEVICE_LIMIT, "C7 05 ? ? ? ? 64 00 00 00 48 8B", 6, u32);
    unsafe { *DEVICE_LIMIT.as_mut() *= 5 };
    mem.find("C6 80 F0 00 00 00 01 E8 ? ? ? ? E8")
        .next().expect("no relative device sorting")
        .add(12).nop(5);

    lazy_static::initialize(&INITIAL_MOUNT);

    vtable!(DEVICE_VTABLE, "Device", "48 21 35 ? ? ? ? 48 8B 74 24 38 48 8D 05", 15);
    vtable!(PACK_FILE_VTABLE, "PackFile", "44 89 41 28 4C 89 41 38 4C 89 41 50 48 8D 05", 15);
    vtable!(RELATIVE_DEVICE_VTABLE, "RelativeDevice", "48 85 C0 74 11 48 83 63 08 00 48", 13);
    vtable!(ENCRYPTING_DEVICE_VTABLE, "EncryptingDevice", "45 33 F6 48 89 85 30 02 00 00 48 8D 45 30 48", -4);
}

extern "C" fn initial_mount() {
    crate::info!("Initial mount");

    INITIAL_MOUNT();

    //super::pre_init();

    /*fn walk(device: &Device, path: &Path) {
        for f in device.entries(path) {
            let path = path.join(f.get_name());
            if f.is_directory() {
                crate::info!("found dir: {}", path.display());
                walk(device, &path);
            } else {
                crate::info!("found file: {} ({} bytes)", path.display(), f.get_size());
                let mut d = Device::get(&path, true).unwrap();
                let mut op = d.open(&path, false)
                    .expect("device opening failed");
                let mut output = File::create(launcher_dir().join(&path))
                    .expect("output file creation failed");
                std::io::copy(&mut op, &mut output);
                let open = PackFile::open(&path, 3)
                    .expect("pack file opening failed");
            }
        }
    }*/

    /*let path = PathBuf::from("platform:/models/farlods.ydd");

    if let Some(mut device) = Device::get(&path, false) {
        device.mount_global("kek:/", true);
        let mut input = device.open("kek:/models/farlods.ydd", false)
            .expect("device opening failed");
        let mut output = std::fs::File::create(launcher_dir().join("farlods.ydd")).unwrap();
        std::io::copy(&mut input, &mut output);

        *//*crate::info!("{:?} len is {}", path, device.len(&path));
        device.mount_global("temp:/", true);
        info!("mounted as temp:/");*//*
        *//*let mut pack_file = PackFile::open("platform:/levels/gta5/_citye/downtown_01/dt1_07.rpf", 3)
            .expect("pack file opening failed");
        crate::info!("opened pack file: {}", pack_file.get_name());
        pack_file.mount("kek:/");
        let mut dev = Device::get("kek:/", true)
            .expect("device getting failed");
        walk(&dev, Path::new("/"));*//*
        *//*let mut input = device.open(&path, false)
            .expect("device opening failed");*//*

    }*/

    /*let mut d = RelativeDevice::new();
    d.set_path("C:/dlc.rpf", true, None);
    crate::info!("dlc pack: {}", d.get_name());*/
    //walk(&d, Path::new("/"));
    //d.mount("kek:/", true).unmount();
}

#[repr(C)]
pub struct DeviceEntry {
    name: [i8; 256],
    size: u64,
    last_write_time: FILETIME,
    attributes: DWORD
}

impl DeviceEntry {
    pub fn get_name(&self) -> RagePath {
        RagePath {
            inner: unsafe { std::mem::transmute(self.name.as_ptr()) }
        }
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
    flag2: u32
}

#[repr(C)]
struct DeviceVTable {
    destructor:         extern "thiscall" fn(this: *mut Device),
    open:               extern "thiscall" fn(this: *mut Device, file_name: RagePath, read_only: bool) -> u64,
    open_bulk:          extern "thiscall" fn(this: *mut Device, file_name: RagePath, ptr: *const u64) -> u64,
    open_bulk_wrap:     extern "thiscall" fn(this: *mut Device, file_name: RagePath, ptr: *const u64, *const ()) -> u64,
    create_local:       extern "thiscall" fn(this: *mut Device, file_name: RagePath) -> u64,
    create:             extern "thiscall" fn(this: *mut Device, file_name: RagePath) -> u64,
    read:               extern "thiscall" fn(this: *mut Device, handle: u64, buffer: *mut u8, to_read: u32) -> u32,
    read_bulk:          extern "thiscall" fn(this: *mut Device, handle: u64, ptr: u64, buffer: *const (), to_read: u32) -> u32,
    write_bulk:         extern "thiscall" fn(this: *mut Device, handle: u64, i32, i32, i32, i32) -> u32,
    write:              extern "thiscall" fn(this: *mut Device, handle: u64, buffer: *const u8, to_write: u32) -> u32,
    seek:               extern "thiscall" fn(this: *mut Device, handle: u64, distance: i32, method: u32) -> u32,
    seek_long:          extern "thiscall" fn(this: *mut Device, handle: u64, distance: i64, method: u32) -> u64,
    close:              extern "thiscall" fn(this: *mut Device, handle: u64) -> i32,
    close_bulk:         extern "thiscall" fn(this: *mut Device, handle: u64) -> i32,
    get_file_len:       extern "thiscall" fn(this: *mut Device, handle: u64) -> i32,
    get_file_len_u:     extern "thiscall" fn(this: *mut Device, handle: u64) -> u64,
    m_40:               extern "thiscall" fn(this: *mut Device, i32) -> i32,
    remove_file:        extern "thiscall" fn(this: *mut Device, file_name: RagePath) -> bool,
    rename_file:        extern "thiscall" fn(this: *mut Device, from: RagePath, to: RagePath) -> i32,
    create_dir:         extern "thiscall" fn(this: *mut Device, dir_name: RagePath) -> i32,
    remove_dir:         extern "thiscall" fn(this: *mut Device, dir_name: RagePath) -> i32,
    m_xx:               extern "thiscall" fn(this: *mut Device),
    get_file_len_l:     extern "thiscall" fn(this: *const Device, file_name: RagePath) -> u64,
    get_file_time:      extern "thiscall" fn(this: *const Device, file_name: RagePath) -> u64,
    set_file_time:      extern "thiscall" fn(this: *mut Device, file_name: RagePath, time: FILETIME),
    find_first:         extern "thiscall" fn(this: *const Device, path: RagePath, data: *mut DeviceEntry) -> u64,
    find_next:          extern "thiscall" fn(this: *const Device, handle: u64, data: *mut DeviceEntry) -> bool,
    find_close:         extern "thiscall" fn(this: *const Device, handle: u64),
    get_unk_device:     extern "thiscall" fn(this: *mut Device) -> *const Device,
    m_xy:               extern "thiscall" fn(this: *mut Device, *const (), i32, *const ()) -> *const (),
    truncate:           extern "thiscall" fn(this: *mut Device, handle: u64) -> bool,
    get_file_attr:      extern "thiscall" fn(this: *const Device, path: RagePath) -> u32,
    m_xz:               extern "thiscall" fn(this: *mut Device) -> bool,
    set_file_attr:      extern "thiscall" fn(this: *mut Device, attributes: u32) -> bool,
    m_yx:               extern "thiscall" fn(this: *mut Device) -> i32,
    read_full:          extern "thiscall" fn(this: *mut Device, handle: u64, buffer: *const (), len: u32) -> bool,
    write_full:         extern "thiscall" fn(this: *mut Device, handle: u64, buffer: *const (), len: u32) -> bool,
    get_res_ver:        extern "thiscall" fn(this: *mut Device, file_name: RagePath, flags: *const ResourceFlags) -> i32,
    m_yy:               extern "thiscall" fn(this: *mut Device) -> i32,
    m_yz:               extern "thiscall" fn(this: *mut Device, *const ()) -> i32,
    m_zx:               extern "thiscall" fn(this: *mut Device, *const ()) -> i32,
    is_collection:      extern "thiscall" fn(this: *mut Device) -> bool,
    m_added_in_1290:    extern "thiscall" fn(this: *mut Device) -> bool,
    get_collection:     extern "thiscall" fn(this: *mut Device) -> *const Device,
    m_ax:               extern "thiscall" fn(this: *mut Device) -> bool,
    get_collection_id:  extern "thiscall" fn(this: *mut Device) -> i32,
    get_name:           extern "thiscall" fn(this: *const Device) -> RagePath
}

#[repr(C)]
pub struct Device {
    v_table: ManuallyDrop<Box<DeviceVTable>>
}

impl Device {
    pub fn get<P>(path: P, allow_root: bool) -> Option<ManuallyDrop<Box<Device>>> where P: Into<RagePath> {
        unsafe {
            GET_DEVICE(path.into(), allow_root)
        }
    }

    pub fn mount_global<P>(&self, mount_point: P, allow_root: bool) -> bool where P: Into<RagePath> {
        unsafe {
            MOUNT_GLOBAL(mount_point.into(), self, allow_root)
        }
    }

    pub fn open<P>(&mut self, file_name: P, read_only: bool) -> Option<DeviceOpenGuard> where P: Into<RagePath> {
        let handle = (self.v_table.open)(self, file_name.into(), read_only);
        if handle != std::u64::MAX {
            Some(DeviceOpenGuard {
                device: self,
                handle
            })
        } else {
            None
        }
    }

    fn close(&self, handle: u64) -> i32 {
        (self.v_table.close)(self as *const _ as *mut _, handle)
    }

    fn read(&mut self, handle: u64, buffer: &mut [u8], to_read: usize) -> IoResult<usize> {
        let read = (self.v_table.read)(self as *const _ as *mut _, handle, buffer.as_mut_ptr(), to_read as u32);
        if read == std::u32::MAX {
            Err(IoError::new(ErrorKind::UnexpectedEof, "unable to read"))
        } else {
            Ok(read as usize)
        }
    }

    fn write(&mut self, handle: u64, buffer: &[u8], to_write: usize) -> IoResult<usize> {
        let written = (self.v_table.write)(self as *const _ as *mut _, handle, buffer.as_ptr(), to_write as u32);
        if written == std::u32::MAX {
            Err(IoError::new(ErrorKind::WriteZero, "unable to write"))
        } else {
            Ok(written as usize)
        }
    }

    fn seek(&mut self, handle: u64, from: SeekFrom) -> IoResult<u64> {
        let (method, distance) = match from {
            SeekFrom::Start(offset) => (FILE_BEGIN, offset as i64),
            SeekFrom::End(offset) => (FILE_END, offset),
            SeekFrom::Current(offset) => (FILE_CURRENT, offset)
        };
        let seek = (self.v_table.seek_long)(self as *const _ as *mut _, handle, distance, method);
        if seek == std::u64::MAX {
            Err(IoError::new(ErrorKind::UnexpectedEof, "unable to seek"))
        } else {
            Ok(seek)
        }
    }

    pub fn get_attributes<P>(&self, path: P) -> u32 where P: Into<RagePath> {
        (self.v_table.get_file_attr)(self, path.into())
    }

    pub fn exists<P>(&self, path: P) -> bool where P: Into<RagePath> {
        self.get_attributes(path) != INVALID_FILE_ATTRIBUTES
    }

    pub fn is_directory<P>(&self, path: P) -> bool where P: Into<RagePath> {
        self.get_attributes(path) & FILE_ATTRIBUTE_DIRECTORY != 0
    }

    pub fn entries<P>(&self, path: P) -> DeviceEntries where P: Into<RagePath> {
        DeviceEntries {
            device: self,
            path: path.into(),
            handle: None
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

    pub fn create_local<P>(&mut self, file_name: P) -> u64 where P: Into<RagePath> {
        (self.v_table.create_local)(self, file_name.into())
    }

    pub fn create<P>(&mut self, file_name: P) -> u64 where P: Into<RagePath> {
        (self.v_table.create)(self, file_name.into())
    }

    pub fn len<P>(&self, file_name: P) -> u64 where P: Into<RagePath> {
        (self.v_table.get_file_len_l)(self, file_name.into())
    }

    pub fn get_name(&self) -> RagePath {
        (self.v_table.get_name)(self)
    }
}

pub struct DeviceOpenGuard<'a> {
    device: &'a mut Device,
    handle: u64
}

impl<'a> Drop for DeviceOpenGuard<'a> {
    fn drop(&mut self) {
        self.device.close(self.handle);
    }
}

impl<'a> Read for DeviceOpenGuard<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.device.read(self.handle, buf, buf.len())
    }
}

impl<'a> Write for DeviceOpenGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.device.write(self.handle, buf, buf.len())
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
    handle: Option<u64>
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
            let handle = self.device.entry_first(self.path.clone(), &mut file);
            if handle != std::u64::MAX {
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

/*pub struct DeviceOpenBulkGuard<'a> {
    device: &'a mut Device,
    handle: u64,
    ptr: u64
}

impl<'a> Drop for DeviceOpenBulkGuard<'a> {
    fn drop(&mut self) {
        self.device.close_bulk(self.handle);
    }
}

impl<'a> Read for DeviceOpenBulkGuard<'a> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.device.read_bulk(self.handle, buf, buf.len())
    }
}

impl<'a> Write for DeviceOpenBulkGuard<'a> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.device.write_bulk(self.handle, buf, buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}*/

const PACK_FILE_SIZE: usize = 368 + (0x650 - 0x590);

#[repr(C)]
pub struct PackFile {
    device: Device,
    inner: [u8; PACK_FILE_SIZE]
}

impl PackFile {
    pub fn open<P>(archive: P, ty: i32) -> Option<PackFile> where P: Into<RagePath> {
        unsafe {
            let mut pack_file = PackFile {
                device: Device {
                    v_table: std::mem::transmute(PACK_FILE_VTABLE)
                },
                inner: [0; PACK_FILE_SIZE]
            };
            PACK_FILE_INIT(&mut pack_file);
            if PACK_FILE_OPEN(&mut pack_file, archive.into(), true, ty, false) {
                Some(pack_file)
            } else {
                None
            }
        }
    }

    pub fn mount<P>(&mut self, mount_point: P) where P: Into<RagePath> {
        unsafe {
            PACK_FILE_MOUNT(self, mount_point.into())
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
    inner: [u8; RELATIVE_DEVICE_SIZE]
}

impl RelativeDevice {
    pub fn new() -> RelativeDevice {
        let mut inner = [0; RELATIVE_DEVICE_SIZE];
        inner[256] = b'\0';
        RelativeDevice {
            device: Device {
                v_table: unsafe { std::mem::transmute(RELATIVE_DEVICE_VTABLE) }
            },
            inner
        }
    }

    pub fn set_path<P>(&mut self, relative_to: P, allow_root: bool, base_device: Option<&Device>) where P: AsRef<OsStr> {
        unsafe {
            RELATIVE_DEVICE_SET_PATH(self, relative_to.into(), allow_root, base_device)
        }
    }

    pub fn mount<P>(mut self, mount_point: P, allow_root: bool) -> MountLock<Self> where P: Into<RagePath> {
        let mount_point = mount_point.into();
        unsafe {
            RELATIVE_DEVICE_MOUNT(&mut self, mount_point.clone(), allow_root)
        }
        MountLock {
            device: ManuallyDrop::new(self),
            mount_point
        }
    }
}

pub struct MountLock<D> where D: Deref<Target=Device> {
    device: ManuallyDrop<D>,
    mount_point: RagePath
}

/*impl<D> MountLock<D> where D: Deref<Target=Device> {
    pub fn unmount(self) -> D {
        unsafe {
            (UNMOUNT.unwrap())(self.mount_point)
        }
        ManuallyDrop::into_inner(self.device)
    }
}*/

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
        unsafe {
            KEY_STATE_INIT(&mut state, key.as_ptr());
        }
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
    pad: AlignAs<[u8; 64], i32>
}

impl EncryptingDevice {
    pub fn new(key: [u8; 32]) -> EncryptingDevice {
        let device = Device {
            v_table: unsafe { std::mem::transmute(ENCRYPTING_DEVICE_VTABLE) }
        };
        EncryptingDevice {
            device,
            key_state: KeyState::new(key),
            m_0010: std::ptr::null(),
            buffer: [0; 4096],
            m_1018: false,
            pad: AlignAs::new([0; 64])
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