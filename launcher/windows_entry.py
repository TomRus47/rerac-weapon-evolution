"""PyInstaller entry point. Player data always lives beside the executable."""
from pathlib import Path
import json
import os
import sys
import traceback

def self_test(destination):
    def progress(stage):
        path = Path(destination).with_suffix('.progress.json')
        path.parent.mkdir(parents=True, exist_ok=True)
        temporary = path.with_suffix('.tmp')
        temporary.write_text(json.dumps({'stage': stage}), encoding='utf-8')
        os.replace(temporary, path)
    progress('loading Python/Tk resources')
    import tkinter as tk
    from tkinter import ttk, filedialog, messagebox, simpledialog, colorchooser
    import launcher_core as core
    import migrate_ini
    import schema
    root = tk.Tk()
    try:
        root.withdraw()
        ttk.Label(root, text='ReRAC').pack()
        root.update_idletasks()
        result = {'frozen': bool(getattr(sys,'frozen',False)), 'launcher_version': core.VERSION,
                  'application_folder': str(core.KIT), 'gui_ok': True,
                  'saved_controls_ui': all(key in schema.BOOLEANS and key in schema.GLOBAL_STANDARD_DEFAULTS for key in ('dpad_quick_select','r1_experimental')),
                  'wrench_trail_ui': 'trail_color' in schema.EXTRA[8].split(),
                  'ngplus_difficulty_ui': list(schema.WORLD_DEFAULTS)[0]=='ngplus_multiplier',
                  'eight_grade_ui': len(schema.XP)==7 and 'hide_ryno' in schema.GLOBAL_STANDARD_DEFAULTS,
                  'additive_nanotech_ui': core.numeric_range('general','nanotech_max_health') == (0,64),
                  'nanotech_xp_bar_ui': ('nanotech_xp_bar_opacity_percent' in schema.HUD
                      and core.numeric_range('hud','nanotech_xp_bar_opacity_percent') == (0,100)),
                  'location_test_ui': core.game_environment(Path('weapons.ini'),Path('.'),env={'RC_WEAPON_LOCATION_TEST':'1'}).get('RC_LEVEL') == '18',
                  'pursuit_ui': 'enemy_chase_range_multiplier' in schema.WORLD_DEFAULTS and core.numeric_range('general','enemy_chase_range_multiplier') == (0.1,10),
                  'encounter_motion_ui': all(key in schema.WORLD_DEFAULTS for key in schema.MOTION_DEFAULTS) and 'golden_bolts_one_weapon_per_planet' in schema.BOOLEANS,
                  'tcl_version': root.tk.call('info','patchlevel'), 'tk_version': str(tk.TkVersion)}
        # Exercise the actual frozen launcher field renderer, not just its schema.
        import launcher as editor
        import i18n
        key = 'move_while_thrown'
        test_keys=(key,'burn_on_hit','hud_quickselect_only','slam_explosion_enabled')
        def descendants(widget):
            for child in widget.winfo_children():
                yield child
                yield from descendants(child)
        for grade in range(1, 9):
            section = f'weapon.wrench.level{grade}'
            frame = ttk.Frame(root)
            app = editor.Launcher.__new__(editor.Launcher)
            app.page = 'wrench'
            standard, experimental = app.weapon_field_groups([*test_keys, 'r1_experimental_max'])
            assert key in standard and key not in experimental
            assert 'r1_experimental_max' not in standard + experimental
            app.root, app.tk, app.ttk = root, tk, ttk
            app.variables = {(section, field): tk.StringVar(master=root, value='false') for field in test_keys}
            app.matches = lambda *_: True
            app.randomizer_field_label = lambda name: i18n.label(name)
            app.show_help = lambda *_: None
            app.fields(frame, section, standard)
            app.fields(frame, section, experimental)
            root.update_idletasks()
            controls = [w for w in descendants(frame) if isinstance(w, ttk.Combobox)]
            assert len(controls) == len(test_keys) and all(tuple(c['values'])==('Off','On') for c in controls)
            for control in controls:
                control.set('On'); control.event_generate('<<ComboboxSelected>>'); root.update()
            assert all(v.get()=='true' for v in app.variables.values())
            for control in controls:
                control.set('Off'); control.event_generate('<<ComboboxSelected>>'); root.update()
            assert all(v.get()=='false' for v in app.variables.values())
            frame.destroy()
        result['feedback_controls_ui'] = True
        result['moving_wrench_gui_grades'] = 8
        result['moving_wrench_gui_options'] = ['Off', 'On']
        progress('checking all preloaded pages')
        from page_smoke import verify
        profile=Path(getattr(sys,'_MEIPASS',Path(__file__).resolve().parent))/'weapons.default.ini'
        result.update(verify(root,profile.read_bytes(),progress=progress))
        result['ok'] = True

    finally:
        root.destroy()
    core.atomic_write(Path(destination), (json.dumps(result,indent=2)+'\n').encode('utf-8'))
    progress('all preloaded-page checks passed')
    return 0

def entry():
    # Windowed executables have no console streams; library notices remain harmless.
    for name in ('stdout','stderr'):
        if getattr(sys,name) is None: setattr(sys,name,open(os.devnull,'w',encoding='utf-8'))
    if len(sys.argv)==3 and sys.argv[1]=='--self-test': return self_test(sys.argv[2])
    import launcher
    return launcher.main()

def main():
    try:
        return entry()
    except Exception as error:
        message = traceback.format_exc()
        if len(sys.argv) == 3 and sys.argv[1] == '--self-test':
            # A failed build must never wait for a hidden modal dialog. Reports
            # live outside the disposable EXE directory and survive cleanup.
            destination = Path(sys.argv[2])
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(json.dumps({'ok': False, 'error': str(error),
                'traceback': message}, indent=2)+'\n', encoding='utf-8')
            destination.with_suffix('.error.log').write_text(message, encoding='utf-8')
            return 2
        from frozen_support import application_folder
        folder = application_folder(__file__)
        try: (folder/'launcher-error.log').write_text(message,encoding='utf-8')
        except OSError: pass
        if sys.platform=='win32':
            import ctypes
            ctypes.windll.user32.MessageBoxW(None,'Unable to open the launcher.\nSee launcher-error.log beside the executable for details.','ReRAC Weapon Evolution',0x10)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
