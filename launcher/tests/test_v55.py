"""Language-neutral profiles, localized widgets, ordering and alternate capacities."""
from pathlib import Path
import sys,tempfile,unittest
from types import SimpleNamespace
from unittest.mock import Mock
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'launcher'))
import launcher as gui
import launcher_core as core
import i18n
from schema import WEAPON_ORDER,EXPERIMENTAL_ENABLERS,EXPERIMENTAL_DEFAULTS,COLORS,BOOLEANS,field_groups
from migrate_ini import convert

class Var:
    def __init__(self,value=''):self.value=value
    def get(self):return self.value
    def set(self,value):self.value=value
    def trace_add(self,*args):pass

class Widget:
    def __init__(self,kind,**options):self.kind=kind;self.options=options;self.bindings={}
    def pack(self,*args,**kwargs):pass
    def grid(self,*args,**kwargs):pass
    def columnconfigure(self,*args,**kwargs):pass
    def bind(self,key,callback):self.bindings[key]=callback

class V55Tests(unittest.TestCase):
    def setUp(self):
        i18n.set_language('en');gui.BASE_LABEL='Default configuration (weapons.ini)'
        self.addCleanup(i18n.set_language,'en')
        self.raw=(ROOT/'launcher/weapons.default.ini').read_bytes()
    def test_every_grade_follows_same_order_and_colors_finish_their_frame(self):
        doc=core.checked_document(self.raw)
        self.assertEqual([s[7:] for s in doc.sections if s.startswith('weapon.') and '.level' not in s],list(WEAPON_ORDER))
        for weapon in WEAPON_ORDER:
            for grade in range(1,6):
                keys=doc.keys(f'weapon.{weapon}.level{grade}');standard,experimental=field_groups(weapon,keys)
                self.assertEqual(keys,standard+experimental)
                for group in (standard,experimental):
                    color_seen=False
                    for key in group:
                        if key in COLORS:color_seen=True
                        elif color_seen:self.fail(f'{weapon}: {key} after colors')
                if experimental:
                    switches=EXPERIMENTAL_ENABLERS[weapon]
                    self.assertEqual(experimental[:len(switches)],list(switches))
                    self.assertEqual(experimental[len(switches)],'experimental_max_ammo')
                gold=[k for k in standard if k.startswith('gold')]
                if gold:self.assertEqual([k for k in standard if k not in COLORS][-len(gold):],gold)
        standard,experimental=field_groups('mine_glove',doc.keys('weapon.mine_glove.level1'))
        for key in ('flame_core_color','flame_outer_color','burn_flame_color'):self.assertIn(key,experimental)
        for key in ('explosion_color','light_color'):self.assertIn(key,standard)
    def test_migration_adds_forty_capacities_from_each_existing_grade_and_preserves_all_values(self):
        doc=core.IniDocument(self.raw)
        old=''.join(line for line in doc.lines if not line.lstrip().startswith('experimental_max_ammo')).encode()
        before=core.checked_document(old);after_raw=convert(old)[0];after=core.checked_document(after_raw)
        self.assertEqual(len(after.entries)-len(before.entries),40)
        for pair,value in before.entries.items():self.assertEqual(after.entries[pair],value)
        for weapon in EXPERIMENTAL_ENABLERS:
            for grade in range(1,6):
                sec=f'weapon.{weapon}.level{grade}'
                self.assertEqual(after.entries[sec,'experimental_max_ammo'],before.entries[sec,'capacity' if weapon=='suck_cannon' else 'max_ammo'])
        self.assertEqual(convert(after_raw)[0],after_raw)
        edited=after.render({('weapon.bomb_glove.level2','experimental_max_ammo'):'77'})
        self.assertEqual(core.checked_document(convert(edited)[0]).entries['weapon.bomb_glove.level2','experimental_max_ammo'],'77')
    def test_alternate_capacity_rejects_invalid_and_unsupported_inputs(self):
        for weapon in EXPERIMENTAL_ENABLERS:
            for value in ('-1','1.5','NaN','10000','On'):
                with self.assertRaises(ValueError):core.validate_value(f'weapon.{weapon}.level1','experimental_max_ammo',value)
        with self.assertRaises(ValueError):core.validate_value('weapon.suck_cannon.level1','experimental_max_ammo','11')
        with self.assertRaises(ValueError):core.validate_value('weapon.wrench.level1','experimental_max_ammo','4')
        self.assertEqual(core.required_config(core.checked_document(self.raw)),56)
        doc=core.IniDocument(self.raw)
        removed={index for (section,key),index in doc.addresses.items() if key=='experimental_max_ammo' or (section.startswith('weapon.wrench.level') and key in {'hit_radius_multiplier','knockback_multiplier'})}
        old=core.checked_document(''.join(line for index,line in enumerate(doc.lines) if index not in removed).encode())
        self.assertEqual(core.required_config(old),54)
    def test_catalogs_cover_all_public_fields_and_weapon_names_in_all_five_languages(self):
        doc=core.checked_document(self.raw)
        for language in i18n.LANGUAGES:
            i18n.set_language(language)
            for section,key in doc.entries:
                self.assertTrue(i18n.label(key));self.assertIn(f'[{section}] {key}',i18n.help_text(section,key))
            self.assertEqual(len({i18n.weapon_name(k) for k in WEAPON_ORDER}),16)
            for catalog in (i18n.TEXT,i18n.FIELD_LABELS,i18n.WEAPON_NAMES,i18n.HELP_TEXT):
                for source,translations in catalog.items():
                    self.assertEqual(set(translations),set(i18n.LANGUAGES));self.assertTrue(translations[language],source)
        i18n.set_language('it');self.assertEqual(i18n.weapon_name('ryno'),'M.I.P.S.')
        i18n.set_language('es');self.assertEqual(i18n.weapon_name('pyrocitor'),'Piromátic')
        i18n.set_language('fr');self.assertEqual(i18n.weapon_name('walloper'),'Repousseur')
    def test_boolean_widgets_use_off_on_and_write_canonical_values_including_wrench(self):
        app=gui.Launcher.__new__(gui.Launcher);widgets=[]
        def factory(kind):
            def create(*args,**kwargs):
                widget=Widget(kind,**kwargs);widgets.append(widget);return widget
            return create
        app.ttk=SimpleNamespace(**{kind:factory(kind) for kind in ('Frame','Label','Combobox','Entry')})
        app.tk=SimpleNamespace(StringVar=Var);app.matches=lambda *_:True
        app.variables={}
        for section in ('weapon.wrench.level1','weapon.bomb_glove.level3','general'):
            for key in ('gold_texture','show_hud','experimental_ammo_unlock_mode'):
                pair=(section,key);var=Var('false');app.variables[pair]=var;widgets.clear()
                app.fields(None,section,[key])
                box=next(w for w in widgets if w.kind=='Combobox')
                self.assertEqual(box.options['values'],('Off','On'))
                self.assertEqual(box.options['textvariable'].get(),'Off')
                box.options['textvariable'].set('On');box.bindings['<<ComboboxSelected>>'](None)
                self.assertEqual(var.get(),'true')
                box.options['textvariable'].set('Off');box.bindings['<<ComboboxSelected>>'](None)
                self.assertEqual(var.get(),'false')
    def test_language_switch_preserves_unsaved_variables_and_active_session(self):
        app=gui.Launcher.__new__(gui.Launcher);app.language_var=Var('Italiano');app.root=Mock()
        app.root.winfo_children.return_value=[Mock()];app.current_name='My preset';app.preset_var=Var()
        app.variables={('weapon.blaster.level2','damage_hp'):Var('789')};original=app.variables
        app.session=object();session=app.session;app.status=Var();app.game_status=Var()
        for method in ('build_ui','refresh_presets','render_page','changed','remember_settings'):setattr(app,method,Mock())
        app.change_language()
        self.assertIs(app.variables,original);self.assertEqual(app.variables['weapon.blaster.level2','damage_hp'].get(),'789')
        self.assertIs(app.session,session);self.assertEqual(app.current_name,'My preset')
        self.assertEqual(app.game_status.get(),'Gioco in esecuzione');app.remember_settings.assert_called_once()
        gui.BASE_LABEL='Default configuration (weapons.ini)'
    def test_translation_module_is_distributed_and_included_in_frozen_cache_identity(self):
        import build_launcher_exe
        self.assertIn('i18n.py',core.DISTRIBUTED_FILES)
        self.assertIn('i18n.py',build_launcher_exe.SOURCES)

    def test_language_setting_round_trip_and_invalid_locale_fallback(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);core.save_settings(root,{'language':'fr','last_preset':'Red','data':'data'})
            self.assertEqual(core.read_settings(root)['language'],'fr')
        i18n.set_language('invalid');self.assertEqual(i18n.language(),'en')
    def test_ui_label_translation_never_changes_raw_ini_or_boolean_literals(self):
        doc=core.checked_document(self.raw)
        for language in i18n.LANGUAGES:
            i18n.set_language(language)
            for key in BOOLEANS:
                self.assertEqual(i18n.boolean_value(i18n.boolean_display('yes')),'true')
                self.assertEqual(i18n.boolean_value(i18n.boolean_display('0')),'false')
            self.assertEqual(doc.render(),self.raw)
            self.assertEqual(i18n.label('experimental_ammo_unlock_mode'),'Ammo-Survival Mode')
