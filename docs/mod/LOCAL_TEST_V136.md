# v136 — custom grapple poses and normal Trespasser kills

Gameplay `0.1.6.136`, ReRAC **0.3.8-alpha**, configuration capability **97**,
launcher **0.30.2**. This complete source kit includes the preceding local changes,
including Freezer HUD, timed ammunition and portable condensation from v135.
Existing balance, INIs, presets, save formats, seeds and icon are retained.

## Build and test

1. Extract the entire ZIP to a new folder. Close the game and launcher.
2. Run `01_COMPILE_AND_UPDATE.bat` and select the existing playable mod directory
   containing `weapons.ini`, the launcher EXE and `runtime/rerac.exe`. Do not select
   the extracted source folder. Previous source patch folders are unnecessary.
3. The installer builds and checks the runtime and matching launcher before
   replacing playable files. It recovers the installed icon and retains player
   configuration and saves. Keep the error window and its diagnostic path if a
   check fails. Launcher 0.30.2 updates the translated Trespasser help.
4. Use `02_OPEN_UPDATED_LAUNCHER.bat` for that same installation.
5. After a successful update, `03_CREATE_PLAYABLE_ZIP.bat` packages the updated
   runtime/launcher and your current `weapons.ini`. Game data are excluded.

## Grapple animation lifecycle

The v135 check accepted a nonzero sequence header, but did not establish that
all decoded keys existed or repair a missing source snapshot. Native pose
blending can preserve snapshot sequence 0xff without a corresponding frame;
the renderer then falls back to identity bone matrices (T-pose). Repeated
transitions can keep restarting the blend. The late custom pose change also
happened after the back attachment had already cached its pose for that tick.

Custom surface pulls now reuse the Swingshot's stable launch/cable-holding
sequence **0x2f**, rather than forcing the full-body **0x30** pose of native
state 0x25 without entering that state. Missing/incomplete sequences fall back
to a complete available stance. The real animation binding verifies decoded
keys and can repair a missing source pose from a valid key before blending.
Native SetAnim callers are unchanged; recovery is exclusive to gadget poses.

The custom pose is applied after native movement/transitions, **before** combat
pose layers, write-back and `fx::end` back-chain evaluation. Ratchet's render pose
and Clank's cached attachment therefore share the same animation. Cable travel,
L1 ascent, collision, enemy pulling, native target priority and the prohibition
on custom yellow-target hanging remain intact.

## Trespasser: death without shooting an enemy copy

Defeating a captured enemy now releases capture and lets native HP/death handling
finish its normal death, drops and XP. The replacement-actor spawner, projectile
simulation, corpse snapshot and hidden-corpse tracking are removed. No defeated
enemy is cloned or launched. A dead actor is not forcibly hidden or deleted in
place of its native death update. Actual damage and XP accounting are retained.

Ground-only capture and audio from v135 remain: jumping cancels capture and the
motor loop. Native Trespasser pads still take priority. English, Italian, French,
German and Spanish launcher help now describes normal enemy death.

## Repository and release behavior

This update synchronizes main with the complete verified local source. It does
not create a tag, release or draft. The draft build workflow is now manually
invoked (`workflow_dispatch`); pushing development metadata no longer creates
or changes a draft automatically. Public release metadata and downloads remain
unchanged. Existing licenses and credits are retained.

## In-game checks

- Grapple elevated walls/ceilings on Novalis and a later planet with Clank on
  Ratchet's back, with and without L1. Verify a stable cable-holding pose during
  pull and normal poses after release, landing or interruption.
- Check original blue/yellow Swingshot targets still behave normally.
- Kill a captured ordinary enemy with Trespasser. Check one normal death, no
  thrown model or extra actor, native drops and no duplicated XP. Test release,
  jumping, damage and checkpoint death while holding the button.
- Check Freezer counter, regeneration, full yellow bar and condensation remain
  functional; this patch does not change their settings or balance.

## Validation and credits

See VERIFICATION.md for exact tests and platform limitations. Real game assets,
GPU appearance and Windows linking require the included local build/test. The
host regressions exercise real animation evaluation and back-chain placement;
they are not a claim of a completed visual playthrough.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, not ReRAC or the original games, is not affiliated with their developers or
rights holders, and claims no rights to the original games or assets. Thanks to
ReRAC's developers and contributors for the port that makes these mods possible.
Their licenses and contributor credits remain intact.
