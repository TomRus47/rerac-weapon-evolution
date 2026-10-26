"""Headless profile, editor schema and localization regression coverage."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from pathlib import Path
import tempfile
import unittest
import launcher_core as core
import migrate_ini
import schema
import i18n

KEY='nanotech_xp_bar_opacity_percent'


def profile():
    lines=['[general]','xp_per_damage = 0.12','[hud]','name_opacity_percent = 35','xp_bar_height_px = 7']
    for name,(item,_) in schema.WEAPONS.items():
        lines.extend([f'[weapon.{name}]',f'id = {item}'])
        lines.extend(f'{key} = {100*(i+1)}' for i,key in enumerate(schema.XP))
        for level in range(1,6):
            lines.append(f'[weapon.{name}.level{level}]')
            if name=='taunter':continue
            field='damage_per_second' if name in {'pyrocitor','tesla_claw'} else 'chicken_damage_hp' if name=='morph_ray' else 'damage_hp'
            lines.append(f'{field} = {level*125}')
    return ('\n'.join(lines)+'\n').encode()


class NanotechUiTests(unittest.TestCase):
    def test_old_profile_adds_one_independent_default_without_changing_existing_values(self):
        raw=profile();before=core.IniDocument(raw)
        changed=migrate_ini.ensure_hud_fields(raw);after=core.IniDocument(changed)
        self.assertEqual(after.entries.pop(('hud',KEY)),'100')
        self.assertEqual(after.entries,before.entries)
        self.assertEqual(migrate_ini.ensure_hud_fields(changed),changed)
        doc=core.profile_document(raw)
        self.assertEqual(doc.entries['hud',KEY],'100')
        self.assertEqual(doc.entries['hud','name_opacity_percent'],'35')
        self.assertEqual(doc.entries['hud','xp_bar_height_px'],'7')
        self.assertEqual(core.required_config(doc),86)

    def test_opacity_validation_requires_an_integer_percentage(self):
        self.assertIn(KEY,schema.HUD);self.assertIn(KEY,schema.INTEGER)
        for value in ['0','50','100']:self.assertEqual(core.validate_value('hud',KEY,value),value)
        for value in ['-1','101','50.5','NaN','off']:
            with self.subTest(value=value),self.assertRaises(ValueError):core.validate_value('hud',KEY,value)

    def test_custom_preset_opacity_and_balance_survive_save_and_reload(self):
        raw=migrate_ini.ensure_hud_fields(profile()).replace((KEY+' = 100').encode(),(KEY+' = 45').encode())
        with tempfile.TemporaryDirectory() as folder:
            store=core.PresetStore(Path(folder));store.save('Classic',raw)
            doc=core.profile_document(store.read('Classic').raw)
            self.assertEqual(doc.entries['hud',KEY],'45')
            self.assertEqual(doc.entries['general','xp_per_damage'],'0.12')
            self.assertEqual(doc.entries['weapon.blaster.level5','damage_hp'],'625')

    def test_global_settings_include_the_field_in_the_base_hud_group(self):
        doc=core.profile_document(profile());base,mode=schema.global_field_groups(doc)
        self.assertIn(('hud',KEY),base);self.assertNotIn(('hud',KEY),mode)

    def test_label_and_help_exist_in_every_supported_launcher_language(self):
        try:
            for language in i18n.LANGUAGES:
                i18n.set_language(language)
                self.assertIn(language,i18n.FIELD_LABELS[KEY]);self.assertIn(language,i18n.HELP_TEXT[KEY])
                self.assertIn('Nanotech',i18n.label(KEY));self.assertNotIn('_',i18n.label(KEY))
                self.assertIn('100',i18n.help_text('hud',KEY))
        finally:i18n.set_language('en')

    def test_ini_migration_preserves_utf8_bom_and_crlf(self):
        raw=b'\xef\xbb\xbf[hud]\r\nname_opacity_percent = 32\r\n'
        changed=migrate_ini.ensure_hud_fields(raw)
        self.assertTrue(changed.startswith(b'\xef\xbb\xbf'))
        self.assertNotIn(b'\n',changed.replace(b'\r\n',b''))
        self.assertIn(b'name_opacity_percent = 32\r\n',changed)
        for raw in [b'[hud]',b'[general]\nfoo=1']:
            self.assertIn(('hud',KEY),core.IniDocument(migrate_ini.ensure_hud_fields(raw)).entries)


if __name__=='__main__':unittest.main()
