# Local v56: wrench progression layout, hit radius and enemy push

Unpublished local test patch 0.1.6.56 for ReRAC 0.3.4-alpha. Launcher 0.16.0,
configuration capability 56. The local updater does not access GitHub.

## Player-visible changes

OmniWrench 8000 is immediately below Global Settings and above Bomb Glove.
Its four existing upgrade requirements are edited together in **XP progression**,
using the same V1→V2 through V4→V5 strip as every other weapon. Grade cards no
longer contain duplicate XP controls or an otherwise empty progression panel.
Existing requirements are not reset or recalculated.

Each `[weapon.wrench.level1]` through `.level5` adds two standard controls:

| Key | Default | Accepted range | Meaning |
|---|---:|---:|---|
| `hit_radius_multiplier` | 1 | 0.1–10 | Scales damaging collision reach/thickness |
| `knockback_multiplier` | 1 | 0–10 | Scales native horizontal enemy push |

Examples: radius 1.5 increases contact reach/thickness by 50%; push 2 doubles
horizontal push speed; push 0 removes horizontal push while retaining damage and
native vertical response. Physical travel distance depends on each enemy's
native drag, gravity and collision response; the value is a strength multiplier,
not a guaranteed distance in units. Native immunities and phase restrictions remain.

These settings apply independently at each grade to all three ground combo swings,
the Magneboots swings, aerial wrench contact, the ground-slam impact, and both
outbound and returning Comet Strike. Throw travel distance and speed remain
controlled by `throw_range` and `throw_speed`. The 3D model, animation pose, player
movement, effect anchors and the separate crate-only `crate_breaker_radius` do not
change. Damage and XP settings retain their previous values.

INI migration adds only missing fields at 1 and preserves every existing value,
including custom XP thresholds and all named presets. Explicit multiplier edits
are preserved. The normal schema ordering and color/experimental grouping remain.
Field labels and wrench-specific help are available in all five launcher languages.

## Runtime implementation

`weapon_evo::wrench::AttackTuning` captures both multipliers at attack entry and at
actual throw release. Each combo swing has its own capture. The detached wrench
stores a copy in `hero::comet::Flight`. A grade earned during a swing/flight therefore
does not change that attack's geometry or push partway through it. The next attack
captures the new grade. No new save fields or save-format changes are required.

- `hero/melee.rs::wrench_update`: scale the collision head and previous head about
  their corresponding native hand positions, then perform the existing five-line
  sweep. Scale the tip sphere's radius and hand-relative center. Store native hand
  and tip history unchanged, and keep cosmetic wall probes on the native pose.
- `hero/melee.rs::jump_attack_shockwave`: multiply the native 0.4 impact radius,
  preserving one-shot consumption of the queued slam. Crate Breaker is separate.
- `hero/comet.rs::thrown_update`: multiply the native 0.4 contact spheres on both
  outbound and return travel using the captured flight tuning. Flight, catch,
  bounces, collection and disappearance remain native/current v55 behavior.
- All three paths scale only `HitTemplate.dir[0..2]` (X/Y). Z and the native
  exact-push marker W are preserved. The creature resolver already reads the
  magnitude via `creature::knock::aim`, so native reaction selection, vertical
  launch and enemy-specific resistance stay intact. Damage/XP do not use this
  multiplier. This introduces no generic position warp or extra damage event.
- Identity branches return the original PS2 float values directly when the
  multiplier is 1. A missing INI entry/disabled profile uses those identity values.

Parser and editor limits are scoped to the wrench. Walloper has identically named
settings and retains its earlier bounds, balance and consuming hooks. Capability
56 is required for wrench profiles containing either new key; v55 is rejected
rather than silently ignoring the settings. Old files still parse with identity
behavior when the new keys are absent.

## Source map

- `crates/rc-game/src/weapon_evo_wrench.rs`: tuning helpers and runtime regression fixtures.
- `crates/rc-game/src/hero/melee.rs`, `hero/comet.rs`: capture and collision/push consumers.
- `crates/rc-game/src/weapon_evo.rs`: supported keys and wrench-specific validation.
- `launcher/schema.py`, `migrate_ini.py`, `launcher_core.py`: order, defaults,
  lossless value migration, validation and capability routing.
- `launcher/launcher.py`: shared four-field XP strip for all weapons.
- `launcher/i18n.py`: localized wrench-specific explanations; existing labels reused.
- `launcher/tests/test_v56.py`: actual XP renderer, navigation/INI order,
  ten-field migration, custom grades, validation and translated help.

## Verification and manual test

Rust fixtures exercise the real in-hand updater for each combo swing, Magneboots
and aerial contact; they verify native joint/model positions and damage remain
unchanged while enlarged collisions reach a previously missed target. Separate
fixtures exercise actual slam consumption, native knockback resolution and both
throw directions with a different held tuning to prove flight snapshot isolation.
Parser tests cover independent five-grade values, identity defaults and invalid
bounds. Python coverage checks that each of the sixteen weapon pages actually
creates four XP entries in the shared strip, with none in grade cards.

On Windows, run `01_UPDATE_BUILD_AND_OPEN.bat` from the new test ZIP. Choose the
existing source and current mod folders so personal INI/preset values migrate.
Later use `02_OPEN_TEST_LAUNCHER.bat`; the old frozen EXE is not rebuilt by this
local test kit. Confirm the new first weapon button and the four XP fields. Then
compare radius 1 versus 2 and push 1 versus 0/2 against a knockback-capable enemy,
using all three swings, throw outbound/return and a ground slam. Check that an
upgrade during a throw affects the next throw and that Crate Breaker remains
controlled by its own setting. Real Tk layout, Windows engine compilation and
disc-data gameplay still require the author's local test before any publication.

Automated validation on 2026-10-21: 1013 Rust game-library tests and 119 Python
tests passed. The source-export checker passed with 1228 source files, 254
indexed mod files and 1359 public INI entries. All 1349 v55 values and the custom
ICO bytes were preserved. Windows engine compilation and live gameplay were
not performed in this preparation environment.
