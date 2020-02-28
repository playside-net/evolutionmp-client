use std::process::{Command, Stdio};
use std::time::Duration;
use winapi::um::winnt::{PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use winapi::um::tlhelp32::TH32CS_SNAPPROCESS;
use evolutionmp::registry::Registry;
use evolutionmp::win::ps::{ProcessIterator, get_process, ModuleEntry, ProcessHandle};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::psapi::LIST_MODULES_ALL;
use evolutionmp::launcher_dir;
use std::path::Path;

fn main() {
    let gta_exe = "GTA5.exe";
    let gta_launcher_exe = "GTAVLauncher.exe";
    let registry = Registry::read().expect("Unable to find GTA5 registry entry!");
    let install_dir = registry.get_install_path();

    if !install_dir.join(gta_exe).exists() {
        panic!("{} not found!", gta_exe);
    }

    if registry.is_retail_key() {
        println!("Found retail version of GTA5");
        start(registry, &install_dir.join(gta_launcher_exe), gta_exe);
    } else if registry.is_steam_key() {
        println!("Found steam version of GTA5");
        start(registry, "steam://rungameid/271590", gta_exe);
    }
}

fn start<P>(registry: Registry, launch_path: P, gta_exe: &str) where P: AsRef<Path> {
    let mut process = Command::new(launch_path.as_ref())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn().expect(&format!("Error starting {:?}", launch_path.as_ref()));

    while !is_process_alive(gta_exe) {
        /*if registry.is_retail_key() {
        }*/
        std::thread::sleep(Duration::from_millis(100));
    }

    let client_dll = launcher_dir().join("evolutionmp.dll");

    let access = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE;
    let proc = get_process(gta_exe, access)
        .expect(&format!("{} not found", gta_exe));
    println!("Found GTA5.exe process with pid: {} ({:p})", proc.get_pid(), proc.inner());
    loop {
        match proc.inject_library(&client_dll) {
            Ok(exit_code) => {
                match exit_code {
                    0 | 1 => {
                        let error = std::io::Error::last_os_error();
                        eprintln!("Module injection failed: {}", error);
                        return;
                    }
                    module => {
                        for m in proc.get_modules(LIST_MODULES_ALL) {
                            if m.get_instance() as u64 & 0xFFFFFFFF == module as u64 {
                                initialize(proc, m, &client_dll);
                                break;
                            }
                        }
                        break;
                    }
                }
            },
            Err(err) => {
                eprintln!("Injection error: {:?}", err);
                return;
            }
        }
    };

    println!("Launcher process exited with code: {}", process.wait().unwrap().code().unwrap());
}

fn is_process_alive<S>(file_name: S) -> bool where S: AsRef<str> {
    ProcessIterator::new(TH32CS_SNAPPROCESS).unwrap().any(move |p|&p.get_name().to_string_lossy() == file_name.as_ref())
}

fn initialize(proc: ProcessHandle, m: ModuleEntry, client_dll: &Path) {
    /*//println!("Looking for set_io in: {:p} ({})", m.get_instance(), m.get_name());
    let p = get_procedure_address(&client_dll.to_string_lossy(), "set_io").expect("no set_io");
    //let p = m.get_procedure_address("set_io").expect("missing set_io procedure in target module");
    //println!("Found set_io procedure: {:?}", p);

    let alloc = proc.virtual_alloc(&0u32, null_mut(), MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE).unwrap();

    match proc.create_thread(p, &alloc) {
        Ok(thread) => {
            thread.wait_for_single_object(INFINITE);
            println!("set_io result: {}", thread.get_exit_code());
        },
        Err(err) => {
            println!("set_io failed: {:?}", err);
        }
    }*/
}



