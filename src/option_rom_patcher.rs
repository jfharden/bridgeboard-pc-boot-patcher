use std::fmt;

use crate::option_rom::{OptionRom, OptionRomError, OPTION_ROM_HEADER};

#[derive(Debug)]
pub enum OptionRomPatcherError {
    CouldntLocateInitialJump,
    OptionRomGenerationError(OptionRomError),
    CouldntLocateReturnToBIOS,
    CouldntLocateFooter,
    NotEnoughBytesAfterReturnToBIOS,
    NotEnoughBytesAtFooter,
    BytesAfterReturnToBIOSDontLookEmpty,
    BytesAtFooterDontLookEmpty,
}

impl fmt::Display for OptionRomPatcherError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionRomPatcherError::CouldntLocateInitialJump => write!(f, "Couldn't find the initial JMP SHORT instruction."),
            OptionRomPatcherError::OptionRomGenerationError(e) => write!(f, "{}", e),
            OptionRomPatcherError::CouldntLocateReturnToBIOS => write!(f, "Coulnd't find the expected return to bios (RETF) instruction"),
            OptionRomPatcherError::NotEnoughBytesAfterReturnToBIOS => write!(f, "Not enough bytes after the return to BIOS (RETF) instruction"),
            OptionRomPatcherError::BytesAfterReturnToBIOSDontLookEmpty => write!(f, "The bytes after the return to BIOS (RETF) instruction do not look empty"),
            OptionRomPatcherError::CouldntLocateFooter => write!(f, "Coulnd't find the expected INT 18h followed by IRET instructions which preceed the empty footer space"),
            OptionRomPatcherError::NotEnoughBytesAtFooter => write!(f, "Not enough bytes available for the footer patch"),
            OptionRomPatcherError::BytesAtFooterDontLookEmpty => write!(f, "The bytes where the footer patch needs to go does not look empty"),
        }
    }
}

const X86_INT: u8 = 0xCD;
const X86_IRET: u8 = 0xCF;
const X86_MOV_INTO_AX: u8 = 0xA1;
const X86_MOV_FROM_AX: u8 = 0xA1;
const X86_PUSH_AX: u8 = 0x50;
const X86_POP_AX: u8 = 0x58;
const X86_POP_DS: u8 = 0x1F;
const X86_POP_ES: u8 = 0x07;
const X86_RET: u8 = 0xC3;
const X86_RETF: u8 = 0xCB;
const X86_JMP_NEAR: u8 = 0xEB;
const X86_CALL: u8 = 0xE8;

const PATCH_HEADER_INSTRUCTIONS: [u8; 8] = [
    X86_PUSH_AX,
    X86_MOV_INTO_AX, 0x64, 0x00,  // MOV AX, [0x0064]
    X86_PUSH_AX,
    X86_MOV_INTO_AX, 0x66, 0x00, // MOV AX, [0x0066]
];

const PATCH_FOOTER_INSTRUCTIONS: [u8; 9] = [
    X86_MOV_FROM_AX, 0x66, 0x00, // MOV [0x0066], AX
    X86_POP_AX,
    X86_MOV_FROM_AX, 0x64, 0x00, // MOV [0x0064], AX
    X86_POP_AX,
    X86_RET,
];

const PATCH_FOOTER_SEARCH: [u8; 3] = [
    X86_INT,
    0x18,
    X86_IRET,
];

const RETURN_TO_BIOS_SEARCH: [u8; 3] = [
    X86_POP_DS,
    X86_POP_ES,
    X86_RETF,
];

const JUMP_TO_FOOTER_INSTRUCTIONS: [u8; 3] = [
    X86_CALL, 0, 0 // The location of this call needs to be filled with fn generate_call_to_footer_at_location
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
    let new_entrypoint_jump_size: u8 = match calculate_new_entrypoint_jump(option_rom) {
        Err(e) => return Err(e),
        Ok(new_entrypoint_jump_size) => new_entrypoint_jump_size,
    };

    let post_header_patch_location: usize = 5 + PATCH_HEADER_INSTRUCTIONS.len();
    let return_to_bios_location = find_location_of_return_to_bios(option_rom)?;
    let footer_location = find_location_of_footer_empty_space(option_rom)?;
    let post_footer_location: usize = footer_location + PATCH_FOOTER_INSTRUCTIONS.len();
    let post_return_to_bios_location: usize = return_to_bios_location + JUMP_TO_FOOTER_INSTRUCTIONS.len() + 1;
    let call_footer_patch_instructions = generate_call_to_footer_at_location(return_to_bios_location, footer_location);

    let mut new_rom_bytes: Vec<u8> = vec![OPTION_ROM_HEADER[0], OPTION_ROM_HEADER[1], option_rom.bytes[2]];  // Option ROM header
    new_rom_bytes.extend_from_slice(&PATCH_HEADER_INSTRUCTIONS);                                             // Our Header patch
    new_rom_bytes.push(X86_JMP_NEAR);                                                                        // JMP SHORT
    new_rom_bytes.push(new_entrypoint_jump_size);                                                            // JMP Location

    new_rom_bytes.extend_from_slice(&option_rom.bytes[post_header_patch_location..return_to_bios_location]); // ROM upto return to bios
    new_rom_bytes.extend_from_slice(&call_footer_patch_instructions);                                        // Call our footer patch code
    new_rom_bytes.push(X86_RETF);                                                                            // Return Far to return control to BIOS
    new_rom_bytes.extend_from_slice(&option_rom.bytes[post_return_to_bios_location..footer_location]);       // The rest of the ROM from the end of the return to BIOS upto the footer
    new_rom_bytes.extend_from_slice(&PATCH_FOOTER_INSTRUCTIONS);                                             // Our footer patch code
    new_rom_bytes.extend_from_slice(&option_rom.bytes[post_footer_location..]);                              // The remainder of the input file

    Ok(new_rom_bytes)
}

fn generate_call_to_footer_at_location(location_of_call_instruction: usize, location_of_footer: usize) -> Vec<u8> {
    let call_offset_from = location_of_call_instruction + 0x3;
    let call_offset = location_of_footer - call_offset_from;

    let call_offset_low_byte = (call_offset & 0x00FF) as u8;
    let call_offset_high_byte = ((call_offset & 0xFF00) >> 8) as u8;

    vec![X86_CALL, call_offset_low_byte, call_offset_high_byte]
}

fn calculate_new_entrypoint_jump(option_rom: &OptionRom) -> Result<u8, OptionRomPatcherError> {
    let existing_entrypoint_jump_size = get_existing_entrypoint_jump(option_rom)?;

    return Ok(existing_entrypoint_jump_size - PATCH_HEADER_INSTRUCTIONS.len() as u8);
}

fn get_existing_entrypoint_jump(option_rom: &OptionRom) -> Result<u8, OptionRomPatcherError> {
    if option_rom.bytes[3] != X86_JMP_NEAR {
        return Err(OptionRomPatcherError::CouldntLocateInitialJump)
    }

    Ok(option_rom.bytes[4])
}

fn find_location_of_return_to_bios(option_rom: &OptionRom) -> Result<usize, OptionRomPatcherError> {
    for i in 0..option_rom.bytes.len()-3 {
        if option_rom.bytes[i] == RETURN_TO_BIOS_SEARCH[0] &&
           option_rom.bytes[i+1] == RETURN_TO_BIOS_SEARCH[1] &&
           option_rom.bytes[i+2] == RETURN_TO_BIOS_SEARCH[2] {
            check_for_free_space_after_return_to_bios(option_rom, i+3)?;
            return Ok(i+2);
        }
    }

    Err(OptionRomPatcherError::CouldntLocateReturnToBIOS)
}

fn find_location_of_footer_empty_space(option_rom: &OptionRom) -> Result<usize, OptionRomPatcherError> {
    for i in 0..option_rom.bytes.len()-3 {
        if option_rom.bytes[i] == PATCH_FOOTER_SEARCH[0] &&
           option_rom.bytes[i+1] == PATCH_FOOTER_SEARCH[1] &&
           option_rom.bytes[i+2] == PATCH_FOOTER_SEARCH[2] {
            check_for_free_space_at_footer(option_rom, i+3)?;
            return Ok(i+3);
        }
    }

    Err(OptionRomPatcherError::CouldntLocateFooter)
}


fn check_for_free_space_after_return_to_bios(option_rom: &OptionRom, start_position: usize) -> Result<(), OptionRomPatcherError> {
    if option_rom.bytes.len() < start_position + JUMP_TO_FOOTER_INSTRUCTIONS.len() {
        return Err(OptionRomPatcherError::NotEnoughBytesAfterReturnToBIOS);
    }

    for i in 0..JUMP_TO_FOOTER_INSTRUCTIONS.len() {
        // I'm not sure why but the pc.boot option rom sometimes has 0 and sometimes has 0x61 as
        // blank space
        if option_rom.bytes[start_position + i] != 0 && option_rom.bytes[start_position + i] != 0x61 {
            return Err(OptionRomPatcherError::BytesAfterReturnToBIOSDontLookEmpty);
        }
    }

    Ok(())
}

fn check_for_free_space_at_footer(option_rom: &OptionRom, start_position: usize) -> Result<(), OptionRomPatcherError> {
    if option_rom.bytes.len() < start_position + PATCH_FOOTER_INSTRUCTIONS.len() {
        return Err(OptionRomPatcherError::NotEnoughBytesAtFooter);
    }

    for i in 0..PATCH_FOOTER_INSTRUCTIONS.len() {
        // I'm not sure why but the pc.boot option rom sometimes has 0 and sometimes has 0x61 as
        // blank space
        if option_rom.bytes[start_position + i] != 0 && option_rom.bytes[start_position + i] != 0x61 {
            return Err(OptionRomPatcherError::BytesAtFooterDontLookEmpty);
        }
    }

    Ok(())
}
