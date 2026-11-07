"""Freezer settings migrate additively and use the checked localized gadget page."""
from pathlib import Path
import sys,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import schema,launcher_core as core,i18n
from migrate_ini import convert
class FreezerTests(unittest.TestCase):
 def test_v134_profile_gains_only_the_missing_recharge_setting(self):
  raw=(ROOT/'weapons.default.ini').read_bytes()
  # A v134 user can have a custom capacity, range, color and enabled frost mode.
  doc=core.IniDocument(raw)
  old=doc.render({('gadget.hydrodisplacer','max_ammo'):'7',('gadget.hydrodisplacer','freeze_radius'):'8.5',('gadget.hydrodisplacer','ice_color'):'ABCD01'})
  old=b'\n'.join(line for line in old.split(b'\n') if not line.startswith(b'recharge_seconds ='))
  before=core.IniDocument(old);out,_=convert(old);after=core.IniDocument(out)
  self.assertTrue(all(after.entries[key]==value for key,value in before.entries.items()))
  self.assertEqual(after.entries[('gadget.hydrodisplacer','recharge_seconds')],'10')
  self.assertEqual(convert(out)[0],out)
  keys=after.keys('gadget.hydrodisplacer')
  self.assertLess(keys.index('recharge_seconds'),keys.index('ice_color'))
 def test_recharge_bounds_and_runtime_requirement(self):
  for value in ['0.1','1','10','3600']:core.validate_value('gadget.hydrodisplacer','recharge_seconds',value)
  for value in ['0','-1','nan','inf','3601']:
   with self.assertRaises(ValueError):core.validate_value('gadget.hydrodisplacer','recharge_seconds',value)
  doc=core.profile_document((ROOT/'weapons.default.ini').read_bytes())
  self.assertEqual(core.required_config(core.checked_document(doc.render({('gadget.hydrodisplacer','enabled'):'true'}))),97)
 def test_localized_setting_is_ordered_before_the_colors_in_every_language(self):
  try:
   for language in i18n.LANGUAGES:
    i18n.set_language(language)
    self.assertNotEqual(i18n.label('recharge_seconds'),'recharge_seconds')
    self.assertIn('recharge_seconds',i18n.help_text('gadget.hydrodisplacer','recharge_seconds'))
    self.assertNotIn('PLACEHOLDER',i18n.tr(schema.GADGET_HELP['hydrodisplacer']))
    self.assertTrue(i18n.tr(schema.GADGET_HELP['trespasser']))
   standard,experimental=schema.gadget_field_groups('hydrodisplacer',schema.GADGET_DEFAULTS['hydrodisplacer'])
   self.assertEqual(standard[:3],['enabled','max_ammo','recharge_seconds'])
   self.assertTrue(all(k.endswith('_color') for k in standard[-4:]));self.assertEqual(experimental,[])
  finally:i18n.set_language('en')
if __name__=='__main__':unittest.main()
