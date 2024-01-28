use std::path::PathBuf;

use crate::FileHandler;
use crate::option_rom::{OptionRom, OptionRomError};
use crate::option_rom_patcher;
use crate::cli::WriteRomArgs;

pub fn write_rom(option_rom: OptionRom, write_rom_args: WriteRomArgs, source_path: PathBuf, rom_start_location: usize) -> Result<String, String> {
    if write_rom_args.output_path.exists() && ! write_rom_args.force {
        return Err("The output file exists and the force option was not specified".into());
    }

    let mut option_rom = match option_rom.validate_checksum() {
        Ok(option_rom) => option_rom,
        Err(OptionRomError::OptionRomChecksumInvalid(mut bad_option_rom)) => {
            if ! write_rom_args.update_checksum {
                let required_checksum_byte = bad_option_rom.required_checksum_byte();
                return Err(format!("Option Rom Checksum Invalid and update_checksum was not specified. Requires checksum byte {:02X?}", required_checksum_byte));
            }
            bad_option_rom.correct_checksum_in_final_byte();
            bad_option_rom
        },
        Err(e) => return Err(format!("Unrecoverable option rom error: {}", e)),
    };

    if write_rom_args.patch_rom {
        option_rom = match option_rom_patcher::patch_rom(&option_rom) {
            Ok(option_rom) => option_rom,
            Err(e) => return Err(format!("Failed patching ROM with error: {}", e)),
        };
    }

    if write_rom_args.rom_only {
        match FileHandler::write_rom_only(&write_rom_args.output_path, option_rom) {
            Ok(..) => Ok(format!("Rom written to {}", write_rom_args.output_path.display())),
            Err(e) => Err(format!("{}", e)),
        }
    } else {
        match FileHandler::write_rom_in_file(&source_path, &write_rom_args.output_path, option_rom, rom_start_location) {
            Ok(..) => Ok(format!("Rom written to {}", write_rom_args.output_path.display())),
            Err(e) => Err(format!("{}", e)),
        }
    }
}
