"""v43: optional behavior adds fields without changing approved balance or presets."""
from pathlib import Path
import json,sys,tempfile,unittest
from types import SimpleNamespace
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as core
from migrate_ini import convert
from schema import EXPERIMENTAL_DEFAULTS,EXPERIMENTAL_RANGES,LABELS,HELP

class ExperimentalTests(unittest.TestCase):
    def setUp(self): self.raw=(ROOT/'weapons.default.ini').read_bytes()
    def test_old_profiles_gain_only_missing_fields_at_all_five_grades(self):
        keys={k for fields in EXPERIMENTAL_DEFAULTS.values() for k in fields}
        old=b''.join(line for line in self.raw.splitlines(True) if line.split(b'=',1)[0].strip().decode() not in keys)
        before=core.checked_document(old);result,_=convert(old);after=core.checked_document(result)
        for key,value in before.entries.items(): self.assertEqual(after.entries[key],value)
        self.assertEqual(len(after.entries)-len(before.entries),sum(len(f)*5 for f in EXPERIMENTAL_DEFAULTS.values()))
        for weapon,fields in EXPERIMENTAL_DEFAULTS.items():
            for grade in range(1,6):
                for key,value in fields.items(): self.assertEqual(after.entries[f'weapon.{weapon}.level{grade}',key],value)
        self.assertEqual(convert(result)[0],result)
        self.assertEqual(core.required_config(before),51)
        self.assertEqual(core.required_config(after),51)
    def test_custom_behavior_and_balance_survive_bom_crlf_and_preset_round_trip(self):
        doc=core.checked_document(self.raw)
        changed=doc.render({('weapon.bomb_glove.level3','cluster_bombs_enabled'):'true',
            ('weapon.bomb_glove.level3','cluster_bomb_count'):'5',('weapon.walloper.level5','held_lunge_duration_percent'):'225',
            ('weapon.pyrocitor.level4','triple_flame_enabled'):'true',('weapon.blaster.level2','damage_hp'):'432'})
        changed=b'\xef\xbb\xbf'+changed.replace(b'\n',b'\r\n')+b'; personal comment\r\n'
        self.assertEqual(convert(changed)[0],changed)
        with tempfile.TemporaryDirectory() as folder:
            store=core.PresetStore(Path(folder));store.save('Experimental',changed)
            self.assertEqual(store.read('Experimental').raw,changed)
    def test_every_new_setting_has_english_metadata_and_matching_limits(self):
        for weapon,fields in EXPERIMENTAL_DEFAULTS.items():
            for key in fields:
                self.assertIn(key,LABELS);self.assertTrue(HELP.get(key))
                if key in EXPERIMENTAL_RANGES:
                    low,high=EXPERIMENTAL_RANGES[key]
                    for value in ['NaN','inf',str(high+1),str(low-1)]:
                        with self.assertRaises(ValueError):core.validate_value(f'weapon.{weapon}.level2',key,value)
    def test_v42_runtime_is_rejected_even_when_experimental_options_are_disabled(self):
        doc=core.profile_document(self.raw)
        with tempfile.TemporaryDirectory() as folder:
            exe=Path(folder)/'rerac.exe';exe.write_bytes(b'exe');(exe.parent/'assets/shaders').mkdir(parents=True)
            info=dict(name='rerac',game='rac1',version='0.3.4',data_format=1,weapon_evolution_config=40)
            with patch.object(core.subprocess,'run',return_value=SimpleNamespace(stdout=json.dumps(info))):
                with self.assertRaises(ValueError):core.probe_exe(exe,core.required_config(doc))
            info['weapon_evolution_config']=51
            with patch.object(core.subprocess,'run',return_value=SimpleNamespace(stdout=json.dumps(info))):
                self.assertEqual(core.probe_exe(exe,core.required_config(doc)),info)

if __name__=='__main__':unittest.main()
