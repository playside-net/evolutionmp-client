use std::cell::UnsafeCell;
use std::error::Error;
use std::ffi::{CStr, CString, OsString, OsStr};
use std::marker::PhantomData;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use winapi::ctypes::c_void;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{DWORD, FALSE, FARPROC, HINSTANCE, HMODULE, LPVOID, MAX_PATH, TRUE};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress, GetModuleHandleA};
use winapi::um::memoryapi::{ReadProcessMemory, VirtualAllocEx, VirtualFreeEx, WriteProcessMemory};
use winapi::um::processthreadsapi::{CreateRemoteThreadEx, CreateThread, GetCurrentProcess, GetProcessId, OpenProcess, OpenProcessToken};
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleFileNameExW};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use winapi::um::winbase::{INFINITE, LookupPrivilegeValueW, THREAD_PRIORITY_HIGHEST};
use winapi::um::winnt::{LUID_AND_ATTRIBUTES, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY};

use crate::win::thread::ThreadHandle;
use wio::wide::{FromWide, ToWide};

pub fn get_current_process() -> ProcessHandle {
    ProcessHandle::from(unsafe { GetCurrentProcess() })
}

pub fn get_process<S>(file_name: S, desired_access: DWORD) -> Option<ProcessHandle> where S: AsRef<str> {
    for process in ProcessIterator::new(TH32CS_SNAPPROCESS)? {
        if &process.get_name().to_string_lossy() == file_name.as_ref() {
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

pub struct ModuleEntry {
    instance: HMODULE,
    name: OsString,
}

impl ModuleEntry {
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

    pub fn get_modules(&self, flags: DWORD) -> Vec<ModuleEntry> {
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
                        result.push(ModuleEntry {
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

    pub fn virtual_alloc<T>(&self, value: &T, address: LPVOID, allocation_type: DWORD, protect: DWORD) -> Result<VirtualAlloc<T>, VirtualAllocError> where T: VirtualData {
        let size = value.get_size();
        let alloc = unsafe { self.virtual_alloc_uninit::<T>(address, size, allocation_type, protect) }?;
        alloc.write(value)?;
        Ok(alloc)
    }

    pub unsafe fn virtual_alloc_uninit<T>(&self, address: LPVOID, size: SIZE_T, allocation_type: DWORD, protect: DWORD) -> Result<VirtualAlloc<T>, VirtualAllocError> where T: VirtualData {
        let inner = VirtualAllocEx(self.inner, address, size, allocation_type, protect);
        if inner.is_null() {
            Err(VirtualAllocError::AllocationFailed(GetLastError()))
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

    pub fn create_thread<T>(&self, start_routine: FARPROC, arg: &VirtualAlloc<T>) -> Result<ThreadHandle, CreateThreadError> where T: VirtualData {
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

impl<'a, T> VirtualAlloc<'a, T> where T: VirtualData {
    pub fn write(&self, data: &T) -> Result<usize, VirtualAllocError> {
        let size = data.get_size();
        let mut bytes_written = 0usize;
        if unsafe { WriteProcessMemory(**self.process, self.inner, data.get_ptr(), size, &mut bytes_written) } == TRUE {
            if size != bytes_written {
                Err(VirtualAllocError::WriteBytesMismatch(size, bytes_written))
            } else {
                Ok(bytes_written)
            }
        } else {
            Err(VirtualAllocError::WriteFailed(unsafe { GetLastError() }))
        }
    }

    pub fn read(&self, size: SIZE_T) -> Result<T, VirtualAllocError> {
        let mut bytes_read = 0usize;
        let mut data = vec![0; size];
        if unsafe { ReadProcessMemory(**self.process, self.inner, data.as_mut_ptr() as *mut _, size, &mut bytes_read) } == TRUE {
            if size != bytes_read {
                Err(VirtualAllocError::ReadBytesMismatch(size, bytes_read))
            } else {
                Ok(T::read(data))
            }
        } else {
            Err(VirtualAllocError::ReadFailed(unsafe { GetLastError() }))
        }
    }

    pub fn read_into(&self, data: &mut T, size: SIZE_T) -> Result<usize, VirtualAllocError> {
        let mut bytes_read = 0usize;
        if unsafe { ReadProcessMemory(**self.process, self.inner, data.get_mut_ptr(), size, &mut bytes_read) } == TRUE {
            if size != bytes_read {
                Err(VirtualAllocError::ReadBytesMismatch(size, bytes_read))
            } else {
                Ok(bytes_read)
            }
        } else {
            Err(VirtualAllocError::ReadFailed(unsafe { GetLastError() }))
        }
    }
}

impl<'a, T> std::ops::Deref for VirtualAlloc<'a, T> where T: VirtualData {
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
        OsString::from_wide_null(&self.inner.szExeFile[..])
    }

    pub fn open(&self, desired_access: DWORD, inherit_handle: bool) -> Result<ProcessHandle, DWORD> {
        let handle = unsafe { OpenProcess(desired_access, if inherit_handle { TRUE } else { FALSE }, self.inner.th32ProcessID) };
        if handle != NULL {
            Ok(ProcessHandle::from(handle))
        } else {
            Err(unsafe { GetLastError() })
        }
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
                    ..Default::default()
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

pub trait VirtualData {
    fn get_ptr(&self) -> *const c_void;
    fn get_mut_ptr(&mut self) -> *mut c_void;
    fn get_size(&self) -> SIZE_T;
    fn read(data: Vec<u8>) -> Self;
}

impl VirtualData for Vec<u8> {
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

impl VirtualData for CString {
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

impl VirtualData for Vec<u16> {
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

impl VirtualData for u8 {
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

impl VirtualData for u16 {
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

impl VirtualData for u32 {
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
    AllocationFailed(VirtualAllocError),
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
pub enum VirtualAllocError {
    AllocationFailed(u32),
    WriteFailed(u32),
    WriteBytesMismatch(usize, usize),
    ReadFailed(u32),
    ReadBytesMismatch(usize, usize),
}

impl Error for VirtualAllocError {}

impl std::fmt::Display for VirtualAllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VirtualAllocError::AllocationFailed(code) => f.write_fmt(format_args!("Virtual allocation failed: {}", code)),
            VirtualAllocError::WriteFailed(code) => f.write_fmt(format_args!("Virtual write failed: {}", code)),
            VirtualAllocError::WriteBytesMismatch(expected, written) => f.write_fmt(format_args!("Virtual write bytes mismatch: {} expected but {} written", expected, written)),
            VirtualAllocError::ReadFailed(code) => f.write_fmt(format_args!("Virtual read failed: {}", code)),
            VirtualAllocError::ReadBytesMismatch(expected, read) => f.write_fmt(format_args!("Virtual read bytes mismatch: {} expected but {} read", expected, read))
        }
    }
}

/*
void killProcessByName(const char *filename)
{
HANDLE hSnapShot = CreateToolhelp32Snapshot(TH32CS_SNAPALL, NULL);
PROCESSENTRY32 pEntry;
pEntry.dwSize = sizeof(pEntry);
BOOL hRes = Process32First(hSnapShot, &pEntry);
while (hRes)
{
if (strcmp(pEntry.szExeFile, filename) == 0)
{
HANDLE hProcess = OpenProcess(PROCESS_TERMINATE, 0,
(DWORD)pEntry.th32ProcessID);
if (hProcess != NULL)
{
TerminateProcess(hProcess, 9);
CloseHandle(hProcess);
}
}
hRes = Process32Next(hSnapShot, &pEntry);
}
CloseHandle(hSnapShot);
}

bool Is64BitProcess(HANDLE hProc)
{
bool Is64BitWin = false;
BOOL Out = 0;
IsWow64Process(GetCurrentProcess(), &Out);
if (Out)
Is64BitWin = true;

if (!IsWow64Process(hProc, &Out))
return false;

if (Is64BitWin && !Out)
return true;

return false;
}*/
