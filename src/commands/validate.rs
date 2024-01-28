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

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::load_option_rom_fixture;

    #[test]
    fn validate_with_valid_rom() -> Result<(), String> {
        let option_rom = load_option_rom_fixture("pc.boot.valid")?;

        match validate(option_rom) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Expected Ok but got error {}", e)),
        }
    }

    #[test]
    fn validate_with_bad_checksum() -> Result<(), String> {
        let option_rom = load_option_rom_fixture("pc.boot.invalid_checksum")?;

        match validate(option_rom) {
            Ok(_) => Err("Expected an error validating checksum on pc.boot.invalid but got Ok".into()),
            Err(message) => {
                assert_eq!(message, "Option Rom Checksum Invalid. Requires checksum byte F1");
                Ok(())
            },
        }
    }
}
