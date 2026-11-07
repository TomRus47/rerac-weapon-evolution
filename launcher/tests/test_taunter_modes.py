"""Taunter R1 migration, per-grade controls and localized editor grouping."""
from pathlib import Path
import sys, unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import launcher_core as core
import migrate_ini, schema, i18n

class TaunterModeTests(unittest.TestCase):
    def test_legacy_placeholder_migrates_once_and_all_balance_is_retained(self):
        raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        # Reconstruct a v129 profile: no mode switch, unsupported zero R1 gate.
        old=b'\n'.join(line for line in raw.split(b'\n') if not line.startswith(b'offensive_taunt_enabled ='))
        d=core.IniDocument(old)
        old=d.render({(f'weapon.taunter.level{g}','r1_experimental_max'):'0' for g in range(1,9)})
        before=core.IniDocument(old);new=migrate_ini.convert(old)[0];after=core.checked_document(new)
        for pair,value in before.entries.items():
            if pair[0].startswith('weapon.taunter.level') and pair[1]=='r1_experimental_max':self.assertEqual(after.entries[pair],'1')
            else:self.assertEqual(after.entries[pair],value)
        for g in range(1,9):self.assertEqual(after.entries[f'weapon.taunter.level{g}','offensive_taunt_enabled'],'true')
        self.assertEqual(migrate_ini.convert(new)[0],new)
        changed=after.render({('weapon.taunter.level3','r1_experimental_max'):'0',('weapon.taunter.level3','offensive_taunt_enabled'):'false'})
        self.assertEqual(migrate_ini.convert(changed)[0],changed)

    def test_editor_places_only_offensive_settings_in_the_experimental_frame(self):
        d=core.profile_document((Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes())
        for g in range(1,9):
            sec=f'weapon.taunter.level{g}';standard,experimental=schema.field_groups('taunter',d.keys(sec))
            self.assertEqual(experimental[0],'offensive_taunt_enabled')
            for k in schema.TAUNTER_OFFENSIVE_FIELDS:self.assertIn(k,experimental);self.assertNotIn(k,standard)
            for k in ('lure_range','lure_cone_degrees','xp_attract_fraction','wave_color'):self.assertIn(k,standard)
            self.assertEqual(standard[-1],'wave_color')
            self.assertEqual(schema.r1_mode_options(sec),('0','1'))
        self.assertEqual(core.required_config(d),95)
        with self.assertRaises(ValueError):core.validate_value('weapon.taunter.level2','r1_experimental_max','2')

    def test_switch_uses_localized_off_on_and_has_help_in_five_languages(self):
        key='offensive_taunt_enabled';self.assertIn(key,schema.BOOLEANS)
        previous=i18n.language()
        try:
            for lang in i18n.LANGUAGES:
                i18n.set_language(lang)
                self.assertTrue(i18n.FIELD_LABELS[key][lang]);self.assertTrue(i18n.HELP_TEXT[key][lang])
                self.assertEqual(i18n.boolean_value(i18n.boolean_display('false')),'false')
                self.assertEqual(i18n.boolean_value(i18n.boolean_display('true')),'true')
        finally:i18n.set_language(previous)
