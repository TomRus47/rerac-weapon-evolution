# Local v54: OmniWrench progression and projectile corrections

Gameplay patch 0.1.6.54, configuration capability 54, English editor 0.14.0.
This is a local test kit for ReRAC 0.3.4-alpha. No GitHub update or release is
performed. All v53 weapon parameters survive; the global wrench Gold option
moves to its five grades. This page supersedes v53 grenade range/fuse notes.

## OmniWrench track and editor

Inventory ID 8 is now an independent XP track, saved per native slot like the
other weapons. Its launcher page contains all five grades. Transition XP is
editable inside V1–V4 cards; V5 is the maximum grade. Names still use the disc's
selected language. The four Boolean grade options use checkboxes.

| Setting | Default | Contract |
|---|---|---|
| `damage_hp` | 100 / 125 / 150 / 175 / 200 | Normal swing/throw HP for V1–V5 |
| `throw_range` | 10 | Outbound Comet Strike travel in game units |
| `throw_speed` | 23 | Outbound speed, game units/s |
| `gold_texture` | false, or migrated global value | Appearance only, independently per grade |
| `show_hud` | true | This weapon's top-left name/vignette/XP only |
| `crate_breaker_enabled` | false | Invisible crate-only shockwave on a ground-slam impact |
| `crate_breaker_radius` | 5 | Three-dimensional radius around the impact, game units |
| `moving_attacks_enabled` | true | Stick locomotion during locked-camera ground combos |
| `bolt_attraction_bonus` | 0 | Passive additional horizontal/vertical normal-bolt attraction radius |

Transition defaults are 1000 / 2000 / 3500 / 5500 XP. Existing non-wrench
balance is unchanged. Configure stronger grades/bonuses before testing them;
Crate Breaker starts disabled and bolt bonus starts zero.

A slam deals twice this grade's normal hit HP, preserving the native 2:1
ratio. Damage is captured on swing/throw entry. One swing/throw may damage a
given eligible enemy once across frames; another swing may hit immediately.
The outward and returning legs belong to the same throw. Native physical
reaction classes and enemy phase shields remain. XP follows accepted actual
HP loss, without credit for overkill or breaking crates.

Comet Strike captures its range/speed/damage at release, travels out at the
configured speed to that range and returns toward the hand. Walls can return
it earlier. Its native spin, whoosh, catch, collection and return acceleration
remain. Do not confuse game units/s with the per-tick internal velocity.

`show_hud = false` does not disable XP, pause-menu XP bars, menu grade labels
or upgrade notifications. It never hides another weapon's HUD. Its Gold option
is moved from `[general] wrench_gold_texture`: explicit grade values take
precedence; the legacy global value fills only missing grade settings.

With `moving_attacks_enabled = false`, locked-camera combos use native fixed
lunges rather than free stick locomotion. Camera lock, normal running, other
weapons and airborne attacks are not disabled. Default true preserves the
previous mod's moving combo behavior; uncheck lower grades to make it an upgrade.

Crate Breaker targets only classes 500, 501, 502, 505 and 511 within the sphere.
Crates carried/fired by the Suck Cannon are excluded.
It sends their normal destruction hit, including the reinforced-crate break
flag. Normal bolt/ammo/nanotech drops, stacking and TNT arming/explosions remain
owned by the crate code. The shockwave adds no direct enemy damage. TNT's own
subsequent native explosion can still hurt nearby things.

## Additive Bolt Grabber bonus

The bonus is passive at the current wrench grade, even with another weapon
held. It increases normal-bolt attraction distance, not bolt value or movement
speed. It adds to the native horizontal and vertical radii:

| Inventory | Bonus 0 | Bonus 2 |
|---|---|---|
| Without Bolt Grabber | 3 / 1.75 | 5 / 3.75 |
| With Bolt Grabber | 12 / 4.5 | 14 / 6.5 |

Ammo/nanotech pickup rules, Gold Bolt placement and non-Ratchet bodies are
unchanged. The native initialization frame keeps its original radii.

## R.Y.N.O. shotgun cores

All 35 physical pellets are still admitted before ammo is charged. Core
particles are created first, are brighter through alpha only (configured hues
are preserved), and keep the same collision/HP ownership as v53. Under particle
pressure a new core can reclaim an optional R.Y.N.O. trail, never an existing
core or an unrelated weapon/weather particle. A pellet with no core receives
no optional tail; a missing core is retried at its current position/lifetime.

Ownership maps are cleaned when records die and checked on projectile-slot
reuse. These are cosmetic records: retry/reclamation does not create another
physical shot, spend ammo or multiply damage. A pool entirely occupied by
unrelated particles can still delay a core until capacity becomes available.

## Visibomb grenade terrain, water, death and audio

A successful grenade launch queues the original Bomb Glove throw feedback
plus the Devastator firing sound, explicitly addressed to the Devastator class.
No extra fire feedback is queued for a failed object creation. Guided native
Visibomb firing remains separate.

The range limit is no longer a fake contact surface: it stops horizontal
travel and the grenade continues falling. The preview uses the same integration
and draws only at a real terrain/object contact. A lower floor receives a lower
marker; an empty pit/death path has no floating marker at Ratchet's height.
A tiny range-rounding residue cannot freeze the vertical fall.

Real contact with terrain, an enemy/object or water detonates the grenade.
Water differs deliberately from the native Bomb Glove, which can sink.
Crossing the level death plane, a pit surface (8/12), or either list of active
class-1039 death cuboids removes it without a blast or damage. Cuboid tests use
swept segment/unit-box intersection, including thin volumes between frames.
Preview volume IDs are collected once per preview, without a persistent cache
of raw object pointers. A 20-second quiet cleanup bounds invalid free flight;
there is no native five-second air detonation in replacement mode.

## Saves and downgrade

WEVO-v3 appends the wrench after the stable first fifteen track IDs. It reads
v1/v2 companions without changing their existing weapon XP or arsenal state;
the new wrench starts at V1/zero XP. Native save bytes and native file binding
are unchanged. Old v53 runtimes cannot decode newly written v3 companions.
Keep the previous save folder if you intend to downgrade after testing.

## Test on Windows

1. Build/open using this kit's `01_UPDATE_BUILD_AND_OPEN.bat`; use the Python
   test editor opened by the BAT to see the new OmniWrench page.
2. Set distinct grade HP/range/speed/Gold/bolt values, upgrade, save/reload and
   change slots. Check normal swings, slams and both legs of a throw.
3. Toggle only wrench `show_hud`; verify pause progress/upgrade notice and every
   other weapon HUD stay available. Toggle moving combos at two grades.
4. Slam near ordinary, reinforced, TNT, ammo and nanotech crates plus a nearby
   enemy. Only the crates receive the added shockwave; their drops remain normal.
5. Compare bolt collection at bonus 0/2 with and without the native Bolt Grabber;
   equip another weapon and verify the bonus remains passive.
6. Fire repeated R.Y.N.O. shotgun volleys among world effects. Compare cores,
   tails, actual hits and cleanup; test near walls and with zero spread.
7. Fire Visibomb grenades over a lower floor, into a deep pit, into water, at an
   enemy and across a native kill volume. Check sound, marker, contact blast,
   Visibomb XP and free movement/camera.

Automated fixtures cover these contracts where no disc data is needed. Model,
audio, collision-data variants and gameplay feel require the in-game checks.
