"""Configurable ammo mode, exact UI grouping and safe local source updates."""
from pathlib import Path
import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'launcher'))
import launcher_core as core
import launcher as gui
from migrate_ini import convert
from schema import AMMO_DROP_DEFAULTS, AMMO_UNLOCK_DEFAULTS, GLOBAL_EXPERIMENTAL_DEFAULTS, global_field_groups, normalized_ammo_chances

spec = importlib.util.spec_from_file_location('local_test_update', ROOT / 'tools/mod/local_test_update.py')
updater = importlib.util.module_from_spec(spec)
spec.loader.exec_module(updater)


class ModeSettingsTests(unittest.TestCase):
    def setUp(self):
        self.raw = (ROOT / 'launcher/weapons.default.ini').read_bytes()

    def test_default_thresholds_and_two_card_groups_put_mode_switch_first(self):
        doc = core.checked_document(self.raw)
        self.assertEqual([doc.entries['general', k] for k in AMMO_UNLOCK_DEFAULTS], ['10000','25000','50000','75000'])
        base, mode = global_field_groups(doc)
        self.assertEqual(mode[0], ('general','experimental_ammo_unlock_mode'))
        self.assertEqual({key for _,key in mode}, set(GLOBAL_EXPERIMENTAL_DEFAULTS))
        self.assertTrue(all(key not in GLOBAL_EXPERIMENTAL_DEFAULTS for _,key in base))
        self.assertIn(('hud','xp_bar_height_px'), base)
        self.assertNotIn(('general','wrench_gold_texture'), base)
        self.assertEqual(core.required_config(doc),90)

    def test_probabilities_normalize_without_implicitly_reenabling_zero_types(self):
        values = dict.fromkeys(AMMO_DROP_DEFAULTS, '0')
        with self.assertRaises(ValueError): normalized_ammo_chances(values)
        values['ammo_drop_chance_bomb_glove']='20'
        values['ammo_drop_chance_ryno']='60'
        chances = normalized_ammo_chances(values)
        self.assertEqual(chances['ammo_drop_chance_bomb_glove'],25)
        self.assertEqual(chances['ammo_drop_chance_ryno'],75)
        self.assertEqual(sum(chances.values()),100)
        self.assertEqual(chances['ammo_drop_chance_tesla_claw'],0)
        doc = core.IniDocument(self.raw)
        core.checked_document(doc.render({('general',k):v for k,v in values.items()}))
        for key,value in [(next(iter(AMMO_UNLOCK_DEFAULTS)),'2.5'), ('ammo_unlock_walloper_bolts','1000000001'),
                          ('ammo_drop_chance_ryno','NaN'),('ammo_drop_chance_blaster','101')]:
            with self.assertRaises(ValueError): core.validate_value('general',key,value)

    def test_v51_profile_migration_preserves_every_old_value_and_is_idempotent(self):
        current = core.IniDocument(self.raw)
        added = {('general', key) for key in (*AMMO_UNLOCK_DEFAULTS,*AMMO_DROP_DEFAULTS)}
        remove = {current.addresses[p] for p in added}
        old = b'\xef\xbb\xbf'+''.join(line for i,line in enumerate(current.lines) if i not in remove).replace('\n','\r\n').encode()
        original = core.IniDocument(old)
        migrated = convert(old)[0]
        loaded = core.checked_document(migrated)
        self.assertTrue(migrated.startswith(b'\xef\xbb\xbf'))
        self.assertIn(b'\r\n',migrated)
        for pair,value in original.entries.items(): self.assertEqual(loaded.entries[pair],value)
        self.assertEqual(convert(migrated)[0],migrated)

    def test_real_global_renderer_keeps_current_cards_and_survival_enabler_first(self):
        app=gui.Launcher.__new__(gui.Launcher)
        app.form=Mock();app.form.winfo_children.return_value=[]
        app.page='global';app.nav_buttons={};app.page_title=Mock();app.context=Mock()
        app.doc=core.checked_document(self.raw);app.ttk=Mock()
        app.tk=SimpleNamespace(StringVar=Mock)
        app.card=Mock(side_effect=lambda *_:Mock());app.fields=Mock();app.matches=lambda *_:True
        app.variables={pair:SimpleNamespace(get=lambda value=value:value) for pair,value in app.doc.entries.items()}
        app.render_page()
        self.assertEqual([call.args[1] for call in app.card.call_args_list],['Base settings','XP Bars and settings','Encounter difficulty and Nanotech','Randomizer','Ammo-Survival Mode'])
        calls=app.fields.call_args_list
        start=next(i for i,c in enumerate(calls) if c.args[2]==['experimental_ammo_unlock_mode'])
        self.assertEqual(calls[start+1].args[2],[list(AMMO_UNLOCK_DEFAULTS)[0]])
        self.assertEqual([c.args[2][0] for c in calls[start+1:]],[*AMMO_UNLOCK_DEFAULTS,*AMMO_DROP_DEFAULTS])


class LocalUpdaterTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='local test kit with spaces ')
        self.addCleanup(self.temp.cleanup)
        root=Path(self.temp.name)
        self.payload=root/'bundle/source'; self.payload.mkdir(parents=True)
        self.source=root/'existing sources';self.source.mkdir()
        self.kit=root/'existing mod';self.kit.mkdir()
        for relative in ('Cargo.toml','Cargo.lock','crates/rc-game/Cargo.toml','crates/rc-engine/Cargo.toml','crates/rc-game/src/moby_update/real_hp.rs'):
            p=self.source/relative;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b'old source')
        p=self.payload/'Cargo.toml';p.write_bytes(b'new source')
        (self.payload/'launcher').mkdir()
        for name in ('weapons.default.ini','launcher.ico'):
            (self.payload/'launcher'/name).write_bytes((ROOT/'launcher'/name).read_bytes())
        self.manifest=root/'bundle/source-files.json'
        files={p.relative_to(self.payload).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in self.payload.rglob('*') if p.is_file()}
        self.manifest.write_text(json.dumps({'patch':'0.1.6.59','files':files}))
        self.ini=(ROOT/'launcher/weapons.default.ini').read_bytes().replace(b'max_ammo = 40',b'max_ammo = 47',1)
        (self.kit/'weapons.ini').write_bytes(self.ini)
        (self.kit/'presets').mkdir();(self.kit/'presets/My build.ini').write_bytes(self.ini)
        (self.kit/'launcher.ico').write_bytes(b'my custom icon')
        (self.kit/'launcher-settings.json').write_text(json.dumps({'exe':'old.exe','data':'D:/disc','last_preset':'My build'}))

    def test_install_preserves_profiles_settings_icon_and_cached_builds_with_backups(self):
        (self.source/'target').mkdir();(self.source/'target/cache').write_bytes(b'cached build')
        backup=updater.install(self.payload,self.source,self.kit,self.manifest)
        self.assertEqual((self.source/'Cargo.toml').read_bytes(),b'new source')
        self.assertEqual((backup/'source/Cargo.toml').read_bytes(),b'old source')
        self.assertEqual((self.source/'target/cache').read_bytes(),b'cached build')
        self.assertEqual((self.kit/'launcher.ico').read_bytes(),b'my custom icon')
        self.assertEqual((self.kit/'weapons.ini').read_bytes(),self.ini)
        self.assertEqual((self.kit/'presets/My build.ini').read_bytes(),self.ini)
        settings=json.loads((self.kit/'launcher-settings.json').read_text())
        self.assertEqual(settings,{'exe':'runtime/rerac.exe','data':'D:/disc','last_preset':'My build'})

    def test_corrupt_or_escaping_payload_stops_before_any_source_write(self):
        (self.payload/'Cargo.toml').write_bytes(b'damaged')
        with self.assertRaisesRegex(ValueError,'Damaged source'):
            updater.install(self.payload,self.source,self.kit,self.manifest)
        self.assertEqual((self.source/'Cargo.toml').read_bytes(),b'old source')
        self.manifest.write_text(json.dumps({'patch':'0.1.6.59','files':{'../outside':'bad'}}))
        with self.assertRaisesRegex(ValueError,'Unsafe source'):
            updater.source_files(self.payload,self.manifest)

    def test_invalid_personal_profile_stops_before_any_source_write(self):
        (self.kit/'weapons.ini').write_bytes(b'not an ini')
        with self.assertRaisesRegex(ValueError,'Fix this configuration'):
            updater.install(self.payload,self.source,self.kit,self.manifest)
        self.assertEqual((self.source/'Cargo.toml').read_bytes(),b'old source')

    def test_failed_write_rolls_back_applied_source_and_launcher_changes(self):
        write=core.atomic_write
        before=(self.kit/'launcher-settings.json').read_bytes()
        def fail_at_opener(path,data):
            if path==self.kit/'OPEN_TEST_LAUNCHER_v59.bat': raise OSError('simulated disk failure')
            return write(path,data)
        with patch.object(core,'atomic_write',side_effect=fail_at_opener):
            with self.assertRaisesRegex(OSError,'simulated'):
                updater.install(self.payload,self.source,self.kit,self.manifest)
        self.assertEqual((self.source/'Cargo.toml').read_bytes(),b'old source')
        self.assertEqual((self.kit/'launcher-settings.json').read_bytes(),before)
        self.assertFalse((self.kit/'weapons.default.ini').exists())


if __name__=='__main__': unittest.main()
