<img src=".github/assets/banner.png">

Grid9 is an esoteric interpreted language implemented in Rust, based on a 3x3 grid of zeros and ones.

The current implementation is on `main`. For changes from the legacy Nim implementation, see the [feature changes](docs/feature_changes.md) and [migration guide](docs/migration.md).

## Installation

### Binaries

See the [releases page](https://github.com/treymouledoux/Grid9/releases) for available binaries. Check the release version: older releases use the legacy Nim implementation.

### Build from source

Install Rust and Cargo, then run these commands from the repository root:

```sh
cargo build --release
cargo run --release -- version
cargo run --release -- interpret src/components/examples/example1.g9
```

The executable is `target/release/grid9` (`grid9.exe` on Windows). Scripts accept an optional TOML file with the same base name, such as `example1.toml` alongside `example1.g9`.

For development checks, run `cargo test` and `cargo clippy --all-targets -- -D warnings`.

## Documentation

Read the [online language documentation](https://treymouledoux.github.io/Grid9/) or open `src/components/documentation/index.html` directly from this checkout. An installation with the documentation component can open it with `grid9 d` or `grid9 documentation`.

The `--example` option and local documentation command expect component files at their configured locations; building the executable alone does not install them. For debug builds, those paths are relative to the working directory and expect execution from `target/debug`.

## Contributing

Pull requests are welcome. For major changes, please [open an issue](https://github.com/treymouledoux/Grid9/issues/new) first to discuss what you would like to change. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[GPL-3.0](LICENSE).

## Credits

This project was inspired by [Brainfuck](https://esolangs.org/wiki/Brainfuck).
