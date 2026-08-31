#[cfg(windows)]
macro_rules! base { () => { r"C:\ProgramData\Grid9\" } }
#[cfg(not(windows))]
macro_rules! base { () => { "/usr/share/Grid9/" } }

pub const MAIN_DIR: &str         = base!();
pub const PARSER_CACHE_DIR: &str = concat!(base!(), "parser_cache/");
pub const LOG_DIR: &str          = concat!(base!(), "logs/");
pub const EXAMPLE_DIR: &str      = concat!(base!(), "examples/");
pub const DOCS_DIR: &str         = concat!(base!(), "documentation/");
