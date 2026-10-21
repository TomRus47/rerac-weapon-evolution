"""UI-only restoration preserves the already tested v58 game and personal files."""
from pathlib import Path
from unittest.mock import patch
import hashlib, importlib.util, json, sys, tempfile, unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'launcher'))
spec=importlib.util.spec_from_file_location('ui_restore_updater',ROOT/'tools/mod/local_test_update.py')
u=importlib.util.module_from_spec(spec);spec.loader.exec_module(u)

class LauncherRestoreTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='launcher restore with spaces ')
        self.addCleanup(self.temp.cleanup)
        base=Path(self.temp.name);self.payload=base/'payload';self.kit=base/'existing mod'
        files={'launcher/launcher.py':b'updated UI','launcher/launcher_core.py':b'updated core','launcher/schema.py':b'model selectors','launcher/weapons.default.ini':b'do not install','crates/rc-game/src/lib.rs':b'do not install Rust'}
        for name,data in files.items():
            p=self.payload/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
        self.manifest=base/'manifest.json';self.manifest.write_text(json.dumps({'patch':'0.1.6.59','files':{name:hashlib.sha256(data).hexdigest() for name,data in files.items()}}))
        self.original={'weapons.ini':b'my INI','presets/Personal.ini':b'my preset','runtime/rerac.exe':b'v58 game','launcher.ico':b'my icon','launcher-settings.json':b'{"language":"it"}','launcher.py':b'old broken UI','launcher_core.py':b'old core'}
        for name,data in self.original.items():
            p=self.kit/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data)
        self.probe=patch.object(u.core,'probe_exe',return_value={'weapon_evolution_patch':'0.1.6.59'})
        self.probe.start();self.addCleanup(self.probe.stop)

    def test_restores_python_only_and_keeps_runtime_profiles_settings_and_icon_byte_identical(self):
        backup=u.install_launcher(self.payload,self.kit,self.manifest)
        self.assertEqual((self.kit/'launcher.py').read_bytes(),b'updated UI')
        self.assertEqual((backup/'launcher.py').read_bytes(),b'old broken UI')
        for name,data in self.original.items():
            if not name.endswith('.py'):self.assertEqual((self.kit/name).read_bytes(),data,name)
        self.assertFalse((self.kit/'weapons.default.ini').exists())
        self.assertFalse((self.kit/'crates').exists())
        self.assertTrue((self.kit/'OPEN_TEST_LAUNCHER_v58_2.bat').is_file())

    def test_corrupt_payload_rejects_before_launcher_files_change(self):
        (self.payload/'launcher/launcher.py').write_bytes(b'corrupt')
        with self.assertRaisesRegex(ValueError,'Damaged source'):u.install_launcher(self.payload,self.kit,self.manifest)
        self.assertEqual((self.kit/'launcher.py').read_bytes(),b'old broken UI')
        self.assertFalse((self.kit/'backup').exists())

    def test_source_launcher_without_runtime_or_weapons_ini_is_supported(self):
        (self.kit/'weapons.ini').unlink()
        (self.kit/'runtime/rerac.exe').unlink();(self.kit/'runtime').rmdir()
        (self.kit/'weapons.example.ini').write_bytes(b'existing source defaults')
        source=self.kit.parent/'source';source.mkdir()
        target=source/'launcher';self.kit.rename(target)
        with patch.object(u.core,'probe_exe',side_effect=AssertionError('runtime must not be probed')):
            self.assertEqual(u.launcher_folder(source),target.resolve())
            u.install_launcher(self.payload,target,self.manifest)
            self.assertEqual((target/'launcher.py').read_bytes(),b'updated UI')
            self.assertFalse((target/'runtime').exists());self.assertFalse((target/'weapons.ini').exists())
            self.assertEqual((target/'weapons.example.ini').read_bytes(),b'existing source defaults')
            with patch.object(u.subprocess,'run') as run:
                run.return_value.returncode=0
                self.assertEqual(u.open_editor(target,verify_runtime=False),0)
                self.assertIn(str(target/'launcher.py'),run.call_args.args[0])

    def test_failed_write_rolls_back_the_python_files_and_leaves_runtime_untouched(self):
        write=u.core.atomic_write
        def fail(path,data):
            if path==self.kit/'OPEN_TEST_LAUNCHER_v58_2.bat':raise OSError('simulated disk failure')
            return write(path,data)
        with patch.object(u.core,'atomic_write',side_effect=fail):
            with self.assertRaises(OSError):u.install_launcher(self.payload,self.kit,self.manifest)
        for name,data in self.original.items():self.assertEqual((self.kit/name).read_bytes(),data,name)
        self.assertFalse((self.kit/'schema.py').exists())
