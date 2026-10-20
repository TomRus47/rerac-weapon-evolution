# v44: crate ammunition, Tesla beams and configuration layout

Runtime capability: 44. Launcher: 0.7.0. Base: Weapon Evolution v43 on
ReRAC 0.3.4-alpha. All options are independent for grades 1–5.

## Permanent configuration policy

Standard fields precede an experimental footer in every grade. Each experiment's
switch appears before its parameters, after all standard fields. The editor uses
`field_groups()` and migration uses `EXPERIMENTAL_DEFAULTS`, so registering future
experiments follows the same rule. Held Walloper duration remains a standard field.

Migration keeps user values and existing on/off choices. The old mini-bomb
defaults 35% / 6 units/s become 40% / 3 units/s. Other size/speed values survive.
Legacy jump height becomes `sqrt(2 * 11 * height)` initial vertical speed.
Legacy symmetric 33-degree flame jets become separate 22-degree left/right jets;
other symmetric angles are copied to both fields. Old runtime keys remain accepted
for compatibility but are removed from the normalized editor profile.

## Mini bombs and flame geometry

Mini bombs bypass the normal bomb update's visibility gate. Their spawn explicitly
clears hidden/no-update/no-animation/kept-matrix flags, sets visible/alpha and builds
the matrix. Model scale, count, planar speed and initial upward speed are configurable.
Default model scale is 40%, planar speed 3 units/s and initial upward speed 4.2 units/s.
The existing 15% blast size, 10% damage and per-grade color snapshots remain.

Pyrocitor has independent `side_flame_left_angle_degrees` and
`side_flame_right_angle_degrees`, default 22. Its lane-local retained particles,
shared ammo clock and per-target damage deduplication remain unchanged.

## Crate ammunition

`classes/cannon_crate.rs` appends its record after the native 0x100-byte crate pvars.
The original instance keeps its class, reward bytes, spawn/save metadata and
nanotech-cluster ownership. The normal cannon slots, capacity, fire velocity and
homing target are reused. Suction detaches the stack without breaking the crate.
Interrupted suction restores collision and native crate behavior.

Only idle crates are taken; an already triggered TNT fuse is not cancelled.
Wood, TNT, steel, ammo and nanotech each have a per-grade allow switch under
`crate_suction_enabled`. Stored crates remain usable after changing grades/options.

Fire snapshots the configured cannon HP. Wood/ammo/nanotech use that damage;
steel uses twice that damage. Nonexplosive impacts use the normal cannon projectile
hit kind, flags and class. TNT uses one cannon-attributed blast (radius 3), without
an additional direct-contact damage event or the native repeated TNT sphere.
Accepted enemy damage and XP go through the existing real-HP resolver.
Destruction invokes native crate break/drop handling once, then deletes the source.
The original crate model remains visible in flight. Nanotech icon particles are
paused while swallowed and released by the original cluster after destruction.

## Tesla

Standard `beam_size_percent` controls player beam width (100 = native).
`beam_range_multiplier` multiplies the existing target-range setting and is shared
by visible reach, target retention and hit delivery. Environmental Tesla copies
keep their native width and colors.

Experimental `dual_beam_enabled` replaces the chain with two independent nozzle-to-
target rays. Each slot reserves a different living target, within the cone/reach
and with a clear line. A ray ignores the other ray's reserved enemy during its
trace. Each selected enemy receives normal damage from one ray. With one enemy,
the second ray has no target and adds no damage. The outer native ammo clock runs
once, regardless of ray count. Native gold chaining remains when the experiment is off.

## Eudora saw bots

The class-340 brawler/saw bot has a private `PUSH` slide velocity independent of
the shared knockback system. Blaster/Pyrocitor/Tesla accepted hits clear that
velocity before quiet settlement and never set a new slide impulse. Native
health, alert, flash, death and other weapon responses retain their code paths.

## Loading investigation

The supplied v43 run log reports 11.1 ms for audio loading, 2291.4 ms in the
moby mesh-build span and 2432.5 ms for the final level swap. Other audio loads in
the same run were about 23 ms. These figures do not demonstrate an audio regression.

The mesh-build span invokes the first class-port recognition through `level_ports()`.
Camera and reaction recognition were also lazy, on initial gameplay setup. v44
prepares these immutable tables on the level loader worker before it reports ready,
and reuses parsed reference overlays. Spawn preparation now explicitly uses the
destination level, rather than the old active-level global on that worker.
The operation changes no save, game RNG or audio state. This removes that lazy
recognition work from the final swap/first gameplay setup, but does not eliminate
GPU shader compilation or all scene-upload costs. Compare final-swap and mesh-build
timings on the same route after compiling. No interactive performance result is
claimed from the preparation environment.
