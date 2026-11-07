# v137 — grapple locomotion, centred drill capture and Freezer equip input

ReRAC **0.3.8-alpha**, gameplay **0.1.6.137**, capability **97**.
The complete source kit includes all preceding changes. Launcher **0.30.2**, its
icon, all existing INIs/defaults, presets, saves and randomizer seeds are retained.
No release, tag, draft or game-data upload is performed by this kit.

## Install and package

1. Extract the whole ZIP to a new folder. Close the game and launcher.
2. Run `01_COMPILE_AND_UPDATE.bat`. Select the existing playable mod folder with
   `weapons.ini`, the launcher EXE and `runtime/rerac.exe`, not the new source
   folder. No previous test-source folder is needed.
3. This update uses `--runtime-only`: build/test and identity verification must
   finish before runtime/shaders are replaced. The installed launcher is reused;
   there are no new editor fields. A failed build retains playable executables.
4. Run `02_OPEN_UPDATED_LAUNCHER.bat` to open that installation.
5. `03_CREATE_PLAYABLE_ZIP.bat` packages the updated runtime, retained launcher and
   current player INI after a successful update. Extracted game data are excluded.

## Swingshot: pose ownership and the actual nozzle

The custom cable retains native launch sequence 0x2f and its bent throw/hold
pose. Its origin now uses item chain 0 in the current item pose, the same native
nozzle reference as the Swingshot. It no longer inherits the chest-height safety
fallback used to prevent Freezer projectiles launching under the floor.

When fire is released, the custom pose relinquishes control before native
movement. If that pose is still active, the current locomotion state determines
walk/run/idle; a jump, damage reaction or cinematic already chosen by native code
is not replaced. This matters when the run state/substate never changed: native
locomotion otherwise does not restart the run sequence after a forced idle.

Only the latched phase filters voluntary movement. A local copy of the gameplay
pad loses left-stick/directional translation and jump, retaining camera, aiming,
fire/release and menu input. Extending or retracting an unattached cable does not
lock movement. The grapple's own movement still goes through the normal capsule
collision resolver, including vertical travel while aiming with L1. Native
Swingshot target priority and yellow-target hanging remain exclusive to the game.

## Trespasser: fixed model centre and steady aim

At capture, the runtime caches the idle model's bounding-sphere centre in local
coordinates, scaled to world units. If unavailable, a valid world bound is
transformed back to local coordinates; the final fallback is a small body-centre
offset. The allocation lease owns this cache, not a reusable slot or the matrix
revision. Boss rejection and ground-only capture/audio remain unchanged.

For each turn, root translation compensates the rotated centre, including
pitch, model scale and mirrored orientation. The same centre stays a short radius
from the real drill tip, so an offset feet/root origin cannot create an orbit.
The actor's matrix is rebuilt immediately after placement because the scheduler
has already processed it. Authored KEEP_ROWS transforms are restored on release
or thaw. No actor copy or enemy projectile is created on death.

During capture, Ratchet holds a decoded Pyrocitor aiming key instead of repeatedly
blending idle/aim sequences. The final pose is applied before hand/back attachment
evaluation. Releasing, jumping or interruption relinquishes its playback lock.
Damage/XP use the existing accepted-damage path and are unchanged by pose pinning.

## Freezer: equip first, fire on the next press

The ready/armed latch is transient hero state, separate from ammunition,
regeneration and saves. Holding Circle, R2 or both through equip cannot consume a
round. A menu handoff that masks gameplay fire while the physical button remains
held does not count as a release. Once the equipped gadget is ready, releasing
fire arms it; the next press
fires normally. Holding after that retains the configured repeat interval.
Switching away, native Hydrodisplacer use or an unavailable gameplay state clears
the latch. No save schema or balance settings change.

## In-game checks

- While running, fire and immediately release a surface/enemy grapple. Repeat
  with directional changes, L1 aiming and jump/damage interruptions. Check normal
  running resumes and the cable starts at the Swingshot rather than Ratchet's head.
- While hooked, hold movement in each direction. Ratchet must not walk freely;
  a surface grapple must still pull him, and release must immediately allow input.
- Capture differently shaped ordinary enemies and turn the camera while drilling.
  Check a stable centre next to the tip, a steady aim pose, normal release/death,
  no duplicate actors and native drops/XP. Jump must cancel capture and its audio.
- Equip Freezer using Circle and then R2. Hold the equip button: no shot or ammo
  loss. Release and press again: one shot; hold to repeat at the configured rate.
  Check its HUD, yellow full bar, regeneration and condensation remain functional.

See VERIFICATION.md for exact host checks and platform limits. Windows linking,
real game assets, visual alignment and GPU behavior require this local test; no
in-game visual playthrough is claimed by the preparation host.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, not ReRAC or the original games, is not affiliated with their developers or
rights holders, and claims no rights to the original games or assets. Thanks to
the ReRAC developers and contributors for the port that enables these mods.
Existing licenses, credits and notices remain applicable.
