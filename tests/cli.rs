use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Script {
    root: PathBuf,
}

impl Script {
    fn new(source: &str, options: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "grid9-cli-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("script.g9"), source).unwrap();
        fs::write(
            root.join("script.toml"),
            format!("[config]\nverbosity = 0\n{options}\n"),
        )
        .unwrap();
        Self { root }
    }

    fn run(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grid9"))
            .arg("interpret")
            .arg(self.root.join("script.g9"))
            .env("GRID9_DATA_DIR", self.root.join("data"))
            .output()
            .unwrap()
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn runtime_failures_have_nonzero_status_even_without_logging() {
    for no_log in [false, true] {
        let script = Script::new("a1p", &format!("no_log = {no_log}"));
        let output = script.run();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("Grid has no glyph"));
    }
}

#[test]
fn preprocessing_and_cache_preserve_nonzero_back_with_leading_zero() {
    for advanced in [false, true] {
        let script = Script::new(
            "f7pi7=1s70b010}p",
            &format!("advanced_preprocess = {advanced}\nno_log = true"),
        );
        // Exercise both freshly preprocessed and cached execution.
        for _ in 0..2 {
            let output = script.run();
            assert!(output.status.success(), "{output:?}");
            assert_eq!(output.stdout, b"a\n \n \n");
        }
    }
}

#[test]
fn legacy_cache_entries_cannot_override_corrected_preprocessing() {
    let source = "f7p";
    let script = Script::new(source, "no_log = true");
    let cache = script.root.join("data/preprocessor_cache");
    fs::create_dir_all(&cache).unwrap();
    let hash: String = Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    fs::write(cache.join(format!("{hash}_ap.g9")), "t").unwrap();
    let output = script.run();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"a\n");
}

#[test]
fn invalid_empty_conditions_are_rejected_before_optimization() {
    let script = Script::new("i99=0}f7p", "no_log = true");
    assert!(!script.run().status.success());
}
