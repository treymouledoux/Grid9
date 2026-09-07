use std::{
    fmt::Write as _,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use scorched::{
    LogData, LogExpect,
    LogImportance::{self, *},
    logf,
};
use sha2::{Digest, Sha256};

use crate::{file_man::PARSER_CACHE_DIR, language::config::Config};

pub fn parse(file_path: &PathBuf, mut cfg: Config) -> String {
    if cfg.verbosity >= 2 {
        logf!(Info, "Parsing script");
    }

    let file = read_to_string(file_path).log_expect(Error, "Failed to read script file");

    let mut file_hash: String = Sha256::digest(file.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    if cfg.advanced_parse {
        file_hash.push_str("_ap");
    }

    // Checks to allow parser cache load if dont_cache is false and if a file exists with the same hash
    if !cfg.dont_cache
        && Path::new(&format!(
            "{}/{}.g9",
            PARSER_CACHE_DIR
                .clone()
                .into_string()
                .log_expect(Error, "Failed to load parser cache"),
            file_hash
        ))
        .exists()
    {
        if cfg.verbosity >= 1 {
            logf!(Info, "Loading cached code from parser cache");
        }

        cfg.dont_cache = true;

        return read_to_string(Path::new(&format!(
            "{}/{}.g9",
            PARSER_CACHE_DIR
                .clone()
                .into_string()
                .log_expect(Error, "Failed to load parser cache"),
            file_hash
        )))
        .unwrap_or({
            logf!(Warning, "Failed to read found cached parsed code, forcing reparse, it is recommended to clean your cache dirrectory with the following command: grid9 c parser_cache");
            parse(file_path, cfg)
        });
    }

    let mut parsed_code = file;

    parsed_code
}
