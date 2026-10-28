"""v45 modes, independent per-grade parameters and icon persistence."""
from pathlib import Path
import sys,unittest,tempfile,shutil
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as core
from migrate_ini import convert
from schema import EXPERIMENTAL_DEFAULTS,STANDARD_DEFAULTS,field_groups
from icon_assets import validate_ico,preserve_icon,extract_ico

class V45Tests(unittest.TestCase):
    def setUp(self):self.raw=(ROOT/'weapons.default.ini').read_bytes()
    def test_ryno_modes_are_exclusive_at_each_grade_but_not_across_grades(self):
        doc=core.profile_document(self.raw)
        first=('weapon.ryno.level1','continuous_missiles_enabled')
        second=('weapon.ryno.level1','shotgun_enabled')
        valid=doc.render({first:'true',second:'false',('weapon.ryno.level2','shotgun_enabled'):'true',
                          ('weapon.ryno.level2','continuous_missiles_enabled'):'false'})
        core.checked_document(valid)
        with self.assertRaisesRegex(ValueError,'either continuous missiles or shotgun'):
            core.checked_document(doc.render({first:'true',second:'true'}))
    def test_every_companion_grade_and_tesla_overlap_is_independent(self):
        doc=core.profile_document(self.raw)
        edits={}
        for grade in range(1,6):
            section=f'weapon.drone_device.level{grade}'
            edits[section,'sand_mouse_enabled']='true'
            edits[section,'sand_mouse_mode']=str((grade-1)%3+1)
            edits[section,'sand_mouse_shot_damage_hp']=str(grade*25)
            edits[f'weapon.tesla_claw.level{grade}','secondary_beam_damage_percent']=str(grade*10)
        result,_=convert(doc.render(edits));loaded=core.checked_document(result)
        for key,value in edits.items():self.assertEqual(loaded.entries[key],value)
        for weapon in EXPERIMENTAL_DEFAULTS:
            for grade in range(1,6):
                keys=loaded.keys(f'weapon.{weapon}.level{grade}')
                a,b=field_groups(weapon,keys);self.assertEqual(keys,a+b)
        self.assertEqual(core.required_config(loaded),90)
    def test_missing_new_keys_are_added_without_touching_user_values(self):
        keys={key for group in (EXPERIMENTAL_DEFAULTS,STANDARD_DEFAULTS) for weapon,fields in group.items() if weapon in {'ryno','mine_glove','drone_device','taunter','tesla_claw'} for key in fields}
        old=b''.join(line for line in self.raw.splitlines(True) if line.split(b'=',1)[0].strip().decode() not in keys)
        before=core.checked_document(old);after=core.checked_document(convert(old)[0])
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value)
        self.assertEqual(after.entries['weapon.tesla_claw.level5','secondary_beam_damage_percent'],'25')
    def test_attached_icon_is_valid_and_an_explicit_icon_wins_over_exe_recovery(self):
        with tempfile.TemporaryDirectory() as tmp:
            kit=Path(tmp);raw=(ROOT/'launcher.ico').read_bytes();validate_ico(raw)
            (kit/'launcher.ico').write_bytes(raw)
            with patch('icon_assets.extract_ico') as extract:
                self.assertEqual(preserve_icon(kit),kit/'launcher.ico')
            extract.assert_not_called();self.assertEqual((kit/'launcher.ico').read_bytes(),raw)
    def test_recovery_prefers_a_pre_update_exe_and_never_runs_it(self):
        with tempfile.TemporaryDirectory() as tmp:
            kit=Path(tmp);raw=(ROOT/'launcher.ico').read_bytes()
            old=kit/'ReRAC_Weapon_Evolution.before-v44-20261019.exe';old.write_bytes(b'old exe')
            current=kit/'ReRAC_Weapon_Evolution.exe';current.write_bytes(b'current exe')
            with patch('icon_assets.extract_ico',return_value=raw) as extract:
                preserve_icon(kit)
            extract.assert_called_once_with(old);self.assertEqual((kit/'launcher.ico').read_bytes(),raw)

if __name__=='__main__':unittest.main()

class PeRecoveryTests(unittest.TestCase):
    def test_multiple_icon_images_do_not_change_the_section_header_offset(self):
        import struct
        data=bytearray(0x600)
        def put(fmt,at,*values):struct.pack_into(fmt,data,at,*values)
        data[:2]=b'MZ';put('<I',60,64);data[64:68]=b'PE\0\0'
        put('<H',70,1);put('<H',84,240);put('<H',88,0x20b)
        put('<II',88+112+16,0x1000,0x200)
        section=88+240;data[section:section+8]=b'.rsrc\0\0\0'
        put('<IIII',section+8,0x400,0x1000,0x400,0x200)
        root=0x200
        def directory(offset,entries):
            put('<HH',root+offset+12,0,len(entries))
            for i,(key,value) in enumerate(entries):put('<II',root+offset+16+i*8,key,value)
        directory(0,[(3,0x80000020),(14,0x80000050)])
        directory(0x20,[(1,0x80000080),(2,0x800000a0)])
        directory(0x50,[(1,0x800000c0)])
        directory(0x80,[(0,0xe0)]);directory(0xa0,[(0,0xf0)]);directory(0xc0,[(0,0x100)])
        images=[b'one!',b'second']
        group=struct.pack('<HHH',0,1,2)+b''.join(struct.pack('<BBBBHHIH',16,16,0,0,1,32,len(image),i+1) for i,image in enumerate(images))
        for offset,at,raw in [(0xe0,0x120,images[0]),(0xf0,0x130,images[1]),(0x100,0x140,group)]:
            put('<IIII',root+offset,0x1000+at,len(raw),0,0);data[root+at:root+at+len(raw)]=raw
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'old.exe';path.write_bytes(data);ico=extract_ico(path)
        validate_ico(ico)
        self.assertTrue(ico.endswith(b'one!second'))
        self.assertEqual(struct.unpack_from('<H',ico,4)[0],2)

class DistributionIconTests(unittest.TestCase):
    def test_player_zip_keeps_the_supplied_icon_beside_the_launcher(self):
        import zipfile
        with tempfile.TemporaryDirectory() as folder:
            base=Path(folder);kit=base/'kit';kit.mkdir()
            raw=(ROOT/'weapons.default.ini').read_bytes()
            icon=(ROOT/'launcher.ico').read_bytes();(kit/'launcher.ico').write_bytes(icon)
            (kit/'LICENSE').write_bytes((ROOT/'LICENSE').read_bytes())
            launcher=kit/'ReRAC_Weapon_Evolution.exe';launcher.write_bytes(b'checked launcher fixture')
            runtime=kit/'runtime/rerac.exe';runtime.parent.mkdir();runtime.write_bytes(b'checked runtime fixture')
            (runtime.parent/'assets/shaders').mkdir(parents=True)
            (runtime.parent/'assets/shaders/test.wgsl').write_text('// shader fixture')
            with patch.object(core,'probe_exe',return_value={'version':'0.3.4','weapon_evolution_config':45}):
                archive=core.package_distribution(kit,base/'Release',raw,runtime,False,launcher)
            with zipfile.ZipFile(archive) as z:self.assertEqual(z.read('Release/launcher.ico'),icon)
