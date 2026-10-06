"""Install and remove real packages on disposable hosts only.

Requires --system because these checks change system installation locations.
Every subprocess has a timeout; pre-existing Grid9 installations are refused.
"""

import argparse
import ctypes
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import tempfile

from check_runtime import PROJECT, check_runtime, run


def one_package(extension):
    paths = list((PROJECT / "target/packages").glob(f"*.{extension}"))
    if len(paths) != 1:
        raise RuntimeError(f"Expected one .{extension} package, found {paths}")
    return paths[0]


def absent(*paths):
    for path in paths:
        if path.exists() or path.is_symlink():
            raise RuntimeError(
                f"Refusing to overwrite an existing installation: {path}"
            )


def runtime(binary, root, name):
    directory = root / name
    directory.mkdir()
    check_runtime(binary, directory)


def check_linux(root):
    package = one_package("deb")
    name = run(["dpkg-deb", "-f", package, "Package"]).stdout.strip()
    status = run(["dpkg-query", "-W", "-f=${Status}", name], check=False)
    if status.returncode == 0 and status.stdout.strip() == "install ok installed":
        raise RuntimeError(f"Refusing to replace installed package {name}")
    binary = Path("/usr/bin/grid9")
    absent(binary)
    architecture = run(["dpkg", "--print-architecture"]).stdout.strip()
    assert architecture in {"amd64", "arm64"}, f"Unsupported test host: {architecture}"
    assert (
        run(["dpkg-deb", "-f", package, "Architecture"]).stdout.strip() == architecture
    )
    dependencies = run(["dpkg-deb", "-f", package, "Depends"]).stdout
    assert "libc6 (>= 2.35)" in dependencies and "libgcc-s1" in dependencies
    sudo = [] if os.geteuid() == 0 else ["sudo", "-n"]
    environment = dict(os.environ, DEBIAN_FRONTEND="noninteractive")
    run([*sudo, "apt-get", "update", "-qq"], env=environment, timeout=300)
    account = f"grid9-ci-{os.getpid()}"
    account_home = root / "test account home ü"
    root.chmod(0o755)
    run(
        [
            *sudo,
            "useradd",
            "--system",
            "--user-group",
            "--no-create-home",
            "--no-log-init",
            "--home-dir",
            account_home,
            account,
        ],
        timeout=120,
    )
    run([*sudo, "mkdir", "-p", account_home])
    run([*sudo, "chown", account, account_home])
    run([*sudo, "chmod", "755", account_home])
    account_uid = run(["id", "-u", account]).stdout.strip()
    default_data = account_home / ".local/share/Grid9"

    def user_cli(*args):
        return run(
            [
                *sudo,
                "runuser",
                "-u",
                account,
                "--",
                "env",
                "-u",
                "GRID9_DATA_DIR",
                "-u",
                "XDG_DATA_HOME",
                binary,
                *args,
            ]
        )

    installed = False
    try:
        control = root / "deb-control"
        run(["dpkg-deb", "--control", package, control])
        assert (control / "postrm").read_bytes() == (
            PROJECT / "packaging/linux-postrm.sh"
        ).read_bytes()
        assert os.access(control / "postrm", os.X_OK)
        for iteration in range(2):
            run(
                [*sudo, "apt-get", "install", "-y", "--reinstall", package],
                env=environment,
                timeout=300,
            )
            installed = True
            assert binary.is_file() and os.access(binary, os.X_OK)
            assert Path(shutil.which("grid9")).resolve() == binary.resolve()
            desktop_files = [
                Path(p)
                for p in run(["dpkg-query", "-L", name]).stdout.splitlines()
                if p.endswith(".desktop")
            ]
            assert desktop_files, "Missing desktop entry"
            assert any("documentation" in p.read_text() for p in desktop_files)
            runtime(binary, root, f"linux-{iteration}")
            user_cli("version")
            for component in ["documentation", "logs", "preprocessor_cache"]:
                assert (default_data / component).is_dir()
            personal_default = default_data / "examples/personal.g9"
            run(
                [
                    *sudo,
                    "runuser",
                    "-u",
                    account,
                    "--",
                    "sh",
                    "-c",
                    'printf f7p > "$1"',
                    "fixture",
                    personal_default,
                ]
            )
            # Reinstall over an existing package, retaining user files.
            sentinel = root / f"linux-{iteration}/user data ü/examples/personal.g9"
            run(
                [*sudo, "apt-get", "install", "-y", "--reinstall", package],
                env=environment,
                timeout=300,
            )
            assert sentinel.read_text() == "f7p"
            assert personal_default.read_text() == "f7p"
            assert (
                default_data / "documentation/index.html"
            ).is_file(), "Upgrade cleaned user data"
            action = "remove" if iteration == 0 else "purge"
            run(
                [
                    *sudo,
                    "env",
                    "-u",
                    "GRID9_DATA_DIR",
                    "-u",
                    "XDG_DATA_HOME",
                    f"SUDO_USER={account}",
                    f"SUDO_UID={account_uid}",
                    "apt-get",
                    action,
                    "-y",
                    name,
                ],
                env=environment,
                timeout=120,
            )
            installed = False
            assert not binary.exists()
            assert all(not p.exists() for p in desktop_files)
            assert sentinel.exists(), "Package removal deleted a custom data directory"
            assert not default_data.exists(), "Removal left the Grid9 data folder"
            # Reinstall regenerates bundled resources, not deleted personal scripts.
            run(
                [*sudo, "apt-get", "install", "-y", package],
                env=environment,
                timeout=300,
            )
            installed = True
            user_cli("version")
            assert (default_data / "documentation/index.html").is_file()
            assert not personal_default.exists()
            run(
                [
                    *sudo,
                    "env",
                    "-u",
                    "SUDO_USER",
                    "-u",
                    "SUDO_UID",
                    "apt-get",
                    "purge",
                    "-y",
                    name,
                ],
                env=environment,
                timeout=120,
            )
            installed = False
            assert (
                default_data / "documentation/index.html"
            ).exists(), "Unverified removal deleted user data"
    finally:
        try:
            if installed:
                run(
                    [
                        *sudo,
                        "env",
                        "-u",
                        "SUDO_USER",
                        "-u",
                        "SUDO_UID",
                        "apt-get",
                        "purge",
                        "-y",
                        name,
                    ],
                    env=environment,
                    check=False,
                    timeout=120,
                )
        finally:
            run([*sudo, "userdel", account], check=False, timeout=120)
            run([*sudo, "rm", "-rf", "--", account_home], check=False)


def check_macos(root):
    package = one_package("pkg")
    app = Path("/Applications/Grid9.app")
    gui = Path("/Applications/Uninstall Grid9.app")
    cli = Path("/usr/local/bin/grid9")
    uninstall = Path("/usr/local/bin/grid9-uninstall")
    data = Path.home() / "Library/Application Support/Grid9"
    receipt = "com.treymouledoux.grid9.installer"
    absent(app, gui, cli, uninstall, data)
    assert run(["pkgutil", "--pkg-info", receipt], check=False).returncode != 0
    installed = False
    try:
        for iteration in range(2):
            run(
                ["sudo", "-n", "installer", "-pkg", package, "-target", "/"],
                timeout=180,
            )
            installed = True
            assert cli.is_symlink() and os.readlink(cli) == str(
                app / "Contents/MacOS/grid9"
            )
            assert Path(shutil.which("grid9")).resolve() == cli.resolve()
            assert os.access(uninstall, os.X_OK)
            assert (
                uninstall.read_bytes()
                == (PROJECT / "packaging/macos-uninstall.sh").read_bytes()
            )
            with (gui / "Contents/Info.plist").open("rb") as file:
                assert (
                    plistlib.load(file)["CFBundleIdentifier"]
                    == "com.treymouledoux.grid9.uninstaller"
                )
            run(["codesign", "--verify", "--strict", gui])
            run(["pkgutil", "--pkg-info", receipt])
            runtime(cli, root, f"macos-{iteration}")
            environment = dict(os.environ)
            environment.pop("GRID9_DATA_DIR", None)
            run([cli, "version"], env=environment)
            assert data.stat().st_uid == os.getuid(), "User data is owned by root"
            personal = data / "examples/personal.g9"
            personal.write_text("f7p")
            run(
                ["sudo", "-n", "installer", "-pkg", package, "-target", "/"],
                timeout=180,
            )
            assert personal.read_text() == "f7p", "Upgrade removed personal files"
            if iteration == 0:
                # A conflicting terminal command must not trigger partial removal.
                run(["sudo", "-n", "ln", "-sfn", "/usr/bin/true", cli])
                failure = run(["sudo", "-n", uninstall], check=False)
                assert failure.returncode != 0 and app.exists() and personal.exists()
                run(["sudo", "-n", "ln", "-sfn", app / "Contents/MacOS/grid9", cli])
            else:
                # Cleanup must also work after Finder has already removed the app.
                run(["sudo", "-n", "rm", "-rf", app])
            run(["sudo", "-n", uninstall, "--user", os.environ["USER"]])
            installed = False
            absent(app, gui, cli, uninstall, data)
            assert run(["pkgutil", "--pkg-info", receipt], check=False).returncode != 0
            assert (
                root / f"macos-{iteration}/user data ü/examples/personal.g9"
            ).exists(), "Removed a custom data directory"
    finally:
        if installed and uninstall.exists():
            # Restore the expected symlink if an assertion interrupted that test.
            run(
                ["sudo", "-n", "ln", "-sfn", app / "Contents/MacOS/grid9", cli],
                check=False,
            )
            run(["sudo", "-n", uninstall], check=False)

    # DMG installation remains app-only; inspect and execute its actual payload.
    mount = root / "mounted-dmg"
    mount.mkdir()
    # cargo-packager embeds our GPL license in the image; hdiutil asks for
    # acceptance on stdin even in a headless CI session.
    run(
        [
            "hdiutil",
            "attach",
            one_package("dmg"),
            "-readonly",
            "-nobrowse",
            "-mountpoint",
            mount,
        ],
        input="Y\n",
        timeout=120,
    )
    try:
        runtime(mount / "Grid9.app/Contents/MacOS/grid9", root, "dmg")
    finally:
        run(["hdiutil", "detach", mount], timeout=60)


def windows_resource(path, resource_type):
    from ctypes import wintypes

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
    kernel.LoadLibraryExW.restype = wintypes.HMODULE
    kernel.FindResourceW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p]
    kernel.FindResourceW.restype = wintypes.HANDLE
    kernel.SizeofResource.argtypes = [wintypes.HMODULE, wintypes.HANDLE]
    kernel.SizeofResource.restype = wintypes.DWORD
    kernel.LoadResource.argtypes = [wintypes.HMODULE, wintypes.HANDLE]
    kernel.LoadResource.restype = wintypes.HANDLE
    kernel.LockResource.argtypes = [wintypes.HANDLE]
    kernel.LockResource.restype = ctypes.c_void_p
    kernel.FreeLibrary.argtypes = [wintypes.HMODULE]
    module = kernel.LoadLibraryExW(str(path), None, 2)
    assert module, f"Cannot load PE resources: {path}"
    try:
        resource = kernel.FindResourceW(module, 1, resource_type)
        assert resource, f"Missing PE resource {resource_type}: {path}"
        return ctypes.string_at(
            kernel.LockResource(kernel.LoadResource(module, resource)),
            kernel.SizeofResource(module, resource),
        )
    finally:
        kernel.FreeLibrary(module)


def nsis_command(executable, directory, *, uninstall=False):
    # NSIS requires /D= and _?= last and unquoted, even with spaces.
    # Pass the raw command line directly to CreateProcess, never a shell.
    prefix = subprocess.list2cmdline([str(executable), "/S"])
    return prefix + (" _?=" if uninstall else " /D=") + str(directory)


def check_windows(root):
    import winreg
    import xml.etree.ElementTree as ET

    installer = one_package("exe")
    manifest = ET.fromstring(windows_resource(installer, 24))
    levels = [
        element.attrib["level"]
        for element in manifest.iter()
        if element.tag.endswith("requestedExecutionLevel")
    ]
    assert levels == ["requireAdministrator"], f"Setup must request elevation: {levels}"
    install_dir = root / "installed program ü"
    data = root / "installer data"
    environment = dict(os.environ, GRID9_DATA_DIR=str(data))
    binary = install_dir / "grid9.exe"
    owner_path = r"Software\Trey Mouledoux\Grid9"
    with winreg.CreateKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
        try:
            original_path = winreg.QueryValueEx(key, "Path")
        except FileNotFoundError:
            original_path = None
    with winreg.CreateKey(winreg.HKEY_CURRENT_USER, owner_path) as key:
        try:
            original_owner = winreg.QueryValueEx(key, "PathAddedByGrid9")
        except FileNotFoundError:
            original_owner = None
    if original_owner:
        raise RuntimeError("Refusing to alter an existing Grid9 PATH registration")
    baseline = os.environ["SystemRoot"] + r"\System32"

    def read_path():
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
            return winreg.QueryValueEx(key, "Path")[0]

    def matching_entries():
        return [
            p
            for p in read_path().split(";")
            if p
            and Path(os.path.expandvars(p.strip('"'))).resolve()
            == install_dir.resolve()
        ]

    installed = False
    try:
        with winreg.OpenKey(
            winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_SET_VALUE
        ) as key:
            winreg.SetValueEx(key, "Path", 0, winreg.REG_EXPAND_SZ, baseline)
        for iteration in range(2):
            run(nsis_command(installer, install_dir), env=environment, timeout=120)
            installed = True
            assert binary.is_file()
            uninstall_manifest = ET.fromstring(
                windows_resource(install_dir / "uninstall.exe", 24)
            )
            uninstall_levels = [
                element.attrib["level"]
                for element in uninstall_manifest.iter()
                if element.tag.endswith("requestedExecutionLevel")
            ]
            assert uninstall_levels == [
                "requireAdministrator"
            ], f"Uninstaller must request elevation: {uninstall_levels}"
            assert windows_resource(binary, 14), "Missing executable icon"
            assert len(matching_entries()) == 1 and baseline in read_path().split(";")
            assert (
                Path(shutil.which("grid9", path=read_path())).resolve()
                == binary.resolve()
            )
            assert (
                data / "documentation/index.html"
            ).is_file(), "Setup did not provision resources"
            runtime(binary, root, f"windows-{iteration}")
            personal = data / "examples/personal.g9"
            personal.write_text("f7p")
            run(nsis_command(installer, install_dir), env=environment, timeout=120)
            assert len(matching_entries()) == 1 and personal.read_text() == "f7p"
            run(
                nsis_command(
                    install_dir / "uninstall.exe", install_dir, uninstall=True
                ),
                env=environment,
                timeout=120,
            )
            installed = False
            assert not binary.exists() and not (install_dir / "grid9-path.ps1").exists()
            assert read_path() == baseline, "Uninstall changed unrelated PATH entries"
            assert personal.exists(), "Uninstall removed user data"
    finally:
        try:
            if installed and (install_dir / "uninstall.exe").exists():
                run(
                    nsis_command(
                        install_dir / "uninstall.exe", install_dir, uninstall=True
                    ),
                    env=environment,
                    check=False,
                    timeout=120,
                )
        finally:
            with winreg.OpenKey(
                winreg.HKEY_CURRENT_USER, "Environment", 0, winreg.KEY_SET_VALUE
            ) as key:
                if original_path is None:
                    try:
                        winreg.DeleteValue(key, "Path")
                    except FileNotFoundError:
                        pass
                else:
                    winreg.SetValueEx(
                        key, "Path", 0, original_path[1], original_path[0]
                    )
            with winreg.CreateKey(winreg.HKEY_CURRENT_USER, owner_path) as key:
                if original_owner is None:
                    try:
                        winreg.DeleteValue(key, "PathAddedByGrid9")
                    except FileNotFoundError:
                        pass
                else:
                    winreg.SetValueEx(
                        key, "PathAddedByGrid9", 0, original_owner[1], original_owner[0]
                    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--system",
        action="store_true",
        help="Allow installation/removal on this disposable host",
    )
    if not parser.parse_args().system:
        parser.error("--system is required; use only on a disposable test machine")
    with tempfile.TemporaryDirectory(prefix="grid9-package-check-") as temporary:
        root = Path(temporary)
        if sys.platform == "win32":
            check_windows(root)
        elif sys.platform == "darwin":
            check_macos(root)
        else:
            check_linux(root)
    print("Install, upgrade, uninstall, and reinstall checks passed.")


if __name__ == "__main__":
    main()
