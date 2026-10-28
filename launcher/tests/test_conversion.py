from pathlib import Path
import hashlib,json,sys,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as c
from migrate_ini import convert,runtime_profile
from schema import LABELS,EDITOR_WEAPONS,HELP,EXPERIMENTAL_DEFAULTS,STANDARD_DEFAULTS,GLOBAL_STANDARD_DEFAULTS,GLOBAL_EXPERIMENTAL_DEFAULTS

class ProfileTests(unittest.TestCase):
    def setUp(self):self.raw=(ROOT/'weapons.default.ini').read_bytes()
    def test_all_approved_balance_values_are_preserved_exactly(self):
        d=c.profile_document(self.raw)
        # All values are pinned to the author's supplied release profile.
        approved=json.loads((ROOT.parent/'mod/default-profile.json').read_text())
        canonical=json.dumps(sorted((s,k,v) for (s,k),v in d.entries.items()),ensure_ascii=True).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(),approved['values_sha256'])
        self.assertEqual(len(d.entries),2592);self.assertEqual(len(d.sections),146)
        self.assertEqual(d.entries['general','xp_per_damage'],'0.12')
    def test_public_profile_has_no_damage_model_or_wrench_and_keeps_functioning_opacity(self):
        d=c.profile_document(self.raw)
        self.assertNotIn(('general','damage_mode'),d.entries)
        self.assertIn('wrench',EDITOR_WEAPONS)
        self.assertTrue(any(s.startswith('weapon.wrench') for s in d.sections))
        self.assertEqual(d.entries['hud','name_opacity_percent'],'100')
        self.assertEqual(LABELS['name_opacity_percent'],'Weapon name opacity (%)')
        self.assertIn('3D',HELP['name_opacity_percent'])
        for value in ['0','50','100']:self.assertEqual(c.validate_value('hud','name_opacity_percent',value),value)
    def test_normalizing_never_rescales_hp_or_xp_and_is_idempotent(self):
        out,rows=convert(self.raw);self.assertEqual(out,self.raw);self.assertEqual(rows,[])
        self.assertEqual(convert(out)[0],out)
    def test_old_real_hp_flag_is_removed_without_changing_other_values(self):
        old=self.raw.replace(b'[general]',b'[general]\ndamage_mode = real_hp',1)
        out,_=convert(old);self.assertEqual(out,self.raw)
    def test_legacy_damage_models_and_multipliers_are_rejected(self):
        with self.assertRaisesRegex(ValueError,'only supports real-HP'):
            c.profile_document(self.raw.replace(b'[general]',b'[general]\ndamage_mode = legacy',1))
        with self.assertRaises(ValueError):
            c.profile_document(self.raw.replace(b'damage_hp = 150',b'damage_multiplier = 1',1))
    def test_runtime_marker_exists_only_in_private_session_copy(self):
        generated=runtime_profile(self.raw)
        self.assertEqual(c.checked_document(generated).entries['general','damage_mode'],'real_hp')
        self.assertEqual(generated.count(b'damage_mode = real_hp'),1)
        self.assertEqual(runtime_profile(generated),generated)
        self.assertNotIn(b'damage_mode',(ROOT/'weapons.default.ini').read_bytes())
    def test_bom_crlf_comments_and_manually_edited_ammo_survive(self):
        edited=c.IniDocument(self.raw).render({('weapon.blaster.level1','max_ammo'):'47'})
        raw=b'\xef\xbb\xbf'+edited.replace(b'\n',b'\r\n')
        self.assertEqual(convert(raw)[0],raw)
        self.assertEqual(c.profile_document(raw).entries['weapon.blaster.level1','max_ammo'],'47')
        self.assertFalse(runtime_profile(raw).startswith(b'\xef\xbb\xbf'))
    def test_old_hidden_fields_are_removed_from_the_editor(self):
        raw=self.raw.replace(b'[hud]',b'[hud]\nxp_bar_opacity_percent = 50',1)

        d=c.profile_document(raw)
        self.assertNotIn(('hud','xp_bar_opacity_percent'),d.entries)
        self.assertIn('weapon.wrench',d.sections)

if __name__=='__main__':unittest.main()
