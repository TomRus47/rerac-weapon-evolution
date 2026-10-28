"""Full-source updater integrity and rollback keep player files outside its scope."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/mod'))
import runtime037_test as update


class Runtime037Tests(unittest.TestCase):
    identity = {'name': 'rerac', 'game': 'rac1', 'version': '0.3.7',
                'weapon_evolution_patch': update.PATCH,
                'weapon_evolution_config': 90,
                'weapon_evolution_audio_diagnostics': False}

    def fixture(self, root):
        source, play, bundle = (root / name for name in ['source', 'play', 'kit'])
        bundle.mkdir()
        shader = source / 'crates/rc-engine/assets/shaders/hud.wgsl'
        shader.parent.mkdir(parents=True)
        shader.write_bytes(b'new shader interface')
        exe = source / 'target/release/rerac.exe'
        exe.parent.mkdir(parents=True)
        exe.write_bytes(b'new executable')
        old = {'runtime/rerac.exe': b'previous runtime',
               'runtime/assets/shaders/hud.wgsl': b'old shader interface',
               'runtime/build-info.json': b'old build identity',
               'weapons.ini': b'personal values', 'presets/custom.ini': b'custom',
               'launcher-settings.json': b'paths', 'saves/slot0.dat': b'native save',
               'saves/slot0.wevo': b'companion', 'ReRAC_Weapon_Evolution.exe': b'GUI and icon'}
        for name, data in old.items():
            path = play / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
        return source, play, bundle, play / 'ReRAC_Weapon_Evolution.exe', old

    def test_update_changes_only_runtime_and_matching_shaders(self):
        with tempfile.TemporaryDirectory() as folder:
            source, play, bundle, launcher, old = self.fixture(Path(folder))
            backup = update.install_runtime(source, play, launcher, self.identity, bundle)
            self.assertTrue((backup / 'restore.json').is_file())
            self.assertEqual((play / 'runtime/rerac.exe').read_bytes(), b'new executable')
            self.assertEqual((play / 'runtime/assets/shaders/hud.wgsl').read_bytes(), b'new shader interface')
            for name, data in old.items():
                if not name.startswith('runtime/'):
                    self.assertEqual((play / name).read_bytes(), data)
            record = json.loads((bundle / 'installed.json').read_text())
            self.assertEqual(record['exe_sha256'], update.sha(b'new executable'))

    def test_late_failure_rolls_back_executable_and_shaders(self):
        with tempfile.TemporaryDirectory() as folder:
            source, play, bundle, launcher, old = self.fixture(Path(folder))
            original = update.write
            def fail_record(path, data):
                if path.name == 'installed.json':
                    raise OSError('simulated late disk failure')
                original(path, data)
            with patch.object(update, 'write', side_effect=fail_record):
                with self.assertRaises(OSError):
                    update.install_runtime(source, play, launcher, self.identity, bundle)
            for name, data in old.items():
                self.assertEqual((play / name).read_bytes(), data)
            self.assertFalse((bundle / 'installed.json').exists())

    def test_identity_failure_cannot_touch_the_playable_installation(self):
        with tempfile.TemporaryDirectory() as folder:
            source, play, bundle, launcher, old = self.fixture(Path(folder))
            with self.assertRaises(ValueError):
                update.install_runtime(source, play, launcher, self.identity | {'version': '0.3.6'}, bundle)
            self.assertFalse((play / 'backup').exists())
            for name, data in old.items():
                self.assertEqual((play / name).read_bytes(), data)

    def test_bundled_source_hash_must_match_before_build(self):
        with tempfile.TemporaryDirectory() as folder:
            source, play, bundle, launcher, _ = self.fixture(Path(folder))
            path = source / 'Cargo.toml'
            path.write_bytes(b'version = "0.3.7"')
            manifest = {'patch': update.PATCH, 'runtime_base': '0.3.7-alpha',
                        'files': {'Cargo.toml': update.sha(path.read_bytes())}}
            (bundle / 'source-files.json').write_text(json.dumps(manifest))
            update.verify_sources(source, bundle)
            path.write_bytes(b'corrupted source')
            with self.assertRaises(ValueError):
                update.verify_sources(source, bundle)

    def test_source_inventory_cannot_escape_bundle(self):
        for name in ['../outside', '/outside', 'C:/outside', 'sub\\outside']:
            with self.assertRaises(ValueError):
                update.safe(ROOT, name)


if __name__ == '__main__':
    unittest.main()
