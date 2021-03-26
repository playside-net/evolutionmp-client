use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::{CStr, CString, OsString, OsStr};
use std::marker::PhantomData;
use std::mem::size_of;
use std::path::PathBuf;
use std::ptr::null_mut;

use winapi::ctypes::c_void;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD, FARPROC, HMODULE, LPVOID, MAX_PATH, TRUE};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::{ReadProcessMemory, VirtualAllocEx, VirtualFreeEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{CreateRemoteThreadEx, CreateThread, GetCurrentProcess, GetProcessId, OpenProcess};
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleFileNameExW};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS, Module32First, Module32Next, MODULEENTRY32};
use winapi::um::winbase::{INFINITE, THREAD_PRIORITY_HIGHEST};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE};
use wio::wide::{FromWide, ToWide};

use crate::win::thread::ThreadHandle;

pub fn get_current_process() -> ProcessHandle {
    ProcessHandle::from(unsafe { GetCurrentProcess() })
}

pub fn get_process<S>(file_name: S, desired_access: DWORD) -> Option<ProcessHandle> where S: AsRef<str> {
    for process in ProcessIterator::new(TH32CS_SNAPPROCESS)? {
        if &process.get_name() == file_name.as_ref() {
            return Some(process.open(desired_access, false).expect(&format!("Error opening process `{}`", file_name.as_ref())));
        }
    }
    None
}

pub fn get_procedure_address<'a>(module_str: &'a CStr, procedure_str: &'a CStr) -> Result<FARPROC, InjectionError<'a>> {
    let module: HMODULE = unsafe {
        GetModuleHandleA(module_str.as_ptr())
    };

    if module.is_null() {
        return Err(InjectionError::MissingModule(module_str));
    }

    let procedure = unsafe {
        GetProcAddress(module, procedure_str.as_ptr())
    };

    if procedure.is_null() {
        Err(InjectionError::MissingModuleProcedure(module_str, procedure_str))
    } else {
        Ok(procedure)
    }
}

type ElevatedThread = unsafe extern "system" fn(LPVOID) -> DWORD;

pub unsafe fn create_elevated_thread(thread: ElevatedThread) -> bool {
    CloseHandle(CreateThread(null_mut(), 0, Some(thread), null_mut(), THREAD_PRIORITY_HIGHEST, null_mut())) == TRUE
}

pub struct ModuleHandle {
    instance: HMODULE,
    name: OsString,
}

impl ModuleHandle {
    pub fn get_instance(&self) -> HMODULE {
        self.instance
    }

    pub fn get_name(&self) -> &OsString {
        &self.name
    }

    pub fn get_procedure_address(&self, procedure: &str) -> Option<FARPROC> {
        let procedure_str = CString::new(procedure).unwrap();
        let procedure = unsafe {
            GetProcAddress(self.instance, procedure_str.as_ptr() as _)
        };
        if procedure.is_null() {
            None
        } else {
            Some(procedure)
        }
    }
}

type TY = unsafe extern "system" fn(LPVOID) -> DWORD;

pub struct ProcessHandle {
    inner: HANDLE
}

impl ProcessHandle {
    pub fn get_pid(&self) -> u32 {
        unsafe { GetProcessId(self.inner) }
    }

    pub fn get_modules(&self, flags: DWORD) -> Vec<ModuleHandle> {
        let mut result = Vec::new();
        let mut modules = vec![std::ptr::null_mut(); 1024];
        let mut count: DWORD = 0;
        unsafe {
            let cb = (1024 * size_of::<HMODULE>()) as DWORD;
            if EnumProcessModulesEx(self.inner, modules.as_mut_ptr(), cb, &mut count, flags) == TRUE {
                let mut mod_name = vec![0; MAX_PATH];
                for i in 0..(count as usize / size_of::<HMODULE>()) {
                    let module = modules[i];
                    if GetModuleFileNameExW(self.inner, module, mod_name.as_mut_ptr(), MAX_PATH as DWORD) != 0 {
                        let name = OsString::from_wide_ptr_null(mod_name.as_ptr());
                        result.push(ModuleHandle {
                            instance: module,
                            name,
                        });
                    }
                }
            }
        }
        result
    }

    pub fn inject_library(&self, dll_path: PathBuf) -> Result<u32, InjectionError> {
        if !dll_path.exists() {
            return Err(InjectionError::FileDoesntExist);
        }

        let load_library_address = get_procedure_address(c_str!("Kernel32.dll"), c_str!("LoadLibraryW"))?;
        let dll_path = dll_path.into_os_string().to_wide_null();
        let alloc = self.virtual_alloc(&dll_path, null_mut(), MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE).unwrap();

        match self.create_thread(load_library_address, &alloc) {
            Ok(thread) => {
                thread.wait_for_single_object(INFINITE);
                Ok(thread.get_exit_code())
            }
            Err(err) => Err(InjectionError::CantCreateThread(err))
        }
    }

    pub fn virtual_alloc<T>(&self, value: &T, address: LPVOID, allocation_type: DWORD, protect: DWORD) -> Result<VirtualAlloc<T>, ProcessMemoryError> where T: RemoteData {
        let size = value.get_size();
        let alloc = unsafe { self.virtual_alloc_uninit::<T>(address, size, allocation_type, protect) }?;
        alloc.write(value)?;
        Ok(alloc)
    }

    pub unsafe fn virtual_alloc_uninit<T>(&self, address: LPVOID, size: SIZE_T, allocation_type: DWORD, protect: DWORD) -> Result<VirtualAlloc<T>, ProcessMemoryError> where T: RemoteData {
        let inner = VirtualAllocEx(self.inner, address, size, allocation_type, protect);
        if inner.is_null() {
            Err(ProcessMemoryError::AllocationFailed(GetLastError()))
        } else {
            Ok(VirtualAlloc {
                process: self,
                address,
                size,
                inner,
                _data: PhantomData,
            })
        }
    }

    pub unsafe fn write<D>(&self, base_address: LPVOID, data: &D) -> Result<usize, ProcessMemoryError> where D: RemoteData {
        let size = data.get_size();
        let mut bytes_written = 0usize;
        if WriteProcessMemory(**self, base_address, data.get_ptr(), size, &mut bytes_written) == TRUE {
            if size != bytes_written {
                Err(ProcessMemoryError::WriteBytesMismatch(size, bytes_written))
            } else {
                Ok(bytes_written)
            }
        } else {
            Err(ProcessMemoryError::WriteFailed(GetLastError()))
        }
    }

    pub unsafe fn read<D>(&self, base_address: LPVOID, size: SIZE_T) -> Result<D, ProcessMemoryError> where D: RemoteData {
        let mut data = vec![0u8; size];
        let _ = self.read_into(base_address, size, &mut data)?;
        Ok(D::read(data))
    }

    pub unsafe fn read_into<D>(&self, base_address: LPVOID, size: SIZE_T, data: &mut D) -> Result<usize, ProcessMemoryError> where D: RemoteData {
        let mut bytes_read = 0usize;
        if ReadProcessMemory(**self, base_address, data.get_mut_ptr() as *mut _, size, &mut bytes_read) == TRUE {
            if size != bytes_read {
                Err(ProcessMemoryError::ReadBytesMismatch(size, bytes_read))
            } else {
                Ok(bytes_read)
            }
        } else {
            Err(ProcessMemoryError::ReadFailed(GetLastError()))
        }
    }

    pub fn create_thread<T>(&self, start_routine: FARPROC, arg: &VirtualAlloc<T>) -> Result<ThreadHandle, CreateThreadError> where T: RemoteData {
        let routine: Option<TY> = Some(unsafe { std::mem::transmute(start_routine) });
        let inner = unsafe { CreateRemoteThreadEx(self.inner, null_mut(), 0, routine, **arg, 0, null_mut(), null_mut()) };
        if inner.is_null() {
            Err(CreateThreadError::Unknown(unsafe { GetLastError() }))
        } else {
            Ok(ThreadHandle::from(inner))
        }
    }

    pub fn inner(&self) -> HANDLE {
        self.inner
    }
}

impl From<HANDLE> for ProcessHandle {
    fn from(inner: HANDLE) -> Self {
        ProcessHandle { inner }
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.inner) };
    }
}

pub struct VirtualAlloc<'a, T> {
    process: &'a ProcessHandle,
    address: LPVOID,
    size: SIZE_T,
    inner: HANDLE,
    _data: PhantomData<T>,
}

impl<'a, T> VirtualAlloc<'a, T> where T: RemoteData {
    pub fn write(&self, data: &T) -> Result<usize, ProcessMemoryError> {
        unsafe { self.process.write(self.inner, data) }
    }

    pub fn read(&self, size: SIZE_T) -> Result<T, ProcessMemoryError> {
        unsafe { self.process.read(self.inner, size) }
    }

    pub fn read_into(&self, data: &mut T, size: SIZE_T) -> Result<usize, ProcessMemoryError> {
        unsafe { self.process.read_into(self.inner, size, data) }
    }
}

impl<'a, T> std::ops::Deref for VirtualAlloc<'a, T> where T: RemoteData {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> Drop for VirtualAlloc<'a, T> {
    fn drop(&mut self) {
        unsafe { VirtualFreeEx(**self.process, self.inner, 0, MEM_RELEASE) };
    }
}

impl std::ops::Deref for ProcessHandle {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ProcessEntry {
    inner: PROCESSENTRY32W
}

impl From<PROCESSENTRY32W> for ProcessEntry {
    fn from(inner: PROCESSENTRY32W) -> Self {
        ProcessEntry { inner }
    }
}

impl ProcessEntry {
    pub fn get_name(&self) -> OsString {
        OsString::from_wide_null(&self.inner.szExeFile)
    }

    pub fn get_pid(&self) -> u32 {
        self.inner.th32ProcessID
    }

    pub fn open(&self, desired_access: DWORD, inherit_handle: bool) -> Result<ProcessHandle, DWORD> {
        let handle = unsafe { OpenProcess(desired_access, inherit_handle as _, self.get_pid()) };
        if handle != NULL {
            Ok(ProcessHandle::from(handle))
        } else {
            Err(unsafe { GetLastError() })
        }
    }

    pub fn get_modules(&self, flags: DWORD) -> Option<ModuleIterator> {
        ModuleIterator::new(self.get_pid(), flags)
    }
}

pub struct ModuleEntry {
    inner: MODULEENTRY32
}

impl From<MODULEENTRY32> for ModuleEntry {
    fn from(inner: MODULEENTRY32) -> Self {
        ModuleEntry { inner }
    }
}

impl ModuleEntry {
    pub fn get_name(&self) -> &OsStr {
        unsafe { std::mem::transmute(CStr::from_ptr(self.inner.szModule[..].as_ptr() as _)) }
    }
}

pub struct ProcessIterator {
    processes_snapshot: HANDLE,
    first: bool,
    entry: PROCESSENTRY32W,
}

impl ProcessIterator {
    pub fn new(flags: DWORD) -> Option<ProcessIterator> {
        let processes_snapshot = unsafe { CreateToolhelp32Snapshot(flags, 0) };
        unsafe { SetLastError(0) };
        if processes_snapshot != INVALID_HANDLE_VALUE {
            Some(ProcessIterator {
                processes_snapshot,
                first: true,
                entry: PROCESSENTRY32W {
                    dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                    .. Default::default()
                },
            })
        } else {
            None
        }
    }
}

impl Iterator for ProcessIterator {
    type Item = ProcessEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            if unsafe { Process32FirstW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ProcessEntry::from(self.entry.clone()));
            }
        } else {
            while unsafe { Process32NextW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ProcessEntry::from(self.entry.clone()));
            }
        }
        None
    }
}

impl Drop for ProcessIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.processes_snapshot) };
    }
}

pub struct ModuleIterator {
    module_snapshot: HANDLE,
    first: bool,
    entry: MODULEENTRY32,
}

impl ModuleIterator {
    pub fn new(pid: u32, flags: DWORD) -> Option<ModuleIterator> {
        let module_snapshot = unsafe { CreateToolhelp32Snapshot(flags, pid) };
        unsafe { SetLastError(0) };
        if module_snapshot != INVALID_HANDLE_VALUE {
            Some(ModuleIterator {
                module_snapshot,
                first: true,
                entry: MODULEENTRY32 {
                    dwSize: std::mem::size_of::<MODULEENTRY32>() as u32,
                    ..Default::default()
                },
            })
        } else {
            None
        }
    }
}

impl Iterator for ModuleIterator {
    type Item = ModuleEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            if unsafe { Module32First(self.module_snapshot, &mut self.entry) } == TRUE {
                return Some(ModuleEntry::from(self.entry.clone()));
            }
        } else {
            while unsafe { Module32Next(self.module_snapshot, &mut self.entry) } == TRUE {
                return Some(ModuleEntry::from(self.entry.clone()));
            }
        }
        None
    }
}

impl Drop for ModuleIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.module_snapshot) };
    }
}

pub trait RemoteData {
    fn get_ptr(&self) -> *const c_void;
    fn get_mut_ptr(&mut self) -> *mut c_void;
    fn get_size(&self) -> SIZE_T;
    fn read(data: Vec<u8>) -> Self;
}

impl RemoteData for Vec<u8> {
    fn get_ptr(&self) -> *const c_void {
        self.as_ptr() as *const _
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut _
    }

    fn get_size(&self) -> usize {
        self.len()
    }

    fn read(data: Vec<u8>) -> Self {
        data
    }
}

impl RemoteData for CString {
    fn get_ptr(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        UnsafeCell::new(self.as_bytes_with_nul().as_ptr()).get() as *mut c_void
    }

    fn get_size(&self) -> usize {
        self.as_bytes_with_nul().len()
    }

    fn read(data: Vec<u8>) -> Self {
        CString::from(unsafe { CStr::from_bytes_with_nul_unchecked(data.as_slice()) })
    }
}

impl RemoteData for Vec<u16> {
    fn get_ptr(&self) -> *const c_void {
        self.as_ptr() as _
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as _
    }

    fn get_size(&self) -> usize {
        (self.len() + 1) * 2
    }

    fn read(data: Vec<u8>) -> Self {
        OsString::from_wide_null(unsafe { std::mem::transmute(data.as_slice()) }).to_wide_null()
    }
}

impl RemoteData for u8 {
    fn get_ptr(&self) -> *const c_void {
        unsafe { std::mem::transmute(self) }
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        unsafe { std::mem::transmute(self) }
    }

    fn get_size(&self) -> usize {
        std::mem::size_of::<u8>()
    }

    fn read(data: Vec<u8>) -> Self {
        unsafe { data.as_ptr().read() }
    }
}

impl RemoteData for u16 {
    fn get_ptr(&self) -> *const c_void {
        unsafe { std::mem::transmute(self) }
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        unsafe { std::mem::transmute(self) }
    }

    fn get_size(&self) -> usize {
        std::mem::size_of::<u16>()
    }

    fn read(data: Vec<u8>) -> Self {
        unsafe { (data.as_ptr() as *mut u16).read() }
    }
}

impl RemoteData for u32 {
    fn get_ptr(&self) -> *const c_void {
        unsafe { std::mem::transmute(self) }
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        unsafe { std::mem::transmute(self) }
    }

    fn get_size(&self) -> usize {
        std::mem::size_of::<u32>()
    }

    fn read(data: Vec<u8>) -> Self {
        unsafe { (data.as_ptr() as *mut u32).read() }
    }
}

#[derive(Debug)]
pub enum CreateThreadError {
    Unknown(u32)
}

impl std::fmt::Display for CreateThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CreateThreadError::Unknown(code) => f.write_fmt(format_args!("Unknown error: {}", code))
        }
    }
}

#[derive(Debug)]
pub enum InjectionError<'a> {
    InvalidProcessHandle,
    FileDoesntExist,
    AllocationFailed(ProcessMemoryError),
    InvalidFile,
    NoX64File,
    NoX86File,
    ImageCantReloc,
    NtdllMissing,
    LdrLoadDllMissing,
    LdrpLoadDllMissing,
    InvalidFlags,
    MissingModule(&'a CStr),
    MissingModuleProcedure(&'a CStr, &'a CStr),
    Unknown(u32),
    CantCreateThread(CreateThreadError),
    Th32Fail,
    CantGetPeb,
    AlreadyInjected,
}

impl<'a> std::fmt::Display for InjectionError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InjectionError::InvalidProcessHandle => f.write_str("Invalid process handle"),
            InjectionError::FileDoesntExist => f.write_str("File doesn't exist"),
            InjectionError::AllocationFailed(err) => f.write_fmt(format_args!("Allocation failed: {}", err)),
            InjectionError::InvalidFile => f.write_str("Invalid file"),
            InjectionError::NoX64File => f.write_str("Not an x64 file"),
            InjectionError::NoX86File => f.write_str("Not an x86 file"),
            InjectionError::ImageCantReloc => f.write_str("Image cannot be relocated"),
            InjectionError::NtdllMissing => f.write_str("ntdll is missing"),
            InjectionError::LdrLoadDllMissing => f.write_str("ldrloaddll is missing"),
            InjectionError::LdrpLoadDllMissing => f.write_str("ldrploaddll is missing"),
            InjectionError::InvalidFlags => f.write_str("Invalid flags"),
            InjectionError::MissingModule(module) => f.write_fmt(format_args!("Missing module `{}`", module.to_string_lossy())),
            InjectionError::MissingModuleProcedure(module, proc) => f.write_fmt(format_args!("Missing procedure `{}` in module `{}`", module.to_string_lossy(), proc.to_string_lossy())),
            InjectionError::Unknown(code) => f.write_fmt(format_args!("Unknown error: {}", code)),
            InjectionError::CantCreateThread(err) => f.write_fmt(format_args!("Cannot create thread: {}", err)),
            InjectionError::Th32Fail => f.write_str("TH32 failed"),
            InjectionError::CantGetPeb => f.write_str("Cannot get peb."),
            InjectionError::AlreadyInjected => f.write_str("Already injected"),
        }
    }
}

#[derive(Debug)]
pub enum ProcessMemoryError {
    AllocationFailed(u32),
    WriteFailed(u32),
    WriteBytesMismatch(usize, usize),
    ReadFailed(u32),
    ReadBytesMismatch(usize, usize),
}

impl Error for ProcessMemoryError {}

impl std::fmt::Display for ProcessMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProcessMemoryError::AllocationFailed(code) => f.write_fmt(format_args!("Virtual allocation failed: {}", code)),
            ProcessMemoryError::WriteFailed(code) => f.write_fmt(format_args!("Virtual write failed: {}", code)),
            ProcessMemoryError::WriteBytesMismatch(expected, written) => f.write_fmt(format_args!("Virtual write bytes mismatch: {} expected but {} written", expected, written)),
            ProcessMemoryError::ReadFailed(code) => f.write_fmt(format_args!("Virtual read failed: {}", code)),
            ProcessMemoryError::ReadBytesMismatch(expected, read) => f.write_fmt(format_args!("Virtual read bytes mismatch: {} expected but {} read", expected, read))
        }
    }
}