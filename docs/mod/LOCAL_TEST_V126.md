# Weapon Evolution v126 local test — ReRAC 0.3.8-alpha

Local test only. No GitHub operations or release publication.
This incremental update uses your verified **v125 Dual Quick Select test kit**.
Its two saved rings, pause option, launcher 0.29.7, icon, INI values, presets,
randomizer seed/layout and existing saves are retained. Configuration capability
remains 94 and save companions remain WEVO v10.

## Install

1. Install/test v125 successfully first. Keep its extracted test-kit folder.
2. Extract this v126 ZIP into a separate folder. Close the game and mod launcher.
3. Run **01_COMPILE_AND_UPDATE.bat**. At the FIRST prompt select the extracted
   **ReRAC_Weapon_Evolution_v125_Dual_Quick_Select_Test** folder containing
   `source-files.json` and `source`. Selecting its `source` folder also works.
   Do not select the playable mod, `source/launcher`, or an older source checkout.
4. At the SECOND prompt select the EXISTING playable mod folder containing
   `weapons.ini` and `runtime/rerac.exe`. This runtime must already report v125.
5. The updater checks the v125 source inventory, applies exact source contexts to
   a private copy under `work/`, runs Rust checks/build and installs the runtime
   with matching shaders only after success. The original source is retained.
   The working launcher EXE/custom icon is retained and does not need rebuilding.
6. Use **02_OPEN_UPDATED_LAUNCHER.bat** to open that same installed launcher.
   **03_CREATE_PLAYABLE_ZIP.bat** packages the successful updated installation.

This is a source update/build kit, not a precompiled Windows runtime. Existing
Rust, Visual Studio x64 C++ tools/Windows SDK and extracted game data are needed,
with the same automatic tool checks as v125. No original game data is included.
A failed compile keeps the previous installed runtime. Read the console error
and `work/v126-*/logs/`; outer updater errors are in `logs/update-error.log`.

## Walloper hand

The body draw adds a composing zero scale at Ratchet's authored palm branch
(joint 55, the first GLOVE_JOINTS entry). Fingers collapse with that branch while
the wrist parent remains visible. It is a render-only modifier, rebuilt on every
upload. Gameplay pose, hitboxes, hand-item attachment, weapon geometry and other
weapons/alternate bodies are not modified, and no hide state survives a swap.

Check idle, lunge, jumping, locked movement, gold texture, changing weapons and
death/reload. The cuff should contain the palm/fingers, with the wrist visible.

## Hoven and underwater Pokitaru water

The water shader previously converted each layer to linear and then blended it
in linear light. Its native FIX equation blends display bytes. Both strip and
ripple layers now use the shared display-byte effect target and premultiplied
FIX blending, as the runtime's other effect draws already do. Positions, height
simulation, UVs, texture selection, fog values, collision and opacity are retained.

The vertex/fragment shader is parsed and validated with Naga 29.0.4. A Linux
compile/type-check does not validate the Windows GPU result: the reported Hoven
puzzle appearance and purple underside on Pokitaru still require an in-game
comparison. This is the rendering correction to test, not a claim that every
possible water artifact has been reproduced or eliminated.

## Batalia turret / Accuracy Counts

An intact, unoccupied turret can be mounted after the main mission is completed.
The mission still gates its completion dialogue/reward. Triangle and the normal
quit request leave the seat without forcing Ratchet's death after completion.
Existing exit distance/cooldown remain; the native fighter hit/skill-point path
is unchanged. Test re-entering, shooting a small fighter and leaving the seat
both on the same visit and after revisiting the planet.

## Veldin II / Going Commando

The native counter was already implemented. An independent wrench hit records
Ratchet's physical origin for position/knockback; interpreting that origin as a
foreign weapon cleared the elite guard's wrench-only flag. The guard now uses
the weapon event's identity and the native wrench marker for skill credit.
Ten qualifying elite-guard kills award the existing skill point/jingle once.
The earlier non-wrench-hit rule remains: even quiet flamethrower/Tesla damage
invalidates that guard before a later wrench finishing blow.

Test with ten previously undamaged elite guards on Veldin II. Other enemy types
do not count. Guards already hit by another weapon remain ineligible; this update
does not retroactively change their damage history or grant the skill point.

## Host checks

- 5 focused runtime regression tests passed, including actual pose evaluation and
  native guard/turret update paths.
- 1,428 rc-game library regression tests passed with locked dependencies and no
  default features. Three unchanged, expensive seed-generation surveys were not
  repeated for these equipment/render/skill fixes.
- rc-engine compile/type checking passed on the Linux host with locked dependencies,
  no default features and workspace-local native build metadata; this is not a
  Windows executable or GPU/playthrough test.
- The actual water vertex/fragment shader plus its shared blending functions
  parsed and validated with Naga 29.0.4, matching the runtime's locked version.
- 6 updater tests passed with both synthetic and reviewed source-file fixtures.

## Verification and source review

The included `changes.patch` is the readable six-file diff; `patches.json` holds
exact contexts applied by `update126.py`. Each source file is validated against
the v125 kit inventory before copying. All patch contexts are checked before any
prepared source is written. The installer rejects unknown source/old runtimes.
The prepared copy gets a fresh full inventory and reviewed source hashes.

The runtime changes have focused tests for actual hand-pose evaluation, native
and evolution hit identity, real guard damage/death/counting, foreign stream
qualification, and completed-mission mount/exit/cooldown. `test_update126.py`
checks preparation, retained v125 pause/ring markers, source isolation, identity,
unknown contexts and unsafe paths. Windows gameplay/visual checks remain manual.

The mod author does not own the original Ratchet & Clank games or assets and is
not involved in developing ReRAC or the original games. Their rights remain with
the respective rights holders. Credit and thanks to the ReRAC developers whose
runtime and source make this independent mod possible.
