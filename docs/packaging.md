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

The Linux package currently targets Debian-compatible distributions; it is not a universal Linux installer. Use the standalone release executable on other distributions. The Debian package requires `libc6 (>= 2.35)`, `libgcc-s1`, and `xdg-utils` (for opening documentation).

CI builds Linux packages on Ubuntu 22.04, with an explicit `libc6 (>= 2.35)` dependency and `libgcc-s1`. Native installation is tested on Ubuntu 22.04 and in an Ubuntu 24.04 container. Older glibc systems are not supported by these binaries; rebuild from source for another baseline. No standalone executable is uploaded by this workflow.

The `Build installers` workflow runs on relevant pushes to `main`, pull requests (including test-only changes), `v*` tags, and manual dispatch. Rust is pinned in `rust-toolchain.toml`; dependency resolution uses `Cargo.lock` and `--locked`.

### Validation order

1. **Before any bundles are generated**, Windows x64, Linux x64, and macOS ARM64 each run Rust formatting, Clippy with warnings denied, debug and release tests, Python packaging tests, and bounded release-executable smoke tests. Windows also exercises PATH ownership, duplicate handling, expandable entries, and installation-directory changes. Unix hosts check uninstaller shell syntax.
2. Packaging waits for **all three** source-check jobs. It restores the compiled packaging tool, verifies its version and any release tag, removes stale output, then generates bundles.
3. Disposable hosts perform real installation, same-version upgrade/reinstallation, removal, and clean reinstallation. Tests verify resources, personal-file preservation during upgrades, CLI behavior, and cleanup. Windows checks the unelevated installer manifest, PE icon, and PATH registration. macOS tests the PKG, receipt, terminal link, graphical-uninstaller bundle, and cleanup after deleting the main app; it also mounts and runs the DMG payload. Linux checks package dependencies, PATH, desktop entries, and removal on Ubuntu 22.04 and 24.04.
4. Each platform uploads its bundles only after its installation checks pass. A `v*` tag creates a **draft** GitHub Release with all platform bundles and `SHA256SUMS`, only after every platform succeeds. The tag must exactly match `v` plus the Cargo version. Review the draft before publishing; retries do not overwrite an existing release.

The tests do not automate clicking graphical installer/uninstaller dialogs, verify Developer ID notarization or SmartScreen reputation, or simulate upgrading from every historical release. Clean Windows installs are checked for `asInvoker`; CI's account is not a substitute for a manual standard-user desktop test. Installation tests require `python packaging/check_installation.py --system` and should run only on disposable hosts: they install and remove software at real system paths. Existing macOS/Linux installations and existing Windows PATH ownership are refused.

### Caching and speed

`Swatinem/rust-cache` reuses Cargo downloads and debug/release build outputs between validation and packaging jobs on each OS. A separate exact-key cache stores the installed cargo-packager executable and installation metadata under `~/.cache/grid9-packager`. Its key includes the runner image label, CPU architecture, packaging-tool version, and Rust toolchain file. On a hit, the expensive `cargo install` step is skipped; the executable version is still checked. The tool cache is saved immediately after installation, even if a later packaging check fails.

Caches accelerate compilation; they never skip tests. Changing code rebuilds the affected crates. Changing the toolchain or cargo-packager version intentionally causes a cache miss. The first run, cache eviction, and GitHub's branch-scoped cache access can still produce cold builds. Push builds on `main` populate caches available to later PRs. Older runs for the same PR are cancelled; tag builds are allowed to finish.

## Installation and command-line access

- Windows: run the per-user installer normally; new installers do not request administrator privileges. It installs `grid9.exe` under `%LOCALAPPDATA%\Grid9` by default and adds the selected directory to that user's PATH. Setup broadcasts the environment change; open a new terminal to use `grid9`. Uninstallation removes only a PATH entry owned by Grid9, preserving other entries, expandable variables, and pre-existing configuration. An older elevated uninstaller may still request elevation when removing an older installation.
- macOS: run the `.pkg` installer. It installs the app in `/Applications/Grid9.app` and a symlink at `/usr/local/bin/grid9`, which is on the default macOS PATH and points to the executable inside the app. Installation requests administrator permission for those system locations. Opening the app in Finder initializes user data and opens the documentation. The optional DMG only installs the app by dragging; it does not configure command-line access. Deleting or moving the app disables the terminal command; deleting it leaves a broken symlink.
- Linux: install the `.deb` with `sudo apt install ./Grid9_*.deb` (using the actual downloaded filename). The executable is installed in `/usr/bin`; `grid9` is available on PATH. The desktop entry opens the documentation.

The packages remain unsigned for public distribution; the graphical uninstaller has only a local ad-hoc signature, which is not Developer ID signing or notarization. Signing requires your Apple/Windows certificates and credentials and is not implied by a passing CI run. The draft-release gate leaves room to finish signing and verification before publishing. Public macOS distribution can use cargo-packager's signing and notarization configuration; Windows signing requires a signing certificate. No credentials are stored in this repository.

If macOS blocks the trusted downloaded `.pkg` installer, remove its quarantine attribute, then open it. Replace the path below with your downloaded `.pkg` filename:

```sh
xattr -d com.apple.quarantine "$HOME/Downloads/Grid9_2026.1.0_arm64.pkg"
open "$HOME/Downloads/Grid9_2026.1.0_arm64.pkg"
```

Follow the installer prompts to install the app and terminal command. A `No such xattr: com.apple.quarantine` message means the installer has no quarantine attribute to remove. This removes download quarantine; it does not sign or notarize the build.

### macOS uninstallation

The current PKG installs **Uninstall Grid9.app** in Applications and `/usr/local/bin/grid9-uninstall`. Open the graphical uninstaller, confirm removal of the app and user data, and approve the administrator prompt. The invoking user is captured before elevation so cleanup targets that user even when another administrator authorizes it. Nothing runs automatically when you drag Grid9 to Trash.

Alternatively, to purge the installed app, terminal integration, and your user data, run:

```sh
sudo /usr/local/bin/grid9-uninstall
```

This removes the app, terminal symlink, both uninstallers, package receipt, and the invoking user's entire `~/Library/Application Support/Grid9` folder. This includes documentation, examples, logs, parser cache, and any personal files stored there. It also works after you have already deleted the app. The uninstaller resolves the original user behind `sudo` rather than using root's home directory. Other users' data and custom directories selected with `GRID9_DATA_DIR` are left untouched; remove those separately if needed. Reinstall the updated PKG to replace an older uninstaller that preserved user data or a separate terminal executable used by older installers.

Dragging the app to Trash disables the terminal command but leaves its broken symlink, the uninstaller, the package receipt, and user data. Use the command above to purge these too. The uninstaller requests administrator access through `sudo` and removes itself when finished.

Older PKGs did not include the uninstaller; reinstall the current PKG first if `/usr/local/bin/grid9-uninstall` is missing. A DMG-only installation has no terminal integration or uninstaller: move `/Applications/Grid9.app` to Trash to remove it, and delete `~/Library/Application Support/Grid9` separately to purge its user data.

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

The Windows installer runs `grid9 version` before installing the executable to initialize the user data layout. macOS and Linux initialize the layout on the first execution, before logging or preprocessing. Initialization creates the cache and log directories without populating them. Commands that log or preprocess scripts subsequently create entries as needed. Each additional user receives their own layout on first run, including when the binary is installed for the whole machine. Running Grid9 as root initializes root's data directory, not another user's. The `--help` and `--version` flags exit before initialization; the `version` subcommand initializes normally.

Initialization is idempotent. Missing bundled files are restored. A changed component fingerprint refreshes the shipped docs and examples, including changes made without a version bump. Files with shipped names are replaced on such an update; keep personal scripts under distinct names. Extra files, existing logs, and cached scripts are retained.

Set `GRID9_DATA_DIR` to override the data directory, useful for portable installations and isolated packaging checks. Debug builds still read docs/examples from the source tree; release builds use the installed data directory.

To check a build manually:

```sh
grid9 version
grid9 interpret --example example1
grid9 documentation
```
