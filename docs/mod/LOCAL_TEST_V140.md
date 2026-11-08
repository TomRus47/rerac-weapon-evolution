# v140 — final-frame grapple release recovery and half-unit drill clearance

ReRAC **0.3.8-alpha**, gameplay **0.1.6.140**, capability **97**;
installed launcher **0.30.2** retained. This complete source kit includes all
preceding fixes. INIs/defaults, balance, presets, save formats, seeds and icon
remain unchanged. The kit performs no GitHub operations or release creation.

## Build and test

1. Extract the complete ZIP to a new folder. Close the game and launcher.
2. Run `01_COMPILE_AND_UPDATE.bat`. Select the existing playable mod directory
   with `weapons.ini`, the launcher EXE and `runtime/rerac.exe`, not this kit's
   source folder. No previous test-source directories are required.
3. This is a runtime-only update: tests, compilation and exact identity checks
   finish before runtime/shaders are replaced. Keep the working launcher/icon;
   no manual launcher rebuild is needed. Failed builds retain playable files.
4. `02_OPEN_UPDATED_LAUNCHER.bat` opens that installation.
5. After success, `03_CREATE_PLAYABLE_ZIP.bat` packages the updated runtime,
   retained launcher and player INI, excluding extracted game data.

## What the earlier handoff did not cover

The v139 recovery ran inside HeroUpdate and recognized ordinary fall/aim states.
Custom pulling can still be active in native jump, running-jump, double-jump or
pack-hover states. Its exit could clear pose ownership without changing those
states to fall. Separately, native deferred hand-item updates execute after that
early check and can issue another SetState/SetAnim before the final render pose.
An isolated HeroUpdate test cannot establish the final state of that whole tick.

## Complete release lifecycle

A transient cable-exit marker now survives the early handoff and the deferred
item pass. For an airborne custom release it enters native fall, retaining the
cable's actual velocity, momentum and effective displacement. Native fall owns
gravity and subsequent falling/landing. The marker remains scoped to this release
until landing/interruption; it is not persistent save state.

The final check runs **after** native `gadgets::after_items` and custom item
updates. It recovers late idle/invalid pose writes rather than depending on a
pose-owner flag which the early handoff already cleared. It selects a fully
decoded native fall key with A=B, without interpolating an old source snapshot,
then allows native playback to advance normally. It does not pin/reset a valid
fall every frame. Original native Swingshot targets, new jump presses, damage,
water/body changes and other interruptions relinquish the recovery.

After a correction the hero's animation fields, modifier layers, write-back,
Clank's cached back chain and the hand-item attachment are refreshed together.
The caller rebuilds the published hero matrix. The hand refresh evaluates its
pose only: it does not advance item animation or emit its triggers a second time.
An existing retracting cable receives the refreshed real muzzle position too.
Normal native animation callers and ordinary item-update advancement remain.

## Trespasser placement

The additional model-centre clearance is now **0.5 game unit**, replacing v139's
1.0 unit. Its existing size-dependent tip offset remains. The enemy is therefore
half a unit closer than v139, while retaining a fixed compensated rotation centre.

Physical-contact-only acquisition from v139 remains. A captured enemy still has
body collision suppressed until release; living victims recover their original
collision state and dead/recycled instances cannot inherit an old lease. Motor
ownership, accepted damage/XP and the Hoven frost detonation sound are unchanged.

## In-game checks

- Grapple an elevated wall from the ground, a jump, a running jump, double jump
  and pack hover. Release halfway through travel and watch the entire descent
  with normal camera, camera lock and L1 aiming. Check a native fall pose and
  Clank's correct attachment until landing; check normal running afterwards.
- Repeat short taps, repeated pulls and release during hook retraction. Confirm
  that a fresh jump press and native blue/yellow targets still behave normally.
- Capture differently sized ordinary enemies with Trespasser. Check the new
  half-unit additional clearance, fixed rotation centre, no body push and proper
  release/death/audio/XP. Acquisition must still require physical contact.

See VERIFICATION.md for exact checks. The new fixture runs complete Game ticks
with native hand updates and a deferred native idle request, in addition to
palette/missing-key and no-double-advance checks. Real disc assets, Windows
linking, GPU appearance and audible output still require the included local test.
No visual playthrough is claimed by the preparation host.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, not ReRAC or the original games, is not affiliated with their developers or
rights holders, and claims no rights to the original games or assets. Thanks to
the ReRAC developers and contributors for the port that enables these mods.
Existing licenses, credits and original-game rights notices remain applicable.
