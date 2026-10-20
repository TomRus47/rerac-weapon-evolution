# Weapon Evolution v47 implementation notes

Independent mod for ReRAC 0.3.4-alpha. Runtime patch identity is 0.1.6.47;
configuration capability remains 46 and launcher version remains 0.9.0.
No configuration fields or balance values change.

## Locked camera self-collision

The native class-1007 helper blocks Ratchet from walking through the camera.
Below 2.5 units above the feet, its update usually parks it away; above that
threshold it stays at the previous frame's camera position. A downward orbit
raises the camera and activates this case. The v46 locked collision path did
not apply the native primitive exclusions, so the camera could hit its own
helper. This explains position changes without an accompanying yaw change.

`Camera::update_type0` now wraps the input moby scene in the existing read-only
`MaskedMobys` view while combat lock is active. Only `cam_moby` is masked,
without mutating the table, grid or collision cache. The wrapper covers pivot
and ceiling queries as well as final arm avoidance; masking only the endpoint
sphere would leave another source of vertical feedback. Scenery, other mobys,
85-degree pitch limits, ten-frame input ramps and menu speed remain unchanged.
The unlocked path receives the original scene.

The regression fixture places a collision-enabled helper at the last published
camera position above the native threshold, looks down, holds, then returns to
the horizon. It compares every frame against the same world without the helper.
Before the fix it fails at tick 10 with about 0.84 units of position difference.

## Experimental R.Y.N.O. animation lifecycle

Both experimental modes bypass the native salvo state 4, whose completion
normally calls `weapons::put_away`. Each successful experimental shot queued
`pending_draw` with no matching cleanup, leaving the persistent firing stance
active after release. An early cooldown return also skipped any release work.

`experimental_fire` advances its cooldown, checks trigger/block/ammo state and
calls the existing `put_away` before the cooldown return when firing ends.
This clears a queued draw, releases the persistent stance and uses the existing
idle blend or arm-layer fade. The timer is not reset, so re-pressing cannot
accelerate the configured firing cadence. Native salvos retain their own code.
The 7-by-5 grid, pellet properties, ammo cost and firing sounds are unchanged.
Tests cover release during cooldown, a quick re-press, a blocked held trigger
and empty ammo.

## Update boundaries

The incremental author installer expects the v46 versions of three Rust files:
`follow_camera.rs`, `hero/ryno.rs` and engine `disc_source.rs`. It verifies all
payloads and source revisions before writes and supports backups/reapply/restore.
It does not replace creature tests or the generated embedded INI. The active
external INI, presets, settings and existing custom icon are preserved.
The developer patch remains cumulative against published v40, not an incremental
patch to apply to the installed v46 checkout.

No audio probes or audio capture are introduced.
