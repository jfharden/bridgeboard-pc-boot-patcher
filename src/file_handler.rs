use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::option_rom::OptionRom;

use std::fmt;

#[derive(Debug)]
pub enum FileHandlerError {
    CouldntReadSourceFile(std::io::Error),
    CouldntWriteOutputFile(std::io::Error),
}

impl fmt::Display for FileHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileHandlerError::CouldntReadSourceFile(e) => write!(f, "Couldn't read source file with error {}", e),
            FileHandlerError::CouldntWriteOutputFile(e) => write!(f, "Couldn't write the output file with error {}", e),
        }
    }
}

pub struct FileHandler {}

impl FileHandler {
    pub fn read_source(path: &PathBuf) -> Result<Vec<u8>, FileHandlerError> {
        match fs::read(path) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(FileHandlerError::CouldntReadSourceFile(e)),
        }
    }

    pub fn write_rom_only(path: &PathBuf, option_rom: OptionRom) -> Result<(), FileHandlerError> {
        match fs::write(path, option_rom.bytes) {
            Ok(..) => Ok(()),
            Err(e) => Err(FileHandlerError::CouldntWriteOutputFile(e)),
        }
    }

    pub fn write_rom_in_file(source_file: &PathBuf, output_path: &PathBuf, option_rom: OptionRom, rom_start_byte: usize) -> Result<(), FileHandlerError> {
        let source_file_bytes = FileHandler::read_source(source_file)?;
        let rom_end_location: usize = rom_start_byte + option_rom.rom_size_in_bytes;

        let mut output_file = match File::create(output_path) {
            Ok(f) => f,
            Err(e) => return Err(FileHandlerError::CouldntWriteOutputFile(e))
        };

        if rom_start_byte != 0 {
            let write_upto_byte = if rom_start_byte <= source_file_bytes.len() {
                rom_start_byte
            } else {
                source_file_bytes.len()
            };
                
            match output_file.write_all(&source_file_bytes[0..write_upto_byte]) {
                Ok(..) => {},
                Err(e) => return Err(FileHandlerError::CouldntWriteOutputFile(e))
            }
        }

        if rom_start_byte > source_file_bytes.len() {
            let empty_space_size = rom_start_byte - source_file_bytes.len();
            let empty_space = vec![0u8; empty_space_size];

            match output_file.write_all(&empty_space) {
                Ok(..) => {},
                Err(e) => return Err(FileHandlerError::CouldntWriteOutputFile(e))
            }
        }

        match output_file.write_all(&option_rom.bytes) {
            Ok(..) => {},
            Err(e) => return Err(FileHandlerError::CouldntWriteOutputFile(e))
        }

        if rom_end_location < source_file_bytes.len() {
            match output_file.write_all(&source_file_bytes[rom_end_location..]) {
                Ok(..) => {},
                Err(e) => return Err(FileHandlerError::CouldntWriteOutputFile(e))
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_helpers::{assert_file_has_bytes, create_temp_dir, fixture_path, load_fixture, load_option_rom_fixture};
    use md5;

    #[test]
    fn test_read_source_with_fs_error() -> Result<(), String> {
        match FileHandler::read_source(&fixture_path("foo")) {
            Ok(_) => Err("Expected an error when the file didn't exist, but got Ok".into()),
            Err(e) => match e {
                FileHandlerError::CouldntReadSourceFile(original_error) => {
                    assert_eq!(original_error.kind(), std::io::ErrorKind::NotFound);
                    Ok(())
                },
                _ => Err(format!("Expected a FileHandlerError::CouldntReadSourceFile but got {}", e)),
            }
        }
    }

    #[test]
    fn test_read_source() -> Result<(), String> {
        match FileHandler::read_source(&fixture_path("pc.boot.valid")) {
            Ok(bytes) => {
                assert_eq!(bytes.len(), 10240);
                let digest = md5::compute(bytes);
                assert_eq!(format!("{:x}", digest), "df1da347ff355aeffb730b7b71cd9056");
                Ok(())
            },
            Err(e) => Err(format!("Failed to load fixture with read_source with error {}", e)),
        }
    }

    #[test]
    fn test_write_rom_only_success() -> Result<(), String> {
        let tempdir = create_temp_dir()?;
        let option_rom = load_option_rom_fixture("pc.boot.valid")?;
        let option_rom_bytes = option_rom.bytes.clone();

        let mut output_path = tempdir.into_path();
        output_path.push("test_output.rom");

        match FileHandler::write_rom_only(&output_path, option_rom) {
            Err(e) => Err(format!("Failed to write rom to output path {} with error {}", output_path.display(), e)),
            Ok(()) => {
                assert_file_has_bytes(&output_path, &option_rom_bytes)?;
                Ok(())
            },
        }
    }

    #[test]
    fn test_write_rom_only_failure() -> Result<(), String> {
        let tempdir = create_temp_dir()?;
        let option_rom = load_option_rom_fixture("pc.boot.valid")?;
        let output_path = tempdir.into_path();

        match FileHandler::write_rom_only(&output_path, option_rom) {
            Ok(()) => Err(format!("Expected failure to write to file {}, but it was successful!", output_path.display())),
            Err(e) => match e {
                FileHandlerError::CouldntWriteOutputFile(_) => Ok(()),
                _ => Err(format!("Expected a FileHandlerError::CouldntWriteOutputFile but got {}", e)),
            }
        }
    }

    #[test]
    fn test_write_rom_in_file_source_file_does_not_exist() -> Result<(), String> {
        let bad_path = fixture_path("foo.rom");
        let output_path = fixture_path("non-existing-output.rom");
        let option_rom = load_option_rom_fixture("pc.boot.valid")?;

        match FileHandler::write_rom_in_file(&bad_path, &output_path, option_rom, 0) {
            Err(e) => {
                match e {
                    FileHandlerError::CouldntReadSourceFile(_) => Ok(()),
                    _ => Err(format!("Expected FileHanderError::CouldntReadSourceFile but got {}", e)),
                }
            },
            Ok(_) => Err("Expected an Error reading a non-existing source file, but got Ok".into()),
        }
    }

    #[test]
    fn test_write_rom_in_file_cannot_write_to_output_file() -> Result<(), String> {
        let source_path = fixture_path("pc.boot.valid");
        let option_rom = load_option_rom_fixture("pc.boot.valid")?;

        let tempdir = create_temp_dir()?;
        let output_path = tempdir.into_path();

        match FileHandler::write_rom_in_file(&source_path, &output_path, option_rom, 0) {
            Ok(()) => Err(format!("Expected failure to write to file {}, but it was successful!", output_path.display())),
            Err(e) => match e {
                FileHandlerError::CouldntWriteOutputFile(_) => Ok(()),
                _ => Err(format!("Expected a FileHandlerError::CouldntWriteOutputFile but got {}", e)),
            }
        }
    }

    fn test_write_rom_in_file_against_expected(expected_output_file: &str, rom_start_byte: usize) -> Result<(), String> {
        let source_file = fixture_path("pc.boot.no-rom");
        let option_rom = load_option_rom_fixture("8k-option-rom")?;
        let expected_output_bytes = load_fixture(expected_output_file)?;

        let tempdir = create_temp_dir()?;
        let mut output_path = tempdir.into_path();
        output_path.push("8k-rom-in-file");

        match FileHandler::write_rom_in_file(&source_file, &output_path, option_rom, rom_start_byte) {
            Err(e) => return Err(format!("Expected Ok writing rom in file, but got error {}", e)),
            Ok(()) => () ,
        };

        assert_file_has_bytes(&output_path, &expected_output_bytes)?;

        Ok(())
    }

    #[test]
    fn test_write_rom_in_file_start_at_0() -> Result<(), String> {
        test_write_rom_in_file_against_expected("pc.boot.8k-at-start", 0)
    }

    #[test]
    fn test_write_rom_in_file_start_with_offset() -> Result<(), String> {
        test_write_rom_in_file_against_expected("pc.boot.8k-in-middle", 0x1000)
    }

    #[test]
    fn test_write_rom_in_file_start_with_offset_at_end_of_file() -> Result<(), String> {
        test_write_rom_in_file_against_expected("pc.boot.8k-at-end", 0x2000)
    }

    #[test]
    fn test_write_rom_in_file_start_with_offset_which_causes_rom_to_overlap_end() -> Result<(), String> {
        test_write_rom_in_file_against_expected("pc.boot.8k-overlapping-end", 0x3000)
    }

    #[test]
    fn test_write_rom_in_file_start_with_offset_which_causes_to_be_appended() -> Result<(), String> {
        test_write_rom_in_file_against_expected("pc.boot.8k-appended", 0x4000)
    }

    #[test]
    fn test_write_rom_in_file_start_with_offset_which_causes_rom_beyond_end_of_file() -> Result<(), String> {
        test_write_rom_in_file_against_expected("pc.boot.8k-beyond-end", 0x4400)
    }
}
