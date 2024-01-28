use crate::option_rom::{OptionRom, OptionRomError};

pub fn validate(option_rom: OptionRom) -> Result<String, String> {
    match option_rom.validate_checksum() {
        Ok(_) => Ok("Option Rom read and validated".into()),
        Err(OptionRomError::OptionRomChecksumInvalid(bad_option_rom)) => {
            let required_checksum_byte = bad_option_rom.required_checksum_byte();
            Err(format!("Option Rom Checksum Invalid. Requires checksum byte {:02X?}", required_checksum_byte))
        },
        Err(e) => Err(format!("{}", e)),
    }
}
