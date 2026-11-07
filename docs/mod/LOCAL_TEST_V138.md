# v138 — collision-safe Trespasser capture and native Freezer impact audio

ReRAC **0.3.8-alpha**, gameplay **0.1.6.138**, capability **97**;
installed launcher **0.30.2** retained. The source includes all preceding fixes.
Existing INIs, balance, presets, save formats, seeds and custom icon are preserved.
This kit does not perform GitHub operations or create a release/tag/draft.

## Build and test

1. Extract the whole ZIP into a new folder and close the game and launcher.
2. Run `01_COMPILE_AND_UPDATE.bat`; select the existing playable mod folder
   containing `weapons.ini`, the launcher EXE and `runtime/rerac.exe`. Do not
   select this kit's source folder. No older test-source folders are required.
3. Tests, runtime compilation and identity verification finish before playable
   runtime/shaders are replaced. This kit uses `--runtime-only`; do not rebuild
   the launcher manually. A failed build retains installed executables.
4. Open the same installation using `02_OPEN_UPDATED_LAUNCHER.bat`.
5. After a successful build, `03_CREATE_PLAYABLE_ZIP.bat` packages that runtime,
   retained launcher and current INI. Extracted game data are excluded.

## Temporary collision ownership

A captured enemy now has `has_collision=false`, so native scene snapshots and
body queries cannot push Ratchet away from the drill. Capture remembers the
previous value under the existing slot/class/allocation-generation lease. It
never resnapshots the temporary false value on subsequent held frames.

Releasing, jumping, switching gadgets, disabling the feature, entering another
hero mode or Ratchet dying restores the prior collision value for a living
victim. An originally collisionless enemy remains collisionless. Native
per-primitive disable masks, targeting and AI/death state are not overwritten.
Dead/deleted/recycled actors cannot inherit a previous victim's collision.

The drill still delivers accepted native hits directly; removing physical
collision does not remove its HP loss, damage clock, native drops or Nanotech XP.
The centred pivot, stable aim pose and normal native death from v137 remain.

## Capture-owned motor sound

Empty fire is silent. The existing motor loop starts only after a living enemy
is captured, and it retains one voice while capture continues. Releasing fire,
defeat, loss of the allocation or another interruption releases that voice.
World cleanup also checks a dead/lost victim before the next hero item update,
so it cannot retain a motor because no further item update runs.

## Freezer detonation sound

The original sinking/freeze-death ice in `hero::fx::MobySpawn::Ice` plays class
`hero::damage::ICE_CLASS` (0x52a), sound **0**, flags **0**. Freezer detonations
now request that same one-shot at their actual impact centre. One detonation
plays once regardless of how many enemies it freezes. No ice actor or loop is
created just to own the sound.

The level audio loader retains native sound definitions when already present.
When needed for an enabled Freezer or live configuration session, it imports the
missing definition/sample from **Hoven (level 12)** through the existing cached,
transactional class-audio importer. No disc reads or source-bank parsing occur
when a projectile lands. Companion audio and other native banks remain intact.
A missing/invalid source bank is reported without disabling native level audio.

## In-game checks

- Capture small and large ordinary enemies near Ratchet. Keep fire held while
  turning: no body collision should push him away or break capture.
- Release while the victim lives; check it has physical collision again. Repeat
  after jumping, switching away and checkpoint death. Check normal damage/drops.
- Fire into empty space: no motor. Capture: one motor. Release or kill the
  captive: motor stops, including while continuing to hold fire after a kill.
- Detonate Freezer on the floor near several enemies and again in empty space.
  Hear one Hoven freeze sound per detonation, spatially at the impact. Repeat on
  Hoven and another planet; check normal music/audio and freeze duration remain.
- Swingshot movement/cable origin, Freezer equip gating, ammo HUD and timed
  regeneration retain the v137 behavior and settings.

See VERIFICATION.md for exact host checks and limitations. Windows linking, real
bank contents, audible output and in-game appearance require this local test.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, not ReRAC or the original games, is not affiliated with their developers or
rights holders, and claims no rights to the original games or assets. Thanks to
the ReRAC developers and contributors for the port that enables these mods.
Existing licenses, credits and rights notices remain applicable.
