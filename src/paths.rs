use std::sync::LazyLock;
use std::path::PathBuf;

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .map(|d| d.join("Grid9"))
        .expect("could not locate a user data directory")
});

pub static PARSER_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("parser_cache"));
pub static LOG_DIR:          LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("logs"));
pub static EXAMPLE_DIR:      LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("examples"));
pub static DOCS_DIR:         LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("documentation"));

// Install locations, unrelated but needed later
//Windows = C://Program Files/Grid9
//Linux /usr/bin
//Mac /usr/local/bin
