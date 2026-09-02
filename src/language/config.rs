use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub config_ver: u8,       // Config Version: no toml key
    pub min_grid9_ver: String, // Minimum Grid9 version: metadata
    pub author: String,       // Script author: metadata
    pub description: String,  // Script description: metadata
    pub version: String,      // Script version: metadata
    pub show_metadata: bool,  // Show metadata: config
    pub advanced_parse: bool, // Advanced parsing option: config
    pub dont_cache: bool,     // Enable Cache option: config
    pub echo_grid_mod: bool,  // Show grid transmutation option: config
    pub no_log: bool,         // Logging toggle: config
    pub verbosity: u8,        // Interpreter verbosity: config
}

// Defaults used when config is missing entirely or when keys are omitted.
impl Default for Config {
    fn default() -> Self {
        Config {
            config_ver: 2,
            min_grid9_ver: "2026.1.0".to_owned(),
            author: "unknown".to_owned(),
            description: "empty".to_owned(),
            version: "0.1.0".to_owned(),
            show_metadata: false,
            advanced_parse: true,
            dont_cache: false,
            echo_grid_mod: false,
            no_log: false,
            verbosity: 1,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    metadata: Metadata,
    #[serde(default)]
    config: Options,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Metadata {
    author: String,
    description: String,
    version: String,
    min_grid9_ver: String,
    show_metadata: bool,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Options {
    advanced_parse: bool,
    dont_cache: bool,
    echo_grid_mod: bool,
    no_log: bool,
    verbosity: u8,
}

impl Default for Metadata {
    fn default() -> Self {
        let d = Config::default();
        Metadata {
            author: d.author,
            description: d.description,
            version: d.version,
            min_grid9_ver: d.min_grid9_ver,
            show_metadata: d.show_metadata,
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        let d = Config::default();
        Options {
            advanced_parse: d.advanced_parse,
            dont_cache: d.dont_cache,
            echo_grid_mod: d.echo_grid_mod,
            no_log: d.no_log,
            verbosity: d.verbosity,
        }
    }
}

impl From<RawConfig> for Config {
    fn from(r: RawConfig) -> Self {
        Config {
            config_ver: Config::default().config_ver,
            min_grid9_ver: r.metadata.min_grid9_ver,
            author: r.metadata.author,
            description: r.metadata.description,
            version: r.metadata.version,
            show_metadata: r.metadata.show_metadata,
            advanced_parse: r.config.advanced_parse,
            dont_cache: r.config.dont_cache,
            echo_grid_mod: r.config.echo_grid_mod,
            no_log: r.config.no_log,
            verbosity: r.config.verbosity,
        }
    }
}

impl Config {
    pub fn from_str(text: &str) -> Result<Config, toml::de::Error> {
        let raw: RawConfig = toml::from_str(text)?;
        Ok(raw.into())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn std::error::Error>> {
        let text = fs::read_to_string(path)?;
        Ok(Config::from_str(&text)?)
    }
}
