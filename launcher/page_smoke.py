"""Exercise real preloaded Tk pages before accepting a standalone launcher EXE."""
from pathlib import Path
import json
import sys
import tempfile
import time


def descendants(widget):
    for child in widget.winfo_children():
        yield child
        yield from descendants(child)


def resource_counts():
    if sys.platform != 'win32':
        return {}
    import ctypes
    from ctypes import wintypes
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    user = ctypes.WinDLL('user32', use_last_error=True)
    kernel.GetCurrentProcess.restype = wintypes.HANDLE
    user.GetGuiResources.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    user.GetGuiResources.restype = wintypes.DWORD
    handle = kernel.GetCurrentProcess()
    counts = {'gdi_objects': user.GetGuiResources(handle, 0),
              'user_objects': user.GetGuiResources(handle, 1)}
    # Leave headroom for dropdown popups, dialogs and the user's desktop setup.
    assert all(0 < count < 8500 for count in counts.values()), counts
    return counts


def verify(root, raw, *, languages=True, progress=None):
    import tkinter as tk
    from tkinter import ttk, filedialog, messagebox, simpledialog, colorchooser
    import launcher
    import launcher_core as core
    import i18n
    from schema import BOOLEANS, MODEL_COLOR_GROUPS, EDITOR_WEAPONS, NAMES

    progress = progress or (lambda stage: None)
    progress('creating all 17 complete editor pages')
    # Closing the verification window is not a test failure. Input is not needed.
    root.protocol('WM_DELETE_WINDOW', lambda: None)
    callbacks = []
    old_error_handler = root.report_callback_exception
    root.report_callback_exception = lambda *error: callbacks.append(str(error))
    previous_language = i18n.language()
    try:
        with tempfile.TemporaryDirectory(prefix='rerac_preloaded_ui_') as temporary:
            kit = Path(temporary)
            (kit/'weapons.ini').write_bytes(raw)
            start = time.perf_counter()
            app = launcher.Launcher(root, kit, tk, ttk, filedialog, messagebox,
                simpledialog, colorchooser, test_mode=True)
            root.protocol('WM_DELETE_WINDOW', lambda: None)
            root.deiconify()
            root.update()
            startup = time.perf_counter()-start
            expected = ['global', *EDITOR_WEAPONS]
            assert list(app._page_views) == expected
            identities = {key: (str(view.canvas), tuple(str(row.canvas) for row in view.rows))
                          for key, view in app._page_views.items()}
            for weapon,keys in [('pyrocitor',{'lava_damage_bonus_percent'}),
                                ('mine_glove',{'flame_mine_max_active_mines','flame_mine_lifetime_seconds'})]:
                pairs={row.pair for row in app._page_views[weapon].rows}
                for grade in range(1,9):
                    assert all((f'weapon.{weapon}.level{grade}',key) in pairs for key in keys)
            widget_count = len(list(descendants(root)))
            assert widget_count < 8000, widget_count
            maximum_resources = resource_counts()
            columns = set()
            switch_times = []
            for width, height in [(1450, 920), (930, 720), (1100, 800)]:
                root.geometry(f'{width}x{height}')
                root.update()
                for key in expected:
                    progress(f'checking {key} at {width}x{height}')
                    switch_start = time.perf_counter()
                    app.select_page(key)
                    root.update()
                    switch_times.append(time.perf_counter()-switch_start)
                    view = app._page_views[key]
                    assert view.frame.winfo_ismapped() and view.canvas.winfo_width() > 200
                    assert view.form.winfo_width() == view.canvas.winfo_width()
                    assert len(view.cards) == (0 if key == 'global' else 8)
                    assert identities[key] == (str(view.canvas), tuple(str(row.canvas) for row in view.rows))
                    for position in [0., 0.5, 1.]:
                        view.canvas.yview_moveto(position)
                        root.update()
                        assert all(row.canvas.winfo_exists() for row in view.rows)
                    if key != 'global':
                        expected_pairs=set()
                        for grade in range(1,9):
                            section=f'weapon.{key}.level{grade}'
                            standard,experimental=app.weapon_field_groups(app.doc.keys(section))
                            expected_pairs.update((section,field) for field in standard+experimental
                                if field not in NAMES and field not in {'xp_bar_opacity_percent','damage_mode'})
                        assert {row.pair for row in view.rows}==expected_pairs, key
                    for row in view.rows:
                        if row.visible:
                            bounds=row.canvas.bbox(row.label)
                            assert bounds[2] <= row.canvas.bbox(row.window)[0]-3, (key,width,row.pair,bounds,row.canvas.bbox(row.window))
                    columns.add(view.columns)
            assert len(list(descendants(root))) == widget_count

            progress('checking palettes, scroll positions and selection controls')
            # A model change reveals existing color rows and keeps hidden values.
            for weapon, selector in [('visibomb', 'grenade_model'), ('mine_glove', 'flame_mine_model')]:
                app.select_page(weapon)
                view = app._page_views[weapon]
                section = f'weapon.{weapon}.level1'
                for model, palette in enumerate(MODEL_COLOR_GROUPS[weapon]):
                    app.select_option(app.variables[section, selector], str(model), selector)
                    root.update()
                    palette_rows = {row.pair[1]: row for row in view.rows if row.pair[0] == section}
                    for group in MODEL_COLOR_GROUPS[weapon]:
                        for key in group:
                            assert palette_rows[key].visible == (key in palette)
                assert identities[weapon][1] == tuple(str(row.canvas) for row in view.rows)

            app.select_page('blaster')
            view = app._page_views['blaster']
            app.canvas.yview_moveto(0.4)
            root.update()
            position = app.canvas.yview()[0]
            pair = ('weapon.blaster.level8', 'projectile_color')
            app.variables[pair].set('12ABEF')
            app.refresh_page_preserving_scroll()
            assert abs(app.canvas.yview()[0]-position) < 0.002
            app.copy_to_grades(pair)
            assert all(app.variables[f'weapon.blaster.level{grade}',pair[1]].get() == '12ABEF'
                       for grade in range(1, 9))
            for row in view.rows:
                if row.pair[1] == 'projectile_color':
                    assert row.canvas.itemcget(row.swatch, 'fill').lower() == '#12abef'
            toggle = next(row for row in view.rows if row.pair[1] in BOOLEANS)
            value = app.variables[toggle.pair].get()
            toggle.editor.event_generate('<MouseWheel>', delta=-120)
            root.update()
            assert app.variables[toggle.pair].get() == value

            app.search.set('damage')
            root.update()
            assert any(row.visible for row in view.rows)
            assert all(not row.visible or 'damage' in (row.pair[1]+' '+i18n.label(row.pair[1])).lower()
                       for row in view.rows)
            app.search.set('')
            root.update()
            assert identities['blaster'][1] == tuple(str(row.canvas) for row in view.rows)

            progress('checking wheel and scrollbar repaint transactions')
            # Exercise the real scrollbar callback, rapid reversals and the
            # same bounded transaction used by wheel events. Navigation must
            # flush the old page, never apply its movement to the new page.
            for fraction in [0.1,0.8,0.3,0.7,0.2,0.6]:
                root.tk.call(view.scrollbar.cget('command'),'moveto',fraction)
            view.scrolling.flush()
            root.update()
            assert abs(view.canvas.yview()[0]-0.6)<0.002
            for _ in range(12):
                view.scroll('scroll',1,'units')
                view.scroll('scroll',-1,'units')
            view.scrolling.flush()
            root.update()
            view.scroll('moveto',0.35)
            app.select_page('global')
            root.update()
            assert not view.scrolling.pending and view.scrolling.job is None
            global_position=app.canvas.yview()
            root.update()
            assert app.canvas.yview()==global_position
            app.select_page('blaster')
            view=app._page_views['blaster']
            view.scroll('moveto',0.45)
            timer=tk.BooleanVar(master=root,value=False)
            root.after(50,lambda: timer.set(True))
            root.wait_variable(timer)
            assert view.scrolling.job is None and not view.scrolling.pending
            assert abs(view.canvas.yview()[0]-0.45)<0.002
            assert identities['blaster'][1]==tuple(str(row.canvas) for row in view.rows)
            assert len(list(descendants(root)))==widget_count
            variables = app.variables
            if languages:
                for language in i18n.LANGUAGES:
                    progress('checking language '+language)
                    app.language_var.set(i18n.LANGUAGES[language])
                    app.change_language()
                    root.update()
                    assert app.variables is variables and app.variables[pair].get() == '12ABEF'
                    assert len(list(descendants(root))) < 8000
                    counts = resource_counts()
                    for name, count in counts.items():
                        maximum_resources[name] = max(maximum_resources.get(name, 0), count)
                    for key in expected:
                        app.select_page(key)
                        root.update_idletasks()
                        assert app._page_views[key].rows and app.canvas.winfo_width() > 200
            progress('checking preset reload and retained page identities')
            # Same-schema preset/save reloads reuse the controls, including combos.
            before = {key: str(view.canvas) for key, view in app._page_views.items()}
            replacement = core.profile_document(raw)
            app.load_document(replacement, 'Other preset')
            root.update()
            assert app.variables is variables
            assert before == {key: str(view.canvas) for key, view in app._page_views.items()}
            assert app.checked_raw() == replacement.raw
            assert not callbacks, callbacks
            return {'preloaded_pages_ui': True, 'scroll_repaint_ui': True, 'experimental_overrides_ui': True, 'page_count': len(expected),
                    'widget_count': widget_count, 'startup_seconds': round(startup, 3),
                    'page_switch_max_ms': round(max(switch_times)*1000, 1),
                    'page_switch_mean_ms': round(sum(switch_times)/len(switch_times)*1000, 1),
                    'tested_columns': sorted(columns), **maximum_resources}
    finally:
        root.report_callback_exception = old_error_handler
        i18n.set_language(previous_language)


def main():
    import argparse
    import tkinter as tk
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', type=Path, default=Path(__file__).with_name('weapons.default.ini'))
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    root = tk.Tk()
    root.withdraw()
    try:
        result = verify(root, args.profile.read_bytes())
        text = json.dumps(result, indent=2)+'\n'
        if args.report:
            args.report.write_text(text, encoding='utf-8')
        print(text)
    finally:
        root.destroy()


if __name__ == '__main__':
    main()
