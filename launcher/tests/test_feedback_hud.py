"""v102 grade, palette, grouping, profile and renderer integration contracts."""
from pathlib import Path
import sys,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as core
import schema,i18n
from migrate_ini import convert
class FeedbackHudTests(unittest.TestCase):
    def setUp(self):self.raw=(ROOT/'weapons.default.ini').read_bytes()
    def test_eight_independent_grades_and_experiment_colors_last(self):
        doc=core.profile_document(self.raw)
        for n in range(1,9):
            section=f'weapon.wrench.level{n}';std,exp=schema.field_groups('wrench',doc.keys(section))
            self.assertIn('burn_on_hit',std);self.assertIn('hud_quickselect_only',std)
            self.assertEqual(exp[0],'slam_explosion_enabled');self.assertEqual(exp[-1],'slam_explosion_color')
        edits={('weapon.wrench.level2','burn_on_hit'):'true',('weapon.wrench.level8','hud_quickselect_only'):'true',('weapon.wrench.level3','slam_explosion_damage_hp'):'123',('weapon.wrench.level3','slam_explosion_color'):'FF22AB'}
        changed=core.profile_document(doc.render(edits))
        for pair,value in edits.items():self.assertEqual(changed.entries[pair],value)
        self.assertEqual(changed.entries['weapon.wrench.level1','burn_on_hit'],'false')
    def test_old_values_survive_and_conversion_is_idempotent(self):
        before=core.IniDocument(self.raw);out=convert(self.raw)[0];after=core.checked_document(out)
        self.assertEqual(before.entries,after.entries);self.assertEqual(convert(out)[0],out)
        self.assertEqual(core.required_config(after),92)
        self.assertIn(('hud','weapon_bar_alternative_position'),after.entries)
    def test_bounds_and_all_five_localizations(self):
        for key,value in [('slam_explosion_damage_hp','-1'),('slam_explosion_radius','0'),('slam_explosion_radius','31'),('burn_on_hit','perhaps'),('hud_quickselect_only','2'),('slam_explosion_color','not-a-color')]:
            with self.assertRaises(ValueError):core.validate_value('weapon.wrench.level1',key,value)
        for key in (*schema.WRENCH_FEEDBACK_STANDARD,*schema.WRENCH_SLAM_DEFAULTS,'weapon_bar_alternative_position','weapon_bar_behind_hud'):
            for lang in i18n.LANGUAGES:
                self.assertTrue(i18n.FIELD_LABELS[key][lang]);self.assertTrue(i18n.HELP_TEXT[key][lang])
        self.assertIn('weapon_bar_alternative_position',schema.BOOLEANS)
    def test_layer_setting_is_added_once_and_preserves_explicit_off(self):
        raw=b'\xef\xbb\xbf[hud]\r\nname_opacity_percent = 43\r\n'
        migrated,_=convert(raw)
        self.assertEqual(convert(migrated)[0],migrated)
        self.assertIn(b'weapon_bar_behind_hud = true\r\n',migrated)
        edited=core.IniDocument(migrated).render({('hud','weapon_bar_behind_hud'):'false'})
        self.assertEqual(core.IniDocument(convert(edited)[0]).entries['hud','weapon_bar_behind_hud'],'false')
        self.assertEqual(core.IniDocument(convert(edited)[0]).entries['hud','name_opacity_percent'],'43')

if __name__=='__main__':unittest.main()
