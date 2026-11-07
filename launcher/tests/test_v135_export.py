"""Source-only kit export must retain build helpers and reject changed inputs."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/mod'))
import create_gadget135_kit as export


class LocalKitExportTests(unittest.TestCase):
    def fixture(self, root):
        source = root / 'source with spaces'
        files = ['Cargo.toml', 'Cargo.lock', 'AGENTS.md', 'mod/development.json',
            'tools/mod/runtime038_test.py', 'tools/mod/rust_build_tools.py',
            'tools/mod/windows_build_env.py', 'tools/mod/verified_artifacts.py',
            'tools/mod/rust_download.py', 'launcher/launcher_core.py', 'launcher/schema.py',
            'docs/mod/LOCAL_TEST_V135.md', 'LICENSE']
        for name in files:
            p = source / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text('reviewed ' + name, encoding='utf-8')
        inventory = root / 'source-files.json'
        inventory.write_text(json.dumps({'patch': export.PATCH, 'runtime_base': '0.3.8-alpha',
            'files': {name: hashlib.sha256((source / name).read_bytes()).hexdigest() for name in files}}))
        return source, inventory

    def test_complete_export_ignores_private_files_and_rebuilds_the_matching_gui(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            source, inventory = self.fixture(root)
            (source / 'unlisted-private.ini').write_text('private')
            with patch.object(export, 'SOURCE', source), patch('verified_artifacts.validate_source_apis'):
                archive = export.create(root / 'out', inventory)
            with zipfile.ZipFile(archive) as z:
                names = z.namelist()
                self.assertFalse(any('unlisted-private' in name for name in names))
                self.assertIn(export.NAME + '/source/tools/mod/windows_build_env.py', names)
                bat = z.read(export.NAME + '/01_COMPILE_AND_UPDATE.bat')
                self.assertNotIn(b'--runtime-only', bat)
                self.assertIn(b'runtime038_test.py" build', bat)
                self.assertNotIn(b'\n', bat.replace(b'\r\n', b''))
                self.assertIsNone(z.testzip())

    def test_changed_source_is_rejected_before_zip_creation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            source, inventory = self.fixture(root)
            (source / 'tools/mod/rust_build_tools.py').write_text('changed')
            with patch.object(export, 'SOURCE', source), patch('verified_artifacts.validate_source_apis'):
                with self.assertRaisesRegex(ValueError, 'integrity'):
                    export.create(root / 'out', inventory)
            self.assertFalse((root / 'out').exists())

    def test_missing_build_helper_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            source, inventory = self.fixture(root)
            data = json.loads(inventory.read_text())
            del data['files']['tools/mod/windows_build_env.py']
            inventory.write_text(json.dumps(data))
            with patch.object(export, 'SOURCE', source), patch('verified_artifacts.validate_source_apis'):
                with self.assertRaisesRegex(ValueError, 'Incomplete'):
                    export.create(root / 'out', inventory)

    def test_wrong_identity_is_rejected(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            source, inventory = self.fixture(root)
            data = json.loads(inventory.read_text())
            data['patch'] = '0.1.6.128'
            inventory.write_text(json.dumps(data))
            with patch.object(export, 'SOURCE', source), patch('verified_artifacts.validate_source_apis'):
                with self.assertRaisesRegex(ValueError, 'v135'):
                    export.create(root / 'out', inventory)


if __name__ == '__main__':
    unittest.main()
