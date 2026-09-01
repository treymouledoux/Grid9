struct Config {
    metaver: u8,
    mingrid9ver: String,
    author: String,
    description: String,
    version: u8,
    showmetadata: bool,
    advancedparse: bool,
    dontcache: bool,
    echogridmod: bool,
    nolog: bool,
    verbosity: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            metaver: 2,
            mingrid9ver: "2026.1.0".to_owned(),
            author: "unknwon".to_owned(),
            description: "unknown".to_owned(),
            version: 0,
            showmetadata: false,
            advancedparse: false,
            dontcache: false,
            echogridmod: false,
            nolog: false,
            verbosity: 0,
        }
    }
}
