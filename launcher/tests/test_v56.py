"""Wrench XP strip, navigation placement and independent collision/push settings."""
from pathlib import Path
import sys,unittest
from unittest.mock import Mock
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'launcher'))
import launcher as gui,launcher_core as core,i18n
from schema import WEAPON_ORDER,EDITOR_WEAPONS,XP,field_groups,MODEL_NEW_KEYS
from migrate_ini import convert
class V56Tests(unittest.TestCase):
    def setUp(self):
        self.raw=(ROOT/'launcher/weapons.default.ini').read_bytes()
        i18n.set_language('en');self.addCleanup(i18n.set_language,'en')
    def test_every_weapon_renders_all_four_xp_requirements_in_one_progression_card(self):
        doc=core.checked_document(self.raw)
        for weapon in WEAPON_ORDER:
            app=gui.Launcher.__new__(gui.Launcher);app.doc=doc;app.page=weapon
            app.variables={pair:Mock(name=str(pair)) for pair in doc.entries}
            app.form=Mock();app.form.winfo_children.return_value=[]
            app.ttk=Mock();app.nav_buttons={weapon:Mock()};app.page_title=Mock();app.context=Mock()
            app.card=Mock();app.fields=Mock();app.arrange_cards=Mock();app.matches=lambda *_:True
            app.render_page()
            xp_entries=app.ttk.Entry.call_args_list
            self.assertEqual(len(xp_entries),4,weapon)
            self.assertEqual([c.kwargs['textvariable'] for c in xp_entries],[app.variables['weapon.'+weapon,key] for key in XP])
            self.assertEqual(app.card.call_args_list[0].args[1],'XP progression')
            self.assertFalse(any(c.args[1]=='weapon.'+weapon for c in app.fields.call_args_list),weapon)
            self.assertEqual(len(app.cards),5)
    def test_wrench_is_first_after_globals_and_ini_matches_navigation_order(self):
        self.assertEqual(list(EDITOR_WEAPONS)[:2],['wrench','bomb_glove'])
        self.assertEqual(WEAPON_ORDER[1:],('bomb_glove','pyrocitor','blaster','glove_of_doom','suck_cannon','taunter','mine_glove','devastator','visibomb','walloper','decoy_glove','drone_device','tesla_claw','morph_ray','ryno'))
        doc=core.checked_document(self.raw)
        self.assertEqual(doc.sections[:3],['general','hud','weapon.wrench'])
    def test_old_profiles_gain_ten_identity_values_and_preserve_every_other_setting(self):
        doc=core.checked_document(self.raw)
        removed={i for (s,k),i in doc.addresses.items() if s.startswith('weapon.wrench.level') and k in {'hit_radius_multiplier','knockback_multiplier'} or k in MODEL_NEW_KEYS}
        old=b'\xef\xbb\xbf'+''.join(line for i,line in enumerate(doc.lines) if i not in removed).replace('\n','\r\n').encode()
        before=core.checked_document(old);result=convert(old)[0];after=core.checked_document(result)
        self.assertEqual(len(after.entries)-len(before.entries),85)
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value)
        for grade in range(1,6):
            for key in ('hit_radius_multiplier','knockback_multiplier'):self.assertEqual(after.entries[f'weapon.wrench.level{grade}',key],'1')
        self.assertEqual(convert(result)[0],result);self.assertIn(b'\r\n',result);self.assertTrue(result.startswith(b'\xef\xbb\xbf'))
        self.assertEqual(core.required_config(before),55);self.assertEqual(core.required_config(after),58)
    def test_independent_grade_settings_limits_and_translated_help(self):
        doc=core.checked_document(self.raw);edits={}
        for n in range(1,6):
            section=f'weapon.wrench.level{n}'
            edits[section,'hit_radius_multiplier']=str(n/2)
            edits[section,'knockback_multiplier']=str(n-1)
        updated=core.checked_document(convert(doc.render(edits))[0])
        for pair,value in edits.items():self.assertEqual(updated.entries[pair],value)
        for key,bad in [('hit_radius_multiplier','0'),('hit_radius_multiplier','10.1'),('knockback_multiplier','-1'),('knockback_multiplier','NaN'),('knockback_multiplier','10.1')]:
            with self.assertRaises(ValueError):core.validate_value('weapon.wrench.level1',key,bad)
        # Walloper retains its earlier, broader limits.
        self.assertEqual(core.validate_value('weapon.walloper.level1','knockback_multiplier','12'),'12')
        for lang in i18n.LANGUAGES:
            i18n.set_language(lang)
            for key in ('hit_radius_multiplier','knockback_multiplier'):
                text=i18n.help_text('weapon.wrench.level1',key)
                self.assertIn(key,text);self.assertTrue(i18n.label(key))
