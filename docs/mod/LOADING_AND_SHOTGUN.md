# Loading and R.Y.N.O. shotgun investigation (v50)

## Pellet lifetime and resource pressure

`blaster_shot::update_ryno` decremented the signed lifetime but never tested it.
It relied on collision, world bounds or comparing remaining range with the
length of this tick's step. A small positive range residue can survive that
comparison. Eventually the squared step length underflows to zero while the
position rounds back to the previous position. The projectile remains alive,
stationary, and keeps a dynamic slot indefinitely. The lifetime check now
bounds that case; range/collision still delete earlier.

The firing loop previously charged ammo and then ignored all 35 allocation
results. With fewer free slots, its row-major loop produced only the first rows;
with no free slots it still charged ammo. Leaked stationary pellets can therefore
cause both symptoms. Other ordinary object pressure can also exhaust the pool.

`hero::ryno::reserve_shotgun` now checks actual allocator capacity, including the
two-tick deletion quarantine, and reserves all 35 slots before charging ammo,
starting cooldown, queuing visuals or probing muzzles. An unexpected sink failure
rolls back every reserved slot through the normal deletion path. The next held
fire tick retries. No global pool expansion or eviction of enemies/effects is
introduced. Near-wall muzzle probes may legitimately terminate individual shots;
that is an impact, not allocation failure.

Experimental pellets explicitly install `blaster_shot::UPDATE_FN` and clear
`NO_UPDATE`. Native class-table absence must not leave a visible object without
a movement/lifetime owner. Native Blaster projectiles retain their original path.

## Shared particle capacity

A full native-style volley requests 35 heads plus 35 x 17 trail sprites: 630
records from the shared 2,048-record particle pool. Repeated fire can exhaust
that pool even when the projectile object table has room. The hero particle
hook now creates every shotgun head before its optional tails. Optional shotgun
tails stop allocating above half of the shared pool; regular Blaster tails and
all projectile movement, collision and damage are unchanged. This preserves
headroom for subsequent heads and world effects without enlarging the global
pool. Under extreme unrelated particle saturation, native particle allocation
can still fail; this does not turn a projectile into a non-damaging shot.

## Repeated level recognition

The v44 loader already prepared class, camera and reaction recognition on its
worker, but each relocation rebuilt the target instruction lookup index. Other
first-frame consumers independently read/parsed/relocated menus for cheats,
the map, menu rendering and model-preview lighting.

`LevelOverlay` now shares lazily constructed instruction and address-reference
indexes across recognizers. `prepare_level_tables(root, destination_index)`
prepares the menu/map overlay too, including the level-01 pause-map reference.
All consumers reuse those resolved tables. Worker preparation uses the explicit
destination index and never changes the live level global.

Parsed and menu overlays share immutable section bytes through `Arc`.
Temporary recognition indexes are explicitly released after preparation, rather
than retaining one large index per visited planet. Concurrent readers hold Arc
leases and remain valid through cache release. Resolved tables remain cached;
a later uncached recognition can rebuild the same index lazily.

## Evidence and limits

The supplied v43 timing evidence, retained in EXPERIMENTAL_WEAPONS_V44.md,
recorded audio work in milliseconds and mesh/finalization work in seconds.
No current user-PC log or disc data is available here. These code-path findings
do not establish that the old audio diagnostics caused the historical slowdown.
No audio probes, recording, artificial waits or new diagnostic environment
variables are introduced.

The standalone `overlay_recognition_perf` example uses synthetic MIPS code,
131,072 target words and 32 repeated recognizers, checking the resolved address
and a checksum. It measures only repeated recognition, not planet loading,
render uploads, shader compilation or the user's frame time. Run with:

```sh
cargo run --release -p rc-formats --example overlay_recognition_perf
```

Manual checks: hold shotgun fire through several volleys in an empty area and
near enemies/walls; wait for every free-flight pellet to disappear; repeat with
high range/low speed and a crowded scene. Check ammo on resource rejection.
Compare the same planet transition and death reload before/after on the same
configuration, including the final loading screen and first playable frames.
