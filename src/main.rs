mod file_man;
mod language;

use std::path::Path;

use file_man::*;
use language::glyphs::{decode, encode};

use clap::{Parser, Subcommand};
use scorched::{
    LogData, LogExpect,
    LogImportance::{self, Error},
    logf, set_logging_path,
};

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

    #[command(alias = "i")]
    Interpret {
        #[arg(short = 'e', long = "example")]
        example: bool,

        input: String,
    },

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
        Command::Interpret { example, input } => {
            let path = match example {
                true => {
                    format!(
                        "{}/{}",
                        EXAMPLE_DIR.to_str().log_expect(
                            Error,
                            "Unable to construct example directory, make sure it is exists"
                        ),
                        input
                    )
                }
                false => input.clone(),
            };

            if Path::new(&path).exists() {
                // Continue interpret
            } else {
                if example {
                    logf!(Warning, "No example found with name '{input}', double check the name of the requested example")
                } else {
                    logf!(Error, "File '{path}' not found, check to make sure the correct path has been specified")
                }
            }
        }
        Command::Convert { mode, input } => {
            let result = match mode.as_str() {
                "encode" => encode(&input), // char -> glyph codes
                "decode" => decode(&input), // glyph codes -> char
                _ => unreachable!("clap value_parser restricts mode to char|glyph"),
            };

            match result {
                Some(out) => println!("{}", out),
                None => {
                    logf!(Warning, "Conversion failed for {input:?}");
                    std::process::exit(1);
                }
            }
        }
        Command::Clean { folder } => {
            match folder.as_str() {
                "parser_cache" | "parser" => {
                    file_man::clean(file_man::Dir::ParserCache);
                }
                "logs" | "log" => {
                    file_man::clean(file_man::Dir::Logs);
                }
                "temp" | "all" | "a" => {
                    file_man::clean(file_man::Dir::All);
                }
                _ => {
                    logf!(
                        Warning,
                        "Invalid folder name '{folder}', try any of the following 'parser_cache', 'parser'; 'logs', 'log'; or 'temp', 'all', 'a'."
                    );
                    std::process::exit(1);
                }
            }
        }
    }
}
