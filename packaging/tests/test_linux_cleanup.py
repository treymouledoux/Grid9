import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_runtime import PROJECT, run


@unittest.skipUnless(sys.platform.startswith("linux"), "Linux removal hook")
class CleanupTests(unittest.TestCase):
    def setUp(self):
        import pwd
        self.temporary = tempfile.TemporaryDirectory(prefix="grid9-cleanup-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.data = self.root / "data with spaces ü" / "Grid9"
        for component in ["documentation", "logs", "preprocessor_cache", "examples"]:
            directory = self.data / component
            directory.mkdir(parents=True)
            (directory / "fixture").write_text("keep")
        (self.data / ".components-revision").write_text("fixture")
        self.account = pwd.getpwnam("nobody") if os.geteuid() == 0 else pwd.getpwuid(os.getuid())
        if os.geteuid() == 0:
            self.root.chmod(0o755)
            for path in [self.root, *self.root.rglob("*")]:
                try:
                    os.chown(path, self.account.pw_uid, self.account.pw_gid)
                except OSError as error:
                    if error.errno == 22:
                        self.skipTest("Execution host cannot map a non-root UID")
                    raise
        self.env = dict(os.environ, SUDO_USER=self.account.pw_name,
                        SUDO_UID=str(self.account.pw_uid), XDG_DATA_HOME=str(self.data.parent))
        self.env.pop("GRID9_DATA_DIR", None)

    def hook(self, action, **environment):
        return run(["sh", PROJECT / "packaging/linux-postrm.sh", action],
                   env=dict(self.env, **environment))

    def test_remove_and_purge_delete_entire_data_folder(self):
        for action in ["remove", "purge", "purge"]:
            # Repopulate for each action, including unknown future components.
            self.data.mkdir(parents=True, exist_ok=True)
            (self.data / "personal-file").write_text("remove")
            if os.geteuid() == 0:
                os.chown(self.data, self.account.pw_uid, self.account.pw_gid)
            self.hook(action)
            self.assertFalse(self.data.exists())
            self.assertTrue(self.data.parent.exists())
            self.hook(action)  # Repeated cleanup is harmless.

    def test_upgrade_unverified_and_custom_directories_are_preserved(self):
        for action in ["upgrade", "failed-upgrade", "abort-install", "disappear"]:
            self.hook(action)
        self.hook("remove", SUDO_USER="")
        self.hook("remove", SUDO_UID="0")
        self.hook("remove", SUDO_UID="999999")
        self.hook("remove", GRID9_DATA_DIR=str(self.data))
        self.assertTrue((self.data / "documentation/fixture").exists())

    def test_symlinked_data_directory_is_preserved(self):
        renamed = self.data.with_name("preserved")
        self.data.rename(renamed)
        self.data.symlink_to(renamed, target_is_directory=True)
        self.hook("remove")
        self.assertTrue((renamed / "documentation/fixture").exists())


if __name__ == "__main__":
    unittest.main()
