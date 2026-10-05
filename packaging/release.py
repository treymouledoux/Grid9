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


def main():
    version = tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"]
    tag = os.environ["GITHUB_REF_NAME"]
    validate_tag(tag, version)
    paths = [p for p in Path("dist").iterdir() if p.suffix in {".exe", ".deb", ".dmg", ".pkg"}]
    if {p.suffix for p in paths} != {".exe", ".deb", ".dmg", ".pkg"}:
        raise RuntimeError("Missing platform bundles; refusing to create a partial release")
    checksum_file = Path("dist/SHA256SUMS")
    checksum_file.write_text(checksums(paths), encoding="utf-8")
    subprocess.run([
        "gh", "release", "create", tag, "--draft", "--verify-tag",
        "--title", f"Grid9 {version}", "--generate-notes",
        *map(str, sorted(paths)), str(checksum_file),
    ], check=True)


if __name__ == "__main__":
    main()
