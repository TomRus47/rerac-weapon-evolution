# Post-0.4.1 reward integration

See [reward handoffs and persistent shops](RANDOMIZER_REWARD_HANDOFFS.md) for
source-claim NPC guards, complete hero-to-host race routing and cross-planet
vendor stock. These changes add no persistent fields. The
[0.4.2 preparation notes](../releases/v0.4.2.md) identify earlier local changes
still requiring the complete prepared source before packaging.

# Current local v116 update

For ReRAC 0.3.8-alpha, gameplay 0.1.6.116, capability 92 and launcher 0.29.5,
see [compact Nanotech](NANOTECH_V116.md) and [verification](VERIFICATION.md).
Normal new games retain four starting HP. The explicit Veldin II test starts at
210/210. This is a local test kit; no publication operations occur.

# Current implementation map

Main is an unreleased ReRAC 0.3.7-alpha development snapshot, gameplay 0.1.6.110 and launcher 0.29.3. `mod/development.json` is authoritative for this snapshot. `mod/release.json` and versioned release notes still describe the previous published package.

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

The complete supplied workspace includes rc-game2 because ReRAC 0.3.7-alpha includes that separate module. Weapon Evolution remains a Ratchet & Clank 1 mod; including the upstream module does not claim a second-game mod or new ownership of port code.

## v102 combat feedback and HUD integration

See [the implementation and local test guide](FEEDBACK_HUD_V102.md). Accepted HP loss, not native stun entry, feeds `moby_update/feedback.rs`. That module owns temporary tint/burn-cycle state; it does not mutate HP, damage clocks, XP or class AI. Standard wrench visual/HUD settings and experimental slam settings share the ordinary per-grade schema/migration. The independent blast retains native bomb hit identity and wrench XP ownership. HUD geometry lives in `hud.rs` and `hud/xp_bar.rs`, with text drawing in `hud_render.rs`. Randomizer display classes and safe poses are separate from its persisted layout and collection logic.

The supplied v97–v101 changes are synchronized here too: surveyed route rules and legacy seed loading, independent Thruster-Pack moves, discovered-ammo policy, access-only explosive/Gemlik generation checks, moving Comet Strike, and the current live-apply/randomizer launcher UI. New development defaults add controls only; all previous balance values are retained. Published release metadata and assets are not changed.

## v103–v106 follow-up fixes

[Wrench blast geometry and upgrade feedback](SLAM_VISUAL_V106.md) documents the confirmed local fixes now synchronized to main: normal-tick feedback and immediate finite ignition, complete HUD batch/index insertion, once-only grounded slam damage and forward impact probing, bright blue Nanotech XP upgrade bursts, and a bounded slam visual using the exact damage sphere. Source index and tests include the incremental Windows updater; personal profiles and saves are not overwritten.

## Local v109 experimental tuning

[Mode-only damage and mine overrides](EXPERIMENTAL_OVERRIDES_V109.md) describes the new per-grade Lavagun DPS bonus and flame-mine limit/lifetime overrides. The current local launcher also includes the v107/v108 complete preloading, scroll repaint and durable build diagnostics work. These local changes have not been published or pushed to GitHub.

## Release and launcher lifecycle

[Launcher lifecycle](LAUNCHER_LIFECYCLE.md) maps page ownership, preloading, scrolling and frozen verification. [Draft preparation](DRAFT_RELEASE.md) explains the separate unpublished descriptor and same-commit Windows/source assets. The approved default is fingerprinted in `mod/default-profile.json`; changing comments must not change any supplied value. The local v109.1 installer repair recognizes two exact editable identity pairs instead of depending on the inactive fixed-build suffix. Full source checkouts use the root build scripts and do not require that incremental installer.
