use winapi::um::consoleapi::AllocConsole;

pub(crate) unsafe fn attach() {
    AllocConsole();
}