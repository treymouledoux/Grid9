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

use crate::{file_man::PREPROCESSOR_CACHE_DIR, language::config::Config};

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

    // Checks to allow preprocessor cache load if dont_cache is false and if a file exists with the same hash
    if !cfg.dont_cache
        && Path::new(&format!(
            "{}/{}.g9",
            PREPROCESSOR_CACHE_DIR
                .clone()
                .into_string()
                .log_expect(Error, "Failed to load preprocessor cache"),
            file_hash
        ))
        .exists()
    {
        if cfg.verbosity >= 1 {
            logf!(Info, "Loading cached code from preprocessor cache");
        }

        return read_to_string(Path::new(&format!(
            "{}/{}.g9",
            PREPROCESSOR_CACHE_DIR
                .clone()
                .into_string()
                .log_expect(Error, "Failed to load preprocessor cache"),
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
        .replace([' ', '\n'], "");

    // Advanced parse
    if cfg.advanced_preprocess {
        let re = Regex::new(r"(?:i\d+[=!][01]\}|w\d+[=!][01]\])").unwrap();
        preprocessed_code = re.replace_all(&preprocessed_code, "").into_owned();
        preprocessed_code = preprocessed_code.replace("b0", "");
    }

    let chars: Vec<char> = preprocessed_code.chars().collect();
    let mut i = 0;

    let mut if_depth: i8 = 0;
    let mut while_depth: i8 = 0;
    //TODO: See if this is still needed
    let mut is_exited = false;

    while i < chars.len() {
        //TODO: Finish command validation code
        match chars[i] {
            's' => {}
            'f' => {}
            'a' => {}
            'p' => {}
            'q' => {
                if matches!(chars.get(i + 1), Some('s' | 'c')) {
                    i += 1;
                } else {
                    logf!(Error, "Invalid operation for queue command");
                    std::process::exit(1);
                }
            }
            'i' => {
                if_depth += 1;
            }
            '}' => {
                if_depth -= 1;
            }
            'w' => {
                while_depth += 1;
            }
            ']' => {
                while_depth -= 1;
            }
            'e' => {}
            'g' => {}
            'b' => {}
            'd' => {}
            't' => {}
            _ => {}
        }

        i += 1;
    }

    while if_depth < 0 {
        let id_error_msg = "If depth is less than 0, ";
        if preprocessed_code.ends_with('}') {
            logf!(Warning, "{}possible fix found", id_error_msg);
            if prompt_user_continue() {
                preprocessed_code.pop();
                if_depth += 1;
            } else {
                break;
            }
        } else {
            logf!(Warning, "{}no auto fixes found", id_error_msg);
            break;
        }
    }

    while if_depth < 0 || while_depth < 0 {
        match preprocessed_code.chars().last() {
            Some('}') if if_depth < 0 => {
                logf!(Warning, "If depth is less than 0, possible fix found");
                if prompt_user_continue() {
                    preprocessed_code.pop();
                    if_depth += 1;
                } else {
                    break;
                }
            }
            Some(']') if while_depth < 0 => {
                logf!(Warning, "While depth is less than 0, possible fix found");
                if prompt_user_continue() {
                    preprocessed_code.pop();
                    while_depth += 1;
                } else {
                    break;
                }
            }
            _ => {
                logf!(
                    Warning,
                    "No auto fixes found for negative control flow depth: (if_depth: {} while_depth: {})",
                    if_depth,
                    while_depth
                );
                break;
            }
        }
    }

    //TODO: Debug line
    println!("{}", preprocessed_code);

    if !cfg.dont_cache {
        //TODO: Write finished preprocessed code to cache
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
