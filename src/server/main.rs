use evolutionmp::setup_logger;

pub fn main() {
    let debug = std::env::args().any(|a| a == "--debug");
    setup_logger("server", debug);



}