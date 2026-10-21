Weapon Evolution v59 - LAUNCHER RESTORE
=======================================

Unofficial mod for ReRAC 0.3.4-alpha.
Runtime is 0.1.6.59 / configuration capability 58.
Corrected Python launcher: 0.17.2. Local test only; no GitHub update.

QUICK RESTORE - NO GAME RECOMPILATION

1. Extract the entire ZIP into a new folder.
2. Close the game and the old launcher.
3. Run 00_RESTORE_LAUNCHER.bat.
4. Paste your existing launcher folder containing launcher.py.
   An installed mod folder OR an older source/launcher folder is supported.
   No runtime directory or compiled game is needed to restore/open the editor.
   A source root is also accepted when it contains launcher/launcher.py.
   Select the older folder being corrected, not this new update bundle.
5. The tool backs up changed launcher Python files and opens the corrected editor.
   Use Save and launch as usual. No preset creation is required.
6. Next time use 02_OPEN_TEST_LAUNCHER.bat here, or
   OPEN_TEST_LAUNCHER_v58_2.bat in your existing mod folder.

Use these BAT files for this test. Your existing standalone launcher EXE is
not rebuilt. Python 3.10+ with Tcl/Tk is required, as in the previous local test.
Cargo and Visual Studio are NOT needed for this launcher-only restoration.

WHAT IS RESTORED

- Original immediate page construction in the existing Canvas form.
- Original navigation, language changes and direct page scrolling.
- No page cache, hidden replacement frames, background preloading or Windows
  composited-painting flag. Sections open as they did in v57.

WHAT IS PRESERVED

- Flame mine size applies to native and alternative mine models.
- All v58 model choices and independent model palettes for Visibomb grenades
  and flame mines, including velocity-aligned missile appearance.
- The v58 native Visibomb duration/steering correction and all previous fixes.
- Your existing weapons.ini, presets, settings, icon, game runtime and saves.
- Mouse-wheel input over dropdowns still scrolls only the page.

The fast restore writes only Python launcher files and a test-launcher BAT.
Backups are stored in the existing mod folder under:
  backup\launcher_v58_2\<timestamp>
RESTORE.json identifies new files and the previous launcher bytes are included.
Failed writes roll back any launcher files already changed.

OPTIONAL FULL SOURCE UPDATE / BUILD

The complete corrected source snapshot remains in source/.
01_UPDATE_BUILD_AND_OPEN.bat is optional and runs the usual source overlay,
Rust tests and Windows game compilation. Only use it if you also want to update
your separate ReRAC source checkout or do not yet have the v58 runtime.
This full-build path requires Rust/Cargo, Visual Studio C++ build tools and a
Windows SDK. It reuses Cargo's target cache and preserves personal profiles.

Developer notes: source/docs/mod/LAUNCHER_RESTORE_V58_1.md.
The old v58 static-page design is historical and has been withdrawn.
No GitHub push, release or remote build is performed.
