use winapi::um::winnt::HANDLE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetThreadId, SuspendThread, ResumeThread, GetThreadContext, OpenThread, SetThreadContext, GetExitCodeThread};
use winapi::shared::minwindef::{DWORD, TRUE, FALSE};
use winapi::um::winnt::CONTEXT;
use winapi::um::tlhelp32::{Thread32First, Thread32Next, CreateToolhelp32Snapshot, THREADENTRY32};
use winapi::shared::ntdef::NULL;
use winapi::um::synchapi::WaitForSingleObject;

pub struct ThreadHandle {
    inner: HANDLE
}

impl ThreadHandle {
    pub fn get_id(&self) -> u32 {
        unsafe { GetThreadId(self.inner) }
    }

    pub fn get_context(&self, flags: DWORD) -> Option<CONTEXT> {
        let mut context = CONTEXT {
            ContextFlags: flags,
            .. Default::default()
        };
        if unsafe { GetThreadContext(self.inner, &mut context) } == TRUE {
            Some(context)
        } else {
            None
        }
    }

    pub fn set_context(&self, context: CONTEXT) -> bool {
        if unsafe { SetThreadContext(self.inner, &context) } == TRUE {
            true
        } else {
            false
        }
    }

    pub fn suspend(&self) -> Option<DWORD> {
        let result = unsafe { SuspendThread(self.inner) };
        if result == -1i32 as DWORD { None } else { Some(result) }
    }

    pub fn resume(&self) -> Option<DWORD> {
        let result = unsafe { ResumeThread(self.inner) };
        if result == -1i32 as DWORD { None } else { Some(result) }
    }

    pub fn wait_for_single_object(&self, timeout: DWORD) -> DWORD {
        unsafe { WaitForSingleObject(self.inner, timeout) }
    }

    pub fn get_exit_code(&self) -> u32 {
        let mut exit_code = 0;
        unsafe { GetExitCodeThread(self.inner, &mut exit_code) };
        exit_code
    }
}

impl From<HANDLE> for ThreadHandle {
    fn from(inner: HANDLE) -> Self {
        ThreadHandle { inner }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.inner) };
    }
}

impl std::ops::Deref for ThreadHandle {
    type Target = HANDLE;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ThreadEntry {
    inner: THREADENTRY32
}

impl ThreadEntry {
    pub fn get_owner_pid(&self) -> u32 {
        self.inner.th32OwnerProcessID
    }

    pub fn get_id(&self) -> u32 {
        self.inner.th32ThreadID
    }

    pub fn open(&self, desired_access: DWORD, inherit_handle: bool) -> Option<ThreadHandle> {
        let handle = unsafe { OpenThread(desired_access, if inherit_handle { TRUE } else { FALSE }, self.get_id()) };
        if handle != NULL {
            Some(ThreadHandle::from(handle))
        } else {
            None
        }
    }
}

impl From<THREADENTRY32> for ThreadEntry {
    fn from(inner: THREADENTRY32) -> Self {
        ThreadEntry { inner }
    }
}

pub struct ThreadIterator {
    tool_help_snapshot: HANDLE,
    first: bool,
    entry: THREADENTRY32
}

impl ThreadIterator {
    pub fn new(flags: DWORD, pid: u32) -> Option<ThreadIterator> {
        let tool_help_snapshot = unsafe { CreateToolhelp32Snapshot(flags, pid) };
        if tool_help_snapshot != NULL {
            Some(ThreadIterator {
                tool_help_snapshot,
                first: true,
                entry: THREADENTRY32 {
                    dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                    .. Default::default()
                }
            })
        } else {
            None
        }
    }
}

impl Iterator for ThreadIterator {
    type Item = ThreadEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            if unsafe { Thread32First(self.tool_help_snapshot, &mut self.entry) } == TRUE {
                return Some(ThreadEntry::from(self.entry.clone()))
            }
        } else {
            while unsafe { Thread32Next(self.tool_help_snapshot, &mut self.entry) } == TRUE {
                return Some(ThreadEntry::from(self.entry.clone()))
            }
        }
        None
    }
}

impl Drop for ThreadIterator {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.tool_help_snapshot) };
    }
}