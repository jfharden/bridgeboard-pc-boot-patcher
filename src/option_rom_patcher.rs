use std::fmt;

use crate::option_rom::{OptionRom, OptionRomError};

#[derive(Debug)]
pub enum OptionRomPatcherError {
    OptionRomGenerationError(OptionRomError),
    CouldntLocateHddReadyCheck,
    CouldntLocateAfterInt13Set,
    JumpLengthTooBig
}

impl fmt::Display for OptionRomPatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionRomPatcherError::CouldntLocateHddReadyCheck => write!(f, "Couldn't find the HDD ready check."),
            OptionRomPatcherError::CouldntLocateAfterInt13Set => write!(f, "Couldn't find the end of the code which sets the INT13 handler."),
            OptionRomPatcherError::JumpLengthTooBig => write!(f, "The distance to JMP to avoid setting INT13 is too big."),
            OptionRomPatcherError::OptionRomGenerationError(e) => write!(f, "{}", e),
        }
    }
}

const X86_INT: u8 = 0xCD;
const X86_MOV_INTO_AH: u8 = 0xb4;
const X86_MOV_INTO_DL: u8 = 0xb2;
const X86_POP_AX: u8 = 0x58;
const X86_POP_DX: u8 = 0x5a;
const X86_POP_ES: u8 = 0x07;
const X86_JC: u8 = 0x72;
const X86_JMP: u8 = 0xeb;

const X86_MOV_SEGMENT_REGISTER_TO_MEMORY_ADDRESS: u8 = 0x8c;
const X86_MOV_GENERAL_REGISTER_TO_MEMORY_ADDRESS: u8 = 0x89;
const X86_ES_SEGMENT_REGISTER: u8 = 0x06;
const X86_DI_GENERAL_REGISTER: u8 = 0x3e;

const HDD_READY_CHECK_SEARCH: [u8; 9] = [
    X86_MOV_INTO_AH, 0x10,
    X86_MOV_INTO_DL, 0x80,
    X86_INT, 0x13,
    X86_POP_DX,
    X86_POP_AX,
    X86_JC,
];

const INT_13_SET_FINISHED_SEARCH: [u8; 9] = [
    X86_MOV_SEGMENT_REGISTER_TO_MEMORY_ADDRESS, X86_ES_SEGMENT_REGISTER, 0x1e, 0x20,
    X86_MOV_GENERAL_REGISTER_TO_MEMORY_ADDRESS, X86_DI_GENERAL_REGISTER, 0x1c, 0x20,
    X86_POP_ES,
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
    let location_of_int_13_set_finished = match find_location_after_int_13_set(option_rom) {
        Err(e) => return Err(e),
        Ok(location) => location,
    };

    // Need to add 2 on the location of the jump since thats where the JMP instruction will count from
    let jump_length: u8 = match u8::try_from(location_of_int_13_set_finished - (location_of_hdd_not_ready_jump+2)) {
        Ok(jump_length) => jump_length,
        Err(_) => return Err(OptionRomPatcherError::JumpLengthTooBig),
    };

    let mut new_rom_bytes: Vec<u8> = option_rom.bytes[0..location_of_hdd_not_ready_jump].to_vec();
    new_rom_bytes.push(X86_JMP);
    new_rom_bytes.push(jump_length);
    new_rom_bytes.extend_from_slice(&option_rom.bytes[location_of_hdd_not_ready_jump+2..]);
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
            return Ok(i+8);
        }
    }

    Err(OptionRomPatcherError::CouldntLocateHddReadyCheck)
}

fn find_location_after_int_13_set(option_rom: &OptionRom) -> Result<usize, OptionRomPatcherError> {
    for i in 0..option_rom.bytes.len()-10 {
        if option_rom.bytes[i] == INT_13_SET_FINISHED_SEARCH[0] &&
           option_rom.bytes[i+1] == INT_13_SET_FINISHED_SEARCH[1] &&
           option_rom.bytes[i+2] == INT_13_SET_FINISHED_SEARCH[2] &&
           option_rom.bytes[i+3] == INT_13_SET_FINISHED_SEARCH[3] &&
           option_rom.bytes[i+4] == INT_13_SET_FINISHED_SEARCH[4] &&
           option_rom.bytes[i+5] == INT_13_SET_FINISHED_SEARCH[5] &&
           option_rom.bytes[i+6] == INT_13_SET_FINISHED_SEARCH[6] &&
           option_rom.bytes[i+7] == INT_13_SET_FINISHED_SEARCH[7] &&
           option_rom.bytes[i+8] == INT_13_SET_FINISHED_SEARCH[8] {
            return Ok(i+9);
        }
    }

    Err(OptionRomPatcherError::CouldntLocateAfterInt13Set)
}
