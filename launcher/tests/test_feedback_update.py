"""Local update transactions preserve the selected playable installation."""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[2]
TOOLS=ROOT/'tools/mod'
sys.path.insert(0,str(TOOLS))
import feedback_update as update

class UpdateTests(unittest.TestCase):
    def fixture(self,root):
        source=root/'Rust source';launcher=root/'Launcher source';playable=root/'Playable mod';bundle=root/'Update kit'
        for p in (source,launcher,playable,bundle):p.mkdir()
        (source/'sample.rs').write_bytes(b'old\n')
        (source/'crates/rc-engine/assets/shaders').mkdir(parents=True)
        (source/'crates/rc-engine/assets/shaders/test.wgsl').write_bytes(b'shader')
        (playable/'runtime').mkdir();(playable/'runtime/rerac.exe').write_bytes(b'old game')
        (playable/'runtime/retained.dll').write_bytes(b'dll')
        (playable/'weapons.ini').write_bytes(b'personal settings')
        (playable/'launcher.ico').write_bytes(b'personal icon')
        target=playable/'My launcher.exe';target.write_bytes(b'old launcher')
        payload=b'new\n';(bundle/'payload/source').mkdir(parents=True)
        (bundle/'payload/source/sample.rs').write_bytes(payload)
        (bundle/'manifest.json').write_text(json.dumps({'patch':update.PATCH,'files':[{
            'root':'source','path':'sample.rs','before':[update.digest(b'old\n')],'after':update.digest(payload)}]}))
        return source,playable,launcher,target,bundle

    def runner(self,source,playable,launcher,fail_build=False):
        def run(args,**kwargs):
            if len(args)>1 and str(args[1]).endswith('stage_feedback_profiles.py'):
                stage=Path(args[-1]);stage.mkdir();new=stage/'0.ini';new.write_bytes(b'personal settings with additive fields')
                (stage/'profiles.json').write_text(json.dumps([{'target':str(playable/'weapons.ini'),'staged':str(new)}]))
            elif args[:2]==['cargo','build']:
                if fail_build:raise subprocess.CalledProcessError(1,args)
                exe=source/'target/release/rerac.exe';exe.parent.mkdir(parents=True);exe.write_bytes(b'new game')
            elif len(args)>1 and args[1]=='--version-json':
                return subprocess.CompletedProcess(args,0,json.dumps({'weapon_evolution_patch':update.PATCH,'weapon_evolution_config':88,'version':'0.3.6'}))
            elif len(args)>1 and str(args[1]).endswith('build_launcher_exe.py'):
                exe=launcher/'launcher-bin/ReRAC_Weapon_Evolution.exe';exe.parent.mkdir();exe.write_bytes(b'new launcher')
            return subprocess.CompletedProcess(args,0,'')
        return run

    def test_complete_transaction_replaces_actual_exe_and_retains_runtime_files(self):
        with tempfile.TemporaryDirectory() as td:
            source,playable,launcher,target,bundle=self.fixture(Path(td));verified=[]
            backup=update.install(source,playable,launcher,target,bundle,self.runner(source,playable,launcher),verified.append)
            self.assertEqual(target.read_bytes(),b'new launcher')
            self.assertEqual((playable/'runtime/rerac.exe').read_bytes(),b'new game')
            self.assertEqual((playable/'runtime/retained.dll').read_bytes(),b'dll')
            self.assertEqual((playable/'runtime/assets/shaders/test.wgsl').read_bytes(),b'shader')
            self.assertEqual((launcher/'launcher.ico').read_bytes(),b'personal icon')
            self.assertEqual((backup/'runtime/rerac.exe').read_bytes(),b'old game')
            self.assertEqual(len(verified),2)
            self.assertEqual(update.plan(source,launcher,bundle),{})
            self.assertEqual(json.loads((bundle/'installed.json').read_text())['exe'],str(target))

    def test_compile_failure_restores_sources_profiles_exes_and_icon(self):
        with tempfile.TemporaryDirectory() as td:
            source,playable,launcher,target,bundle=self.fixture(Path(td))
            with self.assertRaises(subprocess.CalledProcessError):
                update.install(source,playable,launcher,target,bundle,self.runner(source,playable,launcher,True),lambda p:None)
            self.assertEqual((source/'sample.rs').read_bytes(),b'old\n')
            self.assertEqual((playable/'weapons.ini').read_bytes(),b'personal settings')
            self.assertEqual(target.read_bytes(),b'old launcher')
            self.assertEqual((playable/'runtime/rerac.exe').read_bytes(),b'old game')
            self.assertFalse((launcher/'launcher.ico').exists())
            self.assertFalse((bundle/'installed.json').exists())

    def test_installed_gui_verification_failure_also_restores_runtime(self):
        with tempfile.TemporaryDirectory() as td:
            source,playable,launcher,target,bundle=self.fixture(Path(td))
            def verify(path):
                if path==target:raise ValueError('Frozen GUI verification failed')
            with self.assertRaises(ValueError):update.install(source,playable,launcher,target,bundle,self.runner(source,playable,launcher),verify)
            self.assertEqual(target.read_bytes(),b'old launcher')
            self.assertEqual((playable/'runtime/rerac.exe').read_bytes(),b'old game')
            self.assertEqual((source/'sample.rs').read_bytes(),b'old\n')

    def test_unknown_sources_and_escape_paths_fail_before_mutation(self):
        with tempfile.TemporaryDirectory() as td:
            source,playable,launcher,target,bundle=self.fixture(Path(td))
            (source/'sample.rs').write_bytes(b'local edits')
            with self.assertRaises(ValueError):update.plan(source,launcher,bundle)
            self.assertEqual((source/'sample.rs').read_bytes(),b'local edits')
            manifest=json.loads((bundle/'manifest.json').read_text());manifest['files'][0]['path']='../outside'
            (bundle/'manifest.json').write_text(json.dumps(manifest))
            with self.assertRaises(ValueError):update.plan(source,launcher,bundle)

    def test_real_profile_stage_preserves_existing_values_and_is_idempotent(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td);launcher=root/'launcher';launcher.mkdir();playable=root/'playable';playable.mkdir();source=root/'source';source.mkdir()
            for f in (ROOT/'launcher').glob('*.py'):shutil.copy2(f,launcher/f.name)
            raw=(ROOT/'launcher/weapons.default.ini').read_bytes().replace(b'xp_per_hp_damage = 0.1',b'xp_per_hp_damage = 0.23')
            # Remove new fields to exercise additive migration of a real profile.
            raw=b'\n'.join(line for line in raw.splitlines() if not any(line.startswith(k) for k in
                (b'burn_on_hit',b'hud_quickselect_only',b'slam_explosion_',b'weapon_bar_alternative_position')))+b'\n'
            (playable/'weapons.ini').write_bytes(raw);(playable/'presets').mkdir();(playable/'presets/Personal.ini').write_bytes(raw)
            stage=root/'stage'
            subprocess.run([sys.executable,str(TOOLS/'stage_feedback_profiles.py'),str(launcher),str(playable),str(source),str(stage)],check=True)
            rows=json.loads((stage/'profiles.json').read_text());self.assertEqual(len(rows),2)
            for row in rows:
                data=Path(row['staged']).read_bytes();self.assertIn(b'weapon_bar_alternative_position',data);self.assertIn(b'slam_explosion_radius',data)
                Path(row['target']).write_bytes(data)
            again=root/'again'
            subprocess.run([sys.executable,str(TOOLS/'stage_feedback_profiles.py'),str(launcher),str(playable),str(source),str(again)],check=True)
            self.assertEqual(json.loads((again/'profiles.json').read_text()),[])

if __name__=='__main__':unittest.main()
