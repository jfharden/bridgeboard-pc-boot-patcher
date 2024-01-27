use clap::{Parser, Subcommand};
use std::fmt;

#[derive(Debug)]
pub enum ArgumentError {
    SourceFileDoesNotExist,
    OutputFileExists,
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArgumentError::SourceFileDoesNotExist => write!(f, "The specified source file does not exist"),
            ArgumentError::OutputFileExists => write!(f, "The output file exists and the force option was not specified"),
        }
    }
}

#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// The path to the file to read
    pub source_path: std::path::PathBuf,

    /// ROM location in the file (in hex if specified with a leading 0x)
    #[arg(short, long, conflicts_with = "scan")]
    pub location: Option<usize>,

    /// Scan in the source file for an Option Rom
    #[arg(long, conflicts_with = "location")]
    pub scan: bool
}

impl Cli {
    pub fn new() -> Result<Cli, ArgumentError> {
        let args = Cli::parse();

        let source_path = args.source_path.as_path();

        if ! source_path.exists() {
            return Err(ArgumentError::SourceFileDoesNotExist)
        }

        match &args.command {
            Commands::WriteRom { output_path, force, rom_only: _, update_checksum: _, patch_rom: _ } => {
               if output_path.exists() && ! force {
                   return Err(ArgumentError::OutputFileExists)
               }
            },
            _ => {}
        }

        Ok(args)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    Validate {},
    WriteRom {
        /// File path to write the output to
        output_path: std::path::PathBuf,

        /// Force overwrite an existing output file
        #[arg(short, long)]
        force: bool,

        /// Only write the discovered ROM and not the whole file
        #[arg(short, long)]
        rom_only: bool,

        /// Fix the checksum by altering the final byte of the rom
        #[arg(short, long)]
        update_checksum: bool,

        /// Patch the ROM with our hack
        #[arg(short, long)]
        patch_rom: bool,
    }
}
