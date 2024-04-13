use std::{fs, path::Path};

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
        .arg(
            Arg::with_name("boot_bin")
                .short('b')
                .long("boot binary")
                .value_name("BOOT")
                .help("Sets the Boot ROM file to read")
                .default_value(Path::new("assets").join("dmg_boot.bin").to_str().unwrap()),
        )
        .arg(
            Arg::with_name("graphics")
                .long("graphics")
                .help("Enables graphics")
                .takes_value(false)
                .required(false)
                .default_value("true"), // Set default value to true
        )
        .get_matches();

    let boot_bin = matches.value_of("boot_bin").unwrap();
    info!("Loading boot bin {}", boot_bin);
    let contents = fs::read(boot_bin);
    let boot_bin = match contents {
        Ok(fs) => fs,
        Err(e) => {
            debug!("Unable to read file {} due to {}", boot_bin, e.to_string());
            return Err(String::from("Unable to read file"));
        }
    };

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

    let graphics_enabled = matches.is_present("graphics");

    let mut gameboy = GameBoy::new(graphics_enabled);
    gameboy.load_boot(boot_bin);
    gameboy.load_rom(rom_file);
    gameboy.run();

    Ok(())
}
