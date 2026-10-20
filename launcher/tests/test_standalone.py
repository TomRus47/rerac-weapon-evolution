from pathlib import Path
import hashlib,json,os,shutil,struct,subprocess,sys,tempfile,unittest,zipfile
from unittest.mock import patch,Mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
import launcher_core as c
import frozen_support as f
import build_launcher_exe as build
import package_launcher as publish
from migrate_ini import convert

def pe_bytes():
    raw=bytearray(70);raw[:2]=b'MZ';struct.pack_into('<I',raw,60,64)
    raw[64:68]=b'PE\0\0';struct.pack_into('<H',raw,68,0x8664)
    return bytes(raw)

class StandaloneTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='launcher exe con spazi ');self.addCleanup(self.tmp.cleanup)
        self.parent=Path(self.tmp.name).resolve();self.kit=self.parent/'Mod';self.kit.mkdir()
        self.raw=convert((ROOT/'weapons.example.ini').read_bytes())[0]
        (self.kit/'weapons.ini').write_bytes(self.raw)
        (self.kit/'weapons.example.ini').write_bytes(self.raw)
        shutil.copy2(ROOT/'LICENSE',self.kit/'LICENSE')
        self.exe=self.kit/'runtime/rerac.exe';self.exe.parent.mkdir();self.exe.write_bytes(b'game')
        shaders=self.exe.parent/'assets/shaders';shaders.mkdir(parents=True);(shaders/'one.wgsl').write_bytes(b'shader')
        self.launcher=self.kit/'launcher-bin'/build.EXE_NAME;self.launcher.parent.mkdir();self.launcher.write_bytes(pe_bytes())
        self.version={'name':'rerac','game':'rac1','version':'0.3.4','data_format':1,'weapon_evolution_config':38}
    def test_frozen_paths_use_the_exe_folder_and_not_temporary_unpacking_or_cwd(self):
        frozen=self.parent/'Desktop/mod/ReRAC_Weapon_Evolution.exe';unpack=self.parent/'Temporary/_MEI'
        with patch.object(sys,'frozen',True,create=True),patch.object(sys,'executable',str(frozen)),patch.object(sys,'_MEIPASS',str(unpack),create=True):
            self.assertEqual(f.application_folder(unpack/'launcher_core.py'),frozen.parent.resolve())
        self.assertEqual(f.application_folder(self.kit/'launcher_core.py'),self.kit)
    def test_native_dll_directory_is_restored_even_when_game_launch_fails(self):
        with patch.object(sys,'frozen',True,create=True),patch.object(sys,'platform','win32'),patch.object(sys,'_MEIPASS','bundle',create=True),patch.object(f,'_set_dll_directory') as setter:
            with self.assertRaises(RuntimeError):
                with f.native_process_context():raise RuntimeError('game failed')
            self.assertEqual([call.args[0] for call in setter.call_args_list],[None,'bundle'])
    def test_native_environment_keeps_real_paths_and_removes_only_the_unpacking_tree(self):
        unpack=self.parent/'_MEI';system=self.parent/'system'
        original={'PATH':os.pathsep.join([str(unpack),str(unpack/'lib'),str(system)]),'RC_SAVE_DIR':'my saves'}
        with patch.object(sys,'frozen',True,create=True),patch.object(sys,'_MEIPASS',str(unpack),create=True):
            clean=f.native_environment(original)
        self.assertEqual(clean,{'PATH':str(system),'RC_SAVE_DIR':'my saves'})
        self.assertIn(str(unpack),original['PATH'])
    def test_local_runtime_wins_over_a_saved_v35_path(self):
        stale=self.parent/'v35/rerac.exe';stale.parent.mkdir();stale.write_bytes(b'old')
        self.assertEqual(c.preferred_exe(self.kit,str(stale)),self.exe.resolve())
    def test_public_zip_contains_no_python_scripts_or_build_tools_and_moves_as_a_whole(self):
        c.PresetStore(self.kit).save('Purple',self.raw)
        (self.kit/'runs').mkdir();(self.kit/'runs/private.log').write_bytes(b'private')
        (self.kit/'save').mkdir();(self.kit/'save/private.sav').write_bytes(b'private')
        licenses=self.launcher.parent/'licenses';licenses.mkdir();(licenses/'Python.txt').write_text('License')
        destination=self.parent/'Finale'
        with patch.object(c,'probe_exe',return_value=self.version):
            archive=c.package_distribution(self.kit,destination,self.raw,self.exe,launcher_exe=self.launcher)
        with zipfile.ZipFile(archive) as z:
            names=z.namelist();self.assertIsNone(z.testzip())
            self.assertIn('Finale/'+build.EXE_NAME,names)
            self.assertIn('Finale/runtime/rerac.exe',names)
            self.assertIn('Finale/licenses/Python.txt',names)
            self.assertIn('Finale/presets/Purple.ini',names)
            self.assertFalse(any(n.endswith(('.py','.bat','.pyc')) or '/runs/' in n or '/save/' in n for n in names))
            self.assertEqual(z.read('Finale/weapons.ini'),self.raw)
        self.assertIn('do not need to install them',(destination/'LEGGIMI.txt').read_text())
        moved=self.parent/'Desktop copia';shutil.move(destination,moved)
        settings=c.read_settings(moved)
        self.assertEqual(c.preferred_exe(moved,settings['exe']),moved/'runtime/rerac.exe')
        self.assertEqual(settings['data'],'')
        self.assertNotIn(str(self.kit),(moved/'runtime/build-info.json').read_text())
    def test_publisher_builds_an_exe_and_uses_the_default_configuration(self):
        preset=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'71'})
        c.PresetStore(self.kit).save('Personal',preset)
        c.save_settings(self.kit,{'exe':'C:/old/v35/rerac.exe','last_preset':'Personal'})
        with patch.object(c,'KIT',self.kit),patch.object(sys,'argv',['package_launcher.py']),patch.object(c,'probe_exe',return_value=self.version),patch.object(publish,'build_launcher',return_value=self.launcher) as make,patch.object(c,'package_distribution',return_value=self.parent/'final.zip') as package:
            self.assertEqual(publish.main(),0)
        make.assert_called_once_with(self.kit)
        self.assertEqual(package.call_args.args[2],self.raw)
        self.assertEqual(package.call_args.args[3],self.exe.resolve())
        self.assertEqual(package.call_args.kwargs['launcher_exe'],self.launcher)
    def test_an_old_runtime_is_refused_before_building_the_launcher(self):
        with patch.object(c,'KIT',self.kit),patch.object(sys,'argv',['package_launcher.py']),patch.object(c,'probe_exe',side_effect=ValueError('old game')),patch.object(publish,'build_launcher') as make:
            self.assertEqual(publish.main(),2)
        make.assert_not_called()
    def test_smoke_test_requires_frozen_python_tk_and_the_correct_folder(self):
        def report(args,**kwargs):
            Path(args[2]).write_text(json.dumps({'frozen':True,'gui_ok':True,'launcher_version':c.VERSION,
                'application_folder':str(self.launcher.parent),'tcl_version':'8.6','tk_version':'8.6'}))
        with patch.object(build.subprocess,'run',side_effect=report):self.assertTrue(build.smoke_test(self.launcher)['frozen'])
        def bad(args,**kwargs):
            Path(args[2]).write_text(json.dumps({'frozen':False,'gui_ok':True}))
        with patch.object(build.subprocess,'run',side_effect=bad):
            with self.assertRaises(ValueError):build.smoke_test(self.launcher)
        self.launcher.write_bytes(b'not a Windows binary')
        with self.assertRaises(ValueError):build.windows_x64(self.launcher)
    def prepare_sources(self):
        for name in build.SOURCES:shutil.copy2(ROOT/name,self.kit/name)
    def test_build_uses_onefile_windowed_and_checks_it_before_publishing(self):
        self.prepare_sources();python=self.parent/'venv/python.exe';calls=[]
        def freeze(args,**kwargs):
            calls.append(args)
            output=Path(args[args.index('--distpath')+1]);output.mkdir(parents=True)
            (output/build.EXE_NAME).write_bytes(pe_bytes())
        with patch.object(sys,'platform','win32'),patch.object(build,'ensure_tools',return_value=python),patch.object(build.subprocess,'run',side_effect=freeze),patch.object(build,'smoke_test',return_value={'frozen':True,'gui_ok':True}) as smoke,patch.object(build,'collect_licenses',side_effect=lambda py,out:out.mkdir()):
            output=build.build(self.kit)
        self.assertEqual(output,self.launcher)
        self.assertEqual(calls[0][:3],[str(python),'-m','PyInstaller'])
        for flag in ['--onefile','--windowed','--noupx']:self.assertIn(flag,calls[0])
        smoke.assert_called_once()
        self.assertEqual(json.loads((self.launcher.parent/'launcher-build.json').read_text())['launcher_version'],'0.10.0')
    def test_cached_exe_is_reused_only_when_sources_and_binary_match(self):
        self.prepare_sources()
        metadata={'sources_sha256':build.source_digest(self.kit),'exe_sha256':hashlib.sha256(self.launcher.read_bytes()).hexdigest()}
        (self.launcher.parent/'launcher-build.json').write_text(json.dumps(metadata))
        with patch.object(sys,'platform','win32'),patch.object(build,'smoke_test') as smoke,patch.object(build,'ensure_tools') as tools:
            self.assertEqual(build.build(self.kit),self.launcher)
        tools.assert_not_called();smoke.assert_called_once_with(self.launcher)
    def test_failed_compilation_keeps_the_existing_exe_untouched(self):
        self.prepare_sources();before=self.launcher.read_bytes()
        with patch.object(sys,'platform','win32'),patch.object(build,'ensure_tools',return_value=self.parent/'python.exe'),patch.object(build.subprocess,'run',side_effect=subprocess.CalledProcessError(1,['freeze'])):
            with self.assertRaises(subprocess.CalledProcessError):build.build(self.kit)
        self.assertEqual(self.launcher.read_bytes(),before)
    def test_non_windows_build_is_explicitly_rejected(self):
        with patch.object(sys,'platform','linux'):
            with self.assertRaisesRegex(ValueError,'Windows'):build.build(self.kit)

if __name__=='__main__':unittest.main()
