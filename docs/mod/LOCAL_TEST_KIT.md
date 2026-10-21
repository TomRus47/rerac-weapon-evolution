Weapon Evolution v56 - LOCAL TEST KIT
===================================

Unofficial mod for ReRAC 0.3.4-alpha. Gameplay patch 0.1.6.56,
configuration capability 56, launcher 0.16.0. This is not a public release.

QUICK START

1. Extract this entire ZIP into a NEW folder, for example:
   C:\ReRAC-Mod\WeaponEvolution_v56_Test

2. Close ReRAC. Run 01_UPDATE_BUILD_AND_OPEN.bat.
   Use your existing ReRAC SOURCE folder, normally C:\ReRAC-Mod\rerac.
   It must contain Cargo.toml, Cargo.lock and crates, not just rerac.exe.
   For the mod folder, paste the folder containing your current weapons.ini
   and runtime folder. Leave its suggested test-launcher path unchanged if
   you prefer a separate test installation with the supplied defaults.
   The source folder and mod folder must be separate, without nesting.

3. The tool verifies the complete delivered source snapshot, backs up changed
   files, migrates your INI and presets, then tests and builds the editable
   runtime. It reuses the selected source folder's target directory.
   The old runtime is replaced only after a successful build.

4. The UPDATED test editor opens automatically. Use Save and launch.
   For the new mode: Global Settings -> Ammo-Survival Mode -> On.
   No named preset is required. Your previous selected preset is preserved.

5. Next time use 02_OPEN_TEST_LAUNCHER.bat in this kit, or
   OPEN_TEST_LAUNCHER_v56.bat in the selected mod folder. This opens the updated
   Python editor without rebuilding the game. The pre-existing launcher EXE
   is not rebuilt by this test kit; use these BAT files for the new controls.

AUTHORING REQUIREMENTS

Windows x64, Rust/Cargo, Visual Studio C++ build tools with a Windows SDK,
and Python 3.10+ with Tcl/Tk. These are the same local authoring tools used
for earlier patches. MSVC setup is detected automatically when possible.
Extracted game data is needed to PLAY, not to compile. The editor can detect
%LOCALAPPDATA%\rerac\games\rac1\data or accept your selected data folder.

WHAT CHANGED

- OmniWrench is directly below Global Settings, before Bomb Glove.
- All four upgrade XP requirements are in its XP progression strip, just like
  the other weapons. They no longer appear separately in each grade card.
- Each wrench grade adds hit_radius_multiplier and knockback_multiplier.
  Both start at 1, preserving current combat. The hit radius affects all combo
  swings, airborne contact, the throw in both directions and the slam impact.
- Knockback scales horizontal enemy push; vertical response and damage/XP remain
  unchanged. Throw travel range/speed and Crate Breaker radius stay independent.
- Existing INI values, XP requirements, presets, language selection and custom
  icon are preserved. Includes all previous v55 features and fixes.
- Full implementation notes and test checklist:
  source/docs/mod/WRENCH_TUNING_V56.md.

BACKUPS AND LIMITS

Changed source and mod files are backed up under the selected source folder:
  .weapon_evo_backup_v56_<timestamp>
RESTORE.json records the two original locations. source/ and mod/ contain
the previous file bytes; files listed as new had no previous version.
Unrelated files, Cargo's target cache, game data and saves are not copied
or deleted. A complete reviewed source overlay avoids version-specific text
patch failures. Local source edits replaced by the overlay are backed up.

No commit, push, GitHub workflow, release, ZIP player build or PyInstaller
launcher build is performed. Cargo may use the network for missing pinned
dependencies, as in a normal local Rust build. Console windows pause at the
end so success/failure remains visible. Paths containing spaces are supported.

The new runtime reads old WEVO-v1/v2 progression and writes WEVO-v3 to include
the wrench. Older runtimes cannot read newly written v3 companions: keep a
copy of your old save folder if you intend to downgrade after testing. Windows compilation and in-game behavior are
to be tested on your PC; the delivered source was checked with the available
Rust library and Python tests before packaging.

TomRus47 authored Weapon Evolution, not ReRAC or the original game.
No copyrighted disc data or personal saves are distributed.
