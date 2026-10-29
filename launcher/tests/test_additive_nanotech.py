"""Base-plus-XP cap, independent native purchases and old profile migration."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import unittest
import launcher_core as core
import i18n
from nanotech_cap_profile import KEY, LEGACY_KEY, migrate_health_cap

class AdditiveNanotechTests(unittest.TestCase):
    def test_limits_and_runtime_guard(self):
        self.assertEqual(core.numeric_range('general',KEY),(4,206))
        for value in ['4','24','26','206']:self.assertEqual(core.validate_value('general',KEY,value),value)
        for value in ['0','3','207','NaN','1.5']:
            with self.subTest(value=value),self.assertRaises(ValueError):core.validate_value('general',KEY,value)
        doc=core.IniDocument(('[general]\n'+KEY+'=206\n').encode())
        self.assertEqual(core.required_config(doc),92)

    def test_existing_user_balance_is_preserved(self):
        for name in ['weapons.default.ini','weapons.example.ini']:
            raw=(Path(__file__).resolve().parents[1]/name).read_bytes()
            before=core.IniDocument(raw);after=core.profile_document(raw)
            for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value,pair)
            self.assertEqual(core.required_config(after),92)
        legacy=b'\xef\xbb\xbf[general]\r\nnanotech_max_health = 22 ; personal cap\r\nxp_per_damage = 0.12\r\n'
        migrated=migrate_health_cap(legacy)
        self.assertEqual(migrated,b'\xef\xbb\xbf[general]\r\nnanotech_progression_max_health = 26 ; personal cap\r\nxp_per_damage = 0.12\r\n')
        self.assertEqual(migrate_health_cap(migrated),migrated)

    def test_legacy_zero_and_high_values_migrate_once_and_new_key_wins(self):
        for bonus,total in [(0,4),(20,24),(22,26),(64,68),(202,206),(300,206),('22.0',26),('2.2e1',26),('+22',26)]:
            raw=('[general]\n'+LEGACY_KEY+'='+str(bonus)+'\n').encode()
            doc=core.IniDocument(migrate_health_cap(raw))
            self.assertEqual(doc.entries['general',KEY],str(total))
            self.assertNotIn(('general',LEGACY_KEY),doc.entries)
        for order in [(LEGACY_KEY+'=300',KEY+'=26'),(KEY+'=26',LEGACY_KEY+'=300')]:
            raw=('[general]\n'+'\n'.join(order)+'\n').encode()
            self.assertEqual(core.IniDocument(migrate_health_cap(raw)).entries['general',KEY],'26')
        for value in ['-1','301','1.5','NaN']:
            with self.assertRaises(ValueError):migrate_health_cap(('[general]\n'+LEGACY_KEY+'='+value+'\n').encode())
        with self.assertRaises(ValueError):migrate_health_cap(b'[general]\nnanotech_max_health=20\nnanotech_max_health=22\n')

    def test_all_languages_explain_reserved_native_purchases(self):
        try:
            for lang in i18n.LANGUAGES:
                i18n.set_language(lang);text=i18n.help_text('general',KEY)
                for term in ['4-206','Premium','Ultra','210','30']:self.assertIn(term,text)
                self.assertIn(lang,i18n.FIELD_LABELS[KEY])
        finally:i18n.set_language('en')

if __name__=='__main__':unittest.main()
