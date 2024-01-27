use std::fmt;

#[derive(Debug, PartialEq)]
pub enum OptionRomError {
    InvalidOptionRomHeader,
    OptionRomTooSmall,
    OptionRomChecksumInvalid(OptionRom),
    NoOptionRomFoundInScan
}

impl fmt::Display for OptionRomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionRomError::InvalidOptionRomHeader => write!(f, "Invalid Option Rom Header. Must start 0x55AA"),
            OptionRomError::OptionRomTooSmall => write!(f, "The Option Rom is not big enough"),
            OptionRomError::OptionRomChecksumInvalid(_) => write!(f, "The Option Rom had an invalid checksum"),
            OptionRomError::NoOptionRomFoundInScan => write!(f, "No possibly valid Option Rom was found scanning in the source"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OptionRom {
    pub bytes: Vec<u8>,
    pub rom_size_in_bytes: usize,
}

pub const OPTION_ROM_HEADER: [u8; 2] = [0x55, 0xAA];

impl OptionRom {
     pub fn from(bytes: Vec<u8>, start_offset: usize) -> Result<OptionRom, OptionRomError> {
        if ! (bytes[start_offset] == OPTION_ROM_HEADER[0] && bytes[start_offset+1] == OPTION_ROM_HEADER[1]) {
            return Err(OptionRomError::InvalidOptionRomHeader)
        }

        if bytes.len() < 3 {
            return Err(OptionRomError::OptionRomTooSmall)
        }

        let rom_size_in_bytes : usize = (usize::from(bytes[start_offset + 2]) * 512).into();

        let rom_end_in_bytes = start_offset + rom_size_in_bytes;

        if bytes.len() < rom_end_in_bytes {
            return Err(OptionRomError::OptionRomTooSmall)
        }

        let rom_bytes = bytes[start_offset..rom_end_in_bytes].to_vec();
        let option_rom = OptionRom {
            bytes: rom_bytes,
            rom_size_in_bytes
        };

        Ok(option_rom)
    } 

    pub fn find_option_rom_start_in_bytes(bytes: &Vec<u8>) -> Result<usize, OptionRomError> {
        for i in 0..bytes.len()-3 {
            if bytes[i] == OPTION_ROM_HEADER[0] && bytes[i+1] == OPTION_ROM_HEADER[1] {
                let suspected_rom_length = 512 * (bytes[i+2] as usize);
                let rom_end_location = i + suspected_rom_length;

                if rom_end_location <= bytes.len() {
                    return Ok(i)
                }
            }
        }

        Err(OptionRomError::NoOptionRomFoundInScan)
    }

    pub fn validate_checksum(self) -> Result<OptionRom, OptionRomError> {
        match self.calculate_checksum() {
            0 => Ok(self),
            _ => Err(OptionRomError::OptionRomChecksumInvalid(self))
        }
    }

    pub fn required_checksum_byte(&self) -> u8 {
        let remainder: u8 = self.calculate_checksum_remainder();

        match remainder {
            0 => 0,
            _ => u8::from(255 - (remainder - 1)),
        }
    }

    pub fn correct_checksum_in_final_byte(&mut self) {
        if self.calculate_checksum() == 0 {
            return
        }

        let required_checksum = self.required_checksum_byte();

        let bytes_length = self.bytes.len();

        self.bytes[bytes_length - 1] = required_checksum
    }

    fn calculate_checksum_remainder(&self) -> u8 {
        let bytes_total = self.bytes[0..self.bytes.len() - 1].iter().fold(0u32, |acc, byte| acc + (*byte as u32));
        (bytes_total % 0x100) as u8
    }

    fn calculate_checksum(&self) -> u8 {
        let bytes_total = self.bytes.iter().fold(0u32, |acc, byte| acc + (*byte as u32));
        (bytes_total % 0x100) as u8
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::{load_fixture, load_option_rom_fixture};

    #[test]
    fn from_err_invalid_header() {
        let bytes: Vec<u8> = vec![0x54, 0xAA, 0x10, 0x00];
        assert_eq!(OptionRom::from(bytes, 0), Err(OptionRomError::InvalidOptionRomHeader));
    }

    #[test]
    fn from_err_too_short() {
        let bytes: Vec<u8> = vec![0x55, 0xAA];
        assert_eq!(OptionRom::from(bytes, 0), Err(OptionRomError::OptionRomTooSmall));

        let bytes: Vec<u8> = vec![0x55, 0xAA, 0x10, 0x00, 0x00];
        assert_eq!(OptionRom::from(bytes, 0), Err(OptionRomError::OptionRomTooSmall));
    }

    #[test]
    fn from_valid_rom_with_offset() -> Result<(), String> {
        let bytes = load_fixture("pc.boot.8k-in-middle")?;
        let original_bytes= bytes.clone();

        let option_rom = match OptionRom::from(bytes, 0x1000) {
            Ok(option_rom) => option_rom,
            Err(e) => return Err(format!("Failed to load option_rom with error {}", e)),
        };

        assert_eq!(option_rom.rom_size_in_bytes, 8192);
        assert_eq!(option_rom.bytes, original_bytes[0x1000..0x3000]);

        Ok(())
    }

    #[test]
    fn test_validate_checksum_valid() -> Result<(), String> {
        let option_rom = load_option_rom_fixture("pc.boot.valid")?;

        match option_rom.validate_checksum() {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Option Rom checksum validation failure"))
        }
    }
    
    #[test]
    fn test_validate_checksum_invalid() -> Result<(), String> {
        let option_rom = load_option_rom_fixture("pc.boot.invalid_checksum")?;
        let cloned_option_rom = option_rom.clone();

        let returned_option_rom = match option_rom.validate_checksum() {
            Err(OptionRomError::OptionRomChecksumInvalid(returned_option_rom)) => returned_option_rom,
            Err(_) => return Err("Option Rom checksum invalid returned incorrect error type".into()),
            Ok(_) => return Err("Option Rom pc.boot.invalid_checksum should have failed checksum validation".into()),
        };

        assert_eq!(cloned_option_rom, returned_option_rom);

        Ok(())
    }

    #[test]
    fn test_required_checksum_byte() -> Result<(), String> {
        let invalid_option_rom = load_option_rom_fixture("pc.boot.invalid_checksum")?;
        assert_eq!(invalid_option_rom.required_checksum_byte(), 0xF1);
        Ok(())
    }

    #[test]
    fn test_correct_checksum_in_final_byte_on_invalid_rom() -> Result<(), String> {
        let mut invalid_option_rom = load_option_rom_fixture("pc.boot.invalid_checksum")?;

        invalid_option_rom.correct_checksum_in_final_byte();

        assert_eq!(invalid_option_rom.bytes[0x1FFF], 0xF1);
        Ok(())
    }

    #[test]
    fn test_correct_checksum_in_final_byte_on_valid_rom() -> Result<(), String> {
        let mut invalid_option_rom = load_option_rom_fixture("pc.boot.valid")?;
        let invalid_rom_original_bytes = invalid_option_rom.bytes.clone();

        invalid_option_rom.correct_checksum_in_final_byte();

        assert_eq!(invalid_option_rom.bytes, invalid_rom_original_bytes);
        Ok(())
    }

    #[test]
    fn test_find_option_rom_start_in_bytes_at_location_0() -> Result<(), String> {
        let valid_rom_bytes_with_no_offset = load_fixture("pc.boot.valid")?;

        match OptionRom::find_option_rom_start_in_bytes(&valid_rom_bytes_with_no_offset) {
            Ok(position) => assert_eq!(position, 0),
            Err(_) => return Err("Option Rom not found, expected at location 0".into()),
        };

        Ok(())
    }

    #[test]
    fn test_find_option_rom_start_in_bytes_at_offset_location() -> Result<(), String> {
        let valid_rom_bytes_with_no_offset = load_fixture("pc.boot.8k-in-middle")?;

        match OptionRom::find_option_rom_start_in_bytes(&valid_rom_bytes_with_no_offset) {
            Ok(position) => assert_eq!(position, 0x1000),
            Err(_) => return Err("Option Rom not found, expected at location 0x1000".into()),
        };

        Ok(())
    }

    #[test]
    fn test_find_option_rom_start_in_bytes_with_no_valid_rom() -> Result<(), String> {
        let valid_rom_bytes_with_no_offset = load_fixture("pc.boot.no_header")?;

        match OptionRom::find_option_rom_start_in_bytes(&valid_rom_bytes_with_no_offset) {
            Err(OptionRomError::NoOptionRomFoundInScan) => Ok(()),
            Ok(position) => return Err(format!("Option rom was located at position {}, but there should have not been a valid rom located", position)),
            Err(e) => return Err(format!("Unexpected error '{}' returned from find_option_rom_start_in_bytes", e)),
        }
    }

    #[test]
    fn test_find_option_rom_start_in_bytes_when_rom_size_would_exceed_bytes() -> Result<(), String> {
        let valid_rom_bytes_with_no_offset = load_fixture("pc.boot.header_too_late")?;

        match OptionRom::find_option_rom_start_in_bytes(&valid_rom_bytes_with_no_offset) {
            Err(OptionRomError::NoOptionRomFoundInScan) => Ok(()),
            Ok(position) => return Err(format!("Option rom was located at position {}, but there should have not been a valid rom located", position)),
            Err(e) => return Err(format!("Unexpected error '{}' returned from find_option_rom_start_in_bytes", e)),
        }
    }
}
