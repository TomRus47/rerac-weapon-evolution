"""Per-grade trail colors are standard settings, preserve edits and require v78."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from pathlib import Path
import unittest
import launcher_core as core
import schema
import i18n

class WrenchTrailTests(unittest.TestCase):
    def test_all_grades_have_independent_colors_and_existing_values_survive(self):
        raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        before=core.IniDocument(raw);doc=core.profile_document(raw)
        for pair,value in before.entries.items():self.assertEqual(doc.entries[pair],value)
        for grade in range(1,9):self.assertIn((f'weapon.wrench.level{grade}','trail_color'),doc.entries)
        self.assertEqual(core.required_config(doc),89)
        raw=doc.render({('weapon.wrench.level2','trail_color'):'AA11CC',('weapon.wrench.level8','trail_color'):'000000'})
        changed=core.profile_document(raw)
        self.assertEqual(changed.entries['weapon.wrench.level2','trail_color'],'AA11CC')
        self.assertEqual(changed.entries['weapon.wrench.level8','trail_color'],'000000')
        self.assertEqual(core.profile_document(changed.raw).raw,changed.raw)
    def test_color_is_last_in_standard_group_and_help_is_translated(self):
        doc=core.profile_document((Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes())
        standard,experimental=schema.field_groups('wrench',doc.keys('weapon.wrench.level1'))
        self.assertEqual(standard[-1],'trail_color');self.assertNotIn('trail_color',experimental)
        try:
            for lang in i18n.LANGUAGES:
                i18n.set_language(lang)
                self.assertIn('RRGGBB',i18n.help_text('weapon.wrench.level1','trail_color'))
        finally:i18n.set_language('en')

if __name__=='__main__':unittest.main()
