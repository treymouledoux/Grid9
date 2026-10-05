import hashlib
from html.parser import HTMLParser
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from urllib.parse import unquote, urlsplit

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from check_runtime import PROJECT, run
from preflight import validate_tag
from release import checksums


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
    def test_release_tag_must_match_cargo_version(self):
        validate_tag("", "2026.1.0")
        validate_tag("v2026.1.0", "2026.1.0")
        for tag in ["v2026.2.0", "2026.1.0", "v2026.1.0-extra"]:
            with self.assertRaises(ValueError):
                validate_tag(tag, "2026.1.0")

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
            run([sys.executable, "-c", "import sys; print('fixture failure', file=sys.stderr); sys.exit(7)"])
        self.assertEqual(run([sys.executable, "-c", "raise SystemExit(7)"], check=False).returncode, 7)

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
                    self.assertIn(unquote(parsed.fragment), links.ids, f"{page}: missing {url}")

    def test_all_packaging_python_sources_compile(self):
        for source in (PROJECT / "packaging").rglob("*.py"):
            compile(source.read_text(encoding="utf-8"), str(source), "exec")


if __name__ == "__main__":
    unittest.main()
