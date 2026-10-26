"""NG+ coefficient ordering, capability checks and lossless profile migration."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import unittest
import launcher_core as core
import migrate_ini
import schema
import i18n
from test_nanotech_ui import profile

class NgPlusUiTests(unittest.TestCase):
    def test_neutral_default_first_in_encounter_group_and_runtime_guard(self):
        doc=core.profile_document(profile())
        self.assertEqual(list(schema.WORLD_DEFAULTS)[0],'ngplus_multiplier')
        self.assertEqual(doc.entries['general','ngplus_multiplier'],'1')
        self.assertEqual(core.required_config(doc),86)
        self.assertLess(doc.keys('general').index('ngplus_multiplier'),doc.keys('general').index('enemy_health_multiplier'))

    def test_existing_values_comments_and_upper_grades_are_preserved(self):
        doc=core.profile_document(profile())
        raw=doc.render({('general','ngplus_multiplier'):'2',('general','enemy_speed_multiplier'):'1.5'})
        raw=raw.replace(b'ngplus_multiplier = 2',b'ngplus_multiplier = 2 ; custom NG+ balance')
        after=core.profile_document(raw)
        self.assertEqual(after.entries['general','ngplus_multiplier'],'2')
        self.assertEqual(after.entries['general','enemy_speed_multiplier'],'1.5')
        self.assertIn(b'; custom NG+ balance',after.raw)
        self.assertEqual(migrate_ini.convert(after.raw)[0],after.raw)
        for raw in ['0','-1','NaN','inf','4.01']:
            with self.assertRaises(ValueError):core.validate_value('general','ngplus_multiplier',raw)
        for raw in ['0.1','1','1.5','2','4']:core.validate_value('general','ngplus_multiplier',raw)

    def test_every_language_explains_the_proportional_scale(self):
        for lang in i18n.LANGUAGES:
            self.assertIn(lang,i18n.FIELD_LABELS['ngplus_multiplier'])
            self.assertIn(lang,i18n.HELP_TEXT['ngplus_multiplier'])
            self.assertIn('1.5',i18n.HELP_TEXT['ngplus_multiplier'][lang])

if __name__=='__main__':unittest.main()
