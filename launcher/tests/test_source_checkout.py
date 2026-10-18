from pathlib import Path
import shutil,sys,tempfile,unittest
from types import SimpleNamespace
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
import editabile as build
import launcher_core as core

class SourceCheckoutTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='mod source with spaces ')
        self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/'checkout';self.root.mkdir()
        self.kit=self.root/'launcher';self.kit.mkdir()
        self.raw=(ROOT/'weapons.default.ini').read_bytes()
        (self.kit/'weapons.example.ini').write_bytes(self.raw)
        for name in ['Cargo.toml','Cargo.lock','crates/rc-game/Cargo.toml',
                     'crates/rc-engine/Cargo.toml','crates/rc-game/src/moby_update/real_hp.rs']:
            p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text('// source fixture')

    def test_checkout_root_is_relative_and_requires_no_historical_patch_installer(self):
        self.assertEqual(build.DEFAULT_ROOT,ROOT.parent)
        build.check_source_root(self.root)
        (self.root/'Cargo.lock').unlink()
        with self.assertRaisesRegex(ValueError,'Incomplete source checkout'):
            build.check_source_root(self.root)

    def test_first_build_bootstraps_the_approved_ini_and_preserves_later_edits(self):
        target=build.prepare_local_ini(self.kit)
        self.assertEqual(target.read_bytes(),self.raw)
        edited=core.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'61'})
        target.write_bytes(edited)
        self.assertEqual(build.prepare_local_ini(self.kit).read_bytes(),edited)
        self.assertNotIn(('general','damage_mode'),core.checked_document(edited).entries)

    def test_build_tests_then_compiles_this_checkout_and_stages_only_after_success(self):
        with patch.object(build,'os',SimpleNamespace(name='nt')),patch.object(build.shutil,'which',return_value='cargo'), \
             patch.object(build,'ensure_closed'),patch.object(build.subprocess,'run') as run,patch.object(build,'stage_runtime') as stage:
            build.build(self.root,self.kit)
        self.assertEqual([call.args[0][:2] for call in run.call_args_list],[['cargo','test'],['cargo','build']])
        for call in run.call_args_list:
            command=call.args[0]
            self.assertIn('--locked',command);self.assertIn('--no-default-features',command)
            self.assertEqual(command[command.index('--target-dir')+1],str(self.root/'target'))
            self.assertEqual(call.kwargs['cwd'],self.root)
        stage.assert_called_once_with(self.root,self.kit,self.root/'target/release/rerac.exe')
        self.assertFalse((self.root/'.weapon_evo_backup_v01637_editable').exists())

    def test_failed_rust_tests_stop_before_compilation_and_runtime_replacement(self):
        with patch.object(build,'os',SimpleNamespace(name='nt')),patch.object(build.shutil,'which',return_value='cargo'), \
             patch.object(build,'ensure_closed'),patch.object(build.subprocess,'run',side_effect=RuntimeError('Rust tests failed')) as run, \
             patch.object(build,'stage_runtime') as stage:
            with self.assertRaisesRegex(RuntimeError,'Rust tests failed'):build.build(self.root,self.kit)
        self.assertEqual(run.call_count,1);stage.assert_not_called()

    def test_build_action_does_not_require_game_data_or_start_the_game(self):
        with patch.object(sys,'argv',['editabile.py','build',str(self.root)]), \
             patch.object(build,'build') as compile_game,patch.object(build,'launch') as launch:
            self.assertEqual(build.main(),0)
        compile_game.assert_called_once_with(self.root,build.KIT);launch.assert_not_called()

    def test_explicit_build_and_run_action_still_launches_after_compilation(self):
        with patch.object(sys,'argv',['editabile.py','build-and-run',str(self.root)]), \
             patch.object(build,'build') as compile_game,patch.object(build,'launch',return_value=0) as launch:
            self.assertEqual(build.main(),0)
        compile_game.assert_called_once();launch.assert_called_once_with(build.KIT,self.root)

if __name__=='__main__':unittest.main()
