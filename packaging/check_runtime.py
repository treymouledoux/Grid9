"""Bounded smoke tests shared by pre-bundle and installed-binary checks."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile

PROJECT = Path(__file__).resolve().parent.parent


def run(command, *, env=None, cwd=None, input=None, check=True, timeout=30):
    result = subprocess.run(
        [str(arg) for arg in command], env=env, cwd=cwd, input=input,
        capture_output=True, text=True, timeout=timeout,
    )
    if check and result.returncode:
        raise RuntimeError(f"{command!r} exited {result.returncode}\n{result.stdout}\n{result.stderr}")
    return result


def check_runtime(binary, root):
    binary = Path(binary).resolve()
    data = root / "user data ü"
    environment = dict(os.environ, GRID9_DATA_DIR=str(data))

    def cli(*args, **kwargs):
        return run([binary, *args], env=environment, cwd=root, **kwargs)

    for flag in ["--help", "--version"]:
        assert cli(flag).stdout
    assert not data.exists(), "Help/version flags should not initialize user data"
    assert "Version:" in cli("version").stdout
    for directory in ["logs", "preprocessor_cache"]:
        assert (data / directory).is_dir()
        assert not list((data / directory).iterdir())

    components = PROJECT / "src/components"
    for component in ["documentation", "examples"]:
        for source in (components / component).rglob("*"):
            relative = source.relative_to(components)
            if source.is_file() and not any(p.startswith(".") for p in relative.parts):
                assert (data / relative).read_bytes() == source.read_bytes(), relative

    custom = data / "examples/personal.g9"
    custom.write_text("f7p", encoding="utf-8")
    document = data / "documentation/index.html"
    document.unlink()
    cli("version")
    assert document.is_file(), "Missing bundled resources were not restored"
    document.write_text("outdated", encoding="utf-8")
    (data / ".components-revision").write_text("old version", encoding="utf-8")
    cli("version")
    assert document.read_bytes() == (components / "documentation/index.html").read_bytes()
    assert custom.read_text(encoding="utf-8") == "f7p"

    assert cli("convert", "encode", "a").stdout.strip() == "000000010"
    assert cli("convert", "decode", "000000010").stdout.strip() == "a"
    assert cli("convert", "encode", "not a glyph", check=False).returncode != 0
    assert "Hello world" in cli("interpret", "--example", "example1").stdout

    script = root / "script with spaces ü.g9"
    config = script.with_suffix(".toml")
    config.write_text("[config]\nno_log = true\nverbosity = 0\n", encoding="utf-8")
    script.write_text("f7pi7=1s70b010}p", encoding="utf-8")
    for _ in range(2):
        assert cli("interpret", script).stdout == "a\n \n \n"
    script.write_text("ggp", encoding="utf-8")
    assert cli("interpret", script, input="000000010\n").stdout == "a\n"
    assert cli("interpret", script, input="invalid\n", check=False).returncode == 1
    script.write_text("a1p", encoding="utf-8")
    failure = cli("interpret", script, check=False)
    assert failure.returncode == 1 and "Grid has no glyph" in failure.stderr
    config.write_text("[config]\nverbosity = 'invalid'\n", encoding="utf-8")
    assert cli("interpret", script, check=False).returncode != 0
    cli("clean", "preprocessor_cache")
    assert not list((data / "preprocessor_cache").iterdir())
    assert custom.exists(), "Cache cleanup removed a personal script"

    # Exercise installation when the very first command is an example.
    environment["GRID9_DATA_DIR"] = str(root / "first run")
    assert "Hello world" in cli("interpret", "--example", "example1").stdout
    assert (root / "first run/documentation/index.html").is_file()
    print(f"Verified release CLI, resources, repair, configuration, stdin, errors, and cache: {binary}")


def main():
    suffix = ".exe" if sys.platform == "win32" else ""
    binary = PROJECT / f"target/release/grid9{suffix}"
    with tempfile.TemporaryDirectory(prefix="grid9-runtime-") as temporary:
        check_runtime(binary, Path(temporary))


if __name__ == "__main__":
    main()
