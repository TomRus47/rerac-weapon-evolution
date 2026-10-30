Weapon Evolution v121 - Novalis burst tracking local test
Runtime: ReRAC 0.3.8-alpha. No GitHub operations or release publication.

What changes
------------
Novalis robot troopers (class 459, fire glob 722) could keep their initial
firing direction throughout a volley. Their firing state did not turn the
body, and individual shots only corrected aim inside a five-degree cone.
With the existing enemy_burst_tracking setting On, the firing body now turns
toward the current target and every newly emitted glob aims at that target.
Shots already in flight remain ballistic. With the setting Off, the native
firing behavior is retained. Target absence during a cutscene does not enable
tracking. Native animation events, volley count, speed, gravity, muzzle-wall
checks, damage and XP are retained.

Installation
------------
1. Extract this ZIP to a NEW folder. Keep the previously compiled test kit.
2. Close the game and mod launcher. Run 01_COMPILE_AND_UPDATE.bat.
3. PLAYABLE folder: select the folder containing your working launcher EXE,
   weapons.ini and runtime/rerac.exe.
4. PREVIOUS TEST KIT: select the extracted v120 folder you previously compiled
   (or its source subfolder). Prepared v118/v119 sources are also accepted,
   but contain only the fixes already present in that chosen source.
   Do not select the playable folder or an old C:\ReRAC-Mod\rerac checkout here.
   If the earlier test kit has no prepared source yet, compile that kit first.
5. Source integrity is checked in full. A private copy is prepared here;
   unrelated local v117-v120 additions remain exactly as in the selected kit.
6. The recovered Rust compiler and Visual Studio helpers from that source are
   reused. Full local workspace Rust tests run before linking/installing.
7. The game runtime and matching shaders are installed with a rollback backup.
   The existing GUI EXE/icon is reused; no launcher rebuild is needed.
8. Run 02_OPEN_UPDATED_LAUNCHER.bat, or open the same working launcher as usual.
   Enable enemy_burst_tracking in Global Settings and launch with that profile.
   No new game is required. Use 03_CREATE_PLAYABLE_ZIP.bat if desired.

This kit changes only path_enemy.rs plus version/integrity metadata and this
document. It does not replace balance/default INIs, profiles, save slots, seed,
Oltanis/Orxon fixes, launcher UI or any unrelated gameplay implementation.
If integrity/preparation/build fails, the previous playable runtime stays in place.
Logs: logs/v121-*.log and logs/update-error.log inside this test kit.

Verification
------------
The included regression tests emit actual globs at a moving target and compare
On/Off behavior, preserving shot count, speed and shooter. They also check body
turning and the no-target/cutscene case. Host tests use the public runtime source
with only the already-existing v117 settings field supplied as a compatibility
fixture; that fixture is not shipped as replacement gameplay source.
Host verification: 4 Rust regression tests and 6 preparation/integrity tests passed.
Windows linking and visible in-game behavior require this local test build.

Credits and rights
------------------
Weapon Evolution is an independent mod. Its author is not associated with or
part of ReRAC development and does not own the original Ratchet & Clank games,
characters, assets or trademarks. Credit and thanks to the ReRAC developers
whose runtime makes these modifications possible. Original game data is not
included; use your own legally obtained game data. Upstream source retains its
license and attribution.
