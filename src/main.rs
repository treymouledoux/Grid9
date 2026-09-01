mod language;
mod paths;

use language::glyphs::{decode, encode};
use paths::*;

use clap::{Parser, Subcommand};
use scorched::{LogData, LogImportance, logf, set_logging_path};

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

    #[command(alias = "e")]
    Example { name: String },

    #[command(alias = "i")]
    Interpret { path: String },

    #[command(alias = "c")]
    Convert {
        #[arg(value_parser = ["encode", "decode"])]
        mode: String,
        input: String,
    },

    #[command(alias = "cl")]
    Clean { folder: String },
}

fn main() {
    set_logging_path(LOG_DIR.to_str().expect("Failed to get logging dir"));

    let cli = Cli::parse();

    match cli.command {
        Command::About => println!(
            "Grid9 is a esoteric programming language based on a 3x3 grid of memory cells which you use to make 'glyphs' these 'glyphs' are used to output to the scripts terminal.\nThis language was developed by Trey Mouledoux in the Nim programming language but since has been reimplemented in Rust by Trey Mouledoux."
        ),
        Command::Version => println!("Version: {}", env!("CARGO_PKG_VERSION")),
        Command::Documentation => println!("Docs: https://treymouledoux.github.io/Grid9/"),
        Command::Example { name } => {
            // Requires Interpret
        }   //TODO:
        Command::Interpret { path } => {

        } //TODO:
        Command::Convert { mode, input } => {
            let result = match mode.as_str() {
                "encode" => encode(&input),  // char -> glyph codes
                "decode" => decode(&input), // glyph codes -> char
                _ => unreachable!("clap value_parser restricts mode to char|glyph"),
            };

            match result {
                Some(out) => println!("{}", out),
                None => {
                    println!("fail");
                    logf!(Warning, "Conversion failed for {input:?}");
                    std::process::exit(1);
                }
            }
        }
        Command::Clean { folder } => {
            match folder.as_str() {
                "parser_cache" | "parser" => {

                },
                "logs" | "log" => {

                },
                "temp" | "all" | "a" => {

                },
                _ => {
                    logf!(Warning, "Invalid folder name {folder}, try any of the following 'parser_cache', 'parser'; 'logs', 'log'; or 'temp', 'all', 'a'.")
                    std::process::exit(1);
                }
            }
            logf!(Info, "Sucsessfully cleaned {folder}");
        } //TODO:
    }
}
