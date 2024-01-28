use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(author, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[clap(flatten)]
    pub source_args: SourceArgs,
}

impl Cli {
    pub fn new() -> Cli {
        Cli::parse()
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Validate {},
    WriteRom(WriteRomArgs),
}

#[derive(Debug, Args)]
pub struct SourceArgs {
    /// The path to the file to read
    pub source_path: std::path::PathBuf,

    /// ROM location in the file (in hex if specified with a leading 0x)
    #[arg(short, long, conflicts_with = "scan")]
    pub location: Option<usize>,

    /// Scan in the source file for an Option Rom
    #[arg(long, conflicts_with = "location")]
    pub scan: bool
}

#[derive(Debug, Args)]
pub struct WriteRomArgs {
    /// File path to write the output to
    pub output_path: std::path::PathBuf,

    /// Force overwrite an existing output file
    #[arg(short, long)]
    pub force: bool,

    /// Only write the discovered ROM and not the whole file
    #[arg(short, long)]
    pub rom_only: bool,

    /// Fix the checksum by altering the final byte of the rom
    #[arg(short, long)]
    pub update_checksum: bool,

    /// Patch the ROM with our hack
    #[arg(short, long)]
    pub patch_rom: bool,
}
