import hashlib
from html.parser import HTMLParser
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest
from unittest.mock import patch
from urllib.parse import unquote, urlsplit

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_runtime import PROJECT, run
from check_installation import nsis_command
from preflight import validate_tag
import preflight
from release import checksums, validate_bundles


class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        for attribute in ["href", "src"]:
            if attribute in attrs:
                self.links.append(attrs[attribute])
        if "id" in attrs:
            self.ids.add(attrs["id"])


class PackagingTests(unittest.TestCase):
    def test_preflight_refuses_to_continue_after_stale_output_cleanup_fails(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            version = tomllib.loads((PROJECT / "Cargo.toml").read_text())["package"]["version"]
            (root / "Cargo.toml").write_text(f'[package]\nversion = "{version}"\n')
            (root / "target/packages").mkdir(parents=True)
            with (
                patch("preflight.PROJECT", root),
                patch.dict(
                    "os.environ",
                    {"GITHUB_REF_TYPE": "branch", "PACKAGER_VERSION": "0.11.8"},
                ),
                patch(
                    "preflight.subprocess.check_output",
                    return_value="cargo-packager 0.11.8",
                ),
                patch(
                    "preflight.shutil.rmtree",
                    side_effect=PermissionError("stale output"),
                ),
            ):
                with self.assertRaisesRegex(PermissionError, "stale output"):
                    preflight.main()

    def test_nsis_directory_arguments_are_last_and_unquoted(self):
        executable = r"C:\setup files\Grid9.exe"
        directory = r"C:\Users\test user\installed ü"
        self.assertEqual(
            nsis_command(executable, directory), f'"{executable}" /S /D={directory}'
        )
        self.assertEqual(
            nsis_command(executable, directory, uninstall=True),
            f'"{executable}" /S _?={directory}',
        )

    def test_release_tag_must_match_cargo_version(self):
        current = tomllib.loads((PROJECT / "Cargo.toml").read_text())["package"]["version"]
        for version in sorted({current, "2026.1.0", "2026.1.1", "2027.2.3"}):
            with self.subTest(version=version):
                validate_tag("", version)
                validate_tag(f"v{version}", version)
                major, minor, patch_version = map(int, version.split("."))
                wrong_patch = f"v{major}.{minor}.{patch_version + 1}"
                for tag in [wrong_patch, version, f"v{version}-extra"]:
                    with self.subTest(tag=tag), self.assertRaises(ValueError):
                        validate_tag(tag, version)

    def test_release_requires_both_linux_architectures(self):
        paths = [
            Path(name)
            for name in ["setup.exe", "intel.deb", "arm.deb", "mac.dmg", "mac.pkg"]
        ]
        with patch(
            "release.subprocess.check_output", side_effect=["amd64\n", "arm64\n"]
        ):
            validate_bundles(paths)
        with self.assertRaisesRegex(RuntimeError, "Missing or duplicate"):
            validate_bundles([p for p in paths if p.name != "arm.deb"])
        with patch(
            "release.subprocess.check_output", side_effect=["amd64\n", "amd64\n"]
        ):
            with self.assertRaisesRegex(
                RuntimeError, "Expected one amd64 and one arm64"
            ):
                validate_bundles(paths)

    def test_checksums_are_sorted_and_match_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            a, b = root / "a.pkg", root / "b.deb"
            a.write_bytes(b"installer")
            b.write_bytes(b"package")
            expected = f"{hashlib.sha256(b'installer').hexdigest()}  a.pkg\n{hashlib.sha256(b'package').hexdigest()}  b.deb\n"
            self.assertEqual(checksums([b, a]), expected)

    def test_subprocess_failures_include_diagnostics(self):
        with self.assertRaisesRegex(RuntimeError, "fixture failure"):
            run(
                [
                    sys.executable,
                    "-c",
                    "import sys; print('fixture failure', file=sys.stderr); sys.exit(7)",
                ]
            )
        self.assertEqual(
            run([sys.executable, "-c", "raise SystemExit(7)"], check=False).returncode,
            7,
        )

    def test_subprocesses_have_enforced_deadlines(self):
        with self.assertRaises(subprocess.TimeoutExpired):
            run([sys.executable, "-c", "import time; time.sleep(30)"], timeout=0.1)

    def test_documentation_local_links_and_fragments_exist(self):
        root = PROJECT / "src/components/documentation"
        for page in root.rglob("*.html"):
            links = Links()
            links.feed(page.read_text(encoding="utf-8"))
            for url in links.links:
                parsed = urlsplit(url)
                if parsed.scheme or parsed.netloc:
                    continue
                target = page.parent / unquote(parsed.path) if parsed.path else page
                self.assertTrue(target.exists(), f"{page}: missing {url}")
                if parsed.fragment and target == page:
                    self.assertIn(
                        unquote(parsed.fragment), links.ids, f"{page}: missing {url}"
                    )

    def test_all_packaging_python_sources_compile(self):
        for source in (PROJECT / "packaging").rglob("*.py"):
            compile(source.read_text(encoding="utf-8"), str(source), "exec")


if __name__ == "__main__":
    unittest.main()
