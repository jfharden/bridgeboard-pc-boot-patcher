use std::fmt;

use crate::option_rom::{OptionRom, OptionRomError};

#[derive(Debug)]
pub enum OptionRomPatcherError {
    OptionRomGenerationError(OptionRomError),
    CouldntLocateHddReadyCheck,
}

impl fmt::Display for OptionRomPatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionRomPatcherError::CouldntLocateHddReadyCheck => write!(f, "Couldn't find the HDD ready check."),
            OptionRomPatcherError::OptionRomGenerationError(e) => write!(f, "{}", e),
        }
    }
}

const X86_INT: u8 = 0xCD;
const X86_MOV_INTO_AH: u8 = 0xb4;
const X86_MOV_INTO_DL: u8 = 0xb2;
const X86_POP_AX: u8 = 0x58;
const X86_POP_DX: u8 = 0x5a;
const X86_JC: u8 = 0x72;

const HDD_READY_CHECK_SEARCH: [u8; 9] = [
    X86_MOV_INTO_AH, 0x10,
    X86_MOV_INTO_DL, 0x80,
    X86_INT, 0x13,
    X86_POP_DX,
    X86_POP_AX,
    X86_JC,
];

pub fn patch_rom(option_rom: &OptionRom) -> Result<OptionRom, OptionRomPatcherError> {
    println!("ORIGINAL_ROM_SIZE: 0x{:04X}", option_rom.bytes.len());
    let patched_rom_bytes: Vec<u8> = generate_patched_rom(option_rom)?;
    println!("PATCHED_ROM_SIZE: 0x{:04X}", patched_rom_bytes.len());
    let mut patched_rom = match OptionRom::from(patched_rom_bytes, 0) {
        Ok(patched_rom) => patched_rom,
        Err(e) => {
            return Err(OptionRomPatcherError::OptionRomGenerationError(e))
        },
    };

    patched_rom.correct_checksum_in_final_byte();
    Ok(patched_rom)
}

fn generate_patched_rom(option_rom: &OptionRom) -> Result<Vec<u8>, OptionRomPatcherError> {
    let location_of_hdd_not_ready_jump = match find_location_of_hdd_not_ready_jump(option_rom) {
        Err(e) => return Err(e),
        Ok(location) => location,
    };

    let mut new_rom_bytes: Vec<u8> = option_rom.bytes[0..location_of_hdd_not_ready_jump].to_vec();
    new_rom_bytes.push(0x1f);
    new_rom_bytes.extend_from_slice(&option_rom.bytes[location_of_hdd_not_ready_jump+1..]);

    Ok(new_rom_bytes)
}

fn find_location_of_hdd_not_ready_jump(option_rom: &OptionRom) -> Result<usize, OptionRomPatcherError> {
    for i in 0..option_rom.bytes.len()-10 {
        if option_rom.bytes[i] == HDD_READY_CHECK_SEARCH[0] &&
           option_rom.bytes[i+1] == HDD_READY_CHECK_SEARCH[1] &&
           option_rom.bytes[i+2] == HDD_READY_CHECK_SEARCH[2] &&
           option_rom.bytes[i+3] == HDD_READY_CHECK_SEARCH[3] &&
           option_rom.bytes[i+4] == HDD_READY_CHECK_SEARCH[4] &&
           option_rom.bytes[i+5] == HDD_READY_CHECK_SEARCH[5] &&
           option_rom.bytes[i+6] == HDD_READY_CHECK_SEARCH[6] &&
           option_rom.bytes[i+7] == HDD_READY_CHECK_SEARCH[7] &&
           option_rom.bytes[i+8] == HDD_READY_CHECK_SEARCH[8] {
            return Ok(i+9);
        }
    }

    Err(OptionRomPatcherError::CouldntLocateHddReadyCheck)
}
