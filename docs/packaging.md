# Packaging Grid9

Grid9 uses cargo-packager 0.11. Its configuration is in `[package.metadata.packager]` in `Cargo.toml`; `build.rs` embeds all non-hidden files under `src/components/documentation` and `src/components/examples` into the executable. This includes HTML, CSS, images, fonts, `.g9` scripts, and companion TOML files. The Retro Gadgets and BleachBit integrations are not required runtime components and are not bundled.

## Building installers

Install the pinned packaging tool:

```sh
cargo install cargo-packager --version 0.11.8 --locked
```

Run the appropriate command on the target operating system from the repository root. The packaging hook builds Grid9 in release mode automatically. Output goes to `target/packages`.

If cargo-packager reports `Couldn't detect a valid configuration file`, first confirm you are in the Grid9 repository root. It can also report this when Cargo metadata fails because dependencies are missing. Run `cargo fetch --locked`, then retry. Use `cargo packager -vv` or `cargo metadata --format-version 1` to see the underlying error.

| Platform | Command | Output |
| --- | --- | --- |
| Windows | `cargo packager --release --formats nsis` | NSIS `.exe` installer |
| macOS (Apple Silicon) | `cargo packager --release --formats app,dmg`, then `python3 packaging/build_macos_pkg.py` | `.pkg` CLI installer, `Grid9.app`, and optional `.dmg` |
| Linux | `cargo packager --release --formats deb` | Debian/Ubuntu `.deb` package |

The Linux package currently targets Debian-compatible distributions; it is not a universal Linux installer. Use the standalone release executable on other distributions. The Debian package requires `libc6` and `xdg-utils` (for opening documentation).

CI builds Linux packages on Ubuntu 24.04. Those binaries require the build system's glibc baseline; rebuild on an older compatible distribution if you need to support older systems. This also applies to the standalone Linux executable.

The `Build installers` GitHub Actions workflow tests and packages Windows x64, Linux x64, and macOS Apple Silicon. Intel Macs are not supported. It runs manually, on `v*` tags, and on pull requests touching packaging or application files. CI also checks Windows PATH registration/removal and the CLI executable extracted from the macOS PKG. Each job uploads its installer as an Actions artifact; it does not publish releases automatically.

## Installation and command-line access

- Windows: run the installer. It installs `grid9.exe` for the current user under `%LOCALAPPDATA%\Grid9` by default and adds the selected installation directory to the user PATH. Setup broadcasts the environment change; open a new terminal to use `grid9`. Uninstallation removes only a PATH entry added by Grid9, preserving other entries and pre-existing configuration.
- macOS: run the `.pkg` installer. It installs the executable in `/usr/local/bin/grid9`, which is on the default macOS PATH, and the app in `/Applications/Grid9.app`. Installation requests administrator permission for those system locations. Opening the app in Finder initializes user data and opens the documentation. The optional DMG only installs the app by dragging; it does not configure command-line access. The executable installed by the PKG works independently of the app bundle.
- Linux: install the `.deb` with `sudo apt install ./Grid9_*.deb` (using the actual downloaded filename). The executable is installed in `/usr/bin`; `grid9` is available on PATH. The desktop entry opens the documentation.

The packages are unsigned by default. Public macOS distribution can use cargo-packager's signing and notarization configuration; Windows signing requires a signing certificate. No credentials are stored in this repository.

If macOS blocks the trusted downloaded `.pkg` installer, remove its quarantine attribute, then open it. Replace the path below with your downloaded `.pkg` filename:

```sh
xattr -d com.apple.quarantine "$HOME/Downloads/Grid9_2026.1.0_arm64.pkg"
open "$HOME/Downloads/Grid9_2026.1.0_arm64.pkg"
```

Follow the installer prompts to install the app and terminal command. A `No such xattr: com.apple.quarantine` message means the installer has no quarantine attribute to remove. This removes download quarantine; it does not sign or notarize the build.

## User data layout

`file_man.rs` resolves a data directory for the account executing Grid9:

| Platform | Default directory |
| --- | --- |
| Windows | `%APPDATA%\Grid9` |
| macOS | `~/Library/Application Support/Grid9` |
| Linux | `$XDG_DATA_HOME/Grid9`, or `~/.local/share/Grid9` when unset |

Each directory contains:

```text
Grid9/
  documentation/         HTML, CSS, fonts, and images
  examples/              .g9 scripts and .toml configurations
  preprocessor_cache/    empty after initial setup
  logs/                  empty after initial setup
  .components-revision   fingerprint of bundled component contents
```

Windows setup runs `grid9 setup` before installing the executable. macOS and Linux initialize the layout on the first execution, before logging or preprocessing. Setup creates the cache and log directories without populating them. Normal execution subsequently creates log/cache entries as needed. Each additional user receives their own layout on first run, including when the binary is installed for the whole machine. Running a setup command as root initializes root's data directory, not another user's.

`grid9 setup` is idempotent. Missing bundled files are restored. A changed component fingerprint refreshes the shipped docs and examples, including changes made without a version bump. Files with shipped names are replaced on such an update; keep personal scripts under distinct names. Extra files, existing logs, and cached scripts are retained.

Set `GRID9_DATA_DIR` to override the data directory, useful for portable installations and isolated packaging checks. Debug builds still read docs/examples from the source tree; release builds use the installed data directory.

To check a build manually:

```sh
grid9 setup
grid9 interpret --example example1
grid9 documentation
```
