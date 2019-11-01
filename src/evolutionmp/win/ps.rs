use crate::win::thread::ThreadHandle;
use std::ptr::{null, null_mut};
use std::error::Error;
use std::path::{Display, Path};
use std::os::windows::ffi::OsStringExt;
use std::os::windows::ffi::OsStrExt;
use std::ffi::{OsString, OsStr, CString, CStr};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::io::Write;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, TH32CS_SNAPPROCESS, Process32FirstW, PROCESSENTRY32W, THREADENTRY32, MODULEENTRY32W, Process32NextW, TH32CS_SNAPALL, Thread32First, Thread32Next, Module32FirstW, Module32NextW};
use winapi::um::handleapi::{INVALID_HANDLE_VALUE, CloseHandle};
use winapi::um::processthreadsapi::{OpenProcess, OpenProcessToken, GetCurrentProcess, GetProcessId, CreateThread, CreateRemoteThread, CreateRemoteThreadEx};
use winapi::um::winnt::{PROCESS_ALL_ACCESS, TOKEN_QUERY, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, LUID_AND_ATTRIBUTES, SE_PRIVILEGE_ENABLED, SE_PRIVILEGE_REMOVED, MEM_RELEASE, MEM_RESERVE, MEM_COMMIT, PAGE_READWRITE, PAGE_EXECUTE_READWRITE, LPCWSTR};
use winapi::um::winbase::{LookupPrivilegeValueW, THREAD_PRIORITY_HIGHEST, INFINITE};
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::minwinbase::LPTHREAD_START_ROUTINE;
use winapi::um::memoryapi::{VirtualFreeEx, VirtualAllocEx, WriteProcessMemory, ReadProcessMemory};
use winapi::um::errhandlingapi::{GetLastError, SetLastError};
use winapi::um::libloaderapi::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use winapi::um::winuser::{MessageBeep, MB_ICONEXCLAMATION, MB_ICONSTOP, MessageBoxW, MB_ICONHAND, MB_OKCANCEL, GetParent};
use winapi::shared::ntdef::{HANDLE, NULL};
use winapi::shared::minwindef::{HMODULE, TRUE, MAX_PATH, DWORD, FALSE, LPVOID, LPCVOID, __some_function, FARPROC, HINSTANCE};
use winapi::shared::basetsd::SIZE_T;
use winapi::ctypes::c_void;
use widestring::{WideCStr, WideCString};

pub fn get_current_process() -> ProcessHandle {
    ProcessHandle::from(unsafe { GetCurrentProcess() })
}

pub fn get_process<S>(file_name: S, desired_access: DWORD) -> Option<ProcessHandle> where S: AsRef<str> {
    for process in ProcessIterator::new(TH32CS_SNAPPROCESS)? {
        if &process.get_name().to_string_lossy() == file_name.as_ref() {
            return Some(process.open(desired_access, false).expect(&format!("Error opening process `{}`", file_name.as_ref())))
        }
    }
    None
}

unsafe fn set_privilege(privilege: &str, value: bool) -> bool {
    let privilege = WideCString::from_str(privilege).unwrap();
    let mut token: HANDLE = null_mut();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_ADJUST_PRIVILEGES, &mut token) == TRUE {
        let mut token_privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Attributes: if value { SE_PRIVILEGE_ENABLED } else { 0 },
                .. Default::default()
            }]
        };
        if LookupPrivilegeValueW(null_mut(), privilege.as_ptr(), &mut token_privileges.Privileges[0].Luid) == TRUE {
            if AdjustTokenPrivileges(token, FALSE, &mut token_privileges, std::mem::size_of::<TOKEN_PRIVILEGES>() as u32, null_mut(), null_mut()) == TRUE {
                CloseHandle(token);
                return true;
            }
        }
    } else {
        panic!("Process token opening failed: {}", GetLastError());
    }
    CloseHandle(token);
    false
}

fn get_procedure_address(module: &str, procedure: &str) -> Result<FARPROC, InjectionError> {
    let module_str = WideCString::from_str(module).unwrap();
    let procedure_str = CString::new(procedure).unwrap();
    let module: HMODULE = unsafe {
        GetModuleHandleW(module_str.as_ptr())
    };

    if module.is_null() {
        return Err(InjectionError::MissingModule(module_str.to_string_lossy()));
    }

    let procedure = unsafe {
        GetProcAddress(module, procedure_str.as_ptr() as _)
    };

    if procedure.is_null() {
        Err(InjectionError::MissingModuleProcedure(module_str.to_string_lossy(), String::from(procedure_str.to_str().unwrap())))
    } else {
        Ok(procedure)
    }
}

type ElevatedThread = unsafe extern "system" fn(LPVOID) -> DWORD;

pub unsafe fn create_elevated_thread(thread: ElevatedThread) -> bool {
    CloseHandle(CreateThread(null_mut(), 0, Some(thread), null_mut(), THREAD_PRIORITY_HIGHEST, null_mut())) == TRUE
}

pub unsafe fn kill_process_by_name(file_name: String) {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPALL, 0);

}

pub struct ModuleHandle {
    inner: HINSTANCE
}

impl ModuleHandle {
    pub fn get_current() -> ModuleHandle {
        let inner = unsafe { GetModuleHandleW(null_mut()) };
        ModuleHandle { inner }
    }

    pub fn find_by_name(name: String) -> Option<ModuleHandle> {
        let name = WideCString::from_str(&name).unwrap();
        let inner = unsafe { GetModuleHandleW(name.as_ptr()) };
        if inner.is_null() {
            None
        } else {
            Some(ModuleHandle { inner })
        }
    }
}

impl std::ops::Deref for ModuleHandle {
    type Target = HMODULE;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ModuleEntry {
    inner: MODULEENTRY32W
}

impl From<MODULEENTRY32W> for ModuleEntry {
    fn from(inner: MODULEENTRY32W) -> Self {
        Self { inner }
    }
}

impl ModuleEntry {
    pub fn get_name(&self) -> WideCString {
        WideCString::from_vec_with_nul(self.inner.szModule.iter().cloned().collect::<Vec<_>>()).unwrap()
    }

    pub fn get_base_address(&self) -> *mut u8 {
        self.inner.modBaseAddr
    }

    pub fn get_base_size(&self) -> usize {
        self.inner.modBaseSize as usize
    }
}

pub struct ModuleIterator {
    processes_snapshot: HANDLE,
    first: bool,
    entry: MODULEENTRY32W
}

impl ModuleIterator {
    pub fn new(flags: DWORD) -> Option<ModuleIterator> {
        let processes_snapshot = unsafe { CreateToolhelp32Snapshot(flags, 0) };
        unsafe { SetLastError(0) };
        if processes_snapshot != INVALID_HANDLE_VALUE {
            Some(ModuleIterator {
                processes_snapshot,
                first: true,
                entry: MODULEENTRY32W {
                    dwSize: std::mem::size_of::<MODULEENTRY32W>() as u32,
                    .. Default::default()
                }
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
            if unsafe { Module32FirstW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ModuleEntry::from(self.entry.clone()))
            }
        } else {
            while unsafe { Module32NextW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ModuleEntry::from(self.entry.clone()))
            }
        }
        None
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

    pub fn find_module<S>(&self, file_name: S, flags: DWORD) -> Option<ModuleEntry> where S: AsRef<str> {
        for m in ModuleIterator::new(flags)? {
            if &m.get_name().to_string_lossy() == file_name.as_ref() {
                return Some(m);
            }
        }
        None
    }

    pub fn inject_library(&self, dll_path: &Path) -> Result<u32, InjectionError> {
        if !dll_path.exists() {
            return Err(InjectionError::FileDoesntExist);
        }

        let load_library_address = get_procedure_address("Kernel32.dll", "LoadLibraryW")?;
        let dll_path = WideCString::from_str(dll_path).unwrap();
        let alloc = self.virtual_alloc(&dll_path, null_mut(), MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE).unwrap();

        match self.create_thread(load_library_address, &alloc) {
            Ok(thread) => {
                thread.wait_for_single_object(INFINITE);
                Ok(thread.get_exit_code())
            },
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
                _data: PhantomData
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
    _data: PhantomData<T>
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
    pub fn get_name(&self) -> WideCString {
        WideCString::from_vec_with_nul(self.inner.szExeFile.iter().cloned().collect::<Vec<_>>()).unwrap()
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
    entry: PROCESSENTRY32W
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
                }
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
                return Some(ProcessEntry::from(self.entry.clone()))
            }
        } else {
            while unsafe { Process32NextW(self.processes_snapshot, &mut self.entry) } == TRUE {
                return Some(ProcessEntry::from(self.entry.clone()))
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

impl VirtualData for WideCString {
    fn get_ptr(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }

    fn get_mut_ptr(&mut self) -> *mut c_void {
        UnsafeCell::new(self.as_slice_with_nul().as_ptr()).get() as *mut c_void
    }

    fn get_size(&self) -> usize {
        (self.len() + 1) * 2
    }

    fn read(data: Vec<u8>) -> Self {
        WideCString::from(unsafe { WideCStr::from_slice_with_nul_unchecked(std::mem::transmute(data.as_slice())) })
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
            CreateThreadError::Unknown(code) => f.pad(&format!("Unknown error: {}", code)),
            _ => unreachable!()
        }
    }
}

#[derive(Debug)]
pub enum InjectionError {
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
    MissingModule(String),
    MissingModuleProcedure(String, String),
    Unknown(u32),
    CantCreateThread(CreateThreadError),
    Th32Fail,
    CantGetPeb,
    AlreadyInjected
}

impl std::fmt::Display for InjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InjectionError::InvalidProcessHandle => f.pad("Invalid process handle"),
            InjectionError::FileDoesntExist => f.pad("File doesn't exist"),
            InjectionError::AllocationFailed(err) => f.pad(&format!("Allocation failed: {}", err)),
            InjectionError::InvalidFile => f.pad("Invalid file"),
            InjectionError::NoX64File => f.pad("Not an x64 file"),
            InjectionError::NoX86File => f.pad("Not an x86 file"),
            InjectionError::ImageCantReloc => f.pad("Image cannot be relocated"),
            InjectionError::NtdllMissing => f.pad("ntdll is missing"),
            InjectionError::LdrLoadDllMissing => f.pad("ldrloaddll is missing"),
            InjectionError::LdrpLoadDllMissing => f.pad("ldrploaddll is missing"),
            InjectionError::InvalidFlags => f.pad("Invalid flags"),
            InjectionError::MissingModule(module) => f.pad(&format!("Missing module `{}`", module)),
            InjectionError::MissingModuleProcedure(module, proc) => f.pad(&format!("Missing procedure `{}` in module `{}`", module, proc)),
            InjectionError::Unknown(code) => f.pad(&format!("Unknown error: {}", code)),
            InjectionError::CantCreateThread(err) => f.pad(&format!("Cannot create thread: {}", err)),
            InjectionError::Th32Fail => f.pad("TH32 failed"),
            InjectionError::CantGetPeb => f.pad("Cannot get peb."),
            InjectionError::AlreadyInjected => f.pad("Already injected"),
        }
    }
}

#[derive(Debug)]
pub enum VirtualAllocError {
    AllocationFailed(u32),
    WriteFailed(u32),
    WriteBytesMismatch(usize, usize),
    ReadFailed(u32),
    ReadBytesMismatch(usize, usize)
}

impl Error for VirtualAllocError {}

impl std::fmt::Display for VirtualAllocError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VirtualAllocError::AllocationFailed(code) => f.pad(&format!("Virtual allocation failed: {}", code)),
            VirtualAllocError::WriteFailed(code) => f.pad(&format!("Virtual write failed: {}", code)),
            VirtualAllocError::WriteBytesMismatch(expected, written) => f.pad(&format!("Virtual write bytes mismatch: {} expected but {} written", expected, written)),
            VirtualAllocError::ReadFailed(code) => f.pad(&format!("Virtual read failed: {}", code)),
            VirtualAllocError::ReadBytesMismatch(expected, read) => f.pad(&format!("Virtual read bytes mismatch: {} expected but {} read", expected, read))
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
