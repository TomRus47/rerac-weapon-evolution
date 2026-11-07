# v134 local gadget corrections — ReRAC 0.3.8-alpha

This complete source kit builds gameplay patch `0.1.6.134`, configuration
capability 96. It includes the preceding v133 gadget additions and v132 shop
fixes. No GitHub operations or release publication are performed.

## Installation

1. Extract the entire ZIP into a new folder and close the game and mod launcher.
2. Run `01_COMPILE_AND_UPDATE.bat`. Select the existing **playable mod folder**
   containing `weapons.ini`, the launcher EXE and `runtime/rerac.exe`. Do not
   select the extracted source kit. No previous source patch is required.
3. The Windows installer verifies the bundled source, runs Rust checks, builds
   and probes the new runtime before replacing the playable files. Keep the
   window open and retain the log path if a step fails.
4. This update retains the installed v133 launcher 0.30.0 and icon: the gadget
   pages and their settings are unchanged, so rebuilding that EXE is unnecessary.
   Run `02_OPEN_UPDATED_LAUNCHER.bat`, enable the gadget features you want and save.
5. After a successful update, `03_CREATE_PLAYABLE_ZIP.bat` packages that runtime,
   your current `weapons.ini` and the retained launcher. Original game data are
   never included.

INI balance, presets, save formats and randomizer layouts are retained. Existing
saves can be continued. The gadget functions remain opt-in and do not grant
ownership. See [the v133 configuration reference](LOCAL_TEST_V133.md) for keys,
water-tank-independent ammunition and the unchanged Hologuise/Lavagun additions.
The corrections below supersede v133's UID-based lease and drill-tip descriptions.

## Actor lifetimes and the rotating-enemy leak

ReRAC increments the low 16 bits of `Moby.uid_hi` whenever it rebuilds a model
matrix; the high half identifies an array slot. This is not an allocation ID.
The earlier gadget leases incorrectly treated a matrix update as an actor's
replacement. A held enemy could therefore generate a projectile clone each
frame, and the same mistake could release those clones from expiry management.
Enemy hooks and freeze leases also lost their targets after a matrix update.

Gadget references now use slot, class and a separate allocation generation.
`Services::gadget_lifetimes` advances at all three production moby allocation
entry points and resets together with transient level services. It is separate
from `Runtime`, which is temporarily moved out during gadget processing. Native
matrix revisions, renderer identities and the on-disk save format are untouched.

Trespasser conversion now requires a confirmed defeat of the same allocation:
committed HP reaching zero or the existing terminal-damage ledger. Script
removal or slot replacement cancels capture without inventing a kill. A confirmed
defeat creates one managed projectile; collision, timeout and death clean it up.
A living captured enemy remains a single actor through repeated matrix updates.
The speculative separate red-tip joint modifier has been removed: the tip stays
still, while the configured rotation continues to rotate the captured enemy.

## Swingshot pose, first-person ascent and enemy pulls

Custom cable launch/retraction uses the native Swingshot launch pose; surface
travel uses its pull pose. The active hook follows the cable endpoint and remains
visible while aiming with L1. Native launch and contact sounds are reused.

Surface travel supplies a step to Ratchet's normal movement/capsule resolver,
after native look-mode braking and before collision integration. This prevents
L1 from cancelling upward travel. It does not teleport Ratchet after collision
or introduce yellow-target hanging states. Release immediately stops the custom
step. Native Swingshot targets still take priority, and floor surfaces are still
excluded. Enemy pulls retain their target across matrix updates; bosses and
flying ships remain excluded.

## Hydrodisplacer launch and frost impact

The nozzle position is accepted only when it is finite, near Ratchet's chest and
above his feet. A missing/root-level/below-ground hand socket falls back to a
point above the ground in front of his chest. L1 and combat-camera aim supply the
launch direction rather than a fixed horizontal root position.

A swept terrain contact, maximum range or flight timeout creates one frost burst
and freezes targets within the configured radius. The impact now also draws a
bounded translucent filled blue sphere, using `explosion_color`, with the same
centre and maximum radius as the freeze query. The portable crystal and mist
remain; no damaging bomb actor or point light is created. Animation and AI freeze
leases survive ordinary matrix updates and expire on their simulation clock.
Already fired hostile projectiles are unaffected; native boss phase shields and
accepted damage handling are preserved.

## In-game checks

- On Novalis, aim at both a shooting robot and a small ground enemy. Hold fire
  with the Swingshot; verify visible hook/rope, pulling, arrival stun and retraction.
- Grapple an elevated wall or ceiling while holding L1. Verify upward movement,
  launch/pull animation, release, corner collisions and normal native targets.
- Hold the Trespasser on a living enemy for several seconds with low damage.
  Check that only one enemy rotates. Release, kill, change gadget, die and reload;
  no trail of frozen copies or orphan motor loop should remain. On a kill, the
  single enemy projectile should disappear on contact or timeout.
- Fire Hydrodisplacer ice with empty and full tanks at enemies near actual floor.
  Check visible flight, a blue contact burst, frozen pose, damage while frozen
  and thaw. Check native water pads still take precedence.

## Validation limits

Regression fixtures use nonzero level coordinates, actual terrain triangles,
native Novalis robot updates, repeated matrix rebuilding and Ratchet's real L1
physics path. The kit's verification record lists the completed checks. Windows
linking, real asset playback and GPU/in-game appearance cannot be exercised on
this preparation host; the included installer performs the Windows build before
replacing playable files. Visual confirmation remains part of this local test.

Weapon Evolution is an independent mod by TomRus47, who does not develop ReRAC
or the original games and is not affiliated with their developers or rights
holders. No rights to the original games or assets are claimed. Thanks to
ReRAC's developers and contributors for the port that makes these modifications
possible. Their licenses and credits remain intact.
