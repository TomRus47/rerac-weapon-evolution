"""Expanded HP compatibility and save-safe direct test startup."""
from pathlib import Path
import sys
import unittest
import inspect
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'launcher'))
sys.path.insert(0,str(ROOT/'tools/mod'))
import launcher_core as core
import nanotech_test

class LayeredNanotechTests(unittest.TestCase):
    def test_local_health_test_defaults_to_210(self):
        self.assertEqual(inspect.signature(nanotech_test.launch).parameters['hp'].default,210)

    def test_large_bonus_requires_new_runtime_but_existing_profiles_keep_old_requirement(self):
        doc=core.IniDocument(b'[general]\nnanotech_max_health=64\n')
        self.assertLess(core.required_config(doc),91)
        for bonus in [65,100,292,300]:
            doc=core.IniDocument(('[general]\nnanotech_max_health='+str(bonus)+'\n').encode())
            self.assertEqual(core.required_config(doc),91)

    def test_test_uses_explicit_data_and_disables_normal_saves_and_other_test_modes(self):
        env=nanotech_test.test_environment(Path('/test.ini'),Path('/disc'),51,
            {'RC_SAVE_DIR':'/normal','RC_WEAPON_LOCATION_TEST':'1','RC_GEMLIK_GOLD_TEST':'1'})
        self.assertEqual(env['RC_SAVE_DIR'],'0')
        self.assertEqual(env['RC_LEVEL'],'18')
        self.assertEqual(env['RC_FRONTEND'],'0')
        self.assertEqual(env['RC_NANOTECH_TEST_HP'],'51')
        self.assertEqual(env['RC_DATA_DIR'],str(Path('/disc').resolve()))
        self.assertNotIn('RC_WEAPON_LOCATION_TEST',env)
        self.assertNotIn('RC_GEMLIK_GOLD_TEST',env)
        self.assertEqual(nanotech_test.test_environment(Path('/i'),Path('/d'),999,{})['RC_NANOTECH_TEST_HP'],'210')

if __name__=='__main__':unittest.main()
