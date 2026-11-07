"""Additive Quick Select configuration and compatibility with existing profiles."""
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import launcher_core as core
import migrate_ini
import schema
import i18n


class DualQuickSelectTests(unittest.TestCase):
    def test_old_profiles_keep_every_value_and_gain_only_opt_in_pause(self):
        raw = (Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        old = b'\xef\xbb\xbf' + b'\r\n'.join(line for line in raw.splitlines()
            if not line.startswith(b'pause_on_quick_select =')) + b'\r\n'
        old = old.replace(b'max_ammo = 40', b'max_ammo = 43 ; preserved custom value', 1)
        before = core.IniDocument(old)
        migrated, _ = migrate_ini.convert(old)
        after = core.IniDocument(migrated)
        self.assertEqual({k:v for k,v in after.entries.items() if k != ('general','pause_on_quick_select')}, before.entries)
        self.assertEqual(after.entries['general','pause_on_quick_select'], 'false')
        self.assertTrue(migrated.startswith(b'\xef\xbb\xbf'))
        self.assertNotIn(b'\n', migrated.replace(b'\r\n', b''))
        self.assertEqual(migrate_ini.convert(migrated)[0], migrated)

    def test_pause_is_standard_off_on_and_has_five_language_help(self):
        key = 'pause_on_quick_select'
        self.assertEqual(schema.CONTROL_DEFAULTS[key], 'false')
        self.assertIn(key, schema.BOOLEANS)
        for language in i18n.LANGUAGES:
            self.assertTrue(i18n.FIELD_LABELS[key][language])
            self.assertIn('R2', i18n.HELP_TEXT[key][language])
        for value in ('false','true'):
            core.validate_value('general', key, value)
        with self.assertRaises(ValueError): core.validate_value('general', key, '2')

    def test_profile_requires_runtime_with_dual_ring_support(self):
        doc = core.IniDocument(b'[general]\npause_on_quick_select = true\n')
        self.assertEqual(core.required_config(doc), 94)
        doc = core.IniDocument(doc.render({('general','pause_on_quick_select'):'false'}))
        self.assertLess(core.required_config(doc), 94)

if __name__ == '__main__': unittest.main()
