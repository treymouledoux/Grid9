use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use scorched::{LogData, LogExpect, LogImportance, logf};

use crate::file_man::Dir::{All, Logs, PreprocessorCache};

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(path) = std::env::var_os("GRID9_DATA_DIR") {
        return PathBuf::from(path);
    }
    dirs::data_dir()
        .map(|d| d.join("Grid9"))
        .expect("could not locate a user data directory")
});

pub static PREPROCESSOR_CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| DATA_DIR.join("preprocessor_cache"));
pub static LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| DATA_DIR.join("logs"));
pub static EXAMPLE_DIR: LazyLock<PathBuf> = match cfg!(debug_assertions) {
    true => {
        LazyLock::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/components/examples"))
    }
    false => LazyLock::new(|| DATA_DIR.join("examples")),
};
pub static DOCS_DIR: LazyLock<PathBuf> = match cfg!(debug_assertions) {
    true => LazyLock::new(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/components/documentation")
    }),
    false => LazyLock::new(|| DATA_DIR.join("documentation")),
};

include!(concat!(env!("OUT_DIR"), "/components.rs"));

pub fn initialize() -> std::io::Result<()> {
    initialize_at(&DATA_DIR)
}

fn initialize_at(data_dir: &Path) -> std::io::Result<()> {
    use sha2::{Digest, Sha256};

    for dir in ["logs", "preprocessor_cache", "documentation", "examples"] {
        fs::create_dir_all(data_dir.join(dir))?;
    }

    let mut hash = Sha256::new();
    for &(name, bytes) in COMPONENTS {
        hash.update(name.as_bytes());
        hash.update([0]);
        hash.update(bytes);
    }
    let revision: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    let marker = data_dir.join(".components-revision");
    let refresh = fs::read_to_string(&marker).ok().as_deref() != Some(revision.as_str());
    for &(name, bytes) in COMPONENTS {
        let target = data_dir.join(name);
        if refresh || !target.exists() {
            fs::create_dir_all(target.parent().unwrap())?;
            fs::write(target, bytes)?;
        }
    }
    if refresh {
        fs::write(marker, revision)?;
    }
    Ok(())
}

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
            logf!(Info, "Removed \"{}\"", path.into_string().unwrap());
        }
    }
    Ok(())
}

pub fn open_in_browser(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_installs_components_and_preserves_runtime_data() {
        let root = std::env::temp_dir().join(format!("grid9-setup-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        initialize_at(&root).unwrap();
        for directory in ["logs", "preprocessor_cache"] {
            assert_eq!(fs::read_dir(root.join(directory)).unwrap().count(), 0);
        }
        for &(name, bytes) in COMPONENTS {
            assert_eq!(fs::read(root.join(name)).unwrap(), bytes);
        }
        assert!(root.join("documentation/css/style.css").is_file());
        assert!(root.join("documentation/fonts/Quantify.woff").is_file());
        assert!(root.join("examples/example1.toml").is_file());
        fs::write(root.join("logs/keep.log"), "log").unwrap();
        fs::write(root.join("preprocessor_cache/keep.g9"), "f7p").unwrap();
        fs::write(root.join("examples/custom.g9"), "f6p").unwrap();
        fs::write(root.join(".components-revision"), "previous build").unwrap();
        initialize_at(&root).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("logs/keep.log")).unwrap(),
            "log"
        );
        assert_eq!(
            fs::read_to_string(root.join("preprocessor_cache/keep.g9")).unwrap(),
            "f7p"
        );
        assert_eq!(
            fs::read_to_string(root.join("examples/custom.g9")).unwrap(),
            "f6p"
        );
        // Missing assets are restored even without a version change.
        fs::remove_file(root.join("documentation/index.html")).unwrap();
        initialize_at(&root).unwrap();
        assert!(root.join("documentation/index.html").is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
