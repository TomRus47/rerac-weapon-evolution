"""Lossless v52 migration and per-grade grenade/mine editor contract."""
from pathlib import Path
import sys, unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import launcher_core as core
from migrate_ini import convert
from schema import field_groups,EXPERIMENTAL_DEFAULTS,MODEL_NEW_KEYS
class GrenadeMineTests(unittest.TestCase):
    def setUp(self):
        self.raw=(Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()
        self.doc=core.checked_document(self.raw)
    def test_five_grades_and_footer_order(self):
        changes={}
        for n in range(1,6):
            for weapon,key,value in [('visibomb','grenade_size_percent','250'),('mine_glove','flame_mine_size_percent','150')]:
                section=f'weapon.{weapon}.level{n}';keys=self.doc.keys(section)
                # Migration fallback and the approved release balance are
                # independent. Existing personal sizes must remain unchanged.
                self.assertEqual(EXPERIMENTAL_DEFAULTS[weapon][key],value)
                self.assertEqual(core.profile_document(self.raw).entries[section,key],self.doc.entries[section,key])
                base,experimental=field_groups(weapon,keys);self.assertEqual(keys,base+experimental)
                self.assertEqual(experimental[0],next(iter(EXPERIMENTAL_DEFAULTS[weapon])))
            section=f'weapon.visibomb.level{n}';changes[section,'grenade_damage_hp']=str(145*n)
            self.assertIn('explosion_damage_radius',field_groups('visibomb',self.doc.keys(section))[0])
            self.assertEqual(EXPERIMENTAL_DEFAULTS['visibomb']['grenade_launcher_enabled'],'false')
        doc=core.checked_document(self.doc.render(changes))
        for p,v in changes.items():self.assertEqual(doc.entries[p],v)
        self.assertEqual(core.required_config(doc),90)
    def test_migration_adds_55_fields_without_changing_old_values(self):
        new={p for p in self.doc.entries if p[1] not in MODEL_NEW_KEYS and (p[1].startswith('grenade_') or p[1]=='flame_mine_size_percent' or (p[0].startswith('weapon.visibomb.level') and p[1]=='explosion_damage_radius'))}
        self.assertEqual(len(new),88);indexes={self.doc.addresses[p] for p in new}
        old=''.join(line for i,line in enumerate(self.doc.lines) if i not in indexes).encode()
        original=core.IniDocument(old);migrated=convert(old)[0];loaded=core.checked_document(migrated)
        for p,v in original.entries.items():self.assertEqual(loaded.entries[p],v)
        self.assertEqual(len(loaded.entries)-len(original.entries),len(new));self.assertEqual(convert(migrated)[0],migrated)
    def test_invalid_new_values_rejected(self):
        for key,value in [('grenade_size_percent','0'),('grenade_arc_height','NaN'),('grenade_fire_interval_ticks','2.5'),('grenade_max_range','0'),('grenade_explosion_radius','9'),('grenade_damage_hp','-1'),('grenade_color','GG0000'),('grenade_launcher_enabled','perhaps')]:
            with self.assertRaises(ValueError,msg=key):core.validate_value('weapon.visibomb.level1',key,value)
        with self.assertRaises(ValueError):core.validate_value('weapon.mine_glove.level5','flame_mine_size_percent','501')
