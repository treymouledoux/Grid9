# Migrating your Grid9 config

The Rust rewrite of Grid9 renames every multi-word config key to `snake_case`. The `[metadata]` and `[config]` tables keep their names — only the keys inside them change. Rename them in place; `author`, `description`, `version`, and `verbosity` are unchanged.

## Key mapping

| Table Key | Old Grid9 (Nim) | New Grid9 (Rust) |
| --- | --- | --- |
| metadata | `showmetadata` | `show_metadata` |
| metadata | `minGrid9Ver` | `min_grid9_ver` |
| config | `advancedParse` | `advanced_parse` |
| config | `dontCache` | `dont_cache` |
| config | `echoGridMod` | `echo_grid_mod` |
| config | `noLog` | `no_log` |
| experiments | `exampleExperiment` | *(removed)* |

## Target layout

A fully migrated config in the new format (values shown are the current defaults):

```toml
[metadata]
author        = "unknown"
description   = "empty"
version       = "0.1.0"
min_grid9_ver = "2026.1.0"
show_metadata = false

[config]
advanced_parse = true
dont_cache     = false
echo_grid_mod  = false
no_log         = false
verbosity      = 1
```

> [!WARNING]
> Watch out for the following off cases

- **Old keys are ignored silently.** Any leftover camelCase key (`advancedParse`, `noLog`, …) is skipped without warning, and that option reverts to its default. The file still "loads," so a config can quietly run defaults across the board.
- **The `[experiments]` table is deprecated.** `exampleExperiment` and the whole table are no longer read; delete them.
- **Bad value types now fail the load.** Nim caught a malformed TOML and fell back to all-defaults; the Rust loader returns a `toml::de::Error`, so a stray quote around a bool or number surfaces as a hard error. `config_ver` is internal (currently `3`) and has no TOML key — don't add one.