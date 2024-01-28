use crate::cli::{Cli, Commands};
use crate::commands::*;
use crate::FileHandler;
use crate::option_rom::OptionRom;

use validate::validate;
use write_rom::write_rom;

pub fn process(args: Cli) -> Result<String, String> {
    if ! args.source_path.exists() {
        return Err("The specified source file does not exist".into());
    }

    let bytes = match FileHandler::read_source(&args.source_path) {
        Ok(bytes) => bytes,
        Err(file_handler_error) => return Err(format!("{}", file_handler_error)),
    };

    let rom_start_location: usize = if args.scan {
        println!("Scanning for possible option rom");
        match OptionRom::find_option_rom_start_in_bytes(&bytes) {
            Ok(rom_start_location) => {
                println!("Option rom located at byte {:x}", rom_start_location);
                rom_start_location
            },
            Err(e) => return Err(format!("{}", e)),
        }
    } else {
        match args.location {
            Some(location) => location,
            None => 0,
        }
    };

    let option_rom = match OptionRom::from(bytes, rom_start_location) {
        Ok(option_rom) => option_rom,
        Err(option_rom_error) => return Err(format!("Option rom error: {}", option_rom_error)),
    };

    match args.command {
        Commands::Validate {..} => validate(option_rom),
        Commands::WriteRom(write_rom_args) => write_rom(option_rom,write_rom_args, args.source_path, rom_start_location),
    }
}
