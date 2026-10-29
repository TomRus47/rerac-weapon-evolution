"""Separate shotgun axes migrate without resetting per-grade player settings."""
from pathlib import Path
import sys,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as core
from migrate_ini import convert
from schema import field_groups

class V46Tests(unittest.TestCase):
    def setUp(self):self.raw=(ROOT/'weapons.default.ini').read_bytes()
    def test_legacy_spread_becomes_horizontal_and_new_vertical_defaults_to_three(self):
        raw=self.raw
        for grade in range(1,6):
            section=f'[weapon.ryno.level{grade}]'.encode()
            # Start with a v45 style grade and unique legacy spread per grade.
            start=raw.index(section);end=raw.find(b'[',start+1)
            if end<0:end=len(raw)
            block=raw[start:end]
            block=b'\n'.join(line for line in block.split(b'\n') if not line.strip().startswith((b'shotgun_horizontal_spread_degrees',b'shotgun_vertical_spread_degrees',b'shotgun_spread_degrees')))
            block+=f'shotgun_spread_degrees = {grade*5}\n'.encode()
            raw=raw[:start]+block+raw[end:]
        before=core.IniDocument(raw);after=core.checked_document(convert(raw)[0])
        for grade in range(1,6):
            section=f'weapon.ryno.level{grade}'
            self.assertEqual(after.entries[section,'shotgun_horizontal_spread_degrees'],str(grade*5))
            self.assertEqual(after.entries[section,'shotgun_vertical_spread_degrees'],'3')
            self.assertNotIn((section,'shotgun_spread_degrees'),after.entries)
        for pair,value in before.entries.items():
            if pair[1]!='shotgun_spread_degrees':self.assertEqual(after.entries[pair],value)
    def test_explicit_axis_values_are_independent_and_never_replaced_by_legacy_value(self):
        doc=core.profile_document(self.raw);edits={}
        for grade in range(1,6):
            section=f'weapon.ryno.level{grade}'
            edits[section,'shotgun_horizontal_spread_degrees']=str(grade*7)
            edits[section,'shotgun_vertical_spread_degrees']=str(grade)
        raw=doc.render(edits).replace(b'[weapon.ryno.level1]',b'[weapon.ryno.level1]\nshotgun_spread_degrees = 55')
        after=core.checked_document(convert(raw)[0])
        for pair,value in edits.items():self.assertEqual(after.entries[pair],value)
        self.assertEqual(core.required_config(after),92)
    def test_both_zero_axes_are_valid_and_experimental_fields_remain_last(self):
        doc=core.profile_document(self.raw);section='weapon.ryno.level1'
        result=core.checked_document(doc.render({(section,'shotgun_horizontal_spread_degrees'):'0',(section,'shotgun_vertical_spread_degrees'):'0'}))
        keys=result.keys(section);standard,experimental=field_groups('ryno',keys)
        self.assertEqual(keys,standard+experimental)
        for key in ['shotgun_horizontal_spread_degrees','shotgun_vertical_spread_degrees']:self.assertIn(key,experimental)
    def test_bad_axis_bounds_and_nonfinite_values_are_rejected(self):
        doc=core.profile_document(self.raw)
        for key in ['shotgun_horizontal_spread_degrees','shotgun_vertical_spread_degrees']:
            for value in ['-1','61','nan','inf']:
                with self.subTest(key=key,value=value),self.assertRaises(ValueError):
                    core.checked_document(doc.render({('weapon.ryno.level5',key):value}))
