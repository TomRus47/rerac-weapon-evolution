"""Additive health semantics, runtime compatibility and profile preservation."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from pathlib import Path
import unittest
import launcher_core as core
import i18n
import schema

class AdditiveNanotechTests(unittest.TestCase):
    def test_limits_and_runtime_guard(self):
        self.assertEqual(core.numeric_range('general','nanotech_max_health'),(0,64))
        for value in ['0','20','64']:self.assertEqual(core.validate_value('general','nanotech_max_health',value),value)
        for value in ['-1','65','NaN','1.5']:
            with self.subTest(value=value),self.assertRaises(ValueError):core.validate_value('general','nanotech_max_health',value)
        doc=core.IniDocument(b'[general]\nnanotech_max_health=20\nenemy_damage_by_health=false\n')
        self.assertEqual(core.required_config(doc),75)

    def test_existing_user_balance_is_preserved(self):
        for name in ['weapons.ini','weapons.default.ini','weapons.example.ini']:
            raw=(Path(__file__).resolve().parents[1]/('weapons.default.ini' if name=='weapons.ini' else name)).read_bytes()
            before=core.IniDocument(raw);after=core.profile_document(raw)
            for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value,pair)
            self.assertEqual(core.required_config(after),86)

    def test_all_languages_explain_independent_purchases(self):
        try:
            for lang in i18n.LANGUAGES:
                i18n.set_language(lang)
                text=i18n.help_text('general','nanotech_max_health')
                for term in ['0-64','Premium','Ultra']:self.assertIn(term,text)
                self.assertIn(lang,i18n.FIELD_LABELS['nanotech_max_health'])
        finally:i18n.set_language('en')

if __name__=='__main__':unittest.main()
