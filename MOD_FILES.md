# Weapon Evolution source map

Gameplay 0.1.6.51; launcher 0.11.0. This repository contains the complete modified ReRAC 0.3.4-alpha workspace. The following 232 indexed files are mod implementations, integration points or developer tools; the rest is the supplied port source.

Start with [the developer guide](docs/mod/DEVELOPER_GUIDE.md), [current implementation map](docs/mod/CURRENT_IMPLEMENTATION.md) and [respawn lifecycle](docs/mod/RESPAWN_LIFECYCLE.md). Machine-readable hashes are in [mod/source-map.json](mod/source-map.json). Historical changes-since-supplied-snapshot.patch describes the original export boundary, not the full current update.

| Path | Role |
|---|---|
| `.github/workflows/windows-release.yml` | Weapon Evolution implementation or integration |
| `crates/rc-engine/Cargo.toml` | changed since supplied snapshot |
| `crates/rc-engine/src/companion_assets.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/disc_source.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-engine/src/gameplay.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-engine/src/hud_render.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-engine/src/level_load.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/level_switch.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/main.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-engine/src/menu_models.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/menu_render.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/moby_attach.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/moby_render.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/moby_spawn.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/scene_render.rs` | Weapon Evolution implementation or integration |
| `crates/rc-engine/src/tesla_render.rs` | changed since supplied snapshot |
| `crates/rc-engine/src/travel_render.rs` | Weapon Evolution implementation or integration |
| `crates/rc-formats/examples/overlay_recognition_perf.rs` | Weapon Evolution implementation or integration |
| `crates/rc-formats/src/gadget.rs` | Weapon Evolution implementation or integration |
| `crates/rc-formats/src/level_overlay.rs` | Weapon Evolution implementation or integration |
| `crates/rc-formats/src/moby.rs` | Weapon Evolution implementation or integration |
| `crates/rc-formats/src/moby_anim.rs` | changed since supplied snapshot |
| `crates/rc-game/Cargo.toml` | changed since supplied snapshot |
| `crates/rc-game/src/audio.rs` | v40 combat correction or audio diagnosis |
| `crates/rc-game/src/audio/class_sounds.rs` | v40 combat correction or audio diagnosis |
| `crates/rc-game/src/follow_camera.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/follow_camera/level.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/game_state.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/anim.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/blaster.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/combat_legs.rs` | added since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/combat_strafe.rs` | added since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/damage.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/devastator.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/gloves.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/hero/glow.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/guns.rs` | changed since supplied snapshot |
| `crates/rc-game/src/hero/items.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/jump.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/melee.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/morph_ray.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/physics.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/pyrocitor.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/ryno.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/suck_cannon.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/taunter.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/tesla.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/walk.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/walloper.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hero/weapons.rs` | Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/hero/worn.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/hud.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/hud/meters.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/lib.rs` | Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/memcard.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/mod.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/overlay_tests.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/pause.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/pause/gadgets.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/pause/pages.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/pause/weapon_xp.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/quick_select.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/menus/vendor.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/amoeboid.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/blaster_shot.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/bomb.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/moby_update/classes/cannon_crate.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/chicken.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/crate_.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/critter.rs` | changed since supplied snapshot; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/moby_update/classes/decoy.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/devastator_missile.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/doom_bot.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/doom_canister.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/drone.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/flyer.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/gunship.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/mine.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/mine/experimental.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/mod.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/mouse.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/mouse/companion.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/path_enemy.rs` | changed since supplied snapshot; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/moby_update/classes/pickup.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/ryno_missile.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/area_stalker.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/aridia_flamer.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/aridia_sandshark.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/asteroid.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/batalia_bomber.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/batalia_fighter.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/batalia_grenadier.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/batalia_gunship.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/batalia_runner.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/batalia_tank.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/blarg_boss.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/blarg_crawler.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/blarg_trooper.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/blarg_wave_bot.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/buzz_bomb.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/drop_trooper.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/dropship.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/eudora_brawler.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/eudora_flyers.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/eudora_gunner.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/eudora_logger.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/eudora_walker.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/fleet_crew.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/fleet_small.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/fleet_turret.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/flying_biter.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/gemlik_fighter.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/gemlik_launcher.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/gemlik_robot.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/gemlik_ship.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/gemlik_tower.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/grind_mine.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/hop_gunner.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/horny_toad.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/hoven_burrower.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/hoven_copter.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/hoven_drone.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/hoven_gunner.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/hoven_mine.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/hover_ship.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/hover_zapper.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/kalebo_mine.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/kalebo_mine_drone.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/kalebo_trooper.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/kerwan_hound.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/kerwan_layer.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/kerwan_transport.rs` | v40 combat correction or audio diagnosis |
| `crates/rc-game/src/moby_update/classes/units/kerwan_trooper.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/oltanis_arcs.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/oltanis_carrier.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/oltanis_drone.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/oltanis_sentry.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/oltanis_small.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/oltanis_zapper.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/orxon_brawler.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/orxon_drone.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/orxon_flyers.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/orxon_lava.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/pack_biter.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/path_ship.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/pokitaru_biter.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/pokitaru_thrower.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/popup_turret.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/quartu_critter.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/quartu_drone.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/quartu_guard.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/quartu_hover.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/quartu_jet_bot.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/qwark_ship.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/qwark_ship_parts.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/rilgar_biter.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/rilgar_flame_tank.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/rilgar_trail_rider.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/rolling_mine.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/ship_fighter.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/umbris_beast.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/umbris_lobber.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/umbris_mines.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/umbris_swamp.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/veldin_beamer.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/veldin_boss.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/classes/units/veldin_hopper.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/units/veldin_tank.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/classes/visibomb.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/creature.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/creature/attack.rs` | changed since supplied snapshot |
| `crates/rc-game/src/moby_update/creature/damage.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/creature/fx.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/creature/knock.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/creature/react.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/creature/target.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/creature/tests.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/interact.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/real_hp.rs` | added since supplied snapshot |
| `crates/rc-game/src/moby_update/scheduler.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/moby_update/services.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |
| `crates/rc-game/src/moby_update/services_reload_tests.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/particles/type12.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/targeting.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |
| `crates/rc-game/src/weapon_evo.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; Weapon Evolution configuration or module |
| `crates/rc-game/src/weapon_evo_notice.rs` | Weapon Evolution implementation or integration |
| `crates/rc-game/src/weapon_evo_release.ini` | added since supplied snapshot; Weapon Evolution configuration or module |
| `crates/rc-game/src/weapon_evo_save.rs` | added since supplied snapshot; Weapon Evolution implementation or integration; Weapon Evolution configuration or module |
| `crates/rc-game/src/weapon_reach.rs` | added since supplied snapshot; Weapon Evolution implementation or integration |
| `launcher/build_launcher_exe.py` | Weapon Evolution implementation or integration |
| `launcher/editabile.py` | Weapon Evolution implementation or integration |
| `launcher/icon_assets.py` | Weapon Evolution implementation or integration |
| `launcher/launcher.ico` | Weapon Evolution implementation or integration |
| `launcher/launcher.py` | Weapon Evolution implementation or integration |
| `launcher/launcher_core.py` | Weapon Evolution implementation or integration |
| `launcher/migrate_ini.py` | Weapon Evolution implementation or integration |
| `launcher/schema.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_conversion.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_experimental.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_intervals.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_launcher.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_release_behaviour.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_release_tools.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_standalone.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_v44.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_v45.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_v46.py` | Weapon Evolution implementation or integration |
| `launcher/tests/test_v48.py` | Weapon Evolution implementation or integration |
| `launcher/weapons.default.ini` | Weapon Evolution implementation or integration |
| `launcher/weapons.example.ini` | Weapon Evolution implementation or integration |
| `tools/mod/check_source.py` | Weapon Evolution implementation or integration |
| `tools/mod/package_release.py` | Weapon Evolution implementation or integration |
| `tools/mod/refresh_source_map.py` | Weapon Evolution implementation or integration |
| `tools/mod/setup_msvc.bat` | Weapon Evolution implementation or integration |
| `CHANGELOG.md` | Weapon Evolution implementation, integration or developer documentation |
| `README.md` | Weapon Evolution implementation, integration or developer documentation |
| `crates/rc-game/src/hero/pda.rs` | Weapon Evolution implementation, integration or developer documentation |
| `crates/rc-game/src/moby_update/classes/bolt.rs` | Weapon Evolution implementation, integration or developer documentation |
| `crates/rc-game/src/moby_update/classes/item_offer.rs` | Weapon Evolution implementation, integration or developer documentation |
| `crates/rc-game/src/moby_update/classes/units/rilgar_story.rs` | Weapon Evolution implementation, integration or developer documentation |
| `crates/rc-game/src/moby_update/classes/vendor.rs` | Weapon Evolution implementation, integration or developer documentation |
| `crates/rc-game/src/weapon_evo_ammo.rs` | Weapon Evolution implementation, integration or developer documentation |
| `docs/mod/AMMO_UNLOCK_MODE.md` | Weapon Evolution implementation, integration or developer documentation |
| `docs/mod/CONFIG_REFERENCE.md` | Weapon Evolution implementation, integration or developer documentation |
| `docs/mod/CURRENT_IMPLEMENTATION.md` | Weapon Evolution implementation, integration or developer documentation |
| `docs/mod/DEVELOPER_GUIDE.md` | Weapon Evolution implementation, integration or developer documentation |
| `docs/mod/PUBLISHING.md` | Weapon Evolution implementation, integration or developer documentation |
| `docs/mod/VERIFICATION.md` | Weapon Evolution implementation, integration or developer documentation |
| `docs/releases/v0.3.3.md` | Weapon Evolution implementation, integration or developer documentation |
| `launcher/tests/test_v51.py` | Weapon Evolution implementation, integration or developer documentation |
