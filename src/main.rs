use std::fs;

use clap::{App, Arg};
use gb_rs::gb::GameBoy;
use log::{debug, info};

fn main() -> Result<(), String> {
    env_logger::init();

    let matches = App::new("gb-rs")
        .version("1.0")
        .about("A simple program to read a ROM file and emulate it")
        .arg(
            Arg::with_name("rom_file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Sets the ROM file to read")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let rom_file = matches.value_of("rom_file").unwrap();
    info!("Running rom file {}", rom_file);
    let contents = fs::read(rom_file);
    let rom_file = match contents {
        Ok(fs) => fs,
        Err(e) => {
            debug!("Unable to read file {} due to {}", rom_file, e.to_string());
            return Err(String::from("Unable to read file"));
        }
    };

    let mut gameboy = GameBoy::new();
    // gameboy.load_rom(rom_file);
    gameboy.run();

    Ok(())
}
