# Migrating your Grid9 config

The Rust rewrite of Grid9 renames every multi-word config key to `snake_case`. The `[metadata]` and `[config]` tables keep their names — only the keys inside them change. Rename them in place; `author`, `description`, `version`, and `verbosity` are unchanged.

## Key mapping

| Table Key | Old Grid9 (Nim) | New Grid9 (Rust) |
| --- | --- | --- |
| metadata | `showmetadata` | `show_metadata` |
| metadata | `minGrid9Ver` | `min_grid9_ver` |
| config | `advancedParse` | `advanced_preprocess` |
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
advanced_preprocess = true
dont_cache     = false
echo_grid_mod  = false
no_log         = false
verbosity      = 1
```

### Several config behaviors changed in ways that can bite you:
 - **Malformed values now hard-fail.** The Nim loader silently fell back to all defaults on bad TOML; the Rust loader returns a error, so make sure your toml keys have the right types.
 - **Old/unknown keys are ignored silently.** For example any leftover camelCase key (`advancedParse`, `noLog`, …) is skipped without warning and silently applies defaults.
 - **The `[experiments]` table is gone.** `exampleExperiment` and the whole table are no longer read, remove them because of complete removal of functionality.
