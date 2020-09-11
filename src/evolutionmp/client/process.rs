/*
use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt};
use winapi::um::{handleapi, memoryapi, processthreadsapi, tlhelp32, winnt};
use winapi::shared::basetsd::SIZE_T;

pub struct GameProcess {
    handle: winnt::HANDLE,
    pid: u32,
}

pub struct Module {
    base: u32,
    size: u32,
}

impl GameProcess {
    pub fn current_process() -> Self {
        Self::new(unsafe { processthreadsapi::GetCurrentProcess() })
    }

    pub fn new(handle: winnt::HANDLE) -> Self {
        let pid = unsafe { processthreadsapi::GetProcessId(handle) };
        GameProcess { handle, pid }
    }

    pub fn read_memory(&self, address: u32) -> Option<u32> {
        let mut read = unsafe { mem::uninitialized() };
        let mut amount_read: SIZE_T = 0;

        if unsafe {
            memoryapi::ReadProcessMemory(
                self.handle,
                address as *const _,
                &mut read as *mut _ as *mut _,
                mem::size_of::<u32>() as _,
                &mut amount_read as *mut _,
            )
        } != (true as _) || amount_read == 0 {
            mem::forget(read);
            return None;
        }

        Some(read)
    }

    pub fn get_module(&self, module_name: &str) -> Option<Module> {
        let module =
            unsafe { tlhelp32::CreateToolhelp32Snapshot(tlhelp32::TH32CS_SNAPMODULE, self.pid) };
        if module == handleapi::INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry: tlhelp32::MODULEENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<tlhelp32::MODULEENTRY32W>() as _;

        while unsafe { tlhelp32::Module32NextW(module, &mut entry) } != 0 {
            let name = OsString::from_wide(&entry.szModule[..]).into_string();
            let name = match name {
                Err(e) => {
                    eprintln!("Couldn't convert into String: {:?}", e);
                    continue;
                }
                Ok(s) => s,
            };

            if name.contains(module_name) {
                unsafe { handleapi::CloseHandle(module) };

                println!(
                    "Base address of {}: 0x{:016X} @ size of 0x{:016X}",
                    module_name, entry.modBaseAddr as u32, entry.modBaseSize
                );

                return Ok(Module {
                    base: entry.modBaseAddr as _,
                    size: entry.modBaseSize as _,
                });
            }
        }

        None
    }

    pub fn get_pid(&self) -> u32 {
        self.pid
    }
}

impl Module {
    fn fix_offset(&self, offset: usize) -> usize {
        (self.base as usize) + offset
    }

    pub unsafe fn read<T>(&self, offset: usize) -> &T {
        &*(self.fix_offset(offset) as *const T)
    }

    pub unsafe fn read_mut<T>(&self, offset: usize) -> &mut T {
        &mut *(self.fix_offset(offset) as *mut T)
    }

    pub unsafe fn write<T>(&mut self, offset: usize, value: T) {
        *(self.fix_offset(offset) as *mut T) = value;
    }
}*/
