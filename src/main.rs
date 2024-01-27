use std::process::exit;

mod cli;
mod option_rom;
mod option_rom_patcher;
mod file_handler;

#[cfg(test)]
mod test_helpers;

use cli::{Cli, Commands};
use option_rom::{OptionRom, OptionRomError};

use crate::file_handler::FileHandler;

fn main() {
    let args = match Cli::new() {
        Ok(args) => args,
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
    };

    let bytes = match FileHandler::read_source(&args.source_path) {
        Ok(bytes) => bytes,
        Err(file_handler_error) => {
            println!("{}", file_handler_error);
            exit(1)
        }
    };

    let rom_start_location: usize = if args.scan {
        println!("Scanning for possible option rom");
        match OptionRom::find_option_rom_start_in_bytes(&bytes) {
            Ok(rom_start_location) => {
                println!("Option rom located at byte {:x}", rom_start_location);
                rom_start_location
            },
            Err(e) => {
                println!("{}", e);
                exit(1)
            }
        }
    } else {
        match args.location {
            Some(location) => location,
            None => 0,
        }
    };

    let option_rom = match OptionRom::from(bytes, rom_start_location) {
        Ok(option_rom) => option_rom,
        Err(option_rom_error) => {
            println!("Option rom error: {}", option_rom_error);
            exit(1)
        }
    };

    match args.command {
        Commands::Validate {} => {
            match option_rom.validate_checksum() {
                Ok(_) => {
                    println!("Option Rom read and validated");
                    exit(0)
                },
                Err(OptionRomError::OptionRomChecksumInvalid(bad_option_rom)) => {
                    let required_checksum_byte = bad_option_rom.required_checksum_byte();
                    println!("Option Rom Checksum Invalid. Requires checksum byte {:02X?}", required_checksum_byte);
                    exit(1)
                },
                Err(e) => {
                    println!("{}", e);
                    exit(1)
                }
            }
        },
        Commands::WriteRom { output_path, force: _, rom_only, update_checksum, patch_rom } => {
            let mut option_rom = match option_rom.validate_checksum() {
                Ok(option_rom) => option_rom,
                Err(OptionRomError::OptionRomChecksumInvalid(mut bad_option_rom)) => {
                    if ! update_checksum {
                        let required_checksum_byte = bad_option_rom.required_checksum_byte();
                        println!("Option Rom Checksum Invalid and update_checksum was not specified. Requires checksum byte {:02X?}", required_checksum_byte);
                        exit(1)
                    }
                    bad_option_rom.correct_checksum_in_final_byte();
                    bad_option_rom
                },
                Err(e) => {
                    println!("Unrecoverable option rom error: {}", e);
                    exit(1)
                }
            };

            if patch_rom {
                option_rom = match option_rom_patcher::patch_rom(&option_rom) {
                    Ok(option_rom) => option_rom,
                    Err(e) => {
                        println!("Failed patching ROM with error: {}", e);
                        exit(1)
                    },
                };
            }

            if rom_only {
                match FileHandler::write_rom_only(&output_path, option_rom) {
                    Ok(..) => {
                        println!("Rom written to {}", output_path.display());
                        exit(0);
                    },
                    Err(e) => {
                        println!("{}", e);
                        exit(1);
                    }
                }
            } else {
                match FileHandler::write_rom_in_file(&args.source_path, &output_path, option_rom, rom_start_location) {
                    Ok(..) => {
                        println!("Rom written to {}", output_path.display());
                        exit(0);
                    },
                    Err(e) => {
                        println!("{}", e);
                        exit(1);
                    }
                }

            }
        },
    };
}

