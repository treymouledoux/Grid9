# Current Grid9 behavior and legacy changes

Grid9's current implementation is written in Rust and lives on `main`. This document describes its behavior and the changes from the legacy Nim implementation.

## Compound conditions

Conditions on `if` (`i`) and `while` (`w`) support logical operators. Write `i` or `w` once, then join comparisons of the form `<cell><operator><bit>`.

| Operator | Meaning | Example |
| --- | --- | --- |
| `\|` | OR: either comparison is true | `i1=1\|2=1` |
| `&` | AND: both comparisons are true | `i1=1&2=0` |

```text
i1=1|2=1
  f8
}
w0=0&1!1
  f1
]
```

Cells range from 0 to 8, operators are `=` and `!`, and comparison values are literal `0` or `1`. AND binds more tightly than OR. Loops test their conditions before each iteration. `e` exits the innermost enclosing while loop, including when used inside an if block.

## Configuration

Configuration uses `snake_case` keys in `[metadata]` and `[config]`. Missing keys use defaults; malformed TOML or values of the wrong type stop script loading. Unknown keys and tables are ignored without warnings, so legacy key names must be migrated manually. The `[experiments]` table has no effect.

See the [migration guide](migration.md) for key mappings and defaults.

## Preprocessing and caching

- Source contents are hashed with SHA-256. Cache filenames include `_ap` when `advanced_preprocess` is enabled.
- An unreadable cache entry triggers a warning and preprocessing without caching. Readable cached code is returned without preprocessing validation; content corruption is not automatically detected. To remove cached entries, use `grid9 clean preprocessor_cache` or `grid9 cl preprocessor_cache`.
- `advanced_preprocess` removes simple empty if/while blocks and `b0`. It changes the character positions used by Back, so calculate offsets against the preprocessed code.
- Commands and arguments are validated during preprocessing. Unexpected characters produce warnings; invalid command arguments and mismatched closing brackets stop preprocessing.
- Unclosed blocks prompt to append their missing closing brackets. This check also runs with advanced preprocessing disabled.

Uppercase ASCII letters, spaces, tabs, line breaks, parentheses, periods, and commas are removed. Comments are uppercase text, not whole ignored lines: digits and lowercase letters remain executable. Use words such as `FOUR` rather than `4` and avoid lowercase command examples inside comments.

## Interpreter behavior

- `s<cell>r` assigns a random bit; `ar` assigns a random bit to each cell independently.
- `p` prints the queued text when nonempty and clears it; otherwise it prints the current glyph. Each print adds a newline.
- Saved grids start at zero. Load restores all nine cells. Mask combines the current and saved grids with bitwise OR. XOR uses the selected saved grid; `gxc` XORs the current grid with itself, clearing it.
- `gg` reads exactly nine binary digits from one input line.
- Back counts characters in preprocessed code. At a `b` located at index `k`, `bN` resumes at `k - N + 1`, matching the legacy interpreter's final increment. A subtraction before index zero is rejected. `b0` has no effect.
- `dN` waits N whole seconds; `d0` has no effect.

## CLI and tooling

The interpreter command appends `.g9` when omitted. Convert handles one glyph at a time:

```sh
grid9 interpret script
grid9 convert encode a
grid9 convert decode 000000010
grid9 clean preprocessor_cache
```

`c` aliases `convert`; `cl` aliases `clean`. Logging uses [scorched](https://github.com/treymouledoux/scorched). `no_log` suppresses interpreter logging, but preprocessing and CLI logging currently do not consistently honor it. `verbosity` controls selected informational messages, not every diagnostic.

The current example directory contains `.g9` scripts and companion TOML files; the legacy examples subdirectory is absent.
