# Current implementation map: gameplay v51 / launcher 0.11.0

See [ammo-unlock mode](AMMO_UNLOCK_MODE.md) for the v51 inventory, drop, shop and companion-save integration. All evolving weapons expose per-grade Gold appearance; the OmniWrench uses a global visual setting.

Paths below are relative to `crates/rc-game/src` unless qualified. Native ports
intentionally preserve PS2 operation order; read module comments and tests
before changing their timing. Avoid broad reformatting of those routines.

| Weapon | Runtime modules | Contract |
|---|---|---|
| Bomb Glove | `moby_update/classes/bomb.rs` | Visible scaled child models, swept flight, snapshotted size/damage/color/radius; no recursion; explosion owns damage |
| Pyrocitor | `hero/pyrocitor.rs` | Three lane-local retained-flame banks, editable side angles, shared ammo clock and accepted-hit deduplication |
| Suck Cannon | `hero/suck_cannon.rs`, `moby_update/classes/cannon_crate.rs` | Store actual crate type/contents; interrupted suction restores it; normal attack delivery and native drops once |
| Walloper | `hero/walloper.rs` | Held lunge extends propulsion and hit window together; one attack serial; 100% native timing |
| Tesla Claw | `hero/tesla.rs`, engine `tesla_render.rs` | Two native chains including free firing; secondary same-target damage configurable, 25% fallback; independent other target possible |
| R.Y.N.O. | `hero/ryno.rs`, `moby_update/classes/ryno_missile.rs` | Mutually exclusive continuous/shotgun replacements; distinct 7x5 pellet origins and two spread axes; center unaffected; whole-volley admission before ammo, bounded pellet lifetime, trigger release clears stance without resetting cooldown |
| Mine Glove | `moby_update/classes/mine.rs`, `mine/experimental.rs` | Native collision/landing; four flame arms with independent color, DPS/XP, local sound, lifetime and population limit; replaces proximity mode |
| Drone Device | `moby_update/classes/drone.rs`, `mouse/companion.rs` | Up to ten ordered drones; shared native/experimental companion singleton; lifetime/shot/damage/range settings; active companion blocks another summon without spending ammo |
| Taunter | `hero/taunter.rs`, `moby_update/services.rs` | Attraction, damage and stun remain distinct; optional launch on successful stun |

The [configuration reference](CONFIG_REFERENCE.md) is generated from current
keys, bounds and defaults. Earlier [v43](EXPERIMENTAL_WEAPONS_V43.md) and
[v44](EXPERIMENTAL_WEAPONS_V44.md) notes are historical, particularly the old
Tesla target-exclusion design. Current code/reference take precedence.

## Shared boundaries

- `weapon_evo.rs` parses and snapshots grade parameters; `weapon_evo_save.rs`
  exchanges progression with saves. Do not retain XP target caches across slots.
- `moby_update/real_hp.rs` owns attack identity, accepted health settlement and
  stream clocks. One native HP is 100 public HP. XP follows actual health loss;
  overkill must not skip amoeboid generations or inflate XP.
- Creature damage/reaction code, `classes/path_enemy.rs` and enemy handlers
  preserve phase shields. Blaster/Pyrocitor/Tesla reaction limits are three
  seconds; quiet accepted hits still damage and award XP. Eudora blade flyers
  do not receive repeated weapon push impulses.
- `hero/combat_strafe.rs`, `combat_legs.rs`, `worn.rs` and engine attachment
  evaluation keep lower-body travel separate from the aimed upper body/boots.
- `follow_camera.rs` separates requested yaw/pitch from resolved collision
  position; ignores its class-1007 helper, retains scenery collision, and uses
  ten-tick ramps with menu-selected speed.
- `menus/pause/weapon_xp.rs` and engine `menu_render.rs` use live progress;
  menu bars draw outside clipping immediately below the name panel.
- Engine HUD/upgrade-notice code synchronizes fades and clears notices on death.
- [Respawn lifecycle](RESPAWN_LIFECYCLE.md) documents table-bound ownership.
  Reset or remap raw object IDs at every table replacement.

## Adding a field

1. Add a typed runtime value, bounds and a real consumer.
2. Add matching launcher vocabulary/types/bounds and English help. Register
   experiments in `EXPERIMENTAL_DEFAULTS`, after all standard fields per grade.
3. Extend lossless migration; preserve explicit values, comments, BOM/CRLF,
   presets and unknown nonconflicting fields.
4. Test all five grades, invalid values, consumer behavior and capability
   negotiation. Bump capability only when the profile contract changes.
5. Intentionally update default/example/embedded copies, regenerate the
   reference, refresh source hashes and run checks.

Patch 49 introduces no field: capability remains 48 and launcher remains 0.10.0.
The public default reflects the latest attached v48 profile; installed profiles
are preserved by the incremental updater.

- [Loading and shotgun lifecycle](LOADING_AND_SHOTGUN.md) documents atomic volley admission, bounded pellet lifetime and loader-owned shared recognition caches.
