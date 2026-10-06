"""Embed Grid9's removal hook without changing the pinned packager."""
from pathlib import Path
import os
import tempfile

from check_runtime import PROJECT, run


def finalize(package):
    package = Path(package).resolve()
    with tempfile.TemporaryDirectory(prefix="grid9-deb-") as temporary:
        root = Path(temporary) / "package"
        run(["dpkg-deb", "--raw-extract", package, root], timeout=120)
        hook = root / "DEBIAN/postrm"
        if hook.exists():
            raise RuntimeError("Refusing to overwrite an existing postrm hook")
        hook.write_bytes((PROJECT / "packaging/linux-postrm.sh").read_bytes())
        hook.chmod(0o755)
        output = Path(temporary) / package.name
        run(["dpkg-deb", "--build", "--root-owner-group", root, output], timeout=120)
        # Replace atomically even when the temporary directory is on another filesystem.
        staged = package.with_suffix(".deb.tmp")
        try:
            staged.write_bytes(output.read_bytes())
            os.replace(staged, package)
        finally:
            staged.unlink(missing_ok=True)


def main():
    packages = list((PROJECT / "target/packages").glob("*.deb"))
    if len(packages) != 1:
        raise RuntimeError(f"Expected one Debian package, found {packages}")
    finalize(packages[0])


if __name__ == "__main__":
    main()
