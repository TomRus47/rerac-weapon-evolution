"""Saved-control options, capability guards and lossless profile migration."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from pathlib import Path
import unittest
import launcher_core as core
import schema
import i18n
import migrate_ini

class SavedControlsTests(unittest.TestCase):
    def test_defaults_are_opt_in_last_in_base_card_and_not_survival(self):
        doc=core.profile_document((Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes())
        base,survival=schema.global_field_groups(doc)
        self.assertEqual(base[-2:],[('general','dpad_quick_select'),('general','r1_experimental')])
        for key in schema.CONTROL_DEFAULTS:
            self.assertEqual(doc.entries['general',key],'false')
            self.assertIn(key,schema.BOOLEANS)
            self.assertNotIn(('general',key),survival)
            self.assertNotIn(key,schema.WORLD_DEFAULTS)
            for value in ['true','false']:
                core.validate_value('general',key,value)
            for value in ['2','invalid']:
                with self.assertRaises(ValueError):core.validate_value('general',key,value)
        self.assertEqual(core.required_config(doc),88)

    def test_migration_keeps_balance_unknown_fields_comments_bom_and_crlf(self):
        raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        doc=core.IniDocument(raw)
        lines=[line for line in doc.lines if not any(line.strip().startswith(k+' =') for k in schema.CONTROL_DEFAULTS)]
        old=b'\xef\xbb\xbf'+''.join(lines).replace('\r\n','\n').replace('\n','\r\n').encode()
        old=old.replace(b'max_ammo = 40',b'max_ammo = 41 ; custom balance',1)
        result,_=migrate_ini.convert(old)
        before=core.IniDocument(old);after=core.IniDocument(result)
        for key,value in before.entries.items():self.assertEqual(after.entries[key],value,key)
        self.assertTrue(result.startswith(b'\xef\xbb\xbf'))
        self.assertNotIn(b'\n',result.replace(b'\r\n',b''))
        self.assertEqual(migrate_ini.convert(result)[0],result)
        changed=after.render({('general','r1_experimental'):'true'})
        self.assertEqual(core.profile_document(changed).entries['general','r1_experimental'],'true')

    def test_both_options_have_all_five_labels_and_help_translations(self):
        for key in schema.CONTROL_DEFAULTS:
            for lang in i18n.LANGUAGES:
                self.assertTrue(i18n.FIELD_LABELS[key][lang])
                self.assertTrue(i18n.HELP_TEXT[key][lang])
        text=schema.HELP['r1_experimental']
        self.assertIn('crate suction',text)
        self.assertIn('launcher-only',text)

if __name__=='__main__':unittest.main()
