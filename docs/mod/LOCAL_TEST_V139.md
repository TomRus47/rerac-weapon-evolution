# v139 — airborne grapple release and contact-only drill capture

ReRAC **0.3.8-alpha**, gameplay **0.1.6.139**, capability **97**.
Launcher **0.30.2** is retained. The complete kit includes the preceding fixes,
including v138 collision suppression, capture-owned motor and Hoven frost audio.
Existing INIs/defaults, balance, presets, save formats, seeds and icon are unchanged.
This kit performs no GitHub operations and creates no release, draft or tag.

## Install and package

1. Extract the complete ZIP to a new folder. Close the game and launcher.
2. Run `01_COMPILE_AND_UPDATE.bat` and select the existing playable mod directory
   containing `weapons.ini`, the launcher EXE and `runtime/rerac.exe`, not this
   kit's source folder. Previous test-source folders are unnecessary.
3. The installer uses `--runtime-only`. Tests, compilation and exact runtime
   identity verification must pass before replacing playable runtime/shaders.
   The installed launcher/icon are reused. A failed build retains executables.
4. Open that installation using `02_OPEN_UPDATED_LAUNCHER.bat`.
5. After success, `03_CREATE_PLAYABLE_ZIP.bat` packages the updated runtime,
   retained launcher and current INI, without extracted game data.

## Airborne release handoff

The previous custom-pose exit selected walk/run/idle on the ground but did not
select a fall pose in airborne state 6. A surface pull can also leave native fall
as sequence B while the source snapshot is missing. Merely observing a different
sequence B and returning does not repair the renderer's identity-matrix fallback.

On release in ordinary airborne/fall/aim states, a still-owned cable pose now
hands off to decoded native fall sequence **0x0b**. The same recovery applies
when native fall was already selected but its source snapshot is absent. The
gadget-specific pose validator repairs that source before blending. Its loop and
ownership are cleared once; native gravity, momentum, falling and landing then
continue. Independently selected jump, damage and cinematic poses are retained.
Grounded running release keeps the v137/v138 behavior.

The final body pose still runs before write-back and hand/back-chain evaluation,
so Clank's cached attachment and Ratchet's render pose use the same valid keys.

## Physical contact before Trespasser capture

Acquisition now reads the current native capsule hit (`cap_moby` with `cap_hit`)
or wall-contact moby, rather than scanning the nearby aim cone. Both contact
records are produced/reset by Ratchet's normal movement resolver. A nearby enemy,
a large configured range, or a stale capsule ID without a hit cannot qualify.
The contacted actor must be a living eligible enemy in front of Ratchet; bosses
remain excluded and native Trespasser pads retain priority. `drill_range` remains
a defensive upper bound; it cannot create a contact or a ranged capture.

This condition applies only to initial acquisition. Once captured, the victim's
collision remains disabled and the existing allocation lease maintains capture
until release, death or interruption. It does not need to collide again after
being moved to the drill tip. Damage, Nanotech XP and one capture-owned motor
voice retain v138's accepted-damage and cleanup behavior.

The cached model centre is now **one game unit farther forward** along the drill
axis than v138, in addition to its existing size-dependent offset. Rotated-root
compensation and immediate matrix rebuilding keep it fixed while spinning.
This is a placement correction, not an increase in acquisition reach.

## In-game checks

- Grapple a high wall, release fire halfway through travel and continue falling.
  Repeat with L1 aiming, camera lock and Clank on the back. Check a normal fall
  pose throughout descent, then normal landing/run, with no T-pose or detached pack.
- Check early release while running on the ground still restores running, and
  native jump/hit reactions and blue/yellow Swingshot targets remain functional.
- Hold Trespasser fire near an enemy without touching it: no capture or motor.
  Walk into the enemy while continuing to hold: capture begins on physical contact.
- Capture differently sized enemies, rotate the camera and keep fire held. The
  victim should be farther forward with no orbit/body push, and remain held even
  though its collision is disabled. Release: restore collision for a living victim.
- Kill a captive or interrupt capture: check normal death/drops/XP and motor stop.
  Freezer equip input, timed ammo, HUD and Hoven detonation audio remain unchanged.

See VERIFICATION.md for exact host checks. Native logic/palette and collision
fixtures are not a substitute for Windows linking or real-asset/GPU playtesting.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, not ReRAC or the original games, is not affiliated with their developers or
rights holders, and claims no rights to the original games or assets. Thanks to
the ReRAC developers and contributors for the port that enables these mods.
Existing licenses, credits and rights notices remain applicable.
