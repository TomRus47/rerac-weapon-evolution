"""Saved locations are immutable; only new All seeds use the six switches."""
from pathlib import Path
import sys,unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import schema,i18n,migrate_ini,launcher_core as core
class RandomizerSiteTests(unittest.TestCase):
    def test_missing_settings_are_added_without_changing_existing_values(self):
        raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        keys=set(schema.RANDOMIZER_SITE_DEFAULTS)
        old=b'\n'.join(line for line in raw.split(b'\n') if line.split(b'=',1)[0].strip().decode() not in keys)
        before=core.IniDocument(old);result=migrate_ini.convert(old)[0];after=core.profile_document(result)
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value)
        for key,value in schema.RANDOMIZER_SITE_DEFAULTS.items():self.assertEqual(after.entries['general',key],value)
        self.assertEqual(migrate_ini.convert(result)[0],result)
    def test_only_location_flags_require_new_game_and_pack_value_is_live(self):
        raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes();before=core.IniDocument(raw)
        changed=before.render({('general',key):'false' for key in schema.RANDOMIZER_EXTRA_KEYS})
        deferred=core.live_restart_fields(before,core.IniDocument(changed))
        self.assertEqual(set(deferred),set(schema.RANDOMIZER_EXTRA_KEYS))
        changed=before.render({('general','randomizer_bolt_pack_value'):'12345'})
        self.assertFalse(core.live_restart_fields(before,core.IniDocument(changed)))
        self.assertIn('12345',core.validate_value('general','randomizer_bolt_pack_value','12345'))
    def test_all_seven_controls_have_five_translations(self):
        for key in schema.RANDOMIZER_SITE_DEFAULTS:
            for lang in i18n.LANGUAGES:
                self.assertTrue(i18n.FIELD_LABELS[key][lang]);self.assertTrue(i18n.HELP_TEXT[key][lang])

class HovenFallbackHelpTests(unittest.TestCase):
    def test_all_six_controls_explain_the_hoven_fallback_in_every_language(self):
        endings={'en':'If all six switches are Off','it':'Se tutti e sei gli interruttori sono Off',
            'fr':'Si les six options sont Off','de':'Sind alle sechs Optionen Off',
            'es':'Si las seis opciones están en Off'}
        for key in schema.RANDOMIZER_EXTRA_KEYS:
            for lang,phrase in endings.items():
                text=i18n.HELP_TEXT[key][lang]
                self.assertIn(phrase,text);self.assertIn('Hoven',text)
        for key in schema.RANDOMIZER_EXTRA_KEYS:
            self.assertIn('Hoven',schema.HELP[key])
