use clap::{
    builder::styling::{AnsiColor, Styles},
    ArgGroup, Args, ColorChoice, CommandFactory, Parser, Subcommand,
};
use clap_complete::{generate, Shell};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

mod bwtstring;
use bwtstring::*;

#[derive(Parser)]
#[command(
    name = "rust-bwt",
    version = "0.0.1",
    about = "A pure-Rust CLI tool for BWT + RLE compression and decompression",
    author = "Jacob Cohen <jacob@jacobcohen.info>",
    styles=Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default()))
]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compress(CompressArgs),
    Decompress(DecompressArgs),

    /// Generate shell completion scripts with clap_complete
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Args)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["input_file", "input_string"]),
))]
struct CompressArgs {
    #[arg(short, long, value_name = "FILE", conflicts_with = "input_string")]
    input_file: Option<PathBuf>,
    #[arg(
        short = 's',
        long,
        value_name = "STRING",
        conflicts_with = "input_file"
    )]
    input_string: Option<String>,
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

#[derive(Args)]
struct DecompressArgs {
    #[arg(short, long, value_name = "FILE")]
    input_file: PathBuf,
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compress(args) => {
            if let Err(e) = compress(args) {
                eprintln!("Error during compression: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Decompress(args) => {
            if let Err(e) = decompress(args) {
                eprintln!("Error during decompression: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(*shell, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }
}

fn compress(args: &CompressArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read input data
    let input_data = if let Some(input_file) = &args.input_file {
        let mut file = File::open(input_file)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        buffer
    } else if let Some(input_string) = &args.input_string {
        input_string.clone().into_bytes()
    } else {
        unreachable!("Input is required");
    };

    // BWT+RLE compress
    let bwt_str = BWTStr::new(input_data);
    let transformed = bwt_str.forward_transform();

    // Write compressed data
    if let Some(output_file) = &args.output {
        let mut file = File::create(output_file)?;
        transformed.rle_write(&mut file)?;
    } else {
        // Default to writing to stdout
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        transformed.rle_write(&mut handle)?;
    }

    Ok(())
}

fn decompress(args: &DecompressArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read compressed data
    let mut file = File::open(&args.input_file)?;

    // Decompress
    let transformed = BWTStr::rle_read(&mut file)?;
    let original = transformed.reverse_transform();
    let output_data: Vec<u8> = original
        .inner
        .into_iter()
        .filter_map(|bwt_byte| match bwt_byte {
            bwtstring::BWTByte::Byte(b) => Some(b),
            bwtstring::BWTByte::Sentinel => None,
        })
        .collect();

    // Write decompressed data
    if let Some(output_file) = &args.output {
        let mut output = File::create(output_file)?;
        output.write_all(&output_data)?;
    } else {
        // Default stdout
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&output_data)?;
    }

    Ok(())
}
