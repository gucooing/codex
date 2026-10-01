"""Packaging checks that do not compile or publish anything."""
import json
from pathlib import Path
import tempfile
import unittest

import release


class PackageTests(unittest.TestCase):
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
