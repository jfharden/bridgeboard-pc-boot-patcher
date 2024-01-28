use std::process::exit;

mod cli;
mod commands;
mod option_rom;
mod option_rom_patcher;
mod file_handler;

#[cfg(test)]
mod test_helpers;

use cli::Cli;

use crate::file_handler::FileHandler;

fn main() {
    let args = Cli::new();

    match commands::process::process(args) {
        Ok(message) => {
            println!("{}", message);
            exit(0);
        },
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    };
}
