"""Neutral burst-aim migration, grouping, localization and live edit contracts."""
import sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import schema,i18n,launcher_core as core
from migrate_ini import convert
class Combat117Tests(unittest.TestCase):
    def test_old_profile_keeps_balance_and_adds_only_disabled_switch(self):
        raw=(ROOT/'weapons.default.ini').read_bytes()
        old=raw.replace(b'enemy_burst_tracking = false\n',b'')
        before=core.IniDocument(old);after=core.profile_document(convert(old)[0])
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value)
        self.assertEqual(after.entries['general','enemy_burst_tracking'],'false')
        self.assertEqual(convert(convert(old)[0])[0],convert(old)[0])
        self.assertEqual(core.required_config(after),92)
        enabled=core.profile_document(after.render({('general','enemy_burst_tracking'):'true'}))
        self.assertEqual(core.required_config(enabled),93)
    def test_boolean_is_in_encounter_card_with_complete_localizations(self):
        key='enemy_burst_tracking';self.assertIn(key,schema.WORLD_DEFAULTS);self.assertIn(key,schema.BOOLEANS)
        fields=list(schema.WORLD_DEFAULTS)
        self.assertEqual(fields[fields.index('enemy_projectile_speed_multiplier')+1],key)
        for lang in i18n.LANGUAGES:
            self.assertTrue(i18n.FIELD_LABELS[key][lang]);self.assertTrue(i18n.HELP_TEXT[key][lang])
        for value in ['false','true']:core.validate_value('general',key,value)
        with self.assertRaises(ValueError):core.validate_value('general',key,'perhaps')
