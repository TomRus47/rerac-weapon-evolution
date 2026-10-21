"""Release packaging must reject a stale/probed runtime before building a ZIP."""
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("package_release", ROOT / "tools/mod/package_release.py")
release_tool = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release_tool)


class ReleaseToolTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.kit = self.root / "launcher"
        self.kit.mkdir()
        (self.root / "mod").mkdir()
        self.release = json.loads((ROOT / "mod/release.json").read_text())
        (self.root / "mod/release.json").write_text(json.dumps(self.release))
        self.identity = dict(version="0.3.4", weapon_evolution_patch=self.release["patch"],
                             weapon_evolution_config=58, weapon_evolution_audio_diagnostics=False)

    def test_stale_or_audio_probe_runtime_stops_before_packaging(self):
        for changes in [dict(weapon_evolution_patch="0.1.6.48"),
                        dict(weapon_evolution_audio_diagnostics=True),
                        dict(version="0.3.4-weapon-evolution-fixed")]:
            identity = self.identity | changes
            with self.subTest(changes=changes), patch.object(release_tool, "ROOT", self.root), \
                    patch.object(release_tool, "KIT", self.kit), \
                    patch.object(release_tool, "os", SimpleNamespace(name="nt")), \
                    patch.object(release_tool.subprocess, "check_output", return_value=json.dumps(identity)), \
                    patch.object(release_tool.subprocess, "run") as run:
                with self.assertRaisesRegex(ValueError, "clean editable"):
                    release_tool.main()
                run.assert_not_called()

    def test_release_identity_report_uses_actual_archive_and_no_author_presets(self):
        (self.kit / "weapons.default.ini").write_bytes(b"approved profile")
        (self.kit / "launcher.ico").write_bytes(b"supplied icon")

        def package(command, **kwargs):
            self.assertIn("--no-presets", command)
            output = Path(command[command.index("--output") + 1])
            output.parent.mkdir()
            Path(str(output) + ".zip").write_bytes(b"built player ZIP")

        with patch.object(release_tool, "ROOT", self.root), patch.object(release_tool, "KIT", self.kit), \
                patch.object(release_tool, "os", SimpleNamespace(name="nt")), \
                patch.object(release_tool.subprocess, "check_output", side_effect=[json.dumps(self.identity), "a" * 40]), \
                patch.object(release_tool.subprocess, "run", side_effect=package):
            self.assertEqual(release_tool.main(), 0)
        report = json.loads((self.kit / "dist/release-build.json").read_text())
        self.assertEqual(report["archive"], self.release["asset"] + ".zip")
        self.assertEqual(report["runtime"], self.identity)
        self.assertEqual(len(report["sha256"]), 64)
        self.assertIn("Publication approved by the mod author", report["gameplay_validation"])
