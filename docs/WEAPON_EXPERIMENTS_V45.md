# Weapon Evolution v45 implementation notes

This is an independent mod for ReRAC 0.3.4-alpha. The mod author is not
an author of the port or the original game. All asset imports below use the
player's own extracted game data; no disc assets are distributed.

## Configuration and compatibility

The editable runtime reports `weapon_evolution_config = 45` and patch
`0.1.6.45` through `--version-json`. The launcher reports version `0.8.0`.
Each control belongs to one weapon grade (V1–V5). Standard fields precede the
experimental footer in both the INI and editor. `launcher/schema.py` is the
editor vocabulary and migration catalog; `weapon_evo.rs` validates and publishes
the matching runtime fields. Migration adds missing fields, preserves existing
balance/colors/XP, and does not rewrite stored presets.

Public profiles use the real-HP model. The launcher adds the engine's private
model flag only to its per-run INI. Damage and actual-loss XP are independent of
reaction/stun clocks. No temporary audio tracing is included.

## Drone Device companion

Enable `sand_mouse_enabled` in a grade and select `sand_mouse_mode`:

| Mode | Result |
| --- | --- |
| `1` | Sand Mouse replaces normal drones |
| `2` | Sand Mouse and normal drones are deployed together |
| `3` | Sand Mouse while the Sonic Summoner is worn; normal drones otherwise |

The master switch is off by default. Defaults are 90 seconds, 100 shots,
8 ticks between shots, 24 units of shot travel and 50 HP per shot. The color is
`default`; explicit RGB values include black. Time and shot budget are separate
limits; the first one reached begins retirement. All settings are snapshotted
at summon time, and shot damage/range/color are copied to each emitted shot.

`classes/mouse/companion.rs` reuses class 1818's flight, animation, joints and
vanish effects, with separate lifetime/shot controls. It spawns two units ahead
and two units above Ratchet at 1% of native class scale, reaching native scale
over 30 reference ticks. It never depends on owning or keeping the helmet.

The native `mouse::Globals::summoned` singleton is shared by the mod and real
dens. `mouse::idle` checks it before starting a native summon. A new Drone
Device activation returns false while a mouse is active, including during its
vanish, so the caller does not subtract ammo. A successful combined activation
uses one ammo unit in total. Stale/deleted references do not block future use.

Class 1633 shots carry an explicit port-only ownership tag. Only tagged shots
map to weapon 24; native Sonic Summoner shots are not reassigned. Damage and XP
use the same accepted-loss pipeline as the other mod weapons.

`rc-engine::companion_assets` loads only classes 1818/1633, their sequences,
joint lists and required textures from level 01 when either class is missing.
The work happens on the loader worker, is cached by data root, and is skipped
when no grade enables the experiment. `profile_uses_extra` parses the chosen
profile without publishing or resetting gameplay globals. Imports use runtime
texture mappings with usize indices, avoiding the disc table's u8 limit.
Neither summoning nor firing reads disc files. The native sound calls remain
native; this work does not change the runtime's audio system.

## Tesla Claw

`dual_beam_enabled` builds two independent copies of the original electrical
chain, including its waves, decorative strip, arcs, sparks and free firing.
The second ray prefers a different valid enemy. If both rays hit the same enemy,
`secondary_beam_damage_percent` scales only the second ray's HP/s; default 25.
Against a different enemy the second ray uses full base HP/s. The outer firing
update owns the single ammo/warm-up/light/hum clock.

Each ray owns target history and wave phases. No alternate straight-line beam
renderer is used. An empty target list still produces two visible native rays.
The two event channels prevent near-nozzle and targeted hits from multiplying
a ray's damage. Unconsumed stream contact is coalesced only within the same
source/channel/weapon and remains actual contact damage, preventing queues
from growing behind a slow enemy update.

## R.Y.N.O.

Standard controls add `homing_strength_multiplier`, `missile_max_range`,
`explosion_damage_radius` and `explosion_visual_size_multiplier` to the existing
speed/range multipliers and `fire_interval_ticks`.

- Homing 1 preserves native steering; 0 disables steering.
- Maximum range is path distance from the muzzle, including wobble. 0 retains
  the legacy envelope. Lifetime still bounds useful reach and auto-targeting.
- Damage radius 0 preserves native direct-impact-only damage. Set it above 0
  to add area damage; visual size is independent.
- Range/lifetime expiry detonates the missile. Collision is checked only up to
  the remaining travel budget. Obstruction at the muzzle retains its hit target.

The native explosion has no damage sphere. Previously the real-HP adapter
suppressed missile contact as a blast trigger, so the missile could explode
without producing a real-HP event. Detonation now owns direct and optional area
damage under one missile identity. This fixes the common path used by the
Rilgar/Blackwater City flame tank and other eligible enemies; native phase
shields and class state filters remain in place.

Two mutually exclusive experiments fully replace the native seven-missile salvo:

- `continuous_missiles_enabled`: one missile per interval while fire is held,
  one ammo per missile, cycling the nine barrels.
- `shotgun_enabled`: nine independent Blaster-model pellets per volley, one ammo
  per volley. Damage per pellet, speed, travel range, model/glow/trail size,
  projectile/trail/impact RGB and spread are independent of the Blaster.

Shotgun muzzle positions come from all nine attachment lists. Spread follows
each muzzle's horizontal/vertical offset from their average, with a 3×3 fallback
for missing geometry. Spread 0 produces parallel shots. No native missiles or
Gold Blaster ricochets are emitted alongside the shotgun volley.

## Mine Glove

Standard controls add `max_active_mines` (default 8), `mine_lifetime_seconds`
(default 30, starting at release) and independent flame/burn/explosion/light RGB
values. Exceeding capacity retires the oldest deployed mine without a damaging
blast. Lifetime applies to both native and experimental mines. Color choices
are snapshotted per mine rather than read from the held Pyrocitor.

`flame_mine_enabled` replaces native activation, pursuit, contact and explosion
damage. After landing, four native type-12 flame arms form a cross. Defaults:
range 8 units, rotation 90 degrees/second, damage 100 HP/second. Rotation 0 keeps
the cross fixed. World geometry clips each arm. Flame particle speed scales
with reach so longer configured arms also extend visually.

All four arms from one mine share a damage identity per contact tick. Different
mines remain independent. Accepted HP loss awards Mine Glove XP. The shared
three-second reaction gate never delays this HP loss. Enemy burn colors travel
with the attack and are committed only after accepted damage.

Every landed flame mine owns one spatial Pyrocitor-class loop and, when a slot
is available, one dynamic light. Loop ownership is the mine's moby ID, independent
of the player's Pyrocitor and other mines. Retirement releases only its own loop
and light (the light slot is checked against its allocation generation).

## Taunter

`stun_launch_distance` is a standard per-grade value. 0 retains native recoil.
A positive value schedules a short collision-checked push only after a
successful stun roll. Failed rolls and cooldown attempts do not move enemies.
The push runs during the existing stun window; native behavior resumes afterward.

## Launcher icon

The supplied `launcher.ico` is embedded into the Windows EXE and its Tk window,
included in source fingerprints and copied into player ZIPs. The updater backs
up changed sources and preserves an explicit local icon. If an icon is missing,
`icon_assets.py` can recover RT_GROUP_ICON/RT_ICON images from pre-update launcher
backups without executing them. The supplied ICO takes precedence over recovery.

## Validation and remaining gameplay checks

Automated checks cover per-grade parsing/bounds and mode exclusion, singleton
summons and growth, Tesla free firing and overlap damage, independent stream
sources, R.Y.N.O. terminal damage/range expiry and the tank's native HP handler,
INI preservation, icon recovery, source transactions and launcher packaging.
The engine has also been type-checked for Windows from Linux, using the pure
Rust blake3 backend in an isolated validation checkout because MSVC assembly
tools are unavailable on the validation host. Distribution builds retain the
original dependencies and use the normal Windows toolchain.

The actual Windows game and frozen launcher must be built using the included
BAT on the authoring PC. This environment has no extracted disc data or GPU game
session. In-game checks still needed: companion visibility on a planet without
a native den, all three summon modes, simultaneous flame mines/Pyrocitor,
shotgun barrel placement, and two Tesla rays on one or two enemies.


## v45.1 locked vertical aim and installer correction

`follow_camera.rs` keeps an optional requested combat pitch separately from the
collision-resolved orbit. It initializes from the current view when locking,
changes at the menu-selected speed with the existing ten-tick input filter,
and clamps at +/-85 degrees. Unlock, reset and scripted camera transitions
clear this state through the existing combat-stick reset path.

The free orbit accounts for the pivot/look-height difference. Locked rows use
the requested pitch, even when floor/wall collision shortens the camera arm.
The blocked-line fallback retracts the arm rather than invoking the native
30-tick whole-camera reset while locked. Collision contraction is immediate;
outward distance recovery retains the spring. The pole margin is five degrees
only while locked; the native unlocked pole margin and pitch behavior remain.

The regression test runs both vertical extremes at all three menu speeds over
a real hand-built collision floor. It checks 85-degree output, floor clearance,
holding at release, finite rows and continued yaw control. The full game library
suite also covers region trigger isolation, normal camera behavior, input
ramping and scripted camera lifecycle.

The v45.1 author installer verifies the shared native creature-test prefix,
adds `burn_color:None` to recognized fully qualified flat `real_hp::Attack`
test literals and retains earlier appended tests/fixtures in their existing
order. It preserves LF/CRLF and UTF-8 BOM. Unknown native prefixes and unsupported
nested literals fail preflight. Restore derives the expected upgraded file from
the full original backup and refuses later developer edits. Gameplay source
files still require an exact known normalized hash. Only the 37 changed files
are installed; the cumulative developer patch remains based on published v40.

Runtime identity is `0.1.6.45.1`; schema capability remains `45`, and the launcher
remains `0.8.0`. No audio probes, balance changes or new INI settings are added.
