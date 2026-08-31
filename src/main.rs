mod language;
mod paths;

use paths::*;

use scorched::{set_logging_path, log_this};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "Grid9", version, about = "Grid9 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(alias = "a")]
    About,

    #[command(alias = "v")]
    Version,

    #[command(alias = "d")]
    Documentation,

    #[command(alias = "c")]
    Clean { folder: String },

    #[command(alias = "e")]
    Example { name: String },

    #[command(alias = "i")]
    Interpret { path: String },

    GlyphValueGet { glyph: char },
}

fn main() {
    set_logging_path(LOG_DIR);

    let cli = Cli::parse();

    match cli.command {
        Command::About => println!("Grid9 is a esoteric programming language based on a 3x3 grid of memory cells which you use to make 'glyths' these 'glyths' are used to output to the scripts terminal.\nThis language was developed by Trey Mouledoux in the Nim programming language but since has been reimplemented in Rust by Trey Mouledoux."),
        Command::Version => println!("Version: {}", env!("CARGO_PKG_VERSION")),
        Command::Documentation => println!("Docs: https://treymouledoux.github.io/Grid9/"),
        Command::Clean { folder } => {}, //TODO:
        Command::Example { name } => {}, //TODO:
        Command::Interpret { path } => {}, //TODO:
        Command::GlyphValueGet { glyph } => {}, //TODO:
    }
}