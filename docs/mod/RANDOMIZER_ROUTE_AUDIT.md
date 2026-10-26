# Mixed-randomizer route audit: v95

This is the unreleased Weapon Evolution 0.3 development snapshot for ReRAC
0.3.6-alpha, runtime patch 0.1.6.95, configuration capability 86. No release or
published asset is changed by this update.

## Confirmed regression

The reported save has layout version 5. Its native/companion hash and companion
checksum match. Native source 121 was collected and awarded R.Y.N.O.; source 120
is uncollected and contains Quartu coordinates. The author's in-game test binds
121 to the upper interior cage and 120 to the exterior Ratchet/airlock route.
The previous catalog attached the easy pack requirement to the exterior ID.

Corrected rules:

| Source | Native identity | Requirements |
| --- | --- | --- |
| 120 | Blarg exterior Gold Bolt, Ratchet airlock route | O2 Mask and Trespasser |
| 121 | Blarg interior Gold Bolt, upper cage | Heli-Pack or Thruster-Pack |
| 13 | Orxon native Visibomb vendor | O2 Mask / Ratchet access |
| 178 | Added Orxon laboratory alcove | O2 Mask and either pack |
| 181 | Added Novalis upstream dead-end | Hydro-Pack, no O2 Mask |
| 179 | Added Pokitaru seabed | No water gadget requirement |
| 140 | Original Pokitaru hidden Gold Bolt route | Swingshot and either pack; no O2/Hydro requirement |

The original Pokitaru hidden route is distinct from the added underwater site:
it includes a Versa-target and a high ledge. Removing water requirements from the
added site must not make the original route a free check.

Orxon's temporary Clank policy now waits only for sources 28 (Magneboots source)
and 75 (Pokitaru Infobot source). Source 137 is a Ratchet Gold Bolt and must never
be a prerequisite for allowing Ratchet to return. The native section controller,
travel planner and landing renderer share the policy, while actual Mask
ownership remains unchanged. The native death/checkpoint behavior is retained.

## Audit scope and conservative rules

The complete 101-entry list is recorded in [randomizer_locations.csv](randomizer_locations.csv).
The machine-readable counterpart is [randomizer_catalog_snapshot.json](randomizer_catalog_snapshot.json).
Every entry carries its review basis. These are source identities, not a guide's
numbered presentation order.

All original item/coordinate entries were compared with the primary PS3 Casual
logic, including its item-to-planet mapping. ReRAC-specific character access is
additional: a shop available on Orxon in abstract logic is not necessarily
reachable by Clank in the native runtime. All six mod-authored extra routes are
listed separately. Gemlik's extra route now consistently includes Swingshot and
allows either explosive with either pack, rather than two arbitrary pairings.

For native Gold Bolt identities that have not been independently matched to
level pvars/collision, the catalog uses a conservative shared route envelope for
that planet. This intentionally over-gates easier siblings until their identity
can be surveyed. A walkthrough's first entry is never treated as pvar index 0.
Examples include both Quartu bolts requiring the Swingshot/Codebot route envelope,
and Oltanis bolts waiting for the relevant movement/Morph/PDA routes. Eudora's
high ledge uses the Heli-Pack rule; Thruster-Pack alone is not assumed equivalent.
Unknown native Gold Bolt indices or extra-site IDs fail explicitly instead of
falling through to an empty requirement list.

This reduces unsafe early progression placements and may increase backtracking.
It is a conservative model, not a claim that every listed gadget is physically
necessary at every sibling. Matching all remaining native indices to authored
world positions is still a separate data/gameplay survey. No source-only test
can certify all original collision, mission-script and character behavior.

## Seed generation and save compatibility

New layouts use version 6. The reward pool, exact unique reward count, five empty
optional sites, six additional candidates, Novalis start and coordinate pacing
remain intact. At most two discovered-planet coordinates can be on any planet.

With a sparse frontier, the fill avoids spending the last two checks on unrelated
rewards when no escape can be opened. Repeated unsuccessful attempts can prefer
an opening reward; the complete mixed pool remains available on a healthy
frontier. Deterministic bounded retries use the same requested seed. Every
accepted layout still passes complete reward reachability and independent
reachability-sphere pacing checks. No invalid layout is accepted to meet a retry
budget and no replacement seed is silently written.

Versions 1-5 remain readable without regeneration. Being structurally valid does
not mean an old layout passes corrected route requirements. The reported v5
permutation is a regression fixture: it must remain loadable and must fail the
corrected completion check. Its regenerated v6 layout must pass. No original
native save, checksum, personal path or account data is committed in the fixture.
Existing cards, inventory, XP, claims and seeds are never modified by this update.
A new game is required to test the newly generated layouts.

## Reproducing a report

The dependency-free example consumes the actual saved permutation:

```text
cargo run --release --locked --no-default-features -p rc-game --example randomizer_audit -- save.bin matching.wev
cargo run --release --locked --no-default-features -p rc-game --example randomizer_audit -- --seed 123
cargo run --release --locked --no-default-features -p rc-game --example randomizer_audit -- --catalog
```

The companion must match the SHA-1 of the native save; the decoder also validates
its checksum and fields. Reports show the seed/version, claimed source/reward
pairs, uncollected checks allowed by saved rewards, completion under the corrected
model and all placements. They never write a save or regenerate a loaded layout.
Gold Bolt rewards do not grant their original source gadget or coordinates.

The local Windows kit builds this example alongside the game. Its
`03_INSPECT_SAVE.bat` accepts a memory-card ZIP, finds content-matched companions
and writes a text report in the kit's `reports` directory. It reads archive bytes
without extracting arbitrary paths and leaves the archive and live card intact.
No diagnostic is sent anywhere automatically.

## Evidence and limitations

Primary item/coordinate logic:
https://github.com/bordplate/rac1-randomizer/blob/main/logic0_Casual.lua
(blob e965121426450368b6c3084b269c3d52184c3b90).
That implementation uses separate Infobot and item pools; this mod uses a mixed
pool plus native Gold Bolt checks and authored extra sites.

Authored game-route descriptions, used for conservative route envelopes rather
than native index identity:
https://ratchet-galaxy.com/en/games/ps2/ratchet-and-clank/solutions/gold-bolts

The reported Blarg identity is grounded in the author's saved pickup and direct
in-game confirmation. All unresolved native index identities remain marked as
such. Automated tests check the implemented model, not universal physical access,
NPC affordability, combat success or every possible live mission state.

Weapon Evolution is an independent mod by TomRus47. The author develops this mod
only, not ReRAC or the original games, and owns no original-game rights or assets.
Original credits and licenses remain applicable.
