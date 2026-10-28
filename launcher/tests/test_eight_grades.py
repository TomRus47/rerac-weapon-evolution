"""Eight-grade migration preserves personal balance and supplies independent controls."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from pathlib import Path
import tempfile
import re
import unittest
import launcher_core as core
import migrate_ini as migration
import schema
import i18n
def five_grades(raw):
 # Simulate a legacy profile even if the current personal defaults have V6-V8.
 lines=[];skip=False
 for line in raw.decode('utf-8-sig').splitlines(keepends=True):
  if line.lstrip().startswith('['):skip=bool(re.match(r'\s*\[weapon\.[a-z_]+\.level[678]\]',line))
  if not skip:lines.append(line)
 return ''.join(lines).encode()

class EightGradeTests(unittest.TestCase):
 def test_old_profile_keeps_every_value_and_appends_all_eight_grade_sections(self):
  raw=five_grades((Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes());before=core.IniDocument(raw)
  after=core.profile_document(raw)
  for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value,pair)
  for weapon in schema.WEAPONS:
   for grade in range(1,9):self.assertIn(f'weapon.{weapon}.level{grade}',after.sections)
   for key in ['xp_v6_to_v7','xp_v7_to_v8']:self.assertIn(('weapon.'+weapon,key),after.entries)
  self.assertEqual(after.entries['general','hide_ryno'],'false')
  self.assertEqual(core.required_config(after),90)
  self.assertEqual(core.profile_document(after.raw).raw,after.raw)
 def test_upper_grades_are_independent_and_preset_round_trip_keeps_custom_fields(self):
  doc=core.profile_document((Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes())
  changed=doc.render({('weapon.blaster.level8','damage_hp'):'9999',('weapon.blaster.level7','max_ammo'):'77',('general','hide_ryno'):'true'})
  with tempfile.TemporaryDirectory()as t:
   store=core.PresetStore(Path(t));store.save('Test',changed);loaded=store.read('Test')
   self.assertEqual(loaded.entries['weapon.blaster.level8','damage_hp'],'9999');self.assertEqual(loaded.entries['weapon.blaster.level7','max_ammo'],'77')
   self.assertEqual(loaded.entries['general','hide_ryno'],'true')
 def test_migration_keeps_bom_crlf_and_existing_upper_grade_values(self):
  raw=five_grades((Path(__file__).resolve().parents[1]/'weapons.default.ini').read_bytes()).replace(b'\r\n',b'\n').replace(b'\n',b'\r\n')
  raw=b'\xef\xbb\xbf'+raw.removeprefix(b'\xef\xbb\xbf')+b'\r\n[weapon.blaster.level8]\r\ndamage_hp=1234 ; custom\r\nmax_ammo=42\r\n'
  changed=migration.convert(raw)[0];self.assertTrue(changed.startswith(b'\xef\xbb\xbf'));self.assertNotIn(b'\n',changed.replace(b'\r\n',b''))
  self.assertIn(b'damage_hp=1234 ; custom',changed);self.assertEqual(core.IniDocument(changed).entries['weapon.blaster.level8','max_ammo'],'42')
 def test_hide_ryno_and_purchase_gate_are_localized(self):
  for language in i18n.LANGUAGES:
   i18n.set_language(language)
   self.assertTrue(i18n.help_text('general','hide_ryno'))
   self.assertIn(language,i18n.TEXT['Gold purchase (NG+)'])
  i18n.set_language('en')
if __name__=='__main__':unittest.main()
