"""Wrap cargo-packager's app in a macOS installer with a CLI on PATH."""

from pathlib import Path
import json
import platform
import shutil
import subprocess
import tempfile


def build():
    project = Path(__file__).resolve().parent.parent
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--no-deps", "--offline", "--format-version", "1",
        "--manifest-path", str(project / "Cargo.toml"),
    ], text=True))
    version = next(p["version"] for p in metadata["packages"] if p["name"] == "grid9")
    packages = project / "target/packages"
    app = packages / "Grid9.app"
    if not app.is_dir():
        raise RuntimeError("Build Grid9.app with cargo packager --release --formats app first")
    with tempfile.TemporaryDirectory(prefix="grid9-pkg-") as temporary:
        payload = Path(temporary) / "payload"
        destination = payload / "Applications/Grid9.app"
        destination.parent.mkdir(parents=True)
        shutil.copytree(app, destination, symlinks=True)
        cli = payload / "usr/local/bin/grid9"
        cli.parent.mkdir(parents=True)
        # Install an actual executable rather than a link that breaks if the app
        # is relocated or deleted. Both copies contain the required components.
        shutil.copy2(app / "Contents/MacOS/grid9", cli)
        output = packages / f"Grid9_{version}_{platform.machine()}.pkg"
        subprocess.run([
            "pkgbuild", "--root", str(payload),
            "--identifier", "com.treymouledoux.grid9.installer",
            "--version", version, "--install-location", "/",
            "--ownership", "recommended", str(output),
        ], check=True)
        print(output)


if __name__ == "__main__":
    build()
