"""Global control ordering, preservation and compatibility contracts."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import unittest
import launcher_core as core
import migrate_ini as migration
import schema
import i18n
from test_nanotech_ui import profile

class EncounterUiTests(unittest.TestCase):
    def test_all_new_fields_default_neutral_and_require_the_correct_runtime(self):
        doc=core.profile_document(profile())
        self.assertEqual(core.required_config(doc),92)
        for key in schema.MOTION_DEFAULTS:self.assertEqual(doc.entries['general',key],'1')
        self.assertEqual(doc.entries['general','golden_bolts_one_weapon_per_planet'],'false')
        for key in schema.MOTION_DEFAULTS:
            for value in ['0','-1','NaN','11']:
                with self.assertRaises(ValueError):core.validate_value('general',key,value)
            for value in ['0.1','1','2','4']:core.validate_value('general',key,value)

    def test_custom_values_comments_bom_and_crlf_survive_reordering(self):
        raw=b'\xef\xbb\xbf[general]\r\nxp_per_damage = 0.17 ; balance\r\nenemy_speed_multiplier = 2.5 ; fast\r\ncustom = keep\r\nenemy_damage_multiplier = 1.3\r\nenemy_projectile_speed_multiplier = 3\r\n[hud]\r\nnanotech_xp_bar_opacity_percent = 42\r\nlevel_up_sound = off\r\nname_opacity_percent = 65\r\n[weapon.blaster.level1]\r\nmax_ammo = 42\r\n'
        before=core.IniDocument(raw);changed=migration.upgrade_global_settings(raw);after=core.IniDocument(changed)
        for key,value in before.entries.items():self.assertEqual(after.entries[key],value)
        self.assertIn(b'2.5 ; fast\r\n',changed);self.assertTrue(changed.startswith(b'\xef\xbb\xbf'))
        self.assertNotIn(b'\n',changed.replace(b'\r\n',b''));self.assertEqual(migration.upgrade_global_settings(changed),changed)
        keys=after.keys('general');self.assertEqual(keys[keys.index('enemy_damage_multiplier')+1],'enemy_speed_multiplier')
        hud=after.keys('hud');self.assertEqual(hud[:2],['name_opacity_percent','nanotech_xp_bar_opacity_percent'])

    def test_editor_groups_keep_motion_and_visibility_controls_adjacent(self):
        doc=core.profile_document(profile());base,_=schema.global_field_groups(doc)
        keys=list(schema.WORLD_DEFAULTS);at=keys.index('enemy_damage_multiplier')
        self.assertEqual(keys[at:at+4],['enemy_damage_multiplier','enemy_speed_multiplier','enemy_projectile_range_multiplier','enemy_projectile_speed_multiplier'])
        hud=[k for section,k in base if section=='hud'];self.assertEqual(hud[:2],['name_opacity_percent','nanotech_xp_bar_opacity_percent'])
        general=[k for section,k in base if section=='general'];self.assertEqual(general[-5:],['golden_bolts_to_weapons','golden_bolts_early_bias_percent','golden_bolts_one_weapon_per_planet','dpad_quick_select','r1_experimental'])

    def test_new_controls_are_translated_in_all_languages(self):
        for key in [*schema.MOTION_DEFAULTS,'golden_bolts_one_weapon_per_planet']:
            for lang in i18n.LANGUAGES:
                self.assertIn(lang,i18n.FIELD_LABELS[key]);self.assertIn(lang,i18n.HELP_TEXT[key])

if __name__=='__main__':unittest.main()
