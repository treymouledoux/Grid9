use std::{
    fs::read_to_string,
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
        .unwrap_or({
            logf!(Warning, "Failed to read found cached preprocessed code, forcing repreprocess, it is recommended to clean your cache dirrectory with the following command: grid9 c preprocessor_cache");

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
    let mut is_exited = false;

    while i < chars.len() {
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

        if if_depth < 0 {
            let id_error_msg = "If depth is less than 0, ";
            if chars[chars.len() - 1] == '}' {
                //TODO: take user input on possible action to ignore extra closing char at end
                logf!(Error, "{}possible fix found", id_error_msg);
            } else {
                logf!(Error, "{}no auto fixes found", id_error_msg);
            }
        }

        if while_depth < 0 {
            let wd_error_msg = "While depth is less than 0, ";
            if chars[chars.len() - 1] == ']' {
                logf!(Error, "{}possible fix found", wd_error_msg);
                //TODO: take user input on possible action to ignore extra closing char at end
            } else {
                logf!(Error, "{}no auto fixes found", wd_error_msg);
            }
        }

        i += 1;
    }

    //TODO: Debug line
    println!("{}", preprocessed_code);

    preprocessed_code
}
