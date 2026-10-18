"""Packaging checks that do not compile or publish anything."""
import json
import os
from pathlib import Path
import tempfile
import tarfile
import unittest
from unittest.mock import patch

import release


class PackageTests(unittest.TestCase):
    def test_packaging_revision_tag_keeps_the_official_version(self):
        with patch.dict(os.environ, {"GITHUB_REF": f"refs/tags/ccodex-v{release.version()}+1"}):
            self.assertEqual(release.validate(), release.version())

    def test_npm_pack_runs_with_spaces_and_shell_characters_in_paths(self):
        builder = release.npm_builder()
        with tempfile.TemporaryDirectory(prefix="ccodex npm ") as temporary:
            root = Path(temporary)
            stage = root / "stage & literal"
            stage.mkdir()
            builder.stage_sources(stage, "0.159.3", "codex")
            output = root / "output & literal" / "ccodex.tgz"
            builder.run_npm_pack(stage, output)
            with tarfile.open(output) as archive:
                manifest = json.load(archive.extractfile("package/package.json"))
                self.assertEqual(manifest["bin"], {"ccodex": "bin/ccodex.js"})
                self.assertIn("package/bin/ccodex.js", archive.getnames())

    def test_launcher_and_all_platform_aliases_resolve_to_fork(self):
        builder = release.npm_builder()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            launcher = root / "launcher"
            launcher.mkdir()
            builder.stage_sources(launcher, "0.159.3", "codex")
            package = json.loads((launcher / "package.json").read_text())
            self.assertEqual(package["bin"], {"ccodex": "bin/ccodex.js"})
            self.assertTrue((launcher / "bin/ccodex.js").is_file())
            for key, spec in builder.CODEX_PLATFORM_PACKAGES.items():
                stage = root / key
                stage.mkdir()
                builder.stage_sources(stage, "0.159.3", key)
                native = json.loads((stage / "package.json").read_text())
                self.assertEqual(native["name"], "@gucooing/ccodex")
                self.assertEqual(package["optionalDependencies"][spec["npm_name"]], f"npm:{native['name']}@{native['version']}")
                self.assertEqual(native["os"], [spec["os"]])
                self.assertEqual(native["cpu"], [spec["cpu"]])


if __name__ == "__main__":
    unittest.main()
