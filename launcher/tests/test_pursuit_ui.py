"""Local test isolation, pursuit editor validation and lossless profile upgrade."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import unittest
from unittest.mock import patch
from test_nanotech_ui import profile
from pathlib import Path
import launcher_core as core
import migrate_ini
import schema
import i18n

KEY='enemy_chase_range_multiplier'
class PursuitUiTests(unittest.TestCase):
    def test_default_and_custom_values_preserve_all_existing_entries(self):
        for value in [None,'2.5']:
            raw=b'\xef\xbb\xbf[general]\r\nenemy_perception_multiplier=2 ; keep\r\ncustom=abc\r\n[weapon.blaster.level1]\r\nmax_ammo=42\r\n'
            if value:raw=raw.replace(b'custom=abc',b'enemy_chase_range_multiplier=2.5 ; chase\r\ncustom=abc')
            before=core.IniDocument(raw);changed=migrate_ini.upgrade_global_settings(raw);after=core.IniDocument(changed)
            for k,v in before.entries.items():self.assertEqual(after.entries[k],v)
            self.assertEqual(after.entries['general',KEY],value or '1')
            self.assertTrue(changed.startswith(b'\xef\xbb\xbf'))
            self.assertNotIn(b'\n',changed.replace(b'\r\n',b''))
            self.assertEqual(migrate_ini.upgrade_global_settings(changed),changed)
            keys=after.keys('general');self.assertEqual(keys[keys.index('enemy_perception_multiplier')+1],KEY)
            self.assertEqual(core.required_config(after),77)

    def test_range_and_translations(self):
        for v in ['0.1','1','2','10']:core.validate_value('general',KEY,v)
        for v in ['0','-1','11','NaN','inf','on']:
            with self.assertRaises(ValueError):core.validate_value('general',KEY,v)
        for language in i18n.LANGUAGES:
            self.assertIn(language,i18n.FIELD_LABELS[KEY]);self.assertIn(language,i18n.HELP_TEXT[KEY])

    def test_local_test_refuses_the_old_runtime_before_creating_a_session(self):
        for env,minimum in [({},86),({'RC_WEAPON_LOCATION_TEST':'1'},86)]:
            with patch.object(core,'probe_exe',side_effect=ValueError('stop at probe')) as probe:
                with self.assertRaisesRegex(ValueError,'stop at probe'):
                    core.launch_game(Path('kit'),profile(),'default',Path('runtime/rerac.exe'),Path('data'),env=env)
            self.assertEqual(probe.call_args.args[1],minimum)

    def test_location_test_has_an_explicit_local_flag_and_normal_launch_is_unchanged(self):
        ini=Path('mod/runs/test/weapons.ini');data=Path('data')
        normal=core.game_environment(ini,data,env={})
        self.assertNotIn('RC_LEVEL',normal);self.assertEqual(normal['RC_FRONTEND'],'1')
        testing=core.game_environment(ini,data,env={'RC_WEAPON_LOCATION_TEST':'1','RC_FRAME_LOG':'1'})
        self.assertEqual(testing['RC_LEVEL'],'18');self.assertEqual(testing['RC_SHIP'],'2');self.assertEqual(testing['RC_FRONTEND'],'0')
        self.assertEqual(testing['RC_FRAME_LOG'],'1')

if __name__=='__main__':unittest.main()
