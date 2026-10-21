# Launcher 0.17.1: restore the original page lifecycle

**v58.2 updater correction:** source/launcher directories are now supported without weapons.ini or a runtime. Opening the editor skips the runtime check; Save and launch still validates the selected game executable. The launcher is now 0.17.2.

This local launcher correction removes the v58 page cache, offscreen page
construction, background warmup, deferred wheel scrolling and Windows
WS_EX_COMPOSITED painting. The user reported incomplete and misplaced controls
on the target Windows desktop. Controller tests did not establish visual
correctness there; the experimental page implementation has been withdrawn.

`render_page`, `select_page`, `scroll_page`, `load_document`,
`change_language` and `update_ammo_chance_summary` are restored from the supplied
v57 source. Each section rebuilds the existing Canvas form immediately, as it did
before v58. There is no replacement Canvas window or cached page frame. The
mouse wheel still scrolls the page without changing dropdown selections.

The v58 model selector and selected-palette filtering remain in `fields`.
Changing a model renders the current form again; hidden color values stay in the
shared document variables and are saved normally. Gameplay code, shaders,
configuration defaults, model palettes and native Visibomb steering are identical
to v58. The runtime identity remains 0.1.6.58 / configuration capability 58;
only the launcher version advances from 0.17.0 to 0.17.1.

## Apply without recompiling

Run `00_RESTORE_LAUNCHER.bat` from the extracted test bundle. Select the existing
launcher folder containing `launcher.py`, including a source/launcher directory
with example/default INIs. This verifies delivered source hashes and backs up changed Python launcher files,
then installs the corrected Python editor and opens it. It does not invoke
Cargo or PyInstaller. `weapons.ini`, presets, settings, custom icon, runtime and
save files are not rewritten. Use `02_OPEN_TEST_LAUNCHER.bat` subsequently, or
`OPEN_TEST_LAUNCHER_v58_2.bat` in the selected mod folder.

The standalone launcher EXE is unchanged. Run the BAT for this test. The complete
corrected source snapshot is included for the next build; the fast restoration
action updates the selected mod folder only, not a separate source checkout.
The optional full-build BAT still updates a selected source checkout and reuses
its build cache if a complete rebuild is required later.

Python tests cover the retained grade/model UI, original wheel routing,
selected-color filtering, profile preservation and transactional launcher-only
updates, including rollback after a failed file write. No new gameplay tests
are required because every file under `crates/` is byte-identical to v58.
Windows visual verification remains a manual check on the user's computer.
