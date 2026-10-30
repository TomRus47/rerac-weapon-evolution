# Randomizer reward handoffs and persistent shops

## Separate source completion from actual inventory

Randomizer All persists a mapping and a collected bit for each original reward
source. Inventory describes the actual reward already obtained. These answer
different questions: owning an early O2 Mask must not retire Pokitaru's still
unclaimed fighter-mission reward.

The NPC availability guards in `units/pokitaru_commando.rs`, `units/blarg_story.rs`
and `talking_npc.rs` use the shared `source_owned`/`source_planet` helpers. Outside
Randomizer All these retain the supplied native ownership condition. In mixed
mode they consult the original check's receipt. Do not bypass mission completion
or infer a receipt from ownership of the source's native item.

## Follow race completion from hero to receipt

Hoverboard prizes have two gates: the hero sends the winning command, then the
host chooses the prize dialogue. Fixing only the host leaves early-owned native
prizes blocking the command before the host can run.

`units/hoverboard.rs::world` snapshots the source claim into
`hero::hoverboard::BoardWorld.reward_claimed`: source 48 on Rilgar and source 31
on Kalebo III. `None` retains native first-win rules. This transient field is
not serialized. Preserve the native flag's other score/boost/skill-point uses.

The hero finish path checks first place and this optional source claim. Kalebo's
host also checks the original source through `source_owned`. A claimed reward
retains the normal repeat-race flow; a loss never queues a prize. Finish counters
do not prove a winning unclaimed race, so do not retroactively award from them.

Tests in `v123_race_receipt_tests` run the real three-lap finish, dispatch its
commands, and run the host. The Kalebo case follows dialogue completion through
the randomized grant and verifies one receipt with an unchanged layout. Native
mode, early native ownership, prior claims and losses are separate cases.
Fixtures do not play disc-backed dialogue, movies or GPU effects.

## Persistent normal-vendor stock

Native planet entry already registers original Gadgetron sources in
`GameState.global.vendor` (save chunk 12). `menus/vendor.rs::mixed_offer_available`
uses that saved registration, an actual Gadgetron catalog identity, a valid
mapping and an unclaimed source. It does not require the current planet to be the
source planet. Both listing and purchasing use the same predicate.

World-found weapons do not register vendor stock in Randomizer All. Their native
ammo ownership remains separate. The PDA remains ammo-only, and source-specific
NPC purchases are not converted into Gadgetron offers. Normal and weapons-only
mode retain their native rules. No new save field or layout migration is needed.

`menus/vendor/reload_tests.rs` covers all eleven Gadgetron source registrations
through actual planet entry, travel without opening the original vendor,
cross-planet purchase at the source price, duplicate/stale transactions, native
and companion save reload, NG+ retention and the ammo-only PDA policy.

## Integration and compatibility

The 0.4.2 source reconciles the complete prepared v123 workspace and the previous
main-branch fixes. Transient race, shortcut and HUD state remain outside save
storage. Native stock and source claims retain their existing formats. Keep
saves, native files and personal reports out of source exports.

See [credits](CREDITS.md) for mod-only authorship, original-game rights and upstream
license attribution.
