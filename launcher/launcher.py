#!/usr/bin/env python3
"""ReRAC Weapon Evolution · preset launcher, button navigation only."""
from __future__ import annotations
import argparse
import os
import queue
import subprocess
import sys
import threading
from pathlib import Path
import launcher_core as core
import i18n
from i18n import tr
from schema import WORLD_DEFAULTS
from schema import WEAPONS, EDITOR_WEAPONS, XP, NAMES, BOOLEANS, COLORS, title, help_text, field_groups, global_field_groups, AMMO_DROP_DEFAULTS, normalized_ammo_chances, MODEL_OPTIONS, visible_model_field

BASE_LABEL = 'Default configuration (weapons.ini)'
GRADE_COLORS = ('#8997aa', '#b840ff', '#ff3030', '#ff8800', '#ffe600', '#bbff20', '#80e0ff', '#ffffff')

def apply_window_icon(root,tk,folder):
    if sys.platform!='win32':return
    icon=Path(getattr(sys,'_MEIPASS',folder))/'launcher.ico'
    if icon.is_file():
        try:root.iconbitmap(default=str(icon))
        except tk.TclError:pass

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--folder', type=Path, default=core.KIT)
    args = parser.parse_args()
    try:
        import tkinter as tk
        from tkinter import ttk, filedialog, messagebox, simpledialog, colorchooser
    except ImportError as error:
        print('Install Python 3.10 or newer with Tcl/Tk on the authoring PC.', file=sys.stderr)
        print(str(error), file=sys.stderr)
        return 2
    root = tk.Tk()
    apply_window_icon(root,tk,args.folder.resolve())
    Launcher(root, args.folder.resolve(), tk, ttk, filedialog, messagebox, simpledialog, colorchooser)
    root.mainloop()
    return 0

# One editor choice maps to the two existing runtime flags; profile formats stay compatible.
RANDOMIZER_ALL_DESCRIPTION = 'All: randomizes weapons, gadgets, Gold Bolts, planet coordinates and other progression rewards across world pickups and purchases. Includes 6 additional locations. Keeps progression requirements, a saved seed and dedicated randomizer saves.'
RANDOMIZER_WEAPONS_DESCRIPTION = 'Weapons: scatters all 15 weapons across eligible world locations, including 6 additional locations. Removes the remaining Gold Bolts and weapon purchases; ammo can be bought after finding each weapon. In NG+, gold weapon upgrades cost normal bolts only, with no Gold Bolt requirement. Uses dedicated weapon-randomizer saves.'
RANDOMIZER_RESTART_DESCRIPTION = 'Off disables both modes. Choose a mode before starting a new run. Existing saves keep their seed and placements; changing mode or seed requires restarting the game and starting a compatible run.'

class Launcher:
    def __init__(self, root, kit, tk, ttk, filedialog, messagebox, simpledialog, colorchooser):
        global BASE_LABEL
        self.root, self.kit, self.tk, self.ttk = root, kit, tk, ttk
        self.files, self.messages, self.dialogs, self.colors = filedialog, messagebox, simpledialog, colorchooser
        self.store = core.PresetStore(kit)
        self.current_name = ''; self.page = 'global'; self.loading = False
        self.session = None; self.launch_busy = False; self.events = queue.Queue()
        self.log_offset = 0; self.log_text = ''; self.confirmed = False; self.cards = []; self.columns = 0
        self.root.title('ReRAC · Weapon Evolution · '+tr('Launcher')+' '+core.VERSION)
        width = min(1450, root.winfo_screenwidth()-60); height = min(920, root.winfo_screenheight()-80)
        self.root.geometry(f'{width}x{height}'); self.root.minsize(930, 620)
        self.root.configure(bg='#161b26')
        self.root.protocol('WM_DELETE_WINDOW', self.close)
        self.preset_var = tk.StringVar(value=BASE_LABEL); self.search = tk.StringVar()
        self.status = tk.StringVar(value=tr('Ready.')); self.context = tk.StringVar()
        self.document_info = tk.StringVar(); self.game_status = tk.StringVar(value=tr('Game closed'))
        self.settings = {}; startup_warning = ''
        try: self.settings = core.read_settings(kit)
        except (ValueError, OSError) as error: startup_warning = str(error)
        exe = core.preferred_exe(kit, self.settings.get('exe'))
        data = core.resolve_path(self.settings.get('data'), kit) or core.discover_data(kit, exe)
        self.exe_var = tk.StringVar(value=str(exe or ''))
        self.data_var = tk.StringVar(value=str(data or ''))
        self.save_var = tk.StringVar(value=str(core.resolve_path(self.settings.get('save'), kit) or ''))
        i18n.set_language(self.settings.get('language','en'))
        BASE_LABEL = tr('Default configuration (weapons.ini)')
        self.preset_var.set(BASE_LABEL)
        self.status.set(tr('Ready.'));self.game_status.set(tr('Game closed'))
        self.configure_style(); self.build_ui()
        try:
            path, source = core.bootstrap_ini(kit)
            self.load_document(core.profile_document(path.read_bytes()), '')
            previous = self.settings.get('last_preset', '')
            if previous and previous in self.store.names():
                self.load_document(self.store.read(previous), previous)
            self.status.set(tr('Preset loaded: ')+self.current_name if self.current_name else tr('Initial configuration: ')+source)
        except (ValueError, OSError, UnicodeError) as error:
            # Keep the user's invalid file untouched; let them choose a corrected INI.
            self.doc = None; self.variables = {}; self.base_values = {}
            self.render_page(); self.error(error)
        self.refresh_presets()
        self.search.trace_add('write', lambda *_: self.render_page())
        self.root.bind('<Control-s>', lambda _: self.save_preset())
        self.root.after(200, self.poll)
        if getattr(sys,'frozen',False) and not core.valid_data(data):
            self.status.set(tr('First launch: choose your extracted game data folder.'))
            self.root.after(300, self.launch_options)
        if startup_warning: self.messages.showwarning(tr('Launcher settings'), startup_warning, parent=root)

    def restore_value(self,pair):
        self.variables[pair].set(self.base_values[pair]);self.refresh_page_preserving_scroll()

    def change_language(self,_=None):
        selected=next(k for k,v in i18n.LANGUAGES.items() if v==self.language_var.get())
        i18n.set_language(selected)
        global BASE_LABEL
        BASE_LABEL=tr('Default configuration (weapons.ini)')
        # Rebuild labels only; canonical document, unsaved StringVars, preset and
        # running session remain untouched. Tk callbacks are re-bound once.
        for child in self.root.winfo_children():child.destroy()
        self.cards=[];self.columns=0
        self.preset_var.set(self.current_name or BASE_LABEL)
        self.build_ui();self.refresh_presets();self.render_page();self.changed()
        self.game_status.set(tr('Game running') if self.session else tr('Game closed'))
        if self.session and hasattr(self,'apply_button'):self.apply_button.configure(state='normal' if not getattr(self.session,'live_pending',0) else 'disabled')
        self.status.set(tr('Ready.'))
        self.remember_settings()

    def configure_style(self):
        style = self.ttk.Style(self.root)
        if 'clam' in style.theme_names(): style.theme_use('clam')
        style.configure('.', font=('Segoe UI', 10), background='#f3f5f8', foreground='#202939')
        style.configure('Title.TLabel', font=('Segoe UI', 18, 'bold'))
        style.configure('Heading.TLabel', font=('Segoe UI', 12, 'bold'))
        style.configure('Small.TLabel', font=('Segoe UI', 9), foreground='#5e6b80')
        style.configure('Card.TFrame', background='#ffffff')
        style.configure('Card.TLabel', background='#ffffff')
        style.configure('CardSmall.TLabel', background='#ffffff', font=('Segoe UI', 8), foreground='#627088')
        style.configure('Play.TButton', font=('Segoe UI', 11, 'bold'), padding=(20, 10))

    def build_ui(self):
        tk, ttk = self.tk, self.ttk
        header = ttk.Frame(self.root, padding=(14, 10)); header.pack(fill='x')
        ttk.Label(header, text='ReRAC  /  WEAPON EVOLUTION', style='Title.TLabel').pack(side='left')
        ttk.Button(header, text=tr('Launch settings'), command=self.launch_options).pack(side='right')
        self.language_var = tk.StringVar(value=i18n.LANGUAGES[i18n.language()])
        language_box=ttk.Combobox(header,textvariable=self.language_var,values=list(i18n.LANGUAGES.values()),state='readonly',width=12)
        language_box.pack(side='right',padx=8);language_box.bind('<<ComboboxSelected>>',self.change_language)
        ttk.Label(header,text=tr('Language')).pack(side='right')
        controls = ttk.Frame(self.root, padding=(14, 0, 14, 8)); controls.pack(fill='x')
        ttk.Label(controls, text=tr('Preset')).pack(side='left', padx=(0, 7))
        self.preset_box = ttk.Combobox(controls, textvariable=self.preset_var, width=28, state='readonly')
        self.preset_box.pack(side='left'); self.preset_box.bind('<<ComboboxSelected>>', self.select_preset)
        for text, command in [(tr('Save'), self.save_preset), (tr('Save as…'), self.save_as),
                              (tr('Import INI'), self.import_ini), (tr('Export INI'), self.export_ini),
                              (tr('Remove'), self.remove_preset)]:
            ttk.Button(controls, text=text, command=command).pack(side='left', padx=(5, 0))
        body = ttk.Frame(self.root); body.pack(fill='both', expand=True)
        navigation = tk.Frame(body, bg='#161b26', width=205); navigation.pack(side='left', fill='y')
        navigation.pack_propagate(False)
        nav_canvas = tk.Canvas(navigation, bg='#161b26', highlightthickness=0, width=190)
        nav_scroll = ttk.Scrollbar(navigation, orient='vertical', command=nav_canvas.yview)
        nav_scroll.pack(side='right', fill='y'); nav_canvas.pack(side='left', fill='both', expand=True)
        nav_canvas.configure(yscrollcommand=nav_scroll.set)
        nav = tk.Frame(nav_canvas, bg='#161b26'); nav_window = nav_canvas.create_window(0, 0, window=nav, anchor='nw')
        nav.bind('<Configure>', lambda _: nav_canvas.configure(scrollregion=nav_canvas.bbox('all')))
        nav_canvas.bind('<Configure>', lambda e: nav_canvas.itemconfigure(nav_window, width=e.width))
        self.nav_buttons = {}
        for index, (key, label) in enumerate([('global', tr('Global Settings'))]+[(k, i18n.weapon_name(k)) for k in EDITOR_WEAPONS]):
            button = tk.Button(nav, text=label, anchor='w', relief='flat', bd=0,
                font=('Segoe UI', 10, 'bold'), wraplength=180, justify='left', bg='#242d40', fg='#e7edf8', activebackground='#3b4b6a',
                activeforeground='white', padx=14, pady=7, command=lambda p=key: self.select_page(p))
            button.pack(fill='x', padx=8, pady=(8 if index == 0 else 2, 2))
            self.nav_buttons[key] = button
        content = ttk.Frame(body, padding=(14, 0, 14, 4)); content.pack(side='left', fill='both', expand=True)
        top = ttk.Frame(content); top.pack(fill='x', pady=(0, 6))
        self.page_title = ttk.Label(top, style='Heading.TLabel'); self.page_title.pack(side='left')
        ttk.Entry(top, textvariable=self.search, width=24).pack(side='right')
        ttk.Label(top, text=tr('Find setting:')).pack(side='right', padx=6)
        ttk.Label(content, textvariable=self.document_info, style='Small.TLabel').pack(anchor='w', pady=(0, 5))
        frame = ttk.Frame(content); frame.pack(fill='both', expand=True)
        self.canvas = tk.Canvas(frame, highlightthickness=0, bg='#edf0f5')
        scroll = ttk.Scrollbar(frame, orient='vertical', command=self.canvas.yview)
        scroll.pack(side='right', fill='y'); self.canvas.pack(side='left', fill='both', expand=True)
        self.canvas.configure(yscrollcommand=scroll.set)
        self.form = ttk.Frame(self.canvas); self.form_window = self.canvas.create_window(0, 0, window=self.form, anchor='nw')
        self.form.bind('<Configure>', lambda _: self.canvas.configure(scrollregion=self.canvas.bbox('all')))
        self.canvas.bind('<Configure>', self.resize_content)
        self.bind_mousewheel()
        footer = ttk.Frame(self.root, padding=(14, 8)); footer.pack(fill='x')
        help_frame = ttk.Frame(footer); help_frame.pack(side='left', fill='both', expand=True)
        ttk.Label(help_frame, textvariable=self.context, style='Small.TLabel', wraplength=790, justify='left').pack(anchor='w')
        self.play_button = ttk.Button(footer, text=tr('Save and launch'), style='Play.TButton', command=self.play)
        self.play_button.pack(side='right', padx=(15, 0))
        self.apply_button=ttk.Button(footer,text=tr('Apply to running game'),command=self.apply_live,state='disabled')
        self.apply_button.pack(side='right',padx=5)
        ttk.Button(footer, text=tr('Open log'), command=self.open_log).pack(side='right', padx=5)
        bottom = ttk.Frame(self.root, padding=(14, 4, 14, 8)); bottom.pack(fill='x')
        ttk.Label(bottom, textvariable=self.status, wraplength=940).pack(side='left')
        ttk.Label(bottom, textvariable=self.game_status, style='Small.TLabel').pack(side='right')

    def resize_content(self, event):
        self.canvas.itemconfigure(self.form_window, width=event.width)
        columns = max(1, min(5, event.width//355))
        if columns != self.columns:
            self.columns = columns; self.arrange_cards()

    def arrange_cards(self):
        if not hasattr(self, 'card_area'): return
        for i in range(5): self.card_area.columnconfigure(i, weight=0, uniform='')
        columns = self.columns or 2
        for i in range(columns): self.card_area.columnconfigure(i, weight=1, uniform='grades')
        for i, card in enumerate(self.cards): card.grid(row=i//columns, column=i%columns, sticky='new', padx=5, pady=5)

    def bind_mousewheel(self):
        # Override the class wheel binding before Tk changes the selection.
        # Returning break also prevents the all-tag handler scrolling twice.
        for sequence in ('<MouseWheel>', '<Button-4>', '<Button-5>'):
            self.root.bind_all(sequence, self.mousewheel)
            for class_name in ('TCombobox', 'ComboboxListbox'):
                self.root.bind_class(class_name, sequence, self.dropdown_mousewheel)

    def mousewheel(self, event):
        # Only scroll when the pointer is over one of the two scrolling panes.
        widget = self.root.winfo_containing(event.x_root, event.y_root)
        if widget is None: return
        while widget is not None:
            if widget == self.form or widget == self.canvas:
                return self.scroll_page(event)
            widget = getattr(widget, 'master', None)

    def scroll_page(self, event):
        number = getattr(event, 'num', None)
        delta = getattr(event, 'delta', 0)
        if number in (4, 5):
            units = -1 if number == 4 else 1
        elif delta:
            units = -int(delta/120) if abs(delta) >= 120 else (-1 if delta > 0 else 1)
        else:
            return 'break'
        self.canvas.yview_scroll(units, 'units')
        return 'break'

    def dropdown_mousewheel(self, event):
        # Include language/preset dropdowns above the form and open popdowns.
        return self.scroll_page(event)

    def load_document(self, doc, name):
        from migrate_ini import convert
        doc=core.profile_document(convert(doc.raw)[0])
        self.loading = True; self.doc = doc; self.current_name = name
        if not name:
            path=self.kit/'weapons.ini'
            self.base_file_raw=path.read_bytes() if path.is_file() else doc.raw
        self.base_values = dict(doc.entries)
        self.variables = {pair: self.tk.StringVar(value=value) for pair, value in doc.entries.items()}
        for variable in self.variables.values(): variable.trace_add('write', self.changed)
        self.preset_var.set(name or BASE_LABEL); self.loading = False
        self.render_page(); self.changed()

    def dirty(self):
        return hasattr(self, 'variables') and any(v.get() != self.base_values[p] for p, v in self.variables.items())

    def changed(self, *_):
        if self.loading: return
        count = sum(v.get() != self.base_values[p] for p, v in self.variables.items())
        self.document_info.set(tr('{profile}  ·  {count} unsaved changes  ·  configuration values',profile=self.current_name or BASE_LABEL,count=count))
        self.root.title(('• ' if count else '')+'ReRAC · Weapon Evolution · '+tr('Launcher')+' '+core.VERSION)
        self.update_ammo_chance_summary()

    def checked_raw(self):
        if not self.doc: raise ValueError(tr('Load a valid weapons.ini before continuing.'))
        edits = {}
        for pair, variable in self.variables.items():
            if variable.get() != self.base_values[pair]: edits[pair] = core.validate_value(*pair, variable.get())
        raw = self.doc.render(edits); core.profile_document(raw)
        return raw

    def can_discard(self):
        if not self.dirty(): return True
        answer = self.messages.askyesnocancel(tr('Unsaved changes'), tr('Save changes before switching configurations?'), parent=self.root)
        if answer is None: return False
        return self.save_preset() if answer else True

    def refresh_presets(self):
        self.preset_box['values'] = [BASE_LABEL, *self.store.names()]

    def select_preset(self, _=None):
        selected = self.preset_var.get()
        if not self.can_discard(): self.preset_var.set(self.current_name or BASE_LABEL); return
        try:
            name = '' if selected == BASE_LABEL else selected
            doc = core.profile_document((self.kit/'weapons.ini').read_bytes()) if not name else self.store.read(name)
            self.load_document(doc, name); self.status.set(tr('Loaded: ')+(name or BASE_LABEL))
            self.remember_settings()
        except (ValueError, OSError, UnicodeError) as error:
            self.preset_var.set(self.current_name or BASE_LABEL); self.error(error)

    def save_preset(self, new=False):
        try:
            raw = self.checked_raw(); name = '' if new else self.current_name
            if not new and not self.current_name:
                path=core.save_base_profile(self.kit,raw,getattr(self,'base_file_raw',None))
                self.load_document(core.profile_document(path.read_bytes()),'')
                self.refresh_presets();self.remember_settings()
                self.status.set(tr('Default configuration saved: ')+str(path));return True
            if not name:
                name = self.dialogs.askstring(tr('Save'), tr('Preset name:'), initialvalue=self.current_name or tr('Custom'), parent=self.root)
                if name is None: return False
            name = core.preset_name(name)
            if name in set(i18n.TEXT['Default configuration (weapons.ini)'].values()): raise ValueError(tr('Choose a name other than the default configuration label.'))
            path = self.store.path(name)
            if path.exists() and name.casefold() != self.current_name.casefold():
                if not self.messages.askyesno(tr('Existing preset'), tr('Update {name}? The previous version will be backed up.',name=path.stem), parent=self.root): return False
            path = self.store.save(name, raw)
            self.load_document(core.profile_document(raw), path.stem); self.refresh_presets(); self.remember_settings()
            self.status.set(tr('Preset saved: ')+str(path)); return True
        except (ValueError, OSError, UnicodeError) as error: self.error(error); return False

    def save_as(self): self.save_preset(new=True)

    def remove_preset(self):
        if not self.current_name: self.status.set(tr('The default configuration cannot be removed.')); return
        if not self.can_discard(): return
        try:
            base = core.profile_document((self.kit/'weapons.ini').read_bytes())
            destination = self.store.remove(self.current_name)
            self.load_document(base, ''); self.refresh_presets(); self.remember_settings()
            self.status.set(tr('Preset moved to backup: ')+str(destination))
        except (ValueError, OSError, UnicodeError) as error: self.error(error)

    def import_ini(self):
        path = self.files.askopenfilename(title=tr('Import weapons.ini'), filetypes=[('INI', '*.ini'), (tr('All files'), '*.*')], parent=self.root)
        if not path: return
        try:
            from migrate_ini import convert
            converted, _ = convert(Path(path).read_bytes())
            imported = core.profile_document(converted)
            if not self.can_discard(): return
            name = self.dialogs.askstring(tr('Import as preset'), tr('New preset name:'), initialvalue=Path(path).stem, parent=self.root)
            if name is None: return
            name = core.preset_name(name)
            if name in set(i18n.TEXT['Default configuration (weapons.ini)'].values()): raise ValueError(tr('This name is reserved for the default configuration.'))
            if self.store.path(name).exists():
                if not self.messages.askyesno(tr('Existing preset'), tr('Update this preset? The previous version will be backed up.'), parent=self.root): return
            dest = self.store.save(name, imported.raw)
            self.load_document(imported, dest.stem); self.refresh_presets(); self.remember_settings()
            self.status.set(tr('INI imported: ')+str(dest))
        except (ValueError, OSError, UnicodeError) as error: self.error(error)

    def export_ini(self):
        try:
            raw = self.checked_raw()
            selected = self.files.asksaveasfilename(title=tr('Export preset INI'), initialfile=(self.current_name or 'weapons')+'.ini', defaultextension='.ini', filetypes=[('INI', '*.ini')], parent=self.root)
            if not selected: return
            path = Path(selected).resolve()
            if path == (self.kit/'weapons.ini').resolve(): raise ValueError(tr('Use Save to update weapons.ini, or choose another export filename.'))
            if path.exists(): core.atomic_write(self.kit/'backup/export'/core.stamp()/path.name, path.read_bytes())
            core.atomic_write(path, raw); self.status.set(tr('Exported: ')+str(path))
        except (ValueError, OSError, UnicodeError) as error: self.error(error)

    def select_page(self, page):
        self.page = page; self.search.set(''); self.render_page(); self.canvas.yview_moveto(0)

    def matches(self, section, key):
        search = self.search.get().strip().casefold()
        return not search or search in (key+' '+i18n.label(key)+' '+section).casefold()

    def card(self, parent, label, color='#526d99'):
        frame = self.ttk.Frame(parent, style='Card.TFrame', padding=(12, 10))
        self.tk.Frame(frame, bg=color, height=4).pack(fill='x', pady=(0, 8))
        self.ttk.Label(frame, text=label, style='Card.TLabel', font=('Segoe UI', 12, 'bold')).pack(anchor='w', pady=(0, 8))
        return frame

    def weapon_field_groups(self, keys):
        # Hide only the wrench's unsupported R1 selector in the UI. Keep its
        # saved value available to validation and profile migration.
        standard, experimental = field_groups(self.page, keys)
        if self.page == 'wrench':
            standard = [key for key in standard if key != 'r1_experimental_max']
            experimental = [key for key in experimental if key != 'r1_experimental_max']
        return standard, experimental

    def fields(self, card, section, keys):
        for key in keys:
            if key in NAMES or key in {'xp_bar_opacity_percent','damage_mode'} or not self.matches(section, key): continue
            weapon = section.split('.')[1] if section.startswith('weapon.') else ''
            selector = 'grenade_model' if weapon == 'visibomb' else 'flame_mine_model'
            selected = self.variables.get((section, selector))
            if not visible_model_field(weapon, key, selected.get() if selected else '0'): continue
            pair = section, key
            row = self.ttk.Frame(card, style='Card.TFrame'); row.pack(fill='x', pady=5)
            label = self.ttk.Label(row, text=self.randomizer_field_label(key), style='Card.TLabel', wraplength=185)
            label.grid(row=0, column=0, sticky='w', padx=(0, 8)); row.columnconfigure(0, weight=1)
            cell = self.ttk.Frame(row, style='Card.TFrame'); cell.grid(row=0, column=1, sticky='e')
            variable = self.variables[pair]
            if key in BOOLEANS or key in ('level_up_sound','sand_mouse_mode') or key in MODEL_OPTIONS or key=='r1_experimental_max':
                if key=='r1_experimental_max':
                    from schema import r1_mode_options
                    codes=r1_mode_options(section);options=codes;selected=variable.get()
                elif key in MODEL_OPTIONS:
                    codes=('0','1','2'); options=tuple(i18n.model_name(name) for name in MODEL_OPTIONS[key]); selected=options[int(float(variable.get()))]
                elif key in BOOLEANS:
                    codes=('false','true'); options=('Off','On'); selected=i18n.boolean_display(variable.get())
                elif key=='level_up_sound':
                    codes=('off','help_notice');options=('Off',tr('Notification sound'))
                    selected=options[0 if variable.get().lower() in ('off','false','0','no') else 1]
                else:
                    codes=('1','2','3');options=(tr('Companion only'),tr('Companion and drones'),tr('Companion with Sonic Summoner'))
                    selected=options[int(variable.get())-1]
                display=self.tk.StringVar(value=selected)
                widget=self.ttk.Combobox(cell,textvariable=display,values=options,state='readonly',width=10 if key in BOOLEANS else 20)
                widget.bind('<<ComboboxSelected>>',lambda _,v=variable,d=display,o=options,c=codes,k=key:self.select_option(v,c[o.index(d.get())],k))
            else:
                widget = self.ttk.Entry(cell, textvariable=variable, width=12, state='readonly' if key == 'id' else 'normal')
            widget.pack(side='left')
            if key=='r1_experimental_max':
                self.ttk.Label(row,text=tr('Highest R1 mode available at this grade; 0 hides the switch. Requires global R1_Experimental On.'),
                    style='CardSmall.TLabel',wraplength=290).grid(row=1,column=0,columnspan=2,sticky='w',pady=(3,0))
            if key in COLORS:
                swatch = self.tk.Button(cell, width=2, height=1, relief='flat', bd=1,
                    command=lambda p=pair: self.pick_color(p))
                swatch.pack(side='left', padx=(3, 0))
                def paint(*_, var=variable, button=swatch):
                    if not button.winfo_exists(): return
                    text = var.get().lstrip('#'); button.configure(bg='#'+text if core.COLOR.fullmatch(text) else '#b4bdcc')
                paint()
                # Widget-level binding avoids accumulating StringVar traces on each page switch.
                widget.bind('<KeyRelease>', paint)
                swatch.bind('<Enter>', paint)
            for target in (widget, label):
                target.bind('<FocusIn>', lambda _, s=section, k=key: self.show_help(s, k))
                target.bind('<Enter>', lambda _, s=section, k=key: self.show_help(s, k))
            widget.bind('<Button-3>', lambda e, p=pair: self.field_menu(e, p))

    def select_option(self, variable, value, key):
        variable.set(value)
        if key in MODEL_OPTIONS:
            self.refresh_page_preserving_scroll()

    def show_help(self, section, key):
        text = i18n.help_text(section, key)
        if key not in COLORS and key not in BOOLEANS and key not in NAMES and key not in {'id', 'level_up_sound','damage_mode'}:
            low, high = core.numeric_range(section, key)
            text += tr(' Range: {low} … {high}.',low=f'{low:g}',high=f'{high:g}')
        self.context.set(text)

    def pick_color(self, pair):
        variable = self.variables[pair]; value = variable.get().lstrip('#')
        chosen = self.colors.askcolor(color='#'+value if core.COLOR.fullmatch(value) else '#ffffff', title=i18n.label(pair[1]), parent=self.root)[1]
        if chosen: variable.set(chosen[1:].upper()); self.refresh_page_preserving_scroll()

    def field_menu(self, event, pair):
        menu = self.tk.Menu(self.root, tearoff=False)
        menu.add_command(label=tr('Restore saved value'), command=lambda: self.restore_value(pair))
        if core.LEVEL.fullmatch(pair[0]):
            menu.add_command(label=tr('Copy this value to the other grades'), command=lambda: self.copy_to_grades(pair))
        if pair[1] in COLORS: menu.add_command(label=tr('Original color (default)'), command=lambda: self.reset_color(pair))
        menu.tk_popup(event.x_root, event.y_root)

    def reset_color(self, pair): self.variables[pair].set('default'); self.refresh_page_preserving_scroll()

    def copy_to_grades(self, pair):
        try:
            value = core.validate_value(*pair, self.variables[pair].get())
            prefix = pair[0].rsplit('.level', 1)[0]
            for n in range(1, 9):
                target = f'{prefix}.level{n}', pair[1]
                if target in self.variables: self.variables[target].set(core.validate_value(*target, value))
            self.refresh_page_preserving_scroll(); self.status.set(tr('Value copied to all eight grades: ')+i18n.label(pair[1]))
        except ValueError as error: self.error(error)

    def global_fields(self, parent, section, keys):
        for key in keys:
            if (section,key) not in self.variables or not self.matches(section,key): continue
            self.ttk.Separator(parent, orient='horizontal').pack(fill='x', pady=4)
            self.fields(parent, section, [key])

    def refresh_page_preserving_scroll(self):
        if not hasattr(self, 'canvas'):
            self.render_page()
            return
        position = self.canvas.yview()[0]
        self.render_page()
        self.root.update_idletasks()
        self.canvas.configure(scrollregion=self.canvas.bbox('all'))
        self.canvas.yview_moveto(position)

    def render_page(self):
        if not hasattr(self, 'form'): return
        for child in self.form.winfo_children(): child.destroy()
        self.cards = []; self.card_area = self.ttk.Frame(self.form)
        for key, button in self.nav_buttons.items(): button.configure(bg='#405784' if key == self.page else '#242d40')
        self.page_title.configure(text=tr('Global Settings') if self.page == 'global' else i18n.weapon_name(self.page))
        if not getattr(self, 'doc', None):
            self.ttk.Label(self.form, text=tr('Load a valid weapons.ini to get started.'), padding=20).pack(); return
        if self.page == 'global':
            base_fields, mode_fields = global_field_groups(self.doc)
            base_keys = ['xp_per_damage', 'dpad_quick_select', 'r1_experimental',
                         'refill_ammo_on_upgrade', 'hide_ryno']
            random_keys = ['randomizer_mode', 'randomizer_bolt_multiplier',
                           'golden_bolts_to_weapons', 'golden_bolts_early_bias_percent',
                           'golden_bolts_one_weapon_per_planet']
            difficulty_keys = ['bolt_multiplier_max', *WORLD_DEFAULTS]
            def group(label, pairs):
                card = self.card(self.form, tr(label))
                card.pack(fill='x', padx=5, pady=6)
                shown = False
                for section, key in pairs:
                    if (section, key) not in self.variables or not self.matches(section, key): continue
                    if shown: self.ttk.Separator(card, orient='horizontal').pack(fill='x', pady=4)
                    self.fields(card, section, [key])
                    shown = True
                return card
            group('Base settings', [('general', key) for key in base_keys])
            excluded = set(base_keys + random_keys + difficulty_keys)
            hud_keys = ['name_opacity_percent','weapon_bar_alternative_position',
                        'weapon_bar_behind_hud','nanotech_xp_bar_opacity_percent',
                        'xp_bar_height_px','xp_bar_horizontal_inset_px',
                        'show_for_weapons_without_ammo','level_up_popup',
                        'level_up_popup_duration_seconds','level_up_sound']
            hud_pairs = [('hud',key) for key in hud_keys]
            hud_pairs += [(section,key) for section,key in base_fields
                          if (section,key) not in hud_pairs and
                          (section != 'general' or key not in excluded)]
            group('XP Bars and settings', hud_pairs)
            group('Encounter difficulty and Nanotech', [('general',key) for key in difficulty_keys])
            self.render_randomizer_group()
            mode = self.card(self.form, tr('Ammo-Survival Mode'))
            mode.pack(fill='x', padx=5, pady=6)
            self.global_fields(mode, 'general', ['experimental_ammo_unlock_mode'])
            self.ttk.Label(mode, text=tr('New games start with zero Bomb Glove ammo. Existing save ammo is preserved.'),
                           style='CardSmall.TLabel', wraplength=700).pack(anchor='w', pady=(0,8))
            self.ttk.Label(mode, text=tr('Permanent weapon unlocks (collected normal bolts)'), style='Card.TLabel').pack(anchor='w')
            self.global_fields(mode, 'general', [key for _,key in mode_fields if key.startswith('ammo_unlock_')])
            self.ttk.Separator(mode, orient='horizontal').pack(fill='x', pady=(12,6))
            self.ttk.Label(mode, text=tr('Ammo crate pack probabilities'), style='Card.TLabel').pack(anchor='w')
            self.ttk.Label(mode, text=tr('Inputs are normalized to 100%. Set 0 to exclude an ammo type.'),
                           style='CardSmall.TLabel').pack(anchor='w')
            self.global_fields(mode, 'general', [key for _,key in mode_fields if key in AMMO_DROP_DEFAULTS])
            self.ammo_chance_summary = self.tk.StringVar()
            self.ttk.Label(mode, textvariable=self.ammo_chance_summary, style='CardSmall.TLabel',
                           wraplength=700).pack(anchor='w', pady=(8,0))
            self.update_ammo_chance_summary()

        else:
            section = f'weapon.{self.page}'
            profile = self.card(self.form, tr('XP progression'), '#597c9f'); profile.pack(fill='x', padx=5, pady=6)
            if section in self.doc.sections:
                self.ttk.Label(profile, text=tr('Game ID: ')+self.doc.entries.get((section, 'id'), '?')+tr('  ·  XP required for each upgrade'), style='CardSmall.TLabel').pack(anchor='w', pady=(0, 7))
                strip = self.ttk.Frame(profile, style='Card.TFrame'); strip.pack(fill='x')
                for i, key in enumerate(XP):
                    if key=='xp_v5_to_v6' and self.page!='wrench':
                        cell=self.ttk.Frame(strip,style='Card.TFrame');cell.grid(row=i//4,column=i%4,sticky='w',padx=(0,20))
                        self.ttk.Label(cell,text='V5 → V6',style='Card.TLabel').pack(anchor='w')
                        self.ttk.Label(cell,text=tr('Gold purchase (NG+)'),style='CardSmall.TLabel').pack(anchor='w')
                        continue
                    if (section, key) not in self.variables or not self.matches(section, key): continue
                    cell = self.ttk.Frame(strip, style='Card.TFrame'); cell.grid(row=i//4, column=i%4, sticky='w', padx=(0, 20))
                    self.ttk.Label(cell, text=f'V{i+1} → V{i+2}', style='Card.TLabel').pack(anchor='w')
                    entry = self.ttk.Entry(cell, textvariable=self.variables[(section, key)], width=12); entry.pack(pady=3)
                    entry.bind('<FocusIn>', lambda _, s=section, k=key: self.show_help(s, k))
                    entry.bind('<Enter>', lambda _, s=section, k=key: self.show_help(s, k))
            self.card_area.pack(fill='x')
            for n in range(1, 9):
                card = self.card(self.card_area, tr('V{grade}  /  Grade {grade}',grade=n), GRADE_COLORS[n-1]); self.cards.append(card)
                level = section+f'.level{n}'; keys = self.doc.keys(level)
                standard, experimental = self.weapon_field_groups(keys)
                standard_frame=self.ttk.LabelFrame(card,text=tr('Standard settings'),padding=8)
                standard_frame.pack(fill='x',pady=(0,8))
                self.fields(standard_frame, level, standard)
                if experimental:
                    experimental_frame=self.ttk.LabelFrame(card,text=tr('Experimental settings'),padding=8)
                    experimental_frame.pack(fill='x',pady=(0,8))
                    self.fields(experimental_frame, level, experimental)
                if not keys:
                    self.ttk.Label(card, text=tr('No settings for this grade in the INI.'), style='CardSmall.TLabel', wraplength=230).pack(anchor='w')
            self.arrange_cards()
        self.context.set(tr('All eight grades are on this page. Scroll to see them all. Right-click a value to copy it to the other grades.') if self.page != 'global' else tr('Global XP, HUD and level-up settings. In-game weapon names follow the game language.'))

    def update_ammo_chance_summary(self):
        if not hasattr(self, 'ammo_chance_summary'): return
        try:
            chances = normalized_ammo_chances({key: self.variables['general',key].get() for key in AMMO_DROP_DEFAULTS})
            text = tr('Actual chance per pack: ')+ ' · '.join(
                i18n.weapon_name(key.removeprefix('ammo_drop_chance_'))+f' {value:.2f}%'
                for key,value in chances.items())
        except (ValueError, KeyError): text = tr('Enter valid probabilities; at least one ammo type must be above zero.')
        self.ammo_chance_summary.set(text)

    def remember_settings(self):
        settings = {'last_preset': self.current_name, 'language':i18n.language()}
        for key, variable in [('exe', self.exe_var), ('data', self.data_var), ('save', self.save_var)]:
            path = core.resolve_path(variable.get(), self.kit)
            settings[key] = core.portable_path(path, self.kit) if path else ''
        core.save_settings(self.kit, settings); self.settings = settings

    def launch_options(self):
        window = self.tk.Toplevel(self.root); window.title(tr('Launch settings')); window.transient(self.root)
        frame = self.ttk.Frame(window, padding=18); frame.pack(fill='both', expand=True)
        options = [(tr('Extracted game data'), self.data_var, False), (tr('Save folder (leave empty to use the default)'), self.save_var, False)]
        options.insert(0,(tr('Modded ReRAC executable'), self.exe_var, True))
        for row, (label, variable, is_file) in enumerate(options):
            self.ttk.Label(frame, text=label).grid(row=row*2, column=0, columnspan=2, sticky='w', pady=(8, 2))
            self.ttk.Entry(frame, textvariable=variable, width=75, state='readonly' if is_file and getattr(sys,'frozen',False) else 'normal').grid(row=row*2+1, column=0, sticky='ew')
            self.ttk.Button(frame, text=tr('Browse…'), command=lambda v=variable, f=is_file: self.choose_path(v, f, window), state='disabled' if is_file and getattr(sys,'frozen',False) else 'normal').grid(row=row*2+1, column=1, padx=(6, 0))
        self.ttk.Label(frame, text=tr('Choose the folder containing toc.bin and extract-info.json.\nThe included modded game is selected automatically.\nLeave the save folder empty to keep using your existing saves.'), wraplength=650).grid(row=6, column=0, columnspan=2, sticky='w', pady=14)
        def save():
            try: self.remember_settings(); window.destroy()
            except (OSError, ValueError) as error: self.error(error)
        self.ttk.Button(frame, text=tr('Save settings'), command=save).grid(row=7, column=1, sticky='e')
        window.grab_set()

    def choose_path(self, variable, is_file, parent):
        selected = self.files.askopenfilename(title=tr('Choose rerac.exe'), filetypes=[(tr('Executable'), '*.exe'), (tr('All files'), '*.*')], parent=parent) if is_file else self.files.askdirectory(parent=parent)
        if selected: variable.set(selected)

    def prepare_launch_raw(self):
        if self.dirty():
            if not self.save_preset():return None
        elif not self.current_name:
            # Manual changes made while this window was open are also picked up.
            self.load_document(core.profile_document((self.kit/'weapons.ini').read_bytes()),'')
        return self.checked_raw()

    def play(self):
        if self.launch_busy or (self.session and self.session.process.poll() is None): return
        try:
            raw = self.prepare_launch_raw()
            if raw is None:return
            exe = core.resolve_path(self.exe_var.get(), self.kit)
            data = core.resolve_path(self.data_var.get(), self.kit)
            save = core.resolve_path(self.save_var.get(), self.kit)
            if exe is None or data is None:
                self.status.set(tr('Choose the game data folder in Launch settings.')); self.launch_options(); return
            self.remember_settings()
            self.launch_busy = True; self.play_button.configure(state='disabled'); self.status.set(tr('Checking the game and configuration…'))
            preset = self.current_name or BASE_LABEL
            def worker():
                try: self.events.put(('started', core.launch_game(self.kit, raw, preset, exe, data, save)))
                except Exception as error: self.events.put(('error', str(error)))
            threading.Thread(target=worker, daemon=True).start()
        except (OSError, ValueError, UnicodeError) as error: self.error(error)

    def poll(self):
        try:
            while True:
                kind, value = self.events.get_nowait()
                self.launch_busy = False
                if kind == 'started':
                    self.session = value; self.log_offset = 0; self.log_text = ''; self.confirmed = False
                    self.status.set(tr('Started: ')+value.preset+tr(' · waiting for the game to confirm the configuration.'))
                    self.game_status.set(tr('Game running'))
                    self.apply_button.configure(state='normal')
                else:
                    self.play_button.configure(state='normal'); self.error(value)
        except queue.Empty: pass
        if self.session:
            try:
                reply=core.live_reply(self.session)
                if reply is not None:
                    self.apply_button.configure(state='normal')
                    if reply[0]: self.status.set(tr('Live settings applied. Restart required for: ')+getattr(self,'live_deferred','—'))
                    else: self.error(reply[1])
                with self.session.log.open('rb') as stream:
                    stream.seek(self.log_offset); block = stream.read(128*1024); self.log_offset = stream.tell()
                self.log_text = (self.log_text+block.decode('utf-8', errors='replace'))[-512*1024:]
                if not self.confirmed and core.log_confirmation(self.log_text, self.session.ini):
                    self.confirmed = True; self.status.set(tr('Configuration confirmed by the game: ')+self.session.preset)
                code = self.session.process.poll()
                if code is not None:
                    # Drain remaining data on subsequent ticks before reporting exit.
                    if len(block) < 128*1024:
                        self.status.set(tr('Game closed (exit code {code}). ',code=code)+(tr('Configuration confirmed.') if self.confirmed else tr('Configuration loading has not been confirmed. Open the log.')))
                        self.game_status.set(tr('Game closed')); self.play_button.configure(state='normal')
                        self.last_log = self.session.log; self.session = None
                        self.apply_button.configure(state='disabled')
            except OSError as error: self.status.set(tr('Reading log: ')+str(error))
        self.root.after(250, self.poll)

    def apply_live(self):
        if not self.session:return
        try:
            raw=self.prepare_launch_raw()
            if raw is None:return
            fields=core.apply_live(self.session,raw)
            self.live_deferred=', '.join(i18n.label(key) for key in fields) or '—'
            self.apply_button.configure(state='disabled')
            self.status.set(tr('Waiting for live configuration acknowledgement…'))
        except (OSError,ValueError,UnicodeError) as error:self.error(error)

    def open_log(self):
        path = self.session.log if self.session else getattr(self, 'last_log', None)
        if not path or not path.exists(): self.status.set(tr('No log available. Start the game from the launcher.')); return
        try:
            if os.name == 'nt': os.startfile(str(path))
            else: subprocess.Popen(['xdg-open', str(path)])
        except OSError as error: self.error(error)

    def error(self, error):
        self.messages.showerror('Weapon Evolution', tr('Could not complete the operation.')+'\n\n'+tr('Technical details:')+'\n'+str(error), parent=self.root)

    def close(self):
        if self.launch_busy:
            self.status.set(tr('Wait for startup to finish before closing the launcher.')); return
        if not self.can_discard(): return
        try: self.remember_settings()
        except OSError as error: self.error(error); return
        self.root.destroy()

    def randomizer_field_label(self, key):
        prefix = {'randomizer_bolt_multiplier': 'All: ',
                  'golden_bolts_early_bias_percent': 'Weapons: ',
                  'golden_bolts_one_weapon_per_planet': 'Weapons: '}.get(key, '')
        return prefix + i18n.label(key)

    def current_randomizer_choice(self):
        def on(key):
            variable = self.variables.get(('general', key))
            return variable is not None and variable.get().strip().lower() in ('true', '1', 'yes', 'on')
        if on('randomizer_mode'): return 'All'
        if on('golden_bolts_to_weapons'): return 'Weapons'
        return 'Off'

    def select_randomizer_mode(self, choice):
        values = {'Off': ('false', 'false'), 'All': ('true', 'false'),
                  'Weapons': ('false', 'true')}
        if choice not in values: raise ValueError('Unknown randomizer mode')
        for key, value in zip(('randomizer_mode', 'golden_bolts_to_weapons'), values[choice]):
            self.variables['general', key].set(value)
        self.refresh_page_preserving_scroll()

    def render_randomizer_group(self):
        card = self.card(self.form, tr('Randomizer'))
        card.pack(fill='x', padx=5, pady=6)
        row = self.ttk.Frame(card, style='Card.TFrame')
        row.pack(fill='x', pady=5)
        self.ttk.Label(row, text=tr('Randomizer mode'), style='Card.TLabel').pack(side='left')
        self.randomizer_mode_choice = self.tk.StringVar(value=self.current_randomizer_choice())
        selector = self.ttk.Combobox(row, textvariable=self.randomizer_mode_choice,
                                     values=('Off', 'All', 'Weapons'), state='readonly', width=14)
        selector.pack(side='right')
        selector.bind('<<ComboboxSelected>>', lambda _: self.select_randomizer_mode(self.randomizer_mode_choice.get()))
        for text in (RANDOMIZER_ALL_DESCRIPTION, RANDOMIZER_WEAPONS_DESCRIPTION,
                     RANDOMIZER_RESTART_DESCRIPTION):
            self.ttk.Label(card, text=tr(text), style='CardSmall.TLabel',
                           wraplength=720).pack(anchor='w', fill='x', pady=(4, 6))
        for key in ('randomizer_bolt_multiplier', 'golden_bolts_early_bias_percent',
                    'golden_bolts_one_weapon_per_planet'):
            if ('general', key) in self.variables and self.matches('general', key):
                self.ttk.Separator(card, orient='horizontal').pack(fill='x', pady=5)
                self.fields(card, 'general', [key])





if __name__ == '__main__': raise SystemExit(main())
