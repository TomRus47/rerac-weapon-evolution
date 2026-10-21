# Local v53: Visibomb grenades, icons and flame mine geometry

Local test only: gameplay 0.1.6.53, configuration capability 53, editor 0.13.0,
ReRAC 0.3.4-alpha. No GitHub update or public release is performed.

## Icon palette

Every evolving icon uses a neutral source frame, rather than multiplying a
baked native Gold palette by another yellow tint. Equipped/unavailable menu
variants remain. The shared GS modulation brightens V2 fuchsia, V3 red,
V4 orange and V5 golden yellow. V1, alpha and HUD fading are unchanged.
Verify perceived brightness in game across native and added Gold weapons.

## Standard Visibomb missile

`explosion_damage_radius` is an absolute area-damage radius in game units;
default 4 preserves the native missile radius. Visual blast size is unchanged.
Damage and radius are captured at launch after both fin manipulator records.
Later grade changes cannot alter an airborne missile's captured settings.

## Experimental replacement launcher

These settings appear after every standard setting in all five Visibomb grades:

| Key | Default | Meaning |
|---|---|---|
| `grenade_launcher_enabled` | false | Replace new guided missile shots |
| `grenade_size_percent` | 250 | Percent of native Bomb Glove model size |
| `grenade_arc_height` | 3 | Apex height above the higher endpoint, game units |
| `grenade_max_range` | 12 | Maximum horizontal displacement from muzzle |
| `grenade_fire_interval_ticks` | 30 | Successful-shot cooldown at 60 ticks/s |
| `grenade_damage_hp` | 600 | Actual HP per enemy per grenade explosion |
| `grenade_explosion_radius` | 1 | Native Bomb Glove blast multiplier |
| `grenade_explosion_color` | default | RGB explosion tint |
| `grenade_color` | default | RGB textured model ambient/glow tint |

The separate branch in `hero/visibomb.rs` never calls the guided missile
launcher, defers Ratchet's MISSILE state or transfers camera/controls. Ratchet
keeps his movement vectors, including airborne firing in supported groups.
Only successful object creation charges ammo, starts cooldown and queues the
same successful-throw feedback as the Bomb Glove. Missing models or a full
object table consume no shot. A native missile already in flight may finish
before switching to the replacement.

`bomb/visibomb_grenade.rs` reuses class 121, native gravity, spin, collision,
explosion and effects. The Bomb Glove landing marker predicts the same
integration, collision and swept horizontal range limit. Contact, range expiry
or the native five-second fuse detonates the grenade. Bomb Glove clusters and
Gold bonuses are not inherited. Size, range, colors and HP are stored per shot.

`real_hp::Events` recognizes the marker before native class-to-weapon mapping.
Contact triggers detonation without HP; the explosion awards Visibomb XP once
per enemy per grenade. A second grenade may hit immediately. Native enemy
shields/reaction filters still apply. There is no Bomb Glove XP attribution.

The blast multiplier scales native effects and the growing damage sphere.
At 1, the native final radius is 2.5 game units. Model size is independent.
Color modulates the existing texture; it does not replace the texture.
Missing bomb/fireball/flash models/textures are imported on the loader worker
when any grade enables the experiment. No disc reads occur while firing.

## Flame mine body

`flame_mine_size_percent = 150` is in each Mine Glove experimental footer.
100 is native deployed size; 150 is 1.5 times native. Ordinary mines retain
native size. The original scale is captured so resnapshotting cannot compound
size. Ground hull/clearance scale with the body. Landed body yaw and all four
flame arms share their angle; zero flame rotation stops both.

## In-game validation

1. Compare V2–V5 icons across all menus/HUD, particularly native Gold versus
   R.Y.N.O., Taunter, Walloper, Drone Device and Visibomb. V4 must read orange.
2. Enable Visibomb grenade mode. Fire while moving, strafing, jumping and
   camera-locked: no guided missile, immobilization or camera takeover;
   one throw feedback per successful creation.
3. Check spinning 250% model, marker, slopes, walls, water, maximum range,
   colors, damage and Visibomb XP. An airborne shot keeps its grade snapshot.
4. Disable the experiment and verify original guided flight/camera.
5. Deploy flame mines at 100/150/250%; verify clearance and matching rotation,
   including stationary body/flames at zero degrees/s.

Automated tests cover configuration/migration, geometry, snapshot ownership,
HP deduplication, movement preservation and feedback. Model appearance,
subjective audio and full camera behavior still require gameplay validation.
