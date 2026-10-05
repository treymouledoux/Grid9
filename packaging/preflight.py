"""Fail before generating installers if the tool or tag does not match."""

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tomllib

PROJECT = Path(__file__).resolve().parent.parent


def validate_tag(tag, version):
    if tag and tag != f"v{version}":
        raise ValueError(f"Release tag {tag!r} does not match Cargo version v{version}")


def main():
    version = tomllib.loads((PROJECT / "Cargo.toml").read_text())["package"]["version"]
    tag = os.environ.get("GITHUB_REF_NAME", "") if os.environ.get("GITHUB_REF_TYPE") == "tag" else ""
    validate_tag(tag, version)
    actual = subprocess.check_output(["cargo", "packager", "--version"], text=True).strip()
    expected = os.environ.get("PACKAGER_VERSION", "0.11.8")
    if actual.split()[-1] != expected:
        raise RuntimeError(f"Expected cargo-packager {expected}, found {actual!r}")
    # Never accidentally upload an older artifact restored with a build cache.
    shutil.rmtree(PROJECT / "target/packages", ignore_errors=True)
    if sys.platform == "win32":
        # This sidecar may be absent after restoring a Cargo build cache even
        # when build.rs itself is up to date. Always stage it before packaging.
        destination = PROJECT / "target/release/grid9.exe.path.ps1"
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(PROJECT / "packaging/windows-path.ps1", destination)
    print(f"Packaging Grid9 {version} with {actual}")


if __name__ == "__main__":
    main()
