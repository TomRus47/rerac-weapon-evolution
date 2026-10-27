"""Wrench track migration, independent grades, native-save-neutral editor options."""
from pathlib import Path
import unittest,sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import launcher_core as core
from migrate_ini import convert,runtime_profile
from schema import EDITOR_WEAPONS
class WrenchTests(unittest.TestCase):
    def setUp(self):self.raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
    def test_each_grade_accepts_independent_damage_geometry_booleans_and_bolt_bonus(self):
        doc=core.checked_document(self.raw);values={}
        self.assertIn('wrench',EDITOR_WEAPONS)
        for n in range(1,6):
            sec=f'weapon.wrench.level{n}'
            for key,value in [('damage_hp',str(100*n)),('throw_range',str(5*n)),('throw_speed',str(15+n)),('bolt_attraction_bonus',str(n)),('gold_texture',str(n%2==0).lower()),('show_hud',str(n%2==1).lower()),('crate_breaker_enabled',str(n>=3).lower()),('crate_breaker_radius',str(2*n)),('moving_attacks_enabled',str(n>=2).lower())]:values[sec,key]=value
        out=core.checked_document(doc.render(values))
        for p,v in values.items():self.assertEqual(out.entries[p],v)
        self.assertNotIn(('general','wrench_gold_texture'),out.entries)
        self.assertEqual(core.required_config(out),89)
    def test_legacy_global_gold_moves_to_missing_grades_and_explicit_values_win(self):
        doc=core.IniDocument(self.raw)
        remove={i for (s,k),i in doc.addresses.items() if s.startswith('weapon.wrench') and not (s.endswith('level3') and k=='gold_texture')}
        lines=[];in_wrench=False
        for i,line in enumerate(doc.lines):
            if line.lstrip().startswith('['):in_wrench=line.strip().startswith('[weapon.wrench')
            if in_wrench:
                if line.strip()=='[weapon.wrench.level3]':lines.append(line)
                elif i not in remove and '=' in line:lines.append(line)
            else:lines.append(line)
        old=''.join(lines).encode().replace(b'[general]',b'[general]\nwrench_gold_texture = true',1)
        result=convert(old)[0];new=core.checked_document(result)
        for n in range(1,6):self.assertEqual(new.entries[f'weapon.wrench.level{n}','gold_texture'],'false' if n==3 else 'true')
        before=core.IniDocument(old)
        for p,v in before.entries.items():
            if p!=('general','wrench_gold_texture'):self.assertEqual(new.entries[p],v)
        self.assertEqual(convert(result)[0],result)
        self.assertIn(b'damage_mode = real_hp',runtime_profile(result))
    def test_invalid_wrench_geometry_and_toggles_are_rejected(self):
        for key,value in [('throw_range','0'),('throw_speed','0'),('crate_breaker_radius','-1'),('bolt_attraction_bonus','NaN'),('gold_texture','perhaps'),('show_hud','perhaps'),('moving_attacks_enabled','2')]:
            with self.assertRaises(ValueError,msg=key):core.validate_value('weapon.wrench.level1',key,value)
