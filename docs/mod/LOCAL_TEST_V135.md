# v135 local test — gadget poses and Freezer

Weapon Evolution gameplay `0.1.6.135`, ReRAC **0.3.8-alpha**, configuration
capability **97**, launcher **0.30.1**. Includes the complete v134 source and its
actor-lifetime corrections. No GitHub operations or release publication.

## Build and install

1. Extract the whole ZIP into a new folder. Close the game and mod launcher.
2. Run `01_COMPILE_AND_UPDATE.bat`. Select the existing **playable mod folder**
   containing the launcher EXE, `weapons.ini` and `runtime/rerac.exe`. Do not
   select this kit's source folder. Previous source patch folders are unnecessary.
3. This version rebuilds **both the runtime and launcher**. The new recharge field
   needs launcher 0.30.1; keeping the v133/v134 launcher would hide that setting.
   The existing icon is recovered for the new EXE. The installer verifies sources,
   tests/builds/probes the runtime and checks the editor before replacement, with
   backups and rollback. Keep its window and diagnostic path if anything fails.
4. Use `02_OPEN_UPDATED_LAUNCHER.bat`. In Hydrodisplacer, enable gadget combat and
   adjust **Ammo recharge time (seconds)**. Save and start. Existing capacities,
   colors, weapon balance, presets and saves are retained. Missing recharge
   settings migrate to 10 seconds. No seed regeneration or new game is required.
5. After a successful installation, `03_CREATE_PLAYABLE_ZIP.bat` packages the
   updated launcher/runtime and your current `weapons.ini`. Disc data are excluded.

## Poses and grounded Trespasser

Custom Swingshot poses are now checked against the active Ratchet animation
class. Missing native Swingshot sequences (notably on Novalis) fall back to an
available weapon stance or idle pose. Entry frames are clamped to the actual
sequence length and native launch loops require enough frames. No unavailable
sequence is sent to Ratchet or Clank's back-animation follower. Native Swingshot
targets still take priority; custom cable movement and collision are retained.

Trespasser capture, its motor and custom aiming pose are allowed only while
Ratchet is grounded in a normal standing/running/crouching group. Jumping or
falling cancels capture and releases the sound loop. It cannot replace an active
jump/fall animation with a standing pose. The red tip remains stationary. The
existing actual-damage/XP accounting and single defeated-enemy projectile remain.

## Frozen-enemy condensation

Frozen actors now have a dedicated white/ice-blue condensation effect. It uses
a small generated soft-alpha texture and camera-facing puffs rather than relying
on a level's smoke definitions. Emitters follow the current enemy position,
expire on thaw/removal/death, and are capped at 64 actors with four puffs each.
They do not allocate enemies, lights, damage actors or gameplay RNG draws.
The existing ice crystal, impact sphere, tint and freeze duration are retained.

## Freezer HUD and regeneration

When Hydrodisplacer combat is enabled and equipped, the top-left panel shows a
localized **Freezer** name and `current/max` ammunition, without an ammo icon or
weapon level suffix. Captions follow the game's language: Freezer (English),
Congelatore (Italian), Glaceur (French), Froster (German), Congelador (Spanish).

Its progress bar reuses the weapon bar's geometry, style, alternate position,
layer ordering and native ammo-panel fade. It represents **one recovering shot**,
not XP. Below maximum ammo, the bar fills for `recharge_seconds`, grants one shot,
then starts again if another shot is missing. At capacity it is full and yellow;
there is no overflow or stored refill credit.

```ini
[gadget.hydrodisplacer]
max_ammo = 10
recharge_seconds = 10
```

`recharge_seconds` accepts 0.1–3600 seconds and is placed with ammo settings,
before the color controls, in both the INI and all five launcher languages.
Regeneration runs during active gameplay even with another item equipped.
Pausing, cutscenes, death and disabling the feature do not advance its clock.
Live capacity decreases clamp ammo; increases do not grant free extra shots.
Planet arrival still fills the tank. Shops, pickups, upgrades, gadget switching
and checkpoint deaths cannot refill it. Partial recharge survives a checkpoint
reset in the same session. Native game/weapon save formats remain unchanged.

## In-game checks

- On Novalis and a later planet, grapple walls with/without L1 and Clank equipped.
  Verify no T-pose and no displaced backpack; verify original Swingshot targets.
- Capture an enemy with Trespasser, jump while holding fire, then land. Check
  capture and audio stop in the air, the jump animation stays intact and the
  grounded behavior can restart. Check native Trespasser pads separately.
- Freeze several enemies on Novalis. Check visible white/blue vapor, following
  moving targets if another effect displaces them, and complete cleanup on thaw.
- Set `max_ammo=3`, `recharge_seconds=2`. Spend shots: verify timer and count agree,
  a single shot every two gameplay seconds, full yellow bar at three, no overflow.
  Switch items, pause, enter a shop, die and change planet to check the clock.
- Check top-left name/count/timer fade together, no ammo icon, correct wide-screen
  alignment, alternate bar position and existing front/behind-HUD setting.

## Scope and credits

Native logic and Python tests are recorded in VERIFICATION.md. Windows linking,
real asset playback, Tk rendering and GPU appearance require the included local
build and in-game testing; these are not claimed as tested on this host.

Weapon Evolution is an independent mod by TomRus47. The mod author does not
develop ReRAC or the original games, is not affiliated with their developers or
rights holders, and claims no rights to the original games or assets. Thanks to
ReRAC's developers and contributors for the port that makes these modifications
possible. Existing licenses and contributor credits remain intact.
