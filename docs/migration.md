# Migrating legacy Grid9 configuration

Current Grid9 uses `snake_case` configuration keys. When migrating from the legacy Nim implementation, rename multi-word keys as shown below. The `[metadata]` and `[config]` tables keep their names — only the keys inside them change. Rename them in place; `author`, `description`, `version`, and `verbosity` are unchanged.

## Key mapping

| Table Key | Legacy Grid9 (Nim) | Current Grid9 (Rust) |
| --- | --- | --- |
| metadata | `showmetadata` | `show_metadata` |
| metadata | `minGrid9Ver` | `min_grid9_ver` |
| config | `advancedParse` | `advanced_preprocess` |
| config | `dontCache` | `dont_cache` |
| config | `echoGridMod` | `echo_grid_mod` |
| config | `noLog` | `no_log` |
| experiments | `exampleExperiment` | *(removed)* |

## Current layout

A configuration in the current format (values shown are the current defaults):

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

## Loading behavior

- Malformed TOML and values of the wrong type stop script loading instead of reverting the entire configuration to defaults.
- Omitted keys use the defaults shown above. Unknown keys are ignored without warnings. Leaving `advancedParse` or `noLog` in place therefore uses the corresponding current defaults.
- Remove `[experiments]`; its values are ignored.
- Place the TOML file alongside its script with the same base name: `script.toml` configures `script.g9`.
- `no_log` suppresses interpreter logging. CLI and preprocessing messages currently do not consistently honor this option.

See [feature changes](feature_changes.md) for current language syntax and preprocessing behavior.
