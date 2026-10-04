"""Build ccodex with upstream native helpers and stage the official npm layout."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
DIST = ROOT / "dist"
sys.path.insert(0, str(ROOT / "scripts"))


def version():
    return tomllib.loads((ROOT / "codex-rs/Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]


def validate():
    release_version = version()
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[\w.-]+)?", release_version):
        raise ValueError("Invalid Cargo release version")
    ref = os.environ.get("GITHUB_REF", "")
    tag_pattern = rf"refs/tags/ccodex-v{re.escape(release_version)}(?:\+[1-9]\d*)?"
    if ref.startswith("refs/tags/") and not re.fullmatch(tag_pattern, ref):
        raise ValueError("ccodex tag must match the Cargo version, with an optional +N build revision")
    return release_version


def npm_builder():
    spec = importlib.util.spec_from_file_location("ccodex_npm", ROOT / "codex-cli/scripts/build_npm_package.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def build(target):
    release_version = validate()
    DIST.mkdir(exist_ok=True)
    # Official release CI strips binaries after archiving symbols. This npm
    # build has no symbols archive, so strip during Cargo's release build.
    env = {
        **os.environ,
        "CODEX_REPO_ROOT": str(ROOT),
        "CARGO_PROFILE_RELEASE_STRIP": "symbols",
    }
    # Match upstream: bwrap's finalized bytes are hashed before compiling Codex.
    if "linux" in target:
        subprocess.run(["cargo", "build", "--locked", "--release", "--target", target, "--bin", "bwrap"], cwd=ROOT / "codex-rs", env=env, check=True)
        bwrap = ROOT / "codex-rs/target" / target / "release/bwrap"
        env["CODEX_BWRAP_SHA256"] = hashlib.sha256(bwrap.read_bytes()).hexdigest()
    with tempfile.TemporaryDirectory(prefix="ccodex-package-") as temporary:
        vendor = Path(temporary) / "vendor"
        package = vendor / target
        args = [sys.executable, str(ROOT / "scripts/build_codex_package.py"), "--target", target, "--cargo-profile", "release", "--package-dir", str(package)]
        if "linux" in target:
            args.extend(["--bwrap-bin", str(bwrap)])
        subprocess.run(args, cwd=ROOT, env=env, check=True)
        stage_voice_runtime(package, target, Path(temporary))
        from codex_package.archive import write_archive
        write_archive(package, DIST / f"ccodex-package-{target}.tar.gz", force=False)
        if "windows" in target:
            write_archive(package, DIST / f"ccodex-package-{target}.zip", force=False)
        exe = package / "bin" / ("ccodex.exe" if "windows" in target else "ccodex")
        subprocess.run([str(exe), "--version"], check=True)
        subprocess.run([str(exe), "--help"], check=True, stdout=subprocess.DEVNULL)
        module = npm_builder()
        platform = next(key for key, value in module.CODEX_PLATFORM_PACKAGES.items() if value["target_triple"] == target)
        subprocess.run([sys.executable, str(ROOT / "codex-cli/scripts/build_npm_package.py"), "--package", platform, "--version", release_version, "--vendor-src", str(vendor), "--pack-output", str(DIST / f"ccodex-{target}.tgz")], cwd=ROOT, check=True)


def stage_voice_runtime(package, target, temporary):
    """Retain the unchanged upstream voice runtime, with a pinned archive digest."""
    pin = json.loads((ROOT / "scripts/ccodex/upstream.json").read_text(encoding="utf-8"))
    asset = f"codex-package-{target}.tar.gz"
    archive = temporary / "upstream.tar.gz"
    url = f"https://github.com/openai/codex/releases/download/{pin['tag']}/{asset}"
    digest = hashlib.sha256()
    with urllib.request.urlopen(url, timeout=120) as source, archive.open("wb") as output:
        while block := source.read(1024 * 1024):
            digest.update(block)
            output.write(block)
    if digest.hexdigest() != pin["assets"][asset]:
        raise ValueError("Upstream voice runtime archive checksum mismatch")
    with tarfile.open(archive) as source:
        members = [entry for entry in source.getmembers() if entry.name.removeprefix("./").startswith("codex-resources/voice/")]
        if not members:
            raise ValueError("Pinned package did not contain the upstream voice runtime")
        source.extractall(package, members=members, filter="data")


def publish():
    release_version = validate()
    module = npm_builder()
    for platform in module.CODEX_PLATFORM_PACKAGES.values():
        archive = DIST / f"ccodex-{platform['target_triple']}.tgz"
        subprocess.run([*module.npm_command(), "publish", str(archive), "--access", "public", "--tag", platform["npm_tag"]], check=True)
    with tempfile.TemporaryDirectory(prefix="ccodex-npm-") as temporary:
        stage = Path(temporary)
        module.stage_sources(stage, release_version, "codex")
        subprocess.run([*module.npm_command(), "publish", str(stage), "--access", "public", "--tag", "latest"], check=True)


def checksums():
    archives = sorted([*DIST.glob("*.zip"), *DIST.glob("*.tar.gz")])
    (DIST / "ccodex-package_SHA256SUMS").write_text("".join(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}\n" for path in archives), encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["validate", "build", "publish", "checksums"])
    parser.add_argument("--target")
    args = parser.parse_args()
    if args.action == "build":
        if not args.target:
            parser.error("build requires --target")
        build(args.target)
    elif args.action == "publish":
        publish()
    elif args.action == "checksums":
        checksums()
    else:
        print(validate())
