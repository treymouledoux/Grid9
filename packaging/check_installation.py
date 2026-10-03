"""Verify the installed layout using the packaged executable on each CI host."""

import os
from pathlib import Path
import subprocess
import shutil
import sys
import tempfile


def check():
    project = Path(__file__).resolve().parent.parent
    with tempfile.TemporaryDirectory(prefix="grid9-package-check-") as temporary:
        root = Path(temporary)
        data = root / "data"
        environment = dict(os.environ, GRID9_DATA_DIR=str(data))
        if sys.platform == "win32":
            installer = next((project / "target/packages").glob("*.exe"))
            install_dir = root / "program"
            subprocess.run(
                [str(installer), "/S", f"/D={install_dir}"],
                env=environment, check=True, timeout=120,
            )
            binary = install_dir / "grid9.exe"
            import winreg
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                user_path, _ = winreg.QueryValueEx(key, "Path")
            assert sum(p.lower() == str(install_dir).lower() for p in user_path.split(';')) == 1
            environment["PATH"] = user_path + os.pathsep + environment["PATH"]
            assert Path(shutil.which("grid9", path=environment["PATH"])).resolve() == binary.resolve()
            # Verify install-time provisioning before running the installed app.
            assert (data / "documentation/index.html").is_file()
            for directory in ["logs", "preprocessor_cache"]:
                assert (data / directory).is_dir()
                assert not list((data / directory).iterdir()), directory
        elif sys.platform == "darwin":
            package = next((project / "target/packages").glob("*.pkg"))
            extracted = root / "package"
            subprocess.run(["pkgutil", "--expand-full", str(package), str(extracted)], check=True)
            binary = extracted / "Payload/usr/local/bin/grid9"
            assert binary.is_file()
            assert (extracted / "Payload/Applications/Grid9.app/Contents/MacOS/grid9").is_file()
        else:
            package = next((project / "target/packages").glob("*.deb"))
            extracted = root / "package"
            subprocess.run(["dpkg-deb", "--extract", str(package), str(extracted)], check=True)
            binary = extracted / "usr/bin/grid9"

        def run(*args):
            return subprocess.run(
                [str(binary), *args], env=environment, cwd=root,
                check=True, capture_output=True, text=True,
            ).stdout

        print(run("setup").strip())
        for directory in ["logs", "preprocessor_cache"]:
            assert (data / directory).is_dir()
            assert not list((data / directory).iterdir()), directory

        components = project / "src/components"
        count = 0
        for component in ["documentation", "examples"]:
            for source in (components / component).rglob("*"):
                relative = source.relative_to(components)
                if source.is_file() and not any(p.startswith(".") for p in relative.parts):
                    assert (data / relative).read_bytes() == source.read_bytes(), relative
                    count += 1
        # Also exercise automatic initialization without an explicit setup call.
        environment["GRID9_DATA_DIR"] = str(root / "first-run")
        assert "Hello world" in run("interpret", "--example", "example1")
        assert (root / "first-run/documentation/index.html").is_file()
        assert (root / "first-run/preprocessor_cache").is_dir()
        assert (root / "first-run/logs").is_dir()
        print(f"Verified {count} component files, empty initial cache/logs, and first-run example execution.")
        if sys.platform == "win32":
            # Verify uninstallation removes the entry owned by Grid9.
            subprocess.run([
                str(install_dir / "uninstall.exe"), "/S", f"_?={install_dir}",
            ], env=environment, check=True, timeout=120)
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                user_path, _ = winreg.QueryValueEx(key, "Path")
            assert str(install_dir).lower() not in [p.lower() for p in user_path.split(';')]


if __name__ == "__main__":
    check()
