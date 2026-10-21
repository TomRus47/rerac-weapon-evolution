"""Optional ammo progression and visual-only Gold controls."""
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import launcher_core as core
from migrate_ini import convert, runtime_profile
from schema import GLOBAL_EXPERIMENTAL_DEFAULTS, GLOBAL_STANDARD_DEFAULTS, field_groups


class V51Tests(unittest.TestCase):
    def setUp(self):
        self.raw = (ROOT / 'weapons.default.ini').read_bytes()

    def test_old_profiles_gain_only_missing_visual_and_opt_in_controls(self):
        doc = core.checked_document(self.raw)
        removed = {('general', key) for key in [*GLOBAL_STANDARD_DEFAULTS, *GLOBAL_EXPERIMENTAL_DEFAULTS]}
        removed |= {(f'weapon.{w}.level{n}', 'gold_texture')
                    for w in ['visibomb', 'taunter', 'walloper', 'ryno'] for n in range(1, 6)}
        old = ''.join(line for i, line in enumerate(doc.lines)
                      if i not in {doc.addresses[pair] for pair in removed}).encode()
        before = core.checked_document(old)
        migrated, _ = convert(old)
        after = core.checked_document(migrated)
        for pair, value in before.entries.items(): self.assertEqual(after.entries[pair], value)
        self.assertEqual(after.entries['general', 'experimental_ammo_unlock_mode'], 'false')
        self.assertNotIn(('general','wrench_gold_texture'),after.entries)
        self.assertEqual(len(after.entries)-len(before.entries), 36)
        self.assertEqual(convert(migrated)[0], migrated)

    def test_custom_gold_and_mode_values_survive_all_grade_migrations(self):
        doc = core.checked_document(self.raw)
        changes = {('general', 'experimental_ammo_unlock_mode'): 'true',
                   ('weapon.wrench.level2', 'gold_texture'): 'true'}
        changes.update({(f'weapon.{w}.level{n}', 'gold_texture'): 'true'
                        for w in ['visibomb', 'taunter', 'walloper', 'ryno'] for n in range(1, 6)})
        raw = doc.render(changes)
        migrated, _ = convert(raw)
        loaded = core.checked_document(runtime_profile(migrated))
        for pair, value in changes.items(): self.assertEqual(loaded.entries[pair], value)
        self.assertEqual(core.required_config(loaded), 56)
        for n in range(1, 6):
            keys = loaded.keys(f'weapon.ryno.level{n}')
            standard, experimental = field_groups('ryno', keys)
            self.assertIn('gold_texture', standard)
            self.assertTrue(all(keys.index('gold_texture') < keys.index(k) for k in experimental))

    def test_new_global_switches_reject_invalid_values(self):
        for key in ['experimental_ammo_unlock_mode', 'wrench_gold_texture']:
            for invalid in ['perhaps', '2', '-1', 'NaN']:
                with self.assertRaises(ValueError): core.validate_value('general', key, invalid)


if __name__ == '__main__':
    unittest.main()
