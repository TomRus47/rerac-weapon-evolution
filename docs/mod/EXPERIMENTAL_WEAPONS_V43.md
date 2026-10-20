# v43 implementation notes

## Runtime paths

- `rc-formats/moby_anim.rs`: snapshot_layered reuses the native local snapshot
  encoding and applies the same pose layers used by the model evaluator.
  Empty layers preserve the native path.
- `rc-game/hero/worn.rs`: pose_from_host_layered retargets those local keys to
  a replacement boot. The boot's attachment transform owns model-space yaw.
- `rc-engine/moby_attach.rs`: evaluate only the two foot chains with the lower
  body layers/yaw. Other attachments keep their native placement. The same lower
  body layers feed the boots' copied local pose, with no foot-hiding modifiers.
- `rc-game/hero/walloper.rs`: extend propulsion and hit-window clocks together,
  pause the extended pose, restore animation speed on release/limit and preserve
  one attack_serial for the entire punch. The native 100% path stays unchanged.
- `rc-game/moby_update/real_hp.rs`: per-source HP overrides snapshot mini damage.
  Clearing/recycling a source clears its override; copied hit records retain
  their already computed damage and immutable origin.
- `creature/damage.rs` and `classes/path_enemy.rs`: include weapon ID 15 in the
  existing v42 accepted-hit quiet-settlement gate, alongside IDs 16/19. Damage
  and XP keep their own clocks. Native source/state/phase filters run first.
- `classes/bomb.rs`: appended pvars identify a child and snapshot radius, damage,
  color, gold and size. Children use their own swept flight/arming path without
  contact hit templates. Only detonation owns damage. Spawn before parent blast;
  no recursive clustering, no ammo writes, finite lifetime and graceful pool failure.
- `hero/pyrocitor.rs`: three lane-local 15-slot retained-flame banks. Nozzle,
  smoke/ammo handling and per-target event deduplication stay under one fire action.
  Rodrigues rotation around local up produces the two side rays.
- `weapon_evo.rs`: optional typed per-grade fields and matching bounds. No new
  key is accepted without a runtime consumer. Real-HP units and XP scale stay unchanged.

## Launcher and configuration

Launcher 0.6.0 adds 50 fields: 7 Bomb Glove fields, 2 Pyrocitor fields and one
Walloper field for each of five grades. Missing fields are added losslessly;
existing values, colors, thresholds, comments and BOM/CRLF are preserved.
The public approved balance is unchanged apart from optional new defaults.
Profiles containing these fields require runtime configuration capability 43.

The author updater validates payload hashes and known source revisions, backs
up raw bytes, stages executable/shader folders and rebuilds the frozen launcher.
A stale v42 launcher is not reused merely because its filename is identical.
Players keep the existing default/preset workflow; no preset creation is required.

## Validation and practical limits

Automated checks cover boot key retargeting/upper-body isolation, three-second
Blaster/stream reaction timing with uninterrupted damage and XP, mini-bomb spawn,
arc/arming, color/radius snapshots, nonrecursion, per-instance damage, source-slot
reuse, no airborne-fuse clusters, flame geometry, overlap deduplication, one ammo
clock, held-punch limits/release, parser bounds and lossless preset migration.

The native default explosion RNG ledger, damage, ammo, physics, camera, save and
other existing tests remain part of the full suites. See VALIDATION.json for
actual commands/counts. The modified engine placement function is additionally
type-checked against the real format crate in a standalone contract harness.
A full Windows engine build and frozen GUI build are performed by the author BAT,
not by this Linux preparation environment.

Manual game validation remains necessary with original models/collision data:
try both boots on flat ground/slopes, diagonals/backwards, jump and lock release;
Walloper tap/hold/release into walls and multiple targets; Blaster on Hoven and
Gaspar; mini bombs on floor/wall/objects/water and after a grade change; pitched
triple flame against a wall, one overlapping target and separated targets.
