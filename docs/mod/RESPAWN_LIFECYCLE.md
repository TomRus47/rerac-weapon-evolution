# Respawn ownership and the v49 crash fix

## Report and reproduced failure

The supplied log contains two panics in the common pvar reader:
`range start index 4 out of range for slice of length 0`. The game stack is
unsymbolized, so the log alone cannot identify the caller. The reported death
reload is on Oltanis (level 14), before the short main-route grind rail, with
Tesla Claw, Magneboots and Sonic Summoner; an earlier companion had disappeared.

A fixture using the attached v48 code reproduces the exact panic without disc
data: queue the arc keeper's frame callback, replace its slot with a live object
with empty pvars, and run the next frame. This confirms a failing path; the
user's exact save/checkpoint still needs an in-game retest. Other callers could
produce the same unsymbolized panic.

## Failure mechanism

`MobyId` is a slot, not a stable identity. `death_reload` rebuilds a compacted
table; collected/killed placed objects can disappear and move subsequent slots.
The old draw callback lists survived replacement. Their guard checked only
existence/deleted state, so an unrelated live replacement passed it.

Class 1331's `oltanis_arcs::frame` reads texture indices at pvar offsets 4 and 0
before iterating active arcs. Dispatching that old callback to an empty-pvar
replacement produces the logged panic. The correction clears all callback lists
at the table boundary, covering other effects too. It does not hide invalid
identity by returning fake zero pvars or disabling electrical effects.

This path already exists in the supplied v48 and earlier repository snapshot;
it is not a new v48 menu-XP/ten-drone change. The original official upstream
revision was not supplied, so this investigation does not assign the defect
to a particular official ReRAC release.

## Reset contract

`Services::reset_moby_transients` runs after replacement-table/hero preflight
succeeds, immediately before `p.game.mobys = table`. Failed reloads retain their
original state. It clears all deferred draw lists/output, the shared Sand Mouse
singleton, per-slot evolution stun/launch state, and Oltanis/Quartu arc owners
and geometry. Arc-owned lights are released without removing unrelated lights.

XP, grades, thresholds, profile settings, save/death bits, checkpoints, mission
words and loaded class/geometry data are preserved. Existing reload code still
rebuilds hits, groups, grid, particles, drones and the hero. Do not default all
`Services` or `units::Globals`: they also hold immutable level meshes that the
death load pass does not reload.

The companion disappearance path already validates tagged pvars and releases
its singleton. No evidence here directly implicates the expired companion or
boots. Clearing its table-bound singleton also prevents same-class slot reuse.

## Verification

`services_reload_tests.rs` covers all three old callback lists, valid fresh
callbacks after reset, companion/reaction/arc-owner cleanup, and preservation
of progression, checkpoints and class assets. The arc module checks that only
its owned lights are freed. Run the focused fixtures with:

```sh
cargo test --release --locked --no-default-features -p rc-game --lib reload
```

Then run the complete suites. On Windows confirm patch `0.1.6.49`, capability
`48` and audio diagnostics `false` with `runtime/rerac.exe --version-json`.
At the reported Oltanis checkpoint, activate the electrical enemies, let the
companion expire, die and reload repeatedly. Repeat with an active companion
and without one; inspect electrical effects, XP and checkpoint restoration.
Quartu shares this arc keeper and is a useful second case. No audio probe is
needed. Automated tests do not replace this exact-save gameplay check.
