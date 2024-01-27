use std::fmt;

use crate::option_rom::{OptionRom, OptionRomError, OPTION_ROM_HEADER};

#[derive(Debug)]
pub enum OptionRomPatcherError {
    CouldntLocateInitialJump,
    OptionRomGenerationError(OptionRomError),
    CouldntLocateReturnToBIOS,
    NotEnoughBytesAfterReturnToBIOS,
    BytesAfterReturnToBIOSDontLookEmpty,
}

impl fmt::Display for OptionRomPatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionRomPatcherError::CouldntLocateInitialJump => write!(f, "Couldn't find the initial JMP SHORT instruction."),
            OptionRomPatcherError::OptionRomGenerationError(e) => write!(f, "{}", e),
            OptionRomPatcherError::CouldntLocateReturnToBIOS => write!(f, "Coulnd't find the expected INT 18h followed by IRET instructions"),
            OptionRomPatcherError::NotEnoughBytesAfterReturnToBIOS => write!(f, "Not enough bytes after the return (INT 18h followed by IRET) to BIOS"),
            OptionRomPatcherError::BytesAfterReturnToBIOSDontLookEmpty => write!(f, "The bytes after the return (INT 18h followed by IRET) to BIOS do not look empty"),
        }
    }
}

const X86_INT: u8 = 0xCD;
const X86_IRET: u8 = 0xCF;
const X86_MOV_INTO_AX: u8 = 0xA1;
const X86_MOV_FROM_AX: u8 = 0xA1;
const X86_PUSH_AX: u8 = 0x50;
const X86_POP_AX: u8 = 0x58;
const X86_JMP_NEAR: u8 = 0xEB;

const PATCH_HEADER: [u8; 8] = [
    X86_PUSH_AX,
    X86_MOV_INTO_AX, 0x64, 0x00,  // MOV AX, [0x0064]
    X86_PUSH_AX,
    X86_MOV_INTO_AX, 0x66, 0x00, // MOV AX, [0x0066]
];

const PATCH_FOOTER: [u8; 8] = [
    X86_MOV_FROM_AX, 0x66, 0x00, // MOV [0x0066], AX
    X86_POP_AX,
    X86_MOV_FROM_AX, 0x64, 0x00, // MOV [0x0064], AX
    X86_POP_AX,
];

pub fn patch_rom(option_rom: &OptionRom) -> Result<OptionRom, OptionRomPatcherError> {
    let patched_rom_bytes: Vec<u8> = generate_patched_rom(option_rom)?;
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
    let new_entrypoint_jump_size: u8 = match calculate_new_entrypoint_jump(option_rom) {
        Err(e) => return Err(e),
        Ok(new_entrypoint_jump_size) => new_entrypoint_jump_size,
    };
    let post_header_patch_location: usize = 5 + PATCH_HEADER.len();
    let return_to_bios_location = find_location_of_return_to_bios(option_rom)?;
    let post_footer_patch_location: usize = return_to_bios_location + PATCH_FOOTER.len() + 1;

    let mut new_rom_bytes: Vec<u8> = vec![OPTION_ROM_HEADER[0], OPTION_ROM_HEADER[1], option_rom.bytes[2]]; // Option ROM header
    new_rom_bytes.extend_from_slice(&PATCH_HEADER);                     // Our patch code
    new_rom_bytes.push(X86_JMP_NEAR);                                           // JMP SHORT
    new_rom_bytes.push(new_entrypoint_jump_size);                       // JMP Location


    new_rom_bytes.extend_from_slice(&option_rom.bytes[post_header_patch_location..return_to_bios_location]);    // ROM upto return to bios
    new_rom_bytes.extend_from_slice(&PATCH_FOOTER);
    new_rom_bytes.push(X86_IRET);
    new_rom_bytes.extend_from_slice(&option_rom.bytes[post_footer_patch_location..]);

    Ok(new_rom_bytes)
}

fn calculate_new_entrypoint_jump(option_rom: &OptionRom) -> Result<u8, OptionRomPatcherError> {
    let existing_entrypoint_jump_size = get_existing_entrypoint_jump(option_rom)?;

    return Ok(existing_entrypoint_jump_size - PATCH_HEADER.len() as u8);
}

fn get_existing_entrypoint_jump(option_rom: &OptionRom) -> Result<u8, OptionRomPatcherError> {
    if option_rom.bytes[3] != X86_JMP_NEAR {
        return Err(OptionRomPatcherError::CouldntLocateInitialJump)
    }

    Ok(option_rom.bytes[4])
}

fn find_location_of_return_to_bios(option_rom: &OptionRom) -> Result<usize, OptionRomPatcherError> {
    for i in 0..option_rom.bytes.len()-3 {
        if option_rom.bytes[i] == X86_INT && 
           option_rom.bytes[i+1] == 0x18 && 
           option_rom.bytes[i+2] == X86_IRET {
            check_for_free_space(option_rom, i+3)?;
            return Ok(i+2);
        }
    }

    Err(OptionRomPatcherError::CouldntLocateReturnToBIOS)
}

fn check_for_free_space(option_rom: &OptionRom, start_position: usize) -> Result<(), OptionRomPatcherError> {
    if option_rom.bytes.len() < start_position + PATCH_FOOTER.len() {
        return Err(OptionRomPatcherError::NotEnoughBytesAfterReturnToBIOS);
    }

    for i in 0..PATCH_FOOTER.len() {
        // I'm not sure why but the pc.boot option rom sometimes has 0 and sometimes has 0x61 as
        // blank space
        if option_rom.bytes[start_position + i] != 0 && option_rom.bytes[start_position + i] != 0x61 {
            return Err(OptionRomPatcherError::BytesAfterReturnToBIOSDontLookEmpty);
        }
    }

    Ok(())
}
