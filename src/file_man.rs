use std::{fs, path::Path, path::PathBuf, sync::LazyLock};

use scorched::{LogData, LogExpect, LogImportance, logf};

use crate::file_man::Dir::{All, Logs, PreprocessorCache};

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .map(|d| d.join("Grid9"))
        .expect("could not locate a user data directory")
});

pub static PREPROCESSOR_CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_DIR.join("preprocessor_cache"));
pub static LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("logs"));
pub static EXAMPLE_DIR: LazyLock<PathBuf> = match cfg!(debug_assertions) {
    true => LazyLock::new(|| PathBuf::from(r"../../src/components/examples/")),
    false => LazyLock::new(|| DATA_DIR.join("examples")),
};
pub static DOCS_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("documentation"));

pub enum Dir {
    All,
    Logs,
    PreprocessorCache,
}

pub fn clean(clean_type: Dir) {
    match clean_type {
        All => {
            clear_dir(&LOG_DIR).log_expect(LogImportance::Warning, "Failed to clean log dir");
            clear_dir(&PREPROCESSOR_CACHE_DIR).log_expect(
                LogImportance::Warning,
                "Failed to clean preprocessor cache dir",
            );

            logf!(Info, "Finished cleaning all folders");
        }
        Logs => {
            clear_dir(&LOG_DIR).log_expect(LogImportance::Warning, "Failed to clean log dir");

            logf!(Info, "Finished cleaning all logs");
        }
        PreprocessorCache => {
            clear_dir(&PREPROCESSOR_CACHE_DIR).log_expect(
                LogImportance::Warning,
                "Failed to clean preprocessor cache dir",
            );

            logf!(Info, "Finished cleaning all preprocessor cache artifacts");
        }
    }
}

fn clear_dir(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
            logf!(Info, "Removed \"{}\"", path.into_string().unwrap());
        } else {
            fs::remove_file(&path)?;
            logf!(Info, "Removed \"{}\"", path.into_string().unwrap())
        }
    }
    Ok(())
}
