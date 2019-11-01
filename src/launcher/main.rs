use std::process::{Command, Stdio};
use winapi::um::winnt::{PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE};
use evolutionmp::win::ps::{ProcessIterator, get_process};
use evolutionmp::registry::Registry;
use winapi::um::tlhelp32::TH32CS_SNAPPROCESS;
use std::time::Duration;

fn main() {
    let gta_exe = "GTA5.exe";
    let gta_launcher_exe = "GTAVLauncher.exe";
    let client_dll = std::env::current_dir().unwrap().join("target\\debug\\evolutionmp.dll");
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
        let proc = get_process(gta_exe, access).expect(&format!("{} not found", gta_exe));
        println!("Found GTA5.exe process with pid: {}", proc.get_pid());
        loop {
            match proc.inject_library(&client_dll) {
                Ok(exit_code) => {
                    match exit_code {
                        0 => {}
                        1 => {
                            eprintln!("Module injection failed");
                            return;
                        }
                        other => {
                            println!("Injected module at 0x{:X}", other);
                            break;
                        }
                    }
                },
                Err(err) => {
                    eprintln!("Injection error: {:?}", err);
                }
            }
        };

        println!("Launcher process exited with code: {}", process.wait().unwrap());
    } else if registry.is_steam_key() {
        println!("Found steam version of GTA5");
    }
}

fn is_process_alive<S>(file_name: S) -> bool where S: AsRef<str> {
    ProcessIterator::new(TH32CS_SNAPPROCESS).unwrap().any(move |p|&p.get_name().to_string_lossy() == file_name.as_ref())
}



