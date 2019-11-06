use std::process::{Command, Stdio};
use std::time::Duration;
use winapi::um::winnt::{PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use winapi::um::tlhelp32::TH32CS_SNAPPROCESS;
use evolutionmp::registry::Registry;
use evolutionmp::win::ps::{ProcessIterator, get_process};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::psapi::LIST_MODULES_ALL;
use evolutionmp::launcher_dir;

fn main() {
    let gta_exe = "GTA5.exe";
    let gta_launcher_exe = "GTAVLauncher.exe";
    let client_dll = launcher_dir().join("evolutionmp.dll");
    let registry = Registry::read().expect("Unable to find GTA5 registry entry!");
    let install_dir = registry.get_install_path();

    if registry.is_retail_key() {
        println!("Found retail version of GTA5");

        if !install_dir.join(gta_exe).exists() {
            panic!("{} not found!", gta_exe);
        }
        let mut process = Command::new(install_dir.join(gta_launcher_exe))
            .stderr(Stdio::inherit())
            .stdout(Stdio::inherit())
            .spawn().expect(&format!("Error starting {}", gta_launcher_exe));

        while !is_process_alive(gta_exe) {
            if registry.is_retail_key() {
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        let access = PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE;
        let proc = get_process(gta_exe, access)
            .expect(&format!("{} not found", gta_exe));
        println!("Found GTA5.exe process with pid: {} ({:p})", proc.get_pid(), proc.inner());
        loop {
            match proc.inject_library(&client_dll) {
                Ok(exit_code) => {
                    match exit_code {
                        0 | 1 => {
                            let error_code = unsafe { GetLastError() } as i32;
                            let error = std::io::Error::from_raw_os_error(error_code);
                            eprintln!("Module injection failed: {}", error);
                            process.kill().expect("Process kill failed");
                            return;
                        }
                        module => {
                            for m in proc.get_modules(LIST_MODULES_ALL) {
                                if m.get_instance() as u64 & 0xFFFFFFFF == module as u64 {
                                    println!("Injection successful at: {:p} ({})", m.get_instance(), m.get_name());
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
    } else if registry.is_steam_key() {
        println!("Found steam version of GTA5");
    }
}

fn is_process_alive<S>(file_name: S) -> bool where S: AsRef<str> {
    ProcessIterator::new(TH32CS_SNAPPROCESS).unwrap().any(move |p|&p.get_name().to_string_lossy() == file_name.as_ref())
}



