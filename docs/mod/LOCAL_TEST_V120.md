Weapon Evolution v120 / ReRAC 0.3.8-alpha - local test

This incremental update retains the complete v119 source and fixes two issues.
No repository, tag, draft or release operation is performed by this kit.

Oltanis in Randomizer All / NG+
NG+ keeps the seed, collected checks and acquired rewards while native missions
reset. The carrier now accepts the retained claim of the original Morph-o-Ray
location for that exact route mission. It still requires both other native route
missions. Owning a Morph-o-Ray from another location does not bypass the route.
The absent reward does not reappear and no second reward or save grant is made.
The native scene, plaza trigger and missile behavior remain intact. This applies
to existing saves as well as new runs; reload or leave/re-enter Oltanis after
updating. No new game or seed change is needed.

Orxon ground-fire-wave enemies (shared class 1068)
Their separate entrance-wait radius now follows the perception multiplier.
A received Taunter alert wakes the authored entrance path even outside the
waiting cuboid; it no longer gets blocked by the seven-unit waiting check.
Sight is scaled once, and home/pursuit limits are scaled independently once.
The existing entrance movement, attacks, flame visuals, damage and XP remain.
The native default waiting limit is preserved when perception is 1 and no
alert has been received. This is a specific fix for the fire-wave class, not a
claim that every differently modeled Blarg enemy has been identified/tested.
Please check the exact Orxon enemies described in the player's report in game.

INSTALL
1. Extract this entire ZIP to a NEW folder beside your extracted v119 test kit
   and the v117.3 Rust-recovery kit whose compiler already works. Keep v118 too.
   Close both the game and the mod launcher.
2. Run 01_COMPILE_AND_UPDATE.bat. For PLAYABLE folder select the existing mod
   containing ReRAC_Weapon_Evolution.exe, weapons.ini and runtime/rerac.exe.
3. The v119 kit is found automatically when unambiguous. If asked for the v119
   TEST KIT, select its extracted folder containing test_patch.py (not the
   playable mod). Its source folder is accepted too. If v119 was not compiled,
   its own source preparation runs first using v118; no intermediate game build.
4. Wait for tests and compilation. The source copy is created inside THIS kit.
   Actual EXE identity is checked before the existing runtime/shaders are replaced
   transactionally. The launcher EXE/icon, INI, presets and saves are retained.
5. Open 02_OPEN_UPDATED_LAUNCHER.bat or the SAME mod EXE you normally use.

The source and payload are SHA-256 verified. Unsupported code blocks or missing
files stop before changing the playable installation. No older main/v116 source
is substituted for the existing v119 fixes. This ZIP is an incremental compiler
kit, not a precompiled game. Windows C++ x64 tools, Python and Rust/Cargo remain
required; the existing verified compiler recovery and Cargo cache are reused.
Internet is needed only for uncached dependencies/compiler recovery.

03_CREATE_PLAYABLE_ZIP.bat packages the updated installation and your current
weapons.ini. Personal saves, presets and extracted disc assets are not included.
Logs: logs/v120-*.log and logs/update-error.log in this kit.
Backups: backup/runtime_038/<timestamp>/restore.json in the PLAYABLE folder.
Expected runtime identity: ReRAC 0.3.8, Weapon Evolution 0.1.6.120, capability 93.

Tests cover the retained location claim, exact mission identity, incomplete other
routes, ordinary/non-randomized play, reload and absence of repeated rewards;
the existing statue missile test is retained. The Orxon test sends a real Taunter
lure and exercises the native enemy update at default and increased perception.
Host tests do not replace Windows linking or testing the scene/enemies in game.

Weapon Evolution is an independent mod by TomRus47. The author is not associated
with development of ReRAC or the original games and owns no original-game rights
or assets. Thanks to ReRAC's developers and contributors for making mods possible.
Upstream licenses and credits remain applicable.
