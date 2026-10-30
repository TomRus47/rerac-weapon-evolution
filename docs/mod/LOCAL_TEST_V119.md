Weapon Evolution v119 / ReRAC 0.3.8-alpha - local test update

This small ZIP adds the reviewed R2 gadget and Veldin II tank-shell fixes to
an intact v118 Orxon Shortcut Test source kit. It keeps all v117/v118 work,
including the early-O2 Orxon shortcut. No GitHub or release operation is run.

INSTALL
1. Extract this entire ZIP to a NEW folder beside the extracted v118 kit and
   the v117.3 kit whose separate Rust installation already works. Close both
   the game and the mod launcher.
2. Run 01_COMPILE_AND_UPDATE.bat. Select the EXISTING PLAYABLE mod folder:
   it contains ReRAC_Weapon_Evolution.exe, weapons.ini and runtime/rerac.exe.
   Do NOT choose this update folder for that question.
3. The v118 sources are found automatically if exactly one matching kit is
   beside this folder, on Desktop or in Downloads. If asked for the SOURCE,
   select the EXTRACTED v118 Orxon Shortcut Test folder (or its source folder).
   This is a different question from the playable mod folder. A single extra
   Extract All wrapper folder is accepted. The original v118 files are retained.
4. Compilation and tests run before the executable/shaders are installed.
   Do not close the window while Cargo is working. Cold builds can be slow.
5. Run 02_OPEN_UPDATED_LAUNCHER.bat, or open the SAME mod launcher as before.

The preparation copies and SHA-256 verifies the full v118 source inventory,
then applies only unique reviewed code blocks. Unknown/altered source stops
before changing the playable mod. It does not substitute old v116/main sources.
The prepared source copy is created inside THIS update folder after validation;
there is deliberately no ready-made runtime or full source tree in this ZIP.
Runtime identity after success: patch 0.1.6.119, config capability 93, ReRAC 0.3.8.

The working launcher EXE and icon are retained without a redundant GUI rebuild.
Existing weapons.ini, presets, settings, saves and randomizer seeds are retained.
No new INI variable or balance/default edit is needed. The existing v118 build
and rollback routines initialize Visual Studio x64, verify the Rust compiler,
reuse the verified sibling recovery toolchain when available, clear only local
Cargo workspace artifacts, run fresh Rust tests and select Cargo's actual EXE.
Internet is needed for uncached packages or compiler recovery. No security
settings or system-wide Rust installation are changed.

TEST CHECKLIST
- Equip Swingshot: R2 must fire and hold it. Change from R2 to Circle without
  releasing both; it must stay attached. Release both to detach normally.
- Test R2 with Trespasser, Hydrodisplacer, Metal Detector, PDA and Hologuise.
  The Hologuise salute also accepts R2. Passive worn gadgets keep native controls.
- Check that shop, puzzle, turret, vehicle and Clank controls remain native.
- On Veldin II, enable enemy_damage_by_health and set the effective enemy damage
  multiplier to 1: a tank shell, direct or splash, removes 6 Nanotech points.
  Multiplier 2 removes 12. This uses original native tank HP, not remaining HP
  or the enemy-health multiplier. With health-class damage Off, damage is native.
- Continue the same saves; no new game or seed reset is required.

03_CREATE_PLAYABLE_ZIP.bat packages the updated installation with your current
weapons.ini under this folder's dist directory. It does not include personal
presets, saves or extracted disc assets. Players still need their own game data.

Logs are in logs/v119-*.log and logs/update-error.log in THIS update folder.
Rollback backups are in backup/runtime_038/<timestamp> in the PLAYABLE folder;
restore.json identifies every backed-up runtime/shader file. Keep that backup.

Prior host validation: 1,386 rc-game library tests passed for the R2 changes;
41 selected damage tests passed for the tank fix. This ZIP's preparation and
preservation checks are separately tested. Windows linking and actual gameplay
are performed on your PC by this local test; no precompiled Windows EXE is included.

Weapon Evolution is an independent mod by TomRus47. The author is not associated
with development of ReRAC or the original games and owns no original-game assets
or rights. Thanks to ReRAC's developers and contributors for making mods possible.
Upstream licenses and contributor credits remain applicable.
