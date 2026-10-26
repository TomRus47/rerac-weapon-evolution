# Current implementation map

Main is an unreleased ReRAC 0.3.6-alpha development snapshot, gameplay 0.1.6.95 and launcher 0.28.2. `mod/development.json` is authoritative for this snapshot. `mod/release.json` and versioned release notes still describe the previous published package.

| Area | Primary source | Responsibilities |
| --- | --- | --- |
| Profile/grades | `crates/rc-game/src/weapon_evo.rs` | Eight-grade parameters, effective ammo, XP, NG+ upgrade gates and registry publication |
| Save lifecycle | `weapon_evo_save.rs`, `game_state.rs`, engine saves | Content-addressed v9 companions, legacy decoding, slot isolation and native equipment |
| Difficulty/population/health | `weapon_evo_world.rs`, `weapon_evo_world/`, `weapon_evo_motion.rs` | Original-health tiers, enemy tuning, safe duplicates, perception/pursuit, class damage and XP Nanotech |
| Damage identity | `moby_update/real_hp.rs`, creature/hit services | Per-instance acceptance, actual HP loss, stream clocks, XP routing and reaction separation |
| Mixed randomizer | `weapon_evo_randomizer.rs`, `weapon_evo_randomizer/catalog.rs`, `examples/randomizer_audit.rs` | Versioned mixed layouts, corrected character gates, conservative route audit and saved-layout inspection |
| Discovery mode | `weapon_evo_golden.rs`, `weapon_evo_golden/` | Seeded locations, access constraints, underwater pickup and native acquisition flow |
| NG+ shops/rewards | `weapon_evo_gold_shop.rs`, `weapon_evo_bolt_chain.rs` | V6 purchase, Gold displays, collectible spending and shared reward multiplier |
| HUD | `hud.rs`, engine `hud_render.rs`, `menus/pause/weapon_xp.rs` | Weapon/health XP bars, grade icons, notices, Nanotech and fades |
| Saved controls | `weapon_evo_controls.rs`, `pad.rs`, menu callbacks | D-pad equipment, repeat actions, contextual R1 modes and gameplay fire/crouch |
| Combat movement | `hero/combat_strafe.rs`, `combat_legs.rs`, worn/physics, follow camera | Camera lock, directional legs, boots, momentum and aim |
| Weapon experiments | hero and moby weapon modules, `hero/pyro_lava.rs` | Snapshot-owned grenades, mines, projectiles, lava, streams, companions and effects |
| Configuration editor | `launcher/schema.py`, `migrate_ini.py`, `launcher.py`, `i18n.py` | Lossless defaults, five languages, per-grade organization and presets |
| Build/maintenance | `tools/mod/`, launcher build/package tools | Source hashes, references, Windows build/smoke checks and separate release tooling |

Read [saved controls](SAVED_CONTROLS.md), [configuration reference](CONFIG_REFERENCE.md), [developer guide](DEVELOPER_GUIDE.md), [verification](VERIFICATION.md) and [source index](../../MOD_FILES.md) before changing behavior. Versioned v43–v82 notes explain earlier changes; current source and this map supersede obsolete limits and mode catalogs.

Damage acceptance must remain independent of reaction cooldowns and respect native phase shields. XP follows actual accepted HP loss and explicit non-damage weapon policies. Never make clones inherit dead/terminal events from a reused dynamic slot. Keep immutable level-table recognition in the loader, and discard/remap table-owned IDs during reload. Do not reintroduce the removed arena experiment or temporary audio probes.

The complete supplied workspace includes rc-game2 because ReRAC 0.3.6-alpha includes that separate module. Weapon Evolution remains a Ratchet & Clank 1 mod; including the upstream module does not claim a second-game mod or new ownership of port code.
