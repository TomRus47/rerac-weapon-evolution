# Weapon Evolution v46 implementation notes

Independent mod for ReRAC 0.3.4-alpha. The mod author is independent of ReRAC
and the original game's developers. Product strings and developer docs are English.

## Locked camera ownership

`follow_camera.rs` owns requested `combat_pitch` and horizontal `combat_heading`
separately from `cam.off`, the physical camera arm. Stick input keeps the existing
ten-tick filter and menu speed. Lock acquisition initializes from the current
view; unlock/reset clears the request state. Pitch remains limited to +/-85 degrees.

The ideal arm is rebuilt from captured distance and pivot/look heights each tick.
The previous collision-shortened distance never feeds the pitch-height correction.
`avoidance_locked()` clips the line and camera sphere along this requested arm;
it changes distance only, without native sphere-chain re-aim, moby yaw nudges,
minimum-distance extension or the blocked-camera reset. Placement contracts
immediately at an obstacle, recovers outward through the distance spring and
uses the same pivot as the collision query. View rows use the requested heading,
so terrain displacement cannot steer the aim. Unlocked camera collision remains
native. Tests check floor clearance, stationary position/yaw at both vertical
extremes, a nearby wall, menu speeds, release behavior and native region lifecycle.

## Flame mine collision

`mine/experimental.rs::flight()` tests the centre and lower hull with two-sided
world/platform triangle collision. The lower hull has a 0.25-unit radius; sphere
contact also handles grazing faces. Walkable downward contacts land the mine
0.4 units above the face and stop velocity; wall/ceiling contacts reflect it.
Water retains retirement behavior. The class collision volume is enabled after
landing when available. Terrain collision does not depend on a class primitive.

Landed mines keep their existing independent four flame arms, per-mine light and
spatial sound ownership, lifetime, colors, DPS and Mine Glove XP attribution.
Normal mine physics is unchanged. Tests cover ordinary and high-speed descent,
activation, stationary landed position and missing class collision metadata.

## R.Y.N.O. shotgun grid

`hero/ryno.rs` fires 35 independently snapshotted Blaster-model pellets per volley:
7 columns and 5 rows. `shotgun_cell()` returns coordinates x=-1..1 at one-third
steps and y=-1..1 at half steps. The centre is exactly [0,0]. A camera/weapon-aligned
orthonormal basis makes muzzle offsets and directions follow steep aiming too.

Origins interpolate the original muzzle plane, with minimum separation scaled
to pellet size. The two new fields are experimental options at the end of each
grade, following its standard fields:

- `shotgun_horizontal_spread_degrees`: maximum sideways angle of the outer columns.
- `shotgun_vertical_spread_degrees`: maximum vertical angle of the outer rows.

Intermediate offsets scale with their distance from the central column/row.
Directions use independent tangent components and are normalized to preserve
configured pellet speed. The exact central pellet is unaffected by either spread.
With both values zero, all 35 pellets remain parallel with distinct grid origins.
Per-pellet damage, speed, range, colors and one-ammo-per-volley charging remain.
Consequently a full 35-pellet hit has a higher total potential damage than the old
nine-pellet volley; damage is not silently divided or rescaled.

The launcher migration maps `shotgun_spread_degrees` to the new horizontal field
and initializes vertical spread to 3 degrees. Explicit new axis values win.
Runtime still accepts the old key as a compatibility horizontal fallback; the
public INI/editor expose the new independent fields. Both axes allow 0..60 degrees.
Their presence requires runtime configuration capability 46, preventing a v45
runtime from silently ignoring the new controls. Ordering is enforced for every
grade by the shared experimental catalog.

## Transient upgrade notices

`weapon_evo_notice.rs::State` is shared by the renderer and CPU regression tests.
It owns only seen levels, pending announcements, the active popup and the last
observed game tick. Death, disabled notices, a rewound game tick or restored lower
levels clear transient state. Repeated identical ticks preserve pause duration.
Subsequent real upgrades still queue and play their announcement normally.
`hud_render.rs` uses this state and detects zero health / lethal falling states.
The state never edits weapon levels, XP or native save bytes.

## Author updater and compatibility

The v46 delta contains seven changed/added Rust files and an optional default
embedded INI. It does not replace `creature/tests.rs`; retained older/nested tests
stay intact. Existing `weapon_evo_release.ini` is preserved byte-for-byte because
this kit builds with `--no-default-features` and reads the selected launcher's
external INI. Only a missing embedded profile gets the verified package default.
Other Rust source files still require known hashes before any source writes.

The Windows tool backs up sources and the selected launcher/INI, migrates the
active external INI, runs release tests/build, stages the verified runtime and
builds launcher 0.9.0. Presets/settings/reference INIs and an existing custom icon
are preserved. There are no audio probes or disc/model assets in the update.
