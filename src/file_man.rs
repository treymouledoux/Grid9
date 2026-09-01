use std::{fs, path::Path, path::PathBuf, sync::LazyLock};

use scorched::{LogData, LogExpect, LogImportance, logf};

use crate::file_man::Dir::{All, Logs, ParserCache};

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .map(|d| d.join("Grid9"))
        .expect("could not locate a user data directory")
});

pub static PARSER_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("parser_cache"));
pub static LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("logs"));
pub static EXAMPLE_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("examples"));
pub static DOCS_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("documentation"));

pub enum Dir {
    All,
    Logs,
    ParserCache,
}

pub fn clean(clean_type: Dir) {
    match clean_type {
        All => {
            clear_dir(&LOG_DIR).log_expect(LogImportance::Warning, "Failed to clean log dir");
            clear_dir(&PARSER_CACHE_DIR)
                .log_expect(LogImportance::Warning, "Failed to clean parser cache dir");

            logf!(Info, "Sucsessfully cleaned all folders");
        }
        Logs => {
            clear_dir(&LOG_DIR).log_expect(LogImportance::Warning, "Failed to clean log dir");

            logf!(Info, "Sucsessfully cleaned all logs");
        }
        ParserCache => {
            clear_dir(&PARSER_CACHE_DIR)
                .log_expect(LogImportance::Warning, "Failed to clean parser cache dir");

            logf!(Info, "Sucsessfully cleaned all parser cache artifacts");
        }
    }
}

fn clear_dir(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
            logf!(Info, "Removed {path:?}");
        } else {
            fs::remove_file(&path)?;
            logf!(Info, "Removed {path:?}")
        }
    }
    Ok(())
}
