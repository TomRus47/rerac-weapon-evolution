"""Gadget pages share the editor without becoming XP weapon tracks."""
from pathlib import Path
import sys,unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT))
import launcher_core as core, schema, i18n
from migrate_ini import convert
class GadgetTests(unittest.TestCase):
 def test_old_balance_is_retained_and_gadget_sections_are_added_once(self):
  raw=(ROOT/'weapons.default.ini').read_bytes();d=core.IniDocument(raw)
  old=raw[:raw.index(b'[gadget.swingshot]')]
  out,_=convert(old);new=core.IniDocument(out)
  for pair,value in d.entries.items():
   if not pair[0].startswith('gadget.'):self.assertEqual(new.entries[pair],value,pair)
  self.assertEqual(convert(out)[0],out)
  self.assertEqual(list(schema.EDITOR_WEAPONS)[-4:],list(schema.GADGET_DEFAULTS))
 def test_enabled_gadgets_require_new_runtime_without_raising_the_neutral_profile_floor(self):
  raw=(ROOT/'weapons.default.ini').read_bytes();doc=core.profile_document(raw)
  self.assertEqual(core.required_config(doc),95)
  for name in schema.GADGET_DEFAULTS:
   on=doc.render({('gadget.'+name,'enabled'):'true'})
   self.assertEqual(core.required_config(core.checked_document(on)),97 if name=='hydrodisplacer' else 96)
 def test_validation_matches_runtime_bounds_and_keeps_colors_last(self):
  for name,defaults in schema.GADGET_DEFAULTS.items():
   for key,value in defaults.items():core.validate_value('gadget.'+name,key,value)
   standard,experimental=schema.gadget_field_groups(name,defaults)
   colors=[key for key in standard if key.endswith('_color')]
   if colors:self.assertEqual(standard[-len(colors):],colors)
   if experimental:self.assertEqual(experimental[0],'surface_grapple_enabled')
  for key,value in [('freeze_seconds','-1'),('projectile_speed','inf'),('max_ammo','2.5')]:
   with self.assertRaises(ValueError):core.validate_value('gadget.hydrodisplacer',key,value)
 def test_gadget_names_labels_and_help_are_available_in_every_language(self):
  try:
   for lang in i18n.LANGUAGES:
    i18n.set_language(lang)
    for name,defaults in schema.GADGET_DEFAULTS.items():
     self.assertTrue(i18n.weapon_name(name))
     for key in defaults:
      self.assertTrue(i18n.label(key));self.assertIn('[gadget.'+name+']',i18n.help_text('gadget.'+name,key))
     self.assertTrue(i18n.tr(schema.GADGET_HELP[name]))
  finally:i18n.set_language('en')
if __name__=='__main__':unittest.main()
