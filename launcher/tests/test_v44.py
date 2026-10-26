"""Geometry migration, permanent field ordering and per-grade crate/Tesla controls."""
from pathlib import Path
import math,sys,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as core
from migrate_ini import convert
from schema import EXPERIMENTAL_DEFAULTS,STANDARD_DEFAULTS,field_groups

class V44Tests(unittest.TestCase):
    def setUp(self):self.raw=(ROOT/'weapons.default.ini').read_bytes()
    def test_all_standard_fields_precede_the_experimental_switch_and_options(self):
        doc=core.profile_document(self.raw)
        for weapon,fields in EXPERIMENTAL_DEFAULTS.items():
            for level in range(1,6):
                section=f'weapon.{weapon}.level{level}';keys=doc.keys(section)
                standard,experimental=field_groups(weapon,keys)
                self.assertEqual(keys,standard+experimental)
                self.assertEqual(set(experimental),set(fields))
                self.assertEqual(fields[next(iter(fields))],'false')
        for weapon,fields in STANDARD_DEFAULTS.items():
            for key in fields:
                self.assertNotIn(key,EXPERIMENTAL_DEFAULTS.get(weapon,{}))
    def test_v43_defaults_upgrade_once_custom_values_and_legacy_comments_survive(self):
        doc=core.checked_document(self.raw)
        raw=doc.render({('weapon.bomb_glove.level1','cluster_bombs_enabled'):'true',
            **{(f'weapon.bomb_glove.level{n}', 'cluster_jump_speed'):'4.2' for n in range(1,6)},
            ('weapon.bomb_glove.level2','cluster_bomb_size_percent'):'55',
            ('weapon.bomb_glove.level2','cluster_launch_speed'):'7',
            ('weapon.pyrocitor.level3','triple_flame_enabled'):'true'})
        raw=raw.replace(b'cluster_bomb_size_percent = 40',b'cluster_bomb_size_percent = 35')
        raw=raw.replace(b'cluster_launch_speed = 3',b'cluster_launch_speed = 6')
        raw=raw.replace(b'cluster_jump_speed = 4.2',b'cluster_jump_height = 0.8')
        raw=raw.replace(b'side_flame_left_angle_degrees = 22\nside_flame_right_angle_degrees = 22',b'side_flame_angle_degrees = 33')
        raw=b'\xef\xbb\xbf'+raw.replace(b'\n',b'\r\n')+b'; Personal comment\r\n'
        migrated=convert(raw)[0];after=core.checked_document(migrated)
        self.assertTrue(migrated.startswith(b'\xef\xbb\xbf'));self.assertIn(b'; Personal comment\r\n',migrated)
        self.assertEqual(after.entries['weapon.bomb_glove.level1','cluster_bombs_enabled'],'true')
        self.assertEqual(after.entries['weapon.bomb_glove.level1','cluster_bomb_size_percent'],'40')
        self.assertEqual(after.entries['weapon.bomb_glove.level1','cluster_launch_speed'],'3')
        self.assertAlmostEqual(float(after.entries['weapon.bomb_glove.level1','cluster_jump_speed']),math.sqrt(17.6),places=6)
        self.assertEqual(after.entries['weapon.bomb_glove.level2','cluster_bomb_size_percent'],'55')
        self.assertEqual(after.entries['weapon.bomb_glove.level2','cluster_launch_speed'],'7')
        self.assertEqual(after.entries['weapon.pyrocitor.level3','triple_flame_enabled'],'true')
        self.assertEqual(after.entries['weapon.pyrocitor.level3','side_flame_left_angle_degrees'],'22')
        self.assertEqual(after.entries['weapon.pyrocitor.level3','side_flame_right_angle_degrees'],'22')
        self.assertEqual(convert(migrated)[0],migrated)
    def test_custom_asymmetric_angles_and_independent_grade_selectors_remain_independent(self):
        doc=core.profile_document(self.raw)
        edits={('weapon.pyrocitor.level4','side_flame_left_angle_degrees'):'12',
            ('weapon.pyrocitor.level4','side_flame_right_angle_degrees'):'41',
            ('weapon.suck_cannon.level2','suck_steel_crates'):'false',
            ('weapon.suck_cannon.level5','crate_suction_enabled'):'true',
            ('weapon.tesla_claw.level3','dual_beam_enabled'):'true',
            ('weapon.tesla_claw.level3','beam_size_percent'):'175'}
        after=core.profile_document(doc.render(edits))
        for pair,value in edits.items():self.assertEqual(after.entries[pair],value)
        self.assertEqual(after.entries['weapon.suck_cannon.level1','suck_steel_crates'],'true')
        self.assertEqual(after.entries['weapon.tesla_claw.level2','dual_beam_enabled'],'false')
        self.assertEqual(core.required_config(after),86)
    def test_future_experiments_automatically_follow_existing_standard_fields(self):
        fields=EXPERIMENTAL_DEFAULTS.setdefault('blaster',{})
        try:
            fields['future_experiment_enabled']='false';fields['future_setting']='1'
            self.assertEqual(field_groups('blaster',['future_setting','damage_hp','max_ammo','future_experiment_enabled']),
                (['max_ammo','damage_hp'],['future_experiment_enabled','future_setting']))
        finally:EXPERIMENTAL_DEFAULTS.pop('blaster',None)

if __name__=='__main__':unittest.main()
