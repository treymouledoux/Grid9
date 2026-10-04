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
            import winreg
            # An existing Grid9 key without PATH ownership must also work.
            with winreg.CreateKey(winreg.HKEY_CURRENT_USER, r"Software\Trey Mouledoux\Grid9"):
                pass
            installer = next((project / "target/packages").glob("*.exe"))
            install_dir = root / "program"
            subprocess.run(
                [str(installer), "/S", f"/D={install_dir}"],
                env=environment, check=True, timeout=120,
            )
            binary = install_dir / "grid9.exe"
            assert binary.is_file(), f"Installer did not create {binary}"
            # Verify the app icon is embedded in the installed PE executable.
            import ctypes
            from ctypes import wintypes
            kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
            kernel32.LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
            kernel32.LoadLibraryExW.restype = wintypes.HMODULE
            kernel32.FindResourceW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p]
            kernel32.FindResourceW.restype = wintypes.HANDLE
            kernel32.FreeLibrary.argtypes = [wintypes.HMODULE]
            module = kernel32.LoadLibraryExW(str(binary), None, 2)  # LOAD_LIBRARY_AS_DATAFILE
            assert module, f"Could not load executable resources: {ctypes.get_last_error()}"
            try:
                assert kernel32.FindResourceW(module, 1, 14), "Installed grid9.exe has no app icon (RT_GROUP_ICON)"
            finally:
                kernel32.FreeLibrary(module)
            import winreg
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                user_path, _ = winreg.QueryValueEx(key, "Path")
            assert os.environ["SystemRoot"] + r"\System32" in user_path.split(';'), "Installer changed the existing PATH entry"
            def path_directory(entry):
                return Path(os.path.expandvars(entry.strip().strip('"'))).resolve()

            matches = [p for p in user_path.split(';') if p and path_directory(p) == install_dir.resolve()]
            assert len(matches) == 1, f"Expected one PATH entry for {install_dir}; user PATH: {user_path!r}"
            environment["PATH"] = os.path.expandvars(user_path) + os.pathsep + environment["PATH"]
            resolved = shutil.which("grid9", path=environment["PATH"])
            assert resolved and Path(resolved).resolve() == binary.resolve(), f"grid9 resolved to {resolved!r}; expected {binary}"
            # Verify install-time provisioning before running the installed app.
            assert (data / "documentation/index.html").is_file()
            for directory in ["logs", "preprocessor_cache"]:
                assert (data / directory).is_dir()
                assert not list((data / directory).iterdir()), directory
        elif sys.platform == "darwin":
            package = next((project / "target/packages").glob("*.pkg"))
            extracted = root / "package"
            subprocess.run(["pkgutil", "--expand-full", str(package), str(extracted)], check=True)
            import xml.etree.ElementTree as ET
            info = ET.parse(extracted / "PackageInfo").getroot()
            assert info.get("install-location") == "/"
            assert info.get("relocatable") == "false"
            assert not info.findall("relocate/bundle"), "PKG must not relocate Grid9.app to another copy"
            cli = extracted / "Payload/usr/local/bin/grid9"
            assert cli.is_symlink()
            assert os.readlink(cli) == "/Applications/Grid9.app/Contents/MacOS/grid9"
            # The absolute link targets the real install location; run the
            # bundled executable directly while inspecting an extracted PKG.
            binary = extracted / "Payload/Applications/Grid9.app/Contents/MacOS/grid9"
            assert binary.is_file()
            uninstaller = extracted / "Payload/usr/local/bin/grid9-uninstall"
            assert uninstaller.read_bytes() == (project / "packaging/macos-uninstall.sh").read_bytes()
            assert os.access(uninstaller, os.X_OK)
            subprocess.run(["/bin/sh", "-n", str(uninstaller)], check=True)
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

        print(run("version").strip())
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
        # Also exercise initialization when the first command runs an example.
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
            assert not any(p and path_directory(p) == install_dir.resolve() for p in user_path.split(';')), f"Uninstall left {install_dir} in user PATH: {user_path!r}"
            assert user_path == os.environ["SystemRoot"] + r"\System32", f"Uninstall changed the existing PATH: {user_path!r}"
            # Uninstall leaves the key but removes PathAddedByGrid9. Reinstall
            # must succeed in exactly the state shown by the reported failure.
            subprocess.run([str(installer), "/S", f"/D={install_dir}"],
                           env=environment, check=True, timeout=120)
            assert binary.is_file(), "Reinstallation failed"
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                user_path, _ = winreg.QueryValueEx(key, "Path")
            assert sum(bool(p) and path_directory(p) == install_dir.resolve() for p in user_path.split(';')) == 1, f"Reinstall PATH incorrect: {user_path!r}"
            subprocess.run([str(install_dir / "uninstall.exe"), "/S", f"_?={install_dir}"],
                           env=environment, check=True, timeout=120)
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                user_path, _ = winreg.QueryValueEx(key, "Path")
            assert user_path == os.environ["SystemRoot"] + r"\System32", f"Second uninstall changed PATH: {user_path!r}"


if __name__ == "__main__":
    if sys.platform == "win32":
        import winreg
        # Exercise the single-entry PATH regression, preserving the host's
        # original value even if installation or verification fails.
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_READ | winreg.KEY_SET_VALUE) as key:
            try:
                original_path = winreg.QueryValueEx(key, "Path")
            except FileNotFoundError:
                original_path = None
            try:
                winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, os.environ["SystemRoot"] + r"\System32")
                check()
            finally:
                if original_path is None:
                    winreg.DeleteValue(key, "Path")
                else:
                    winreg.SetValueEx(key, "Path", 0, original_path[1], original_path[0])
    else:
        check()
