"""Stage a draft release only after every platform has passed installation checks."""

import hashlib
import os
from pathlib import Path
import subprocess
import tomllib

from preflight import validate_tag


def checksums(paths):
    return "".join(
        f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n"
        for path in sorted(paths)
    )


def validate_bundles(paths):
    expected_counts = {".exe": 1, ".deb": 2, ".dmg": 1, ".pkg": 1}
    counts = {suffix: sum(p.suffix == suffix for p in paths) for suffix in expected_counts}
    if counts != expected_counts:
        raise RuntimeError("Missing or duplicate platform bundles; refusing to create a partial release")
    architectures = [
        subprocess.check_output(
            ["dpkg-deb", "-f", str(path), "Architecture"], text=True, timeout=30,
        ).strip()
        for path in paths if path.suffix == ".deb"
    ]
    if sorted(architectures) != ["amd64", "arm64"]:
        raise RuntimeError(f"Expected one amd64 and one arm64 Debian package, found {architectures}")


def main():
    version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
    tag = os.environ["GITHUB_REF_NAME"]
    validate_tag(tag, version)
    paths = [p for p in Path("dist").iterdir() if p.suffix in {".exe", ".deb", ".dmg", ".pkg"}]
    validate_bundles(paths)
    checksum_file = Path("dist/SHA256SUMS")
    checksum_file.write_text(checksums(paths), encoding="utf-8")
    subprocess.run([
        "gh", "release", "create", tag, "--draft", "--verify-tag",
        "--title", f"Grid9 {version}", "--generate-notes",
        *map(str, sorted(paths)), str(checksum_file),
    ], check=True)


if __name__ == "__main__":
    main()
