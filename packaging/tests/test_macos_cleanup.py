"""Exercise the uninstaller with system paths and privileged commands replaced."""

import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_runtime import PROJECT, run


@unittest.skipIf(sys.platform == "win32", "Requires a POSIX shell")
class MacCleanupTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="grid9-macos-cleanup-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.home = self.root / "user home ü"
        self.data = self.home / "Library/Application Support/Grid9"
        self.data.mkdir(parents=True)
        (self.data / "personal.g9").write_text("keep")
        self.app = self.root / "Grid9.app"
        self.gui = self.root / "Uninstall Grid9.app"
        self.cli = self.root / "grid9"
        self.uninstaller = self.root / "grid9-uninstall"
        self.app.mkdir()
        self.gui.mkdir()
        self.cli.symlink_to(self.app / "Contents/MacOS/grid9")
        self.command("uname", "echo Darwin")
        self.command(
            "id",
            'case "$*" in "-u") echo 0 ;; "-u root") echo 0 ;; *) echo 501 ;; esac',
        )
        self.command("dscl", 'printf "NFSHomeDirectory: %s\\n" "$TEST_HOME"')
        self.command(
            "PlistBuddy",
            'case "$3" in *"Uninstall Grid9.app"*) echo com.treymouledoux.grid9.uninstaller ;; *) echo com.treymouledoux.grid9 ;; esac',
        )
        self.command("pkgutil", "exit 1")
        # Record the requested identity, then run only the redirected fixture cleanup.
        self.command(
            "sudo",
            'printf "%s\\n" "$*" > "$TEST_SUDO_LOG"\n[ "$1" = -u ] && [ "$2" = tester ] && [ "$3" = -- ] || exit 1\nshift 3\nexec "$@"',
        )
        source = (PROJECT / "packaging/macos-uninstall.sh").read_text()
        replacements = {
            "/Applications/Uninstall Grid9.app": str(self.gui),
            "/Applications/Grid9.app": str(self.app),
            "/usr/local/bin/grid9-uninstall": str(self.uninstaller),
            "/usr/local/bin/grid9": str(self.cli),
            "/usr/bin/dscl": str(self.bin / "dscl"),
            "/usr/libexec/PlistBuddy": str(self.bin / "PlistBuddy"),
            "/usr/bin/sudo": str(self.bin / "sudo"),
            "/usr/sbin/pkgutil": str(self.bin / "pkgutil"),
        }
        for original, replacement in replacements.items():
            source = source.replace(original, replacement)
        self.uninstaller.write_text(source)
        self.env = dict(
            os.environ,
            PATH=f"{self.bin}:/usr/bin:/bin",
            SUDO_USER="tester",
            TEST_HOME=str(self.home),
            TEST_SUDO_LOG=str(self.root / "sudo.log"),
        )

    def command(self, name, body):
        command = self.bin / name
        command.write_text(f"#!/bin/sh\n{body}\n")
        command.chmod(0o755)

    def uninstall(self, *args):
        return run(["/bin/sh", self.uninstaller, *args], env=self.env, check=False)

    def test_removes_data_using_selected_user_and_handles_missing_app(self):
        self.app.rmdir()
        result = self.uninstall("--user", "tester")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.data.exists())
        self.assertFalse(self.cli.is_symlink())
        self.assertFalse(self.gui.exists())
        self.assertFalse(self.uninstaller.exists())
        self.assertTrue(
            (self.root / "sudo.log").read_text().startswith("-u tester -- /bin/sh")
        )

    def test_preserves_symlinked_data(self):
        preserved = self.data.with_name("preserved")
        self.data.rename(preserved)
        self.data.symlink_to(preserved, target_is_directory=True)
        result = self.uninstall()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((preserved / "personal.g9").exists())
        self.assertTrue(self.data.is_symlink())

    def test_conflicting_cli_is_rejected_before_cleanup(self):
        self.cli.unlink()
        self.cli.write_text("unrelated command")
        self.assertNotEqual(self.uninstall().returncode, 0)
        self.assertTrue(self.app.exists())
        self.assertTrue((self.data / "personal.g9").exists())
        self.assertFalse((self.root / "sudo.log").exists())

    def test_failed_user_cleanup_leaves_system_installation_intact(self):
        self.command("sudo", "exit 1")
        self.assertNotEqual(self.uninstall().returncode, 0)
        self.assertTrue(self.app.exists())
        self.assertTrue(self.cli.is_symlink())
        self.assertTrue(self.uninstaller.exists())

    def test_invalid_accounts_and_failed_home_lookup_are_rejected(self):
        for account in ["root", "-tester", "bad/name"]:
            with self.subTest(account=account):
                self.assertNotEqual(self.uninstall("--user", account).returncode, 0)
        self.command("dscl", "exit 1")
        self.assertNotEqual(self.uninstall().returncode, 0)
        self.assertTrue((self.data / "personal.g9").exists())


if __name__ == "__main__":
    unittest.main()
