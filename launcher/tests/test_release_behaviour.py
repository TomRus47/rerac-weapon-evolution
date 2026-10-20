from pathlib import Path
import ast,json,re,shutil,sys,tempfile,unittest
from types import SimpleNamespace
from unittest.mock import patch,Mock
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as c
import launcher as gui
import final_release as release
import editabile as manual
from schema import EDITOR_WEAPONS

class Var:
    def __init__(self,value=''):self.value=value
    def get(self):return self.value
    def set(self,value):self.value=value
    def trace_add(self,*args):pass

class DefaultWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(prefix='default release ');self.addCleanup(self.tmp.cleanup)
        self.kit=Path(self.tmp.name);self.raw=(ROOT/'weapons.default.ini').read_bytes()
        for name in ['weapons.ini','weapons.example.ini','weapons.default.ini']:(self.kit/name).write_bytes(self.raw)
        self.exe=self.kit/'runtime/rerac.exe';self.exe.parent.mkdir();self.exe.write_bytes(b'game')
        data=self.kit/'data';data.mkdir();(data/'toc.bin').touch();(data/'extract-info.json').write_text('{}')
    def app(self):
        root=Mock();root.winfo_screenwidth.return_value=1920;root.winfo_screenheight.return_value=1080
        messages=Mock();dialogs=Mock()
        def build_ui(a):a.preset_box={};a.play_button=Mock()
        with patch.object(gui.Launcher,'configure_style'),patch.object(gui.Launcher,'build_ui',build_ui),patch.object(gui.Launcher,'render_page'):
            a=gui.Launcher(root,self.kit,SimpleNamespace(StringVar=Var),Mock(),Mock(),messages,dialogs,Mock())
        a.render_page=Mock();a.error=Mock()
        return a
    def test_startup_restores_the_last_selected_preset(self):
        old=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'81'})
        c.PresetStore(self.kit).save('Previous',old)
        c.save_settings(self.kit,{'last_preset':'Previous','exe':'C:/old/v35/rerac.exe'})
        current=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'61'})
        (self.kit/'weapons.ini').write_bytes(current)
        a=self.app();self.assertEqual(a.current_name,'Previous')
        self.assertEqual(a.variables['weapon.blaster.level1','max_ammo'].get(),'81')
        self.assertEqual(Path(a.exe_var.get()),self.exe)
    def test_deleted_remembered_preset_falls_back_to_weapons_ini_without_a_prompt(self):
        c.save_settings(self.kit,{'last_preset':'Deleted preset'})
        a=self.app();self.assertEqual(a.current_name,'')
        self.assertEqual(a.prepare_launch_raw(),self.raw)
        a.dialogs.askstring.assert_not_called()
    def test_selecting_default_is_remembered_and_manual_edits_are_loaded_next_time(self):
        c.PresetStore(self.kit).save('Previous',self.raw)
        c.save_settings(self.kit,{'last_preset':'Previous'})
        a=self.app();self.assertEqual(a.current_name,'Previous')
        a.preset_var.set(gui.BASE_LABEL);a.select_preset()
        self.assertEqual(c.read_settings(self.kit)['last_preset'],'')
        updated=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'66'})
        (self.kit/'weapons.ini').write_bytes(updated)
        restarted=self.app();self.assertEqual(restarted.current_name,'')
        self.assertEqual(restarted.prepare_launch_raw(),updated)
        restarted.dialogs.askstring.assert_not_called()
    def test_unchanged_default_launch_requires_no_preset_name_or_write(self):
        a=self.app();raw=a.prepare_launch_raw()
        self.assertEqual(raw,self.raw);self.assertFalse(a.dirty())
        a.dialogs.askstring.assert_not_called();a.messages.askyesnocancel.assert_not_called()
        self.assertFalse((self.kit/'presets').exists());self.assertFalse((self.kit/'backup').exists())
    def test_manual_ini_edits_are_reloaded_immediately_before_launch(self):
        a=self.app()
        updated=c.IniDocument(self.raw).render({('weapon.pyrocitor.level2','damage_per_second'):'147'})
        (self.kit/'weapons.ini').write_bytes(updated)
        self.assertEqual(a.prepare_launch_raw(),updated)
        self.assertEqual(a.variables['weapon.pyrocitor.level2','damage_per_second'].get(),'147')
        a.dialogs.askstring.assert_not_called()
    def test_editor_changes_save_to_weapons_ini_with_a_backup_and_no_name_prompt(self):
        a=self.app();a.variables['weapon.blaster.level1','max_ammo'].set('62')
        out=a.prepare_launch_raw();self.assertIsNotNone(out)
        self.assertEqual(c.checked_document(out).entries['weapon.blaster.level1','max_ammo'],'62')
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),out)
        backups=list((self.kit/'backup/weapons.ini').glob('*/weapons.ini'))
        self.assertEqual(len(backups),1);self.assertEqual(backups[0].read_bytes(),self.raw)
        self.assertEqual(a.current_name,'');a.dialogs.askstring.assert_not_called()
        self.assertFalse((self.kit/'presets').exists())
    def test_save_as_creates_an_optional_named_preset_without_changing_the_default(self):
        a=self.app();a.variables['weapon.blaster.level1','max_ammo'].set('63')
        a.dialogs.askstring.return_value='Optional'
        self.assertTrue(a.save_preset(new=True));self.assertEqual(a.current_name,'Optional')
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),self.raw)
        self.assertEqual(a.store.read('Optional').entries['weapon.blaster.level1','max_ammo'],'63')
        restarted=self.app();self.assertEqual(restarted.current_name,'Optional')
        self.assertEqual(restarted.variables['weapon.blaster.level1','max_ammo'].get(),'63')
        restarted.dialogs.askstring.assert_not_called()
    def test_external_changes_are_preserved_when_the_editor_also_has_unsaved_changes(self):
        a=self.app();a.variables['weapon.blaster.level1','max_ammo'].set('64')
        updated=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'65'})
        (self.kit/'weapons.ini').write_bytes(updated)
        self.assertIsNone(a.prepare_launch_raw());a.error.assert_called_once()
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),updated)
    def test_approved_default_is_installed_once_and_manual_changes_survive_rebuilds(self):
        old=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'70'})
        (self.kit/'weapons.ini').write_bytes(old)
        self.assertTrue(release.install_default(self.kit))
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),self.raw)
        backup=list((self.kit/'backup/defaults_before_v05').glob('*/weapons.ini'))
        self.assertEqual(len(backup),1);self.assertEqual(backup[0].read_bytes(),old)
        (self.kit/'weapons.ini').write_bytes(old)
        self.assertFalse(release.install_default(self.kit));self.assertEqual((self.kit/'weapons.ini').read_bytes(),old)
    def test_invalid_default_is_rejected_before_touching_the_existing_file(self):
        (self.kit/'weapons.default.ini').write_bytes(self.raw.replace(b'max_ammo = 40',b'max_ammo = -2',1))
        with self.assertRaises(ValueError):release.install_default(self.kit)
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),self.raw)
        self.assertFalse((self.kit/'defaults-installed-v05.json').exists())
    def test_manual_game_launcher_also_injects_real_hp_only_into_the_session_copy(self):
        (self.exe.parent/'build-info.json').write_text(json.dumps({'mode':'editable','mod_version':'0.1.6.44','root':str(self.kit)}))
        observed={}
        def run(args,**kwargs):
            observed.update(kwargs['env'])
            kwargs['stdout'].write('weapon evolution: config loaded from '+kwargs['env']['RC_WEAPON_EVOLUTION_INI']+'\n')
            return SimpleNamespace(returncode=0)
        with patch.object(manual,'editable_version'),patch.object(manual,'ensure_closed'),patch.dict(manual.os.environ,{'RC_DATA_DIR':str(self.kit/'data')}),patch.object(manual.subprocess,'run',side_effect=run):
            self.assertEqual(manual.launch(self.kit,self.kit),0)
        session=Path(observed['RC_WEAPON_EVOLUTION_INI'])
        self.assertEqual(c.checked_document(session.read_bytes()).entries['general','damage_mode'],'real_hp')
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),self.raw)
    def test_every_editor_weapon_and_setting_has_english_metadata(self):
        self.assertEqual(len(EDITOR_WEAPONS),15);self.assertNotIn('wrench',EDITOR_WEAPONS)
        d=c.profile_document(self.raw)
        from schema import LABELS
        self.assertFalse({key for _,key in d.entries}-{*LABELS})
        self.assertNotIn(('general','damage_mode'),d.entries)
    def test_authored_product_strings_and_default_comments_are_english(self):
        forbidden=re.compile(r'\b(?:Pronto|Rimuovi|Personalizzato|Eseguibile|codice|Salva|avvia|Scegli|scegli|Gioco|gioco|cartella|colore|valori|sorgenti|compilazione|modifiche|Nessun|opacità|ERRORE)\b')
        for name in ['launcher.py','launcher_core.py','schema.py','migrate_ini.py','build_launcher_exe.py','windows_entry.py','package_launcher.py','editabile.py','valida_ini.py','final_release.py']:
            for node in ast.walk(ast.parse((ROOT/name).read_text())):
                if isinstance(node,ast.Constant) and isinstance(node.value,str):
                    self.assertFalse(forbidden.search(node.value),(name,node.lineno,node.value))
        for line in self.raw.decode().splitlines():
            if line.startswith(';'):self.assertFalse(forbidden.search(line),line)

if __name__=='__main__':unittest.main()
