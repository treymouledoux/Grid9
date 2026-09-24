use std::{
    fs::read_to_string,
    io::stdin,
    path::{Path, PathBuf},
};

use regex::Regex;
use scorched::{
    LogData, LogExpect,
    LogImportance::{self, *},
    logf,
};
use sha2::{Digest, Sha256};

use crate::{
    file_man::PREPROCESSOR_CACHE_DIR,
    language::{
        config::Config,
        preprocessor::ControlDepth::{If, While},
    },
};

#[derive(PartialEq)]
enum ControlDepth {
    If,
    While,
}

pub fn preprocess(file_path: &PathBuf, mut cfg: Config) -> String {
    if cfg.verbosity >= 2 {
        logf!(Info, "Parsing script");
    }

    let file = read_to_string(file_path).log_expect(Error, "Failed to read script file");

    let mut file_hash: String = Sha256::digest(file.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if cfg.advanced_preprocess {
        file_hash.push_str("_ap");
    }
    let cache_dir = PREPROCESSOR_CACHE_DIR
        .clone()
        .into_string()
        .log_expect(Error, "Failed to load preprocessor cache");

    // Checks to allow preprocessor cache load if dont_cache is false and if a file exists with the same hash
    if !cfg.dont_cache && Path::new(&format!("{}/{}.g9", cache_dir, file_hash)).exists() {
        if cfg.verbosity >= 1 {
            logf!(Info, "Loading cached code from preprocessor cache");
        }

        return read_to_string(Path::new(&format!(
            "{}/{}.g9",
            cache_dir,
            file_hash
        )))
        .unwrap_or_else(|_| {
            logf!(Warning, "Failed to read found cached preprocessed code, forcing repreprocess, it is recommended to clean your cache directory with the following command: grid9 c preprocessor_cache");

            cfg.dont_cache = true;
            preprocess(file_path, cfg)
        });
    }

    // Comment cleanup
    let mut preprocessed_code = file
        .replace(|c: char| c.is_ascii_uppercase(), "")
        .replace([' ', '\n', '\t', '\r'], "");

    // Advanced parse
    if cfg.advanced_preprocess {
        let re = Regex::new(r"(?:i\d+[=!][01]\}|w\d+[=!][01]\])").unwrap();
        preprocessed_code = re.replace_all(&preprocessed_code, "").into_owned();
        preprocessed_code = preprocessed_code.replace("b0", "");
    }

    let chars: Vec<char> = preprocessed_code.chars().collect();
    let mut i = 0;

    let mut control_depth: Vec<ControlDepth> = Vec::new();

    while i < chars.len() {
        //TODO: Redo some error messages and make them more specfic other than a general missing / invalid message
        match chars[i] {
            's' => {
                if chars.get(i + 1).is_some_and(|c| ('0'..='8').contains(c)) {
                    i += 1;
                    if matches!(chars.get(i + 1), Some('0' | '1')) {
                        i += 1;
                    } else {
                        logf!(Error, "Invalid or missing set value for set command");
                        std::process::exit(1);
                    }
                } else {
                    logf!(Error, "Invalid or missing grid cell for set command");
                    std::process::exit(1);
                }
            }
            'f' => {
                if chars.get(i + 1).is_some_and(|c| ('0'..='8').contains(c)) {
                    i += 1;
                } else {
                    logf!(Error, "Invalid or missing grid cell for flip command");
                    std::process::exit(1);
                }
            }
            'a' => {
                if matches!(chars.get(i + 1), Some('0' | '1' | 'r')) {
                    i += 1;
                } else {
                    match chars.get(i + 1) {
                        Some(set_value) => {
                            logf!(
                                Error,
                                "Invalid set value for set all command: {}",
                                set_value
                            );
                        }
                        None => {
                            logf!(Error, "Missing set value for set all command");
                        }
                    }
                    std::process::exit(1);
                }
            }
            'p' | 't' => {}
            'q' => {
                if matches!(chars.get(i + 1), Some('s' | 'c')) {
                    i += 1;
                } else {
                    logf!(Error, "Invalid operation for queue command");
                    std::process::exit(1);
                }
            }
            'i' | 'w' => {
                let (kind, name) = if chars[i] == 'i' {
                    (If, "if")
                } else {
                    (While, "while")
                };
                i = validate_condition(&chars, i, name);
                control_depth.push(kind);
            }
            '}' | ']' => {
                let (kind, name) = if chars[i] == '}' {
                    (If, "if")
                } else {
                    (While, "while")
                };

                if control_depth.pop_if(|c| *c == kind).is_none() {
                    logf!(
                        Error,
                        "Invalid exit of {} statement when not avalible",
                        name
                    );
                    std::process::exit(1);
                }
            }
            'e' => match control_depth.pop() {
                Some(_) => {}
                None => {
                    logf!(
                        Error,
                        "Unable to exit control flow due to control depth being zero"
                    );
                    std::process::exit(1);
                }
            },
            'g' => {
                if chars.get(i + 1).is_some() {
                    i += 1;
                    match chars.get(i).unwrap() {
                        'g' => {}
                        's' | 'l' | 'm' => {
                            let sub = chars[i];
                            i += 1;
                            if !matches!(chars.get(i), Some('0'..='8')) {
                                logf!(Error, "Invalid or missing grid id for grid \"{}\" command", sub);
                                std::process::exit(1);
                            }
                        }
                        'x' => {
                            i += 1;
                            if !matches!(chars.get(i), Some('0'..='8') | Some('c')) {
                                match chars.get(i) {
                                    Some(c) => logf!(
                                        Error,
                                        "Invalid grid id \"{}\" for grid xor command",
                                        c
                                    ),
                                    None => logf!(Error, "Missing grid id for grid xor command"),
                                }
                                std::process::exit(1);
                            }
                        }
                        _ => {
                            logf!(
                                Error,
                                "Invalid subcommand provided to grid statement: {}",
                                chars.get(i).unwrap()
                            );
                            std::process::exit(1);
                        }
                    }
                } else {
                    logf!(Error, "No subcommand provided to grid statement");
                    std::process::exit(1);
                }
            }
            'b' | 'd' => {
                let start = i + 1;
                let mut j = start;
                while chars.get(j).is_some_and(|c| c.is_ascii_digit()) {
                    j += 1;
                }
                if j == start {
                    logf!(
                        Error,
                        "Missing numeric argument for \"{}\" command at {}",
                        chars[i],
                        i
                    );
                    std::process::exit(1);
                }
                i = j - 1;
            }
            _ => {
                logf!(
                    Warning,
                    "Invalid charcater \"{}\" found at index {}",
                    chars.get(i).unwrap(),
                    i
                )
            }
        }

        i += 1;
    }

    while let Some(block) = control_depth.last() {
        let (closer, name) = match block {
            If => ('}', "if"),
            While => (']', "while"),
        };

        logf!(
            Warning,
            "Unclosed {} block at end of file, possible fix: append '{}'",
            name,
            closer
        );
        if prompt_user_continue() {
            preprocessed_code.push(closer);
            control_depth.pop();
        } else {
            break;
        }
    }

    if !control_depth.is_empty() {
        logf!(
            Error,
            "{} unclosed control-flow block(s) remain, cannot continue",
            control_depth.len()
        );
        std::process::exit(1);
    }

    if !cfg.dont_cache {
        match std::fs::write(
            format!("{}/{}.g9", cache_dir, file_hash),
            &preprocessed_code,
        ) {
            Ok(_) => {
                return preprocessed_code;
            }
            Err(e) => logf!(
                Warning,
                "Failed to write preprocessed file to preprocessor cache: {}",
                e
            ),
        }
    }

    preprocessed_code
}

fn prompt_user_continue() -> bool {
    println!("Would you like to continue with this action? (y/n): ");
    let mut input = String::new();

    match stdin().read_line(&mut input) {
        Ok(_) => match input.chars().next() {
            Some('y') => true,
            Some('n') => false,
            _ => {
                logf!(
                    Warning,
                    "Invalid response from user, continuing without action"
                );
                false
            }
        },
        Err(e) => {
            logf!(Warning, "Failed to get user input: {}", e);
            false
        }
    }
}

fn validate_condition(chars: &[char], mut i: usize, name: &str) -> usize {
    loop {
        // cell [0-8], operator [=!], value [01]
        if !chars.get(i + 1).is_some_and(|c| ('0'..='8').contains(c)) {
            logf!(Error, "Invalid or missing grid cell for {} condition", name);
            std::process::exit(1);
        }
        if !matches!(chars.get(i + 2), Some('=' | '!')) {
            logf!(
                Error,
                "Invalid or missing comparison operator for {} condition",
                name
            );
            std::process::exit(1);
        }
        if !matches!(chars.get(i + 3), Some('0' | '1')) {
            logf!(
                Error,
                "Invalid or missing comparison value for {} condition",
                name
            );
            std::process::exit(1);
        }
        i += 3;

        match chars.get(i + 1) {
            Some('&' | '|') => i += 1,
            _ => break,
        }
    }
    i
}
