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

pub fn parse(file_path: &PathBuf, cfg: Config) -> String {
    if cfg.verbosity >= 2 {
        logf!(Info, "Parsing script");
    }

    let file = read_to_string(file_path).log_expect(Error, "Failed to read script file");

    let digest = Sha256::digest(file.as_bytes());
    let mut file_hash = String::with_capacity(64);
    for byte in digest {
        let _ = write!(file_hash, "{:02x}", byte);
    }

    // Checks to allow parser cache load
    // Is allowed if dont_cache is false and if a file exists with the same hash
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

        return read_to_string(file_path).log_expect(Error, "Failed to read parser cache file");
    }

    "gurt".to_string()
}
