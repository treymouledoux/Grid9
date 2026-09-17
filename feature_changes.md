# Grid9 Feature Changes Nim -> Rust Rewrite

This document tracks language and tooling changes introduced by the Rust rewrite (the `rust` branch) compared to the original Nim implementation.

> [!NOTE]
> Migrating an existing config? The full key-by-key mapping lives in [migration.md](src/components/examples/migration.md).

## Conditional operators (new)

Conditions on `if` (`i`) and `while` (`w`) statements can now be combined with logical operators instead of being limited to a single comparison.

| Operator | Meaning | Example |
| --- | --- | --- |
| `\|` | Logical **OR** — passes if either condition is true | `i1=1\|i2=1` |
| `&` | Logical **AND** — passes only if both conditions are true | `i1=1&i2=0` |

```
i1=1|i2=1
  f8
}
w0=0&1!1
  f1
]
```

Both operators combine the existing `i<pos><op><bit>` / `w<pos><op><bit>` condition form, where `<op>` is `=` (equals) or `!` (not equals).

## Configuration (changed)

Every multi-word key is now `snake_case` (previously camelCase), and loading is far stricter and can cause warnings/errors for invalid keys for more guided development.

### Several config behaviors changed in ways that can bite you:
 - **Malformed values now hard-fail.** The Nim loader silently fell back to all defaults on bad TOML; the Rust loader returns a error, so make sure your toml keys have the right types.
 - **Old/unknown keys are ignored silently.** For example any leftover camelCase key (`advancedParse`, `noLog`, …) is skipped without warning and silently applies defaults.
 - **The `[experiments]` table is gone.** `exampleExperiment` and the whole table are no longer read, remove them because of complete removal of functionality.

For a complete before/after mapping and a ready-to-copy migrated config, see the [migration guide](src/components/examples/README.md).

## Preprocessor (new?)

The ~~parser~~ preprocessor was rebuilt from scratch and does substantially more work and validation than the Nim version.

- **Content-hashed cache.** Scripts are now hashed with SHA-256 instead of md5; hashes may get an `_ap` suffix when advanced preprocessing is on, so cached versions of the same script but different output due to different optimizations never collide.
- **Cache safety + recovery.** A corrupt / non functional cache entry logs a warning and forces a repreprocess, with a hint to clean the cache directory via `grid9 c preprocessor_cache` as a possible fix.
- **Dead-statement elimination.** With `advanced_preprocessor` on, empty `if` and `while` blocks are removed automatically, also `b0` 
- **Per-command validation.** Each command is walked and checked. For example, the queue command `q` must be followed by `s` or `c`, otherwise it errors with `Invalid operation for queue command`, previously only specific control flow commands were checked.
- **Bracket-depth checking.** `if` and `while` nesting depth is tracked; a negative depth is reported as an error, and the preprocessor checks for likely fixs such as a trailing `}` or `]`.

## CLI & tooling (updated)

- **Automatic `.g9` extension.** The CLI now appends `.g9` for you when running a script.
- **Conversion command.** A CLI conversion command handles encoding/decoding between text and Grid9.
- **Structured logging.** Logging for cli and scripts runs through my own personally developed logging library: [scorched](https://github.com/treymouledoux/scorched)

## Removed / breaking changes ⚠️

- Config keys moved from camelCase to `snake_case` (manual migration required).
- The `[experiments]` config table was removed.
- Legacy example scripts are fully deprecated and no longer released.
