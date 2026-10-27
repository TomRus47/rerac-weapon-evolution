"""Exact-block v106 installation remains additive, atomic and idempotent."""
from pathlib import Path
import json,shutil,subprocess,sys,tempfile,unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/mod'))
import slam_visual_update as update
class SlamUpdateTests(unittest.TestCase):
    def fixture(self,root):
        source=root/'source';bundle=root/'kit';play=root/'playable';bundle.mkdir();play.mkdir();(play/'runtime').mkdir()
        (play/'runtime/rerac.exe').write_bytes(b'previous runtime');(play/'weapons.ini').write_bytes(b'personal settings')
        launcher=play/'Custom launcher.exe';launcher.write_bytes(b'previous launcher')
        data=json.loads((ROOT/'tools/mod/v106_slam_manifest.json').read_text())
        # Reverse the exact-block edits against this checkout to obtain the
        # supported previous interface, preserving unrelated source comments.
        for name in {row['path'] for row in data['edits']}:
            text=(ROOT/name).read_text()
            for row in reversed(data['edits']):
                if row['path']==name:
                    self.assertEqual(text.count(row['after']),row['count'])
                    text=text.replace(row['after'],row['before'])
            target=source/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_text(text+'\n// Unrelated local comment.\n')
        (bundle/'payload').mkdir()
        for row in data['files']:
            shutil.copy2(ROOT/row['path'],bundle/'payload'/row['file'])
        (bundle/'manifest.json').write_text(json.dumps(data))
        return source,bundle,play,launcher
    def test_complete_preflight_and_second_install_preserve_unrelated_source(self):
        with tempfile.TemporaryDirectory() as td:
            source,bundle,play,launcher=self.fixture(Path(td))
            for path,body in update.plan(source,bundle).items():update.write(path,body)
            self.assertEqual(update.plan(source,bundle),{})
            self.assertTrue((source/'crates/rc-game/src/hero/melee.rs').read_text().endswith('// Unrelated local comment.\n'))
            self.assertEqual((play/'weapons.ini').read_bytes(),b'personal settings')
    def test_unknown_source_rejected_before_any_writes(self):
        with tempfile.TemporaryDirectory() as td:
            source,bundle,play,launcher=self.fixture(Path(td))
            target=source/'crates/rc-game/src/hero/melee.rs';target.write_text(target.read_text().replace('blast.radius*2.','custom_radius()'))
            before={p:p.read_bytes() for p in source.rglob('*') if p.is_file()}
            with self.assertRaises(ValueError):update.plan(source,bundle)
            self.assertEqual(before,{p:p.read_bytes() for p in source.rglob('*') if p.is_file()})
    def test_compile_failure_rolls_back_source_and_playable_runtime(self):
        with tempfile.TemporaryDirectory() as td:
            source,bundle,play,launcher=self.fixture(Path(td));before={p:p.read_bytes() for p in source.rglob('*') if p.is_file()}
            def fail(*args,**kwargs):raise subprocess.CalledProcessError(1,args[0])
            with self.assertRaises(subprocess.CalledProcessError):update.install(source,play,launcher,bundle,fail)
            self.assertEqual(before,{p:p.read_bytes() for p in source.rglob('*') if p.is_file()})
            self.assertEqual((play/'runtime/rerac.exe').read_bytes(),b'previous runtime')
            self.assertEqual(launcher.read_bytes(),b'previous launcher')
    def test_success_replaces_only_verified_runtime(self):
        with tempfile.TemporaryDirectory() as td:
            source,bundle,play,launcher=self.fixture(Path(td))
            def run(args,**kwargs):
                if args[:2]==['cargo','build']:
                    exe=source/'target/release/rerac.exe';exe.parent.mkdir(parents=True);exe.write_bytes(b'new runtime')
                return subprocess.CompletedProcess(args,0,json.dumps({'weapon_evolution_patch':update.PATCH,'weapon_evolution_config':89,'version':'0.3.6'}))
            update.install(source,play,launcher,bundle,run)
            self.assertEqual((play/'runtime/rerac.exe').read_bytes(),b'new runtime')
            self.assertEqual((play/'weapons.ini').read_bytes(),b'personal settings')
            self.assertEqual(launcher.read_bytes(),b'previous launcher')
            self.assertEqual(update.plan(source,bundle),{})
if __name__=='__main__':unittest.main()
