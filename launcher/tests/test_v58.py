"""Original page rendering, retained v58 palettes and lossless v57 migration."""
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock
import sys, unittest
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT/'launcher'))
import launcher as gui, launcher_core as core, i18n
from schema import (MODEL_OPTIONS, MODEL_COLOR_GROUPS, MODEL_NEW_KEYS,
                    EDITOR_WEAPONS, field_groups, visible_model_field)
from migrate_ini import convert

class Value:
    def __init__(self, value=''): self.value=value
    def get(self): return self.value
    def set(self, value): self.value=value

class PageTests(unittest.TestCase):
    def test_model_changes_rebuild_the_existing_form_and_keep_hidden_values(self):
        app=gui.Launcher.__new__(gui.Launcher);app.render_page=Mock()
        model=Value('0');hidden=Value('AABBCC')
        app.select_option(model,'2','grenade_model')
        self.assertEqual(model.get(),'2');self.assertEqual(hidden.get(),'AABBCC')
        app.render_page.assert_called_once_with()
        app.render_page.reset_mock();app.select_option(model,'true','gold_texture')
        app.render_page.assert_not_called()

    def test_navigation_renders_immediately_and_resets_scroll_as_before(self):
        app=gui.Launcher.__new__(gui.Launcher)
        app.search=Value('damage');app.canvas=Mock();app.render_page=Mock()
        app.select_page('visibomb')
        self.assertEqual(app.page,'visibomb');self.assertEqual(app.search.get(),'')
        app.render_page.assert_called_once_with();app.canvas.yview_moveto.assert_called_once_with(0)

    def test_model_palette_filter_builds_only_selected_rows(self):
        doc=core.checked_document((ROOT/'launcher/weapons.default.ini').read_bytes())
        for weapon,selector in [('visibomb','grenade_model'),('mine_glove','flame_mine_model')]:
            for model,group in enumerate(MODEL_COLOR_GROUPS[weapon]):
                section=f'weapon.{weapon}.level1';app=gui.Launcher.__new__(gui.Launcher)
                app.ttk=Mock();app.tk=Mock();app.matches=lambda *_:True
                app.variables={pair:Value(value) for pair,value in doc.entries.items()}
                app.variables[section,selector].set(str(model))
                palette_keys=[key for keys in MODEL_COLOR_GROUPS[weapon] for key in keys]
                app.fields(Mock(),section,palette_keys)
                self.assertEqual([call.kwargs['textvariable'] for call in app.ttk.Entry.call_args_list],
                                 [app.variables[section,key] for key in group])

class ModelTests(unittest.TestCase):
    def setUp(self):
        self.raw=(ROOT/'launcher/weapons.default.ini').read_bytes()
        self.doc=core.checked_document(self.raw)
        self.addCleanup(i18n.set_language,'en')

    def test_old_v57_values_are_unchanged_and_only_75_missing_entries_are_added(self):
        removed={i for (s,k),i in self.doc.addresses.items() if k in MODEL_NEW_KEYS}
        old=b'\xef\xbb\xbf'+''.join(line for i,line in enumerate(self.doc.lines) if i not in removed).replace('\n','\r\n').encode()
        before=core.checked_document(old);out=convert(old)[0];after=core.checked_document(out)
        self.assertEqual(core.required_config(before),56);self.assertEqual(core.required_config(after),58)
        self.assertEqual(len(after.entries)-len(before.entries),75)
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value)
        self.assertEqual(convert(out)[0],out);self.assertTrue(out.startswith(b'\xef\xbb\xbf'));self.assertIn(b'\r\n',out)

    def test_selectors_precede_colors_and_only_selected_palette_is_shown_for_every_grade(self):
        edits={}
        for weapon,selector in [('visibomb','grenade_model'),('mine_glove','flame_mine_model')]:
            for n in range(1,6):
                section=f'weapon.{weapon}.level{n}';keys=self.doc.keys(section)
                standard,experiment=field_groups(weapon,keys)
                self.assertEqual(keys,standard+experiment)
                self.assertTrue(experiment[0].endswith('_enabled'))
                index=experiment.index(selector)
                self.assertTrue(all(key.endswith('_color') for key in experiment[index+1:]))
                for model,group in enumerate(MODEL_COLOR_GROUPS[weapon]):
                    edits[section,selector]=str(model)
                    for other in MODEL_COLOR_GROUPS[weapon]:
                        for key in other:self.assertEqual(visible_model_field(weapon,key,str(model)+'.0'),key in group)
                    for key in group:edits[section,key]=f'{n:02X}{model:02X}A0'
        result=core.checked_document(convert(self.doc.render(edits))[0])
        for pair,value in edits.items():self.assertEqual(result.entries[pair],value)
        for selector in MODEL_OPTIONS:
            weapon='visibomb' if selector=='grenade_model' else 'mine_glove'
            for bad in ('-1','3','1.5','NaN','inf','true'):
                with self.assertRaises(ValueError):core.validate_value(f'weapon.{weapon}.level1',selector,bad)
        for language in i18n.LANGUAGES:
            i18n.set_language(language)
            for choices in MODEL_OPTIONS.values():
                for name in choices:self.assertTrue(i18n.model_name(name))
