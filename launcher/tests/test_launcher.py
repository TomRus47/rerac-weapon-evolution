from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch, Mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import launcher_core as c
import launcher as gui

class Tests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='rerac launcher spazi ')
        self.addCleanup(self.temp.cleanup)
        self.parent = Path(self.temp.name).resolve()
        self.kit = self.parent/'Launcher con spazi'; self.kit.mkdir()
        self.raw = (ROOT/'weapons.example.ini').read_bytes()
        (self.kit/'weapons.example.ini').write_bytes(self.raw)

    def make_runtime(self, fixed=False):
        exe = self.parent/'Runtime con spazi'/'rerac.exe'; exe.parent.mkdir(exist_ok=True)
        (exe.parent/'assets/shaders').mkdir(parents=True, exist_ok=True)
        (exe.parent/'assets/shaders/test.wgsl').write_text('// shader')
        version = dict(name='rerac', game='rac1', data_format=1, weapon_evolution_config=51, version='0.3.4-fixed' if fixed else '0.3.4')
        code = '''#!/usr/bin/env python3
import hashlib,json,os,sys
from pathlib import Path
VERSION=REPLACE_VERSION
if '--version-json' in sys.argv:
 print(json.dumps(VERSION));sys.exit(0)
ini=Path(os.environ['RC_WEAPON_EVOLUTION_INI'])
print('weapon evolution: config loaded from '+str(ini),flush=True)
Path('observed.json').write_text(json.dumps({'args':sys.argv[1:],'env':dict(os.environ),'cwd':str(Path.cwd()),'sha256':hashlib.sha256(ini.read_bytes()).hexdigest()}))
'''.replace('REPLACE_VERSION', repr(version))
        exe.write_text(code); exe.chmod(0o755)
        data = self.parent/'Dati del gioco'; data.mkdir(exist_ok=True)
        (data/'toc.bin').touch(); (data/'extract-info.json').write_text('{}')
        return exe, data

    def test_all_approved_values_and_every_color_have_metadata(self):
        from schema import LABELS, COLORS
        doc = c.checked_document(self.raw)
        self.assertEqual(len(doc.sections), 92); self.assertEqual(len(doc.entries), 1190)
        self.assertEqual(doc.render(), self.raw)
        self.assertEqual({key for _, key in doc.entries}-{*LABELS}, set())
        self.assertTrue(all(key in COLORS for _, key in doc.entries if key.endswith('_color')))
        self.assertEqual(doc.entries['weapon.blaster','xp_v1_to_v2'], '500')
        self.assertEqual(doc.entries['weapon.bomb_glove','xp_v4_to_v5'], '5000')

    def test_lossless_edits_preserve_bom_crlf_color_comment_and_unedited_values(self):
        raw = b'\xef\xbb\xbf'+self.raw.replace(b'\n', b'\r\n')
        raw = raw.replace(b'projectile_color = default', b'projectile_color = #AABBCC  ; prova commento', 1)
        doc = c.checked_document(raw); self.assertEqual(doc.render(), raw)
        pair = 'weapon.blaster.level1', 'projectile_color'
        output = doc.render({pair: 'FA1200'})
        self.assertEqual(output, raw.replace(b'#AABBCC  ; prova commento', b'FA1200  ; prova commento', 1))
        self.assertEqual(c.checked_document(output).entries[pair], 'FA1200')

    def test_last_line_without_newline_and_comment_whitespace_survive(self):
        raw = b'[x]\r\n  key  =  1.00  ; comment'
        doc = c.IniDocument(raw)
        self.assertEqual(doc.render({('x','key'): '2.5'}), b'[x]\r\n  key  =  2.5  ; comment')
        with self.assertRaises(ValueError): doc.render({('x','key'): '2\nother=3'})
        with self.assertRaises(ValueError): c.IniDocument(b'[x]\na=1\na=2\n')

    def test_invalid_parameters_rejected_before_saving(self):
        examples = [
            ('weapon.bomb_glove.level1','max_ammo','0'), ('weapon.bomb_glove.level1','max_ammo','40.5'),
            ('weapon.bomb_glove.level1','throw_range','6'), ('weapon.pyrocitor.level1','flame_range','4'),
            ('weapon.blaster.level1','damage_multiplier','NaN'), ('weapon.blaster.level1','projectile_color','GG0000'),
            ('weapon.blaster.level1','gold_effects','maybe'), ('weapon.taunter.level1','wave_alpha','129'),
            ('weapon.taunter.level1','lure_cone_degrees','181'), ('weapon.taunter.level1','stun_efficiency_percent','101'),
            ('weapon.taunter.level1','wave_interval_ticks','0'), ('weapon.suck_cannon.level1','capacity','11'),
            ('weapon.drone_device.level1','drone_count','11'), ('hud','xp_bar_height_px','1'),
            ('hud','level_up_popup_duration_seconds','16'), ('general','xp_per_damage','0'),
            ('weapon.blaster','id','16'), ('weapon.blaster','xp_v1_to_v2','0'),
            ('general','xp_per_damage','1e-100'), ('weapon.blaster.level1','damage_multiplier','1_000'),
        ]
        doc = c.checked_document(self.raw)
        for section, key, value in examples:
            with self.subTest(section=section,key=key,value=value):
                with self.assertRaises(ValueError): c.checked_document(doc.render({(section,key): value}))
        with self.assertRaises(ValueError): c.checked_document(self.raw.replace(b'[weapon.tesla_claw.level5]', b'[weapon.unknown.level5]'))
        with self.assertRaises(ValueError): c.checked_document(self.raw.replace(b'xp_v1_to_v2 = 400', b'xp_unknown = 400', 1))
        for section, key in [('weapon.bomb_glove.level1','auto_lock_range'), ('weapon.decoy_glove.level1','lure_range')]:
            c.validate_value(section,key,'default')

    def test_store_complete_presets_backups_case_insensitive_and_reversible_delete(self):
        base = self.kit/'weapons.ini'; base.write_bytes(self.raw)
        store = c.PresetStore(self.kit); store.save('Il mio preset', self.raw)
        changed = c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'): '48'})
        store.save('IL MIO PRESET', changed)
        self.assertEqual(store.names(), ['Il mio preset'])
        self.assertEqual(store.read('il mio preset').raw, changed)
        self.assertEqual(base.read_bytes(), self.raw)
        backups = list((self.kit/'backup/presets').rglob('*.ini'))
        self.assertEqual(len(backups), 1); self.assertEqual(backups[0].read_bytes(), self.raw)
        removed = store.remove('Il mio preset')
        self.assertEqual(store.names(), []); self.assertEqual(removed.read_bytes(), changed)
        store.save('Ripristinato', removed.read_bytes()); self.assertEqual(store.read('Ripristinato').raw, changed)

    def test_invalid_preset_names_cannot_escape_the_folder(self):
        store = c.PresetStore(self.kit)
        for name in ('../escape', '..', 'C:\\escape', 'a/b', 'CON', 'NUL.ini', 'COM1', 'A'*61, 'x\nconfig'):
            with self.subTest(name=name), self.assertRaises(ValueError): store.save(name, self.raw)
        self.assertEqual(c.preset_name('Viola e giallo'), 'Viola e giallo')

    def test_bootstrap_prefers_latest_and_never_overwrites_local_ini(self):
        latest = self.parent/c.PREVIOUS[0]; latest.mkdir()
        older = self.parent/c.PREVIOUS[1]; older.mkdir()
        changed = c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'): '48'})
        (latest/'weapons.ini').write_bytes(changed); (older/'weapons.ini').write_bytes(self.raw)
        local, source = c.bootstrap_ini(self.kit)
        self.assertEqual(local.read_bytes(), changed); self.assertEqual(source, str(latest/'weapons.ini'))
        local.write_bytes(self.raw); c.bootstrap_ini(self.kit)
        self.assertEqual(local.read_bytes(), self.raw); self.assertEqual((latest/'weapons.ini').read_bytes(), changed)

    def test_environment_is_process_local_and_preserves_real_saves(self):
        env = {'RC_SAVE_DIR':'custom saves', 'RC_LEVEL':'3', 'RC_GIVE_ITEMS':'all', 'RC_WEAPON_EVOLUTION_INI':'wrong.ini'}
        original = dict(env); ini = self.kit/'runs/path with spaces/weapons.ini'; data = self.parent/'dati'
        got = c.game_environment(ini, data, env=env)
        self.assertEqual(env, original)
        self.assertEqual(got['RC_SAVE_DIR'], 'custom saves'); self.assertNotIn('RC_LEVEL', got)
        self.assertEqual(got['RC_WEAPON_EVOLUTION_INI'], str(ini.resolve()))
        self.assertEqual(got['RC_COMBAT_STRAFE'],'1'); self.assertEqual(got['RC_FRONTEND'],'1')
        self.assertNotIn('RC_SAVE_DIR', c.game_environment(ini, data, env={'RC_SAVE_DIR':'0'}))
        save = self.kit/'save separati'
        self.assertEqual(c.game_environment(ini, data, save, env=env)['RC_SAVE_DIR'], str(save.resolve()))

    @unittest.skipIf(os.name == 'nt', 'POSIX shebang fixture; Windows launch path is covered by mocked process tests')
    def test_real_child_reads_exact_session_ini_and_selected_values(self):
        exe, data = self.make_runtime()
        raw = b'\xef\xbb\xbf'+c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'): '57'})
        (self.kit/'weapons.ini').write_bytes(self.raw)
        env = dict(os.environ, RC_LEVEL='3', RC_SAVE_DIR='actual saves')
        session = c.launch_game(self.kit, raw, 'Preset viola', exe, data, env=env)
        self.assertEqual(session.process.wait(timeout=5), 0)
        observed = json.loads((exe.parent/'observed.json').read_text())
        self.assertEqual(observed['args'], ['--data-dir',str(data.resolve())])
        self.assertEqual(observed['env']['RC_WEAPON_EVOLUTION_INI'], str(session.ini.resolve()))
        self.assertNotIn('RC_LEVEL', observed['env']); self.assertEqual(observed['env']['RC_SAVE_DIR'],'actual saves')
        self.assertEqual(observed['cwd'],str(exe.parent))
        self.assertEqual(c.checked_document(session.ini.read_bytes()).entries['weapon.blaster.level1','max_ammo'],'57')
        self.assertFalse(session.ini.read_bytes().startswith(b'\xef\xbb\xbf'))
        self.assertTrue(c.log_confirmation(session.log.read_text(), session.ini))
        c.PresetStore(self.kit).save('Preset viola', self.raw)
        from migrate_ini import runtime_profile
        self.assertEqual(session.ini.read_bytes(), runtime_profile(raw))
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),self.raw)

    def test_mocked_windows_probe_and_launch_accept_paths_with_spaces(self):
        exe, data = self.make_runtime()
        version = {'name':'rerac','game':'rac1','data_format':1,'weapon_evolution_config':51,'version':'0.3.4'}
        process = Mock()
        with patch.object(c.subprocess, 'run', return_value=SimpleNamespace(stdout=json.dumps(version))), \
             patch.object(c, 'ensure_closed'), patch.object(c.subprocess, 'Popen', return_value=process) as launch:
            session = c.launch_game(self.kit, self.raw, 'Test', exe, data)
        args = launch.call_args
        self.assertIs(session.process,process)
        self.assertEqual(args.args[0][0],str(exe.resolve()))
        self.assertEqual(args.kwargs['env']['RC_WEAPON_EVOLUTION_INI'],str(session.ini.resolve()))
        self.assertNotIn('shell',args.kwargs)

    def test_fixed_build_and_missing_data_do_not_start_game(self):
        exe, data = self.make_runtime()
        fixed = {'name':'rerac','game':'rac1','data_format':1,'weapon_evolution_config':51,'version':'0.3.4-fixed'}
        with patch.object(c.subprocess,'run',return_value=SimpleNamespace(stdout=json.dumps(fixed))), \
             patch.object(c.subprocess,'Popen') as launch:
            with self.assertRaisesRegex(ValueError,'fixed-profile'): c.launch_game(self.kit,self.raw,'Test',exe,data)
            launch.assert_not_called()
        self.assertFalse((self.kit/'runs').exists())
        with patch.object(c,'probe_exe'),patch.object(c.subprocess,'Popen') as launch:
            with self.assertRaises(ValueError): c.launch_game(self.kit,self.raw,'Test',exe,self.parent/'missing')
            launch.assert_not_called()

    def test_log_requires_the_exact_ini_path(self):
        ini = self.kit/'runs/session/weapons.ini'
        self.assertFalse(c.log_confirmation('weapon evolution: config loaded from wrong.ini',ini))
        self.assertFalse(c.log_confirmation('weapon evolution: embedded fixed profile',ini))
        self.assertTrue(c.log_confirmation('weapon evolution: config loaded from '+str(ini.resolve())+'\n',ini))

    def test_distribution_contains_runtime_and_presets_without_personal_paths_or_saves(self):
        import zipfile
        exe,data = self.make_runtime()
        for name in c.DISTRIBUTED_FILES: shutil.copy2(ROOT/name,self.kit/name)
        c.PresetStore(self.kit).save('Alternativa',self.raw)
        (self.kit/'save').mkdir();(self.kit/'save/personal.sav').write_bytes(b'private')
        (self.kit/'runs').mkdir();(self.kit/'runs/private.log').write_bytes(b'private')
        destination = self.parent/'Finale mod'
        version = {'name':'rerac','game':'rac1','data_format':1,'weapon_evolution_config':51,'version':'0.3.4'}
        with patch.object(c,'probe_exe',return_value=version):
            archive = c.package_distribution(self.kit,destination,self.raw,exe)
        with zipfile.ZipFile(archive) as z:
            self.assertIsNone(z.testzip()); names=z.namelist()
            self.assertIn('Finale mod/runtime/rerac.exe',names)
            self.assertIn('Finale mod/runtime/assets/shaders/test.wgsl',names)
            self.assertIn('Finale mod/presets/Alternativa.ini',names)
            self.assertFalse(any('/save/' in p or '/runs/' in p or '/data/' in p for p in names))
        settings = json.loads((destination/'launcher-settings.json').read_text())
        self.assertEqual(settings, {'exe':'runtime/rerac.exe','data':'','save':'','last_preset':''})
        self.assertNotIn(str(self.parent),(destination/'runtime/build-info.json').read_text())
        expected=hashlib.sha256(archive.read_bytes()).hexdigest()
        self.assertTrue(Path(str(archive)+'.sha256').read_text().startswith(expected))
        with patch.object(c,'probe_exe',return_value=version),self.assertRaises(ValueError):
            c.package_distribution(self.kit,destination,self.raw,exe)

    def test_relative_settings_move_with_the_package(self):
        runtime = self.kit/'runtime/rerac.exe'
        text = c.portable_path(runtime,self.kit)
        self.assertEqual(c.resolve_path(text,self.kit),runtime.resolve())
        c.save_settings(self.kit,{'exe':text,'last_preset':'Viola'})
        moved = self.parent/'spostato';shutil.copytree(self.kit,moved)
        self.assertEqual(c.resolve_path(c.read_settings(moved)['exe'],moved),moved/'runtime/rerac.exe')

    def test_publisher_uses_weapons_ini_as_the_distribution_default(self):
        import package_launcher
        exe, data = self.make_runtime()
        changed=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'59'})
        c.PresetStore(self.kit).save('Ultimo preset',changed)
        (self.kit/'weapons.ini').write_bytes(self.raw)
        c.save_settings(self.kit,{'exe':str(exe),'last_preset':'Ultimo preset'})
        with patch.object(c,'KIT',self.kit),patch.object(sys,'argv',['package_launcher.py','--no-presets']), \
             patch.object(package_launcher,'build_launcher',return_value=self.kit/'Launcher.exe'), patch.object(c,'probe_exe'), \
             patch.object(c,'package_distribution',return_value=self.parent/'final.zip') as package:
            self.assertEqual(package_launcher.main(),0)
        from migrate_ini import convert
        self.assertEqual(package.call_args.args[2],convert(self.raw)[0])
        self.assertEqual(package.call_args.args[3],exe)
        self.assertFalse(package.call_args.args[4])

class GuiWorkflowTests(unittest.TestCase):
    """Exercise saving and copying through the actual GUI controller, without Tk."""
    class Var:
        def __init__(self,value=''):self.value=value
        def get(self):return self.value
        def set(self,value):self.value=value
        def trace_add(self,*_):pass

    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.app=gui.Launcher.__new__(gui.Launcher);a=self.app
        a.kit=Path(self.temp.name).resolve();a.store=c.PresetStore(a.kit)
        a.root=Mock();a.tk=SimpleNamespace(StringVar=self.Var)
        a.preset_var=self.Var();a.document_info=self.Var();a.status=self.Var()
        a.render_page=Mock();a.refresh_presets=Mock();a.remember_settings=Mock()
        a.dialogs=Mock();a.messages=Mock();a.error=Mock()
        a.load_document(c.checked_document((ROOT/'weapons.example.ini').read_bytes()),'')

    def test_controller_saves_then_loads_all_five_grades(self):
        a=self.app;pair='weapon.blaster.level2','projectile_color'
        a.variables[pair].set('AA33FF');a.dialogs.askstring.return_value='Viola'
        self.assertTrue(a.save_preset(new=True));self.assertFalse(a.dirty())
        a.load_document(a.store.read('Viola'),'Viola')
        self.assertEqual(a.variables[pair].get(),'AA33FF')
        self.assertEqual(len(a.variables),1190)

    def test_copy_to_grades_updates_only_that_weapons_parameter(self):
        a=self.app;pair='weapon.blaster.level2','projectile_color'
        a.variables[pair].set('AA33FF');a.copy_to_grades(pair)
        for n in range(1,6):self.assertEqual(a.variables[f'weapon.blaster.level{n}','projectile_color'].get(),'AA33FF')
        self.assertEqual(a.variables['weapon.blaster.level1','max_ammo'].get(),'40')
        self.assertEqual(a.variables['weapon.tesla_claw.level1','beam_core_color'].get(),a.base_values['weapon.tesla_claw.level1','beam_core_color'])

if __name__=='__main__':unittest.main()
