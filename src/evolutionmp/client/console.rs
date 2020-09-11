use winapi::um::consoleapi::AllocConsole;

pub(crate) fn attach() {
    unsafe { AllocConsole() };
    ansi_term::enable_ansi_support().expect("enabling console ansi support failed");
}