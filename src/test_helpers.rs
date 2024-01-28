use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

use crate::option_rom::OptionRom;

pub fn fixture_path(fixture_file: &str) -> PathBuf {
    let all_path_parts: Vec<&str> = vec![
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "fixtures",
        fixture_file,
    ];

    all_path_parts.iter().collect::<PathBuf>()
}

pub fn load_fixture(fixture_file: &str) -> Result<Vec<u8>, String> {
    let fixture_path = fixture_path(fixture_file);

    match std::fs::read(fixture_path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(format!("Error reading fixture: {}", e)),
    }
}

pub fn load_option_rom_fixture(fixture_file: &str) -> Result<OptionRom, String> {
    let fixture_file_bytes = load_fixture(fixture_file)?;

    match OptionRom::from(fixture_file_bytes, 0) {
        Ok(option_rom) => Ok(option_rom),
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn create_temp_dir() -> Result<TempDir, String> {
    match tempdir() {
        Ok(path) => Ok(path),
        Err(e) => Err(format!("Failed to create tempdir with error {}", e)),
    }
}

pub fn assert_file_has_bytes(file_path: &PathBuf, expected_bytes: &Vec<u8>) -> Result<(), String> {
    match std::fs::read(file_path) {
        Ok(file_bytes) => {
            assert_eq!(file_bytes, *expected_bytes);
            Ok(())
        },
        Err(e) => Err(format!("Failed to open file {} to check bytes with error {}", file_path.display(), e)),
    }
}
