from pathlib import Path
import sys,unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import launcher_core as core
from migrate_ini import convert
from schema import WEAPONS
class RandomizerSchemaTests(unittest.TestCase):
    def test_current_ini_keeps_all_new_controls_and_requires_capability_86(self):
        raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        doc=core.profile_document(raw)
        self.assertEqual(core.required_config(doc),86)
        self.assertEqual(convert(raw)[0],raw)
        for key in ['randomizer_mode','randomizer_bolt_multiplier','bolt_multiplier_max']:
            self.assertIn(('general',key),doc.entries)
        for weapon,(item,_) in WEAPONS.items():
            limit=2 if item==23 else 1 if item in [13,16,17] else 0
            for grade in range(1,9):
                section=f'weapon.{weapon}.level{grade}'
                core.validate_value(section,'r1_experimental_max',str(limit))
                with self.assertRaises(ValueError):core.validate_value(section,'r1_experimental_max',str(limit+1))
        for v in ['1','3','100','1.1']:core.validate_value('general','bolt_multiplier_max',v)
        for v in ['0','100.1','1.05','NaN']:
            with self.assertRaises(ValueError):core.validate_value('general','bolt_multiplier_max',v)
