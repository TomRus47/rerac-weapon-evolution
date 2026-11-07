# v133 local gadget combat test — ReRAC 0.3.8-alpha

This source-only kit builds Weapon Evolution patch `0.1.6.133`, configuration
capability 96 and launcher 0.30.0. No GitHub operations or release publication
are performed. All v132 shop-video and original NPC-price fixes are included.

## Install and test

1. Extract the complete kit to a new folder. Do not copy individual Rust files.
2. Close the game and mod launcher. Run `01_COMPILE_AND_UPDATE.bat`.
3. Select the existing playable mod directory containing `weapons.ini`, the
   launcher EXE and `runtime/rerac.exe`. The source kit is not that directory.
4. Both the runtime and matching editor must build and pass the installer checks
   before replacement. Keep the window open if any check fails.
5. Open the same playable launcher, then use the four new pages after R.Y.N.O.
   Turn on the desired gadget functions and save. Original weapon balance,
   colors, XP, presets and native/companion saves are retained. Existing saves
   can be continued; this update does not regenerate randomizer layouts.
6. `03_CREATE_PLAYABLE_ZIP.bat` creates a playable package after a successful build.
   Original disc/extracted game data are never included.

Gadgets have one settings page each, without XP grades. The new functions are
Off by default. They do not grant ownership of gadgets. Circle and R2 use the
same logical fire action; holding both does not create a second shot.

## Camera and Swingshot

L2 camera lock accepts equipped hand gadgets and their equip transitions, with
the existing water, magnetic-surface, death, cinematic and L1 restrictions.
Original Swingshot targets always win over the optional actions. Only original
yellow targets allow hanging.

`[gadget.swingshot]` supports `enabled`, `pull_range`, `pull_speed` and
`stun_seconds`. Aim toward an ordinary enemy, fire and hold to extend the native
rope, pull the enemy toward Ratchet and stun it on arrival. Bosses and flying
ships cannot be pulled. Releasing retracts the cable without creating a hang.

The experimental footer contains `surface_grapple_enabled`, `grapple_range` and
`grapple_speed`. In the absence of an enemy/native target, aim at a wall,
ceiling or crate to pull Ratchet there. Floor normals are rejected. The travel
sweep stops short of geometry. Native cable rendering and Swingshot sounds are
reused, without importing level-local models or converting surfaces into yellow
Swingshot targets.

## Trespasser

`[gadget.trespasser]`: `enabled`, `damage_per_second`, `drill_range` and
`rotation_degrees_per_second`. Native Trespasser pads have priority.

Hold fire to run the motor. A nearby ordinary enemy is held in one idle pose,
rotated with the drill and damaged through the real-HP contact clock. Ratchet
uses the standing Pyrocitor aiming animation and cannot walk while capturing.
Releasing stops the motor and tip immediately; the victim returns upright over
six frames with a small collision-resolved forward push. A defeated victim is
replaced by a swept projectile displaying that enemy's model, using the current
Suck Cannon grade's damage (200 HP fallback), native collision queries and one
accepted hit. Cosmetic projectile copies cannot become fresh XP donors.

The drill has no weapon XP track. Only actual committed HP loss may award the
existing optional Nanotech progression. A held pose, repeated collision samples,
shields and dead actors do not award extra XP.

Asset-dependent details need in-game verification: the tip modifier is applied
only when the Trespasser exposes a separate non-root joint-list target. The
whole gadget is never rotated as a fallback. No explicitly named drill enemy or
standalone drill sample was identified in the ported class catalog; this test
uses the native Suck Cannon motor loop as a portable mechanical sound. Its
availability and timbre should be checked on multiple planets.

## Hydrodisplacer frost shots

Native water-pad interactions have priority, including their cutaways. Empty or
full water tanks may fire the independent ice ammunition elsewhere.

| Key | Purpose | Default |
| --- | --- | --- |
| `enabled` | Enables frost shots | Off |
| `max_ammo` | Shots per planet visit | 10 |
| `fire_interval_ticks` | Minimum interval at 60 Hz | 60 |
| `freeze_radius` | Area radius in world units | 6 |
| `freeze_seconds` | Animation/AI freeze duration | 4 |
| `projectile_range` | Maximum travelled distance | 50 |
| `projectile_speed` | Launch speed | 18 |
| `projectile_gravity` | Ballistic acceleration | 18 |
| `ice_size_percent` | Crystal visual size | 100 |
| `ice_color` | Faceted crystal color | C0EEFF |
| `mist_color` | Flight mist color | 80CCFF |
| `explosion_color` | Frost burst color | 80DCFF |
| `frozen_enemy_color` | Frozen tint and enemy mist | 408CFF |

The ice crystal is a small generated faceted mesh with a mist trail, so no
planet-specific ice prop is required. Contact, range expiry or flight timeout
produces one frost burst. It freezes enemy animation and AI, including bosses;
it neither damages targets nor freezes already launched bombs, bullets or
missiles. Frozen bosses retain their original phase shields and HP intake rules. Damage from other attacks still works. The Veldin beam manager retains
the existing beam endpoints when its owner is frozen rather than re-aiming or
removing that beam. Frozen leases expire by simulation time and verify slot,
class and UID; respawned or recycled enemies cannot inherit a stale freeze.

Ice ammunition is a separate session-level planet-visit counter. Shops, pickups,
weapon upgrades, switching gadgets and checkpoint respawns cannot replenish it.
Changing planets refills it; starting/loading a session is a fresh arrival. It
is not inserted into native shop ammo tables or the weapon-evolution save format.
Live changes to capacity do not replenish a visit already in progress.

## Hologuise and continuous Lavagun

`[gadget.hologuise]`: `enabled`, `movement_multiplier` and
`animation_multiplier`. Movement is scaled before native collision, while the
native distance-linked walk animation rate has its own multiplier. These settings
apply to the disguise, not Ratchet's ordinary running speed.

Lavagun droplets now continue through enemy bodies while their accepted-contact
HP clock keeps one damage pulse per target/tick across overlapping droplets and
triple jets. Enemy bodies no longer remove flight samples or break the ribbon.
Static world geometry is tested independently, even when a creature is directly
in front of a thin wall. Terrain and other solid obstacles still stop the stream.
No Pyrocitor damage, XP, colors or ammo balance values are rescaled.

## In-game verification checklist

- Test L2 with each gadget, L1 first person and real Swingshot targets nearby.
- Pull ground/flying ordinary enemies and check that ships/bosses are excluded.
- Grapple walls, ceilings and crates; reject floors, check corners and ledges,
  release midway and verify visible cable retraction. No free hanging.
- Use Trespasser on a real pad, then capture a small and a large enemy elsewhere.
  Release, change gadgets, take damage and die; ensure no motor loop or held actor
  survives cancellation. Verify tip isolation and aiming pose on the real model.
- Fire frost with empty/full water tanks, on a water pad, at a normal enemy and a
  boss. Check thaw, damage while frozen, active Veldin beams and moving hostile
  projectiles. Fire the last shot, switch gadgets, die and revisit the planet.
- Test Hologuise speeds against walls and through story interactions.
- Sweep single/triple Lavagun jets through enemies directly in front of walls;
  check continuous visuals, damage and unchanged ammo consumption.

## Verification scope

Rust logic and launcher/profile checks are run in this environment. Windows
linking, real asset playback, full Tk page painting and GPU/in-game behavior must
be verified with the included local installer and game. This is an experimental
local test, not a claim that every native enemy animation has been exercised.

Weapon Evolution is an independent mod by TomRus47, who does not develop ReRAC
or the original games and is not affiliated with their developers. Original-game
rights remain with their respective owners. ReRAC's developers and contributors
make this work possible; their port and existing license/credits remain intact.
