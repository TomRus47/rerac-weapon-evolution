"""Ten-drone profiles retain per-grade values and require the matching runtime."""
from pathlib import Path
import sys, unittest
ROOT = Path(__file__).resolve().parents[1]; sys.path.insert(0, str(ROOT))
import launcher_core as core
from migrate_ini import convert
from schema import field_groups

class V48Tests(unittest.TestCase):
    def test_each_grade_accepts_ten_and_keeps_the_standard_field_before_experiments(self):
        doc = core.profile_document((ROOT/'weapons.default.ini').read_bytes())
        edits = {(f'weapon.drone_device.level{n}', 'drone_count'): str(n+5) for n in range(1, 6)}
        after = core.checked_document(convert(doc.render(edits))[0])
        for pair, value in edits.items():
            self.assertEqual(after.entries[pair], value)
            standard, experimental = field_groups('drone_device', after.keys(pair[0]))
            self.assertIn('drone_count', standard)
            self.assertEqual(after.keys(pair[0]), standard+experimental)
        self.assertEqual(core.required_config(after),89)
    def test_invalid_counts_are_rejected_at_every_grade(self):
        for grade in range(1, 6):
            section = f'weapon.drone_device.level{grade}'
            for value in ['-1', '10.5', '11', 'nan']:
                with self.subTest(grade=grade, value=value):
                    with self.assertRaises(ValueError): core.validate_value(section, 'drone_count', value)
            for value in ['0', '6', '7', '10']:
                self.assertEqual(core.validate_value(section, 'drone_count', value), value)
