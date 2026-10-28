from pathlib import Path
import sys,tempfile,unittest,json
from unittest.mock import patch
from types import SimpleNamespace
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
import launcher_core as c
from migrate_ini import convert,ensure_stream_fields
from schema import LABELS,HELP,INTEGER

class IntervalTests(unittest.TestCase):
    def setUp(self):
        self.raw=(ROOT/'weapons.example.ini').read_bytes()
    def test_missing_fields_are_added_to_every_grade_without_changing_existing_values(self):
        before=c.checked_document(self.raw)
        out,_=convert(self.raw);after=c.checked_document(out)
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value,pair)
        for weapon in ['pyrocitor','tesla_claw']:
            for n in range(1,6):
                section=f'weapon.{weapon}.level{n}'
                self.assertIn((section,'ammo_interval_ticks'),after.entries)
                self.assertEqual(after.entries[section,'damage_interval_ticks'],'1')
        self.assertEqual(convert(out)[0],out)
    def test_custom_intervals_are_never_overwritten_and_bom_crlf_comments_survive(self):
        out,_=convert(self.raw)
        doc=c.checked_document(out)
        edits={(f'weapon.{weapon}.level{n}',key):str(n*3) for weapon in ['pyrocitor','tesla_claw'] for n in range(1,6) for key in ['ammo_interval_ticks','damage_interval_ticks']}
        out=doc.render(edits)
        out=b'\xef\xbb\xbf'+out.replace(b'\n',b'\r\n')+b'; custom comment\r\n'
        normalized=convert(out)[0]
        self.assertEqual(c.IniDocument(normalized).entries,c.IniDocument(out).entries)
        self.assertIn(b'; custom comment\r\n',normalized)
        self.assertEqual(convert(normalized)[0],normalized)
        for key in ['ammo_interval_ticks','damage_interval_ticks']:
            self.assertIn(key,INTEGER);self.assertIn(key,LABELS);self.assertIn(key,HELP)
    def test_limits_are_enforced_for_both_weapons(self):
        for weapon in ['pyrocitor','tesla_claw']:
            section=f'weapon.{weapon}.level3'
            for key,high in [('ammo_interval_ticks',9999),('damage_interval_ticks',600)]:
                for value in ['0','-1','1.5','NaN','inf',str(high+1)]:
                    with self.assertRaises(ValueError,msg=(section,key,value)):c.validate_value(section,key,value)
                for value in ['1',str(high)]:self.assertEqual(c.validate_value(section,key,value),value)
    def test_old_executables_are_refused_when_intervals_are_present(self):
        out,_=convert(self.raw);doc=c.checked_document(out)
        self.assertEqual(c.required_config(doc),90)
        old=c.checked_document(self.raw);self.assertEqual(c.required_config(old),90)
        with tempfile.TemporaryDirectory() as directory:
            exe=Path(directory)/'rerac.exe';exe.touch();(exe.parent/'assets/shaders').mkdir(parents=True)
            version=dict(name='rerac',game='rac1',data_format=1,version='0.3.4',weapon_evolution_config=37)
            with patch.object(c.subprocess,'run',return_value=SimpleNamespace(stdout=json.dumps(version))):
                with self.assertRaises(ValueError):c.probe_exe(exe,c.required_config(doc))
            version['weapon_evolution_config']=90
            with patch.object(c.subprocess,'run',return_value=SimpleNamespace(stdout=json.dumps(version))):
                self.assertEqual(c.probe_exe(exe,c.required_config(doc)),version)
    def test_loading_old_preset_adds_fields_without_writing_its_file(self):
        with tempfile.TemporaryDirectory() as directory:
            store=c.PresetStore(Path(directory));store.save('Personal',self.raw)
            raw=store.path('Personal').read_bytes();converted,_=convert(store.read('Personal').raw)
            self.assertEqual(store.path('Personal').read_bytes(),raw)
            self.assertIn(('weapon.pyrocitor.level5','ammo_interval_ticks'),c.checked_document(converted).entries)

if __name__=='__main__':unittest.main()
