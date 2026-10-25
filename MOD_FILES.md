# Weapon Evolution source map

Unreleased development snapshot: gameplay 0.1.6.83, launcher 0.27.1, ReRAC 0.3.6-alpha. The complete supplied workspace is included; the 446 reviewed entries below cover mod code, integrations, maintenance and base synchronization. A file being indexed does not attribute its upstream code to the mod author.

Start with the [current implementation](docs/mod/CURRENT_IMPLEMENTATION.md), [saved controls](docs/mod/SAVED_CONTROLS.md), [developer guide](docs/mod/DEVELOPER_GUIDE.md) and [credits](docs/mod/CREDITS.md). Machine-readable hashes are in [mod/source-map.json](mod/source-map.json). The historical snapshot patch remains an account of the original export, not a complete current delta.

| Path | Role |
| --- | --- |

| `.github/workflows/windows-release.yml` | Weapon Evolution implementation or integration |

| `AGENTS.md` | Weapon Evolution implementation, integration or documentation |

| `CHANGELOG.md` | Weapon Evolution implementation or integration |

| `Cargo.lock` | Weapon Evolution implementation, integration or documentation |

| `Cargo.toml` | Weapon Evolution implementation, integration or documentation |

| `MOD_FILES.md` | Weapon Evolution implementation or integration |

| `README.md` | Weapon Evolution implementation or integration |

| `crates/rc-engine/Cargo.toml` | changed since supplied snapshot |

| `crates/rc-engine/assets/shaders/hud.wgsl` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/assets/shaders/screen_tint.wgsl` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/assets/shaders/tfrag.wgsl` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/assets/shaders/tie.wgsl` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/assets/shaders/world_lights.wgsl` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/audio_out.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/companion_assets.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/disc_source.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-engine/src/discovery_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/fly_cam.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/fog_state.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/frame_log.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/frame_pace.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/fx_draw.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/gameplay.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |

| `crates/rc-engine/src/gold_display.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/gs_state.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/hud_render.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |

| `crates/rc-engine/src/interact_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/level_load.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/level_switch.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/main.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-engine/src/menu_models.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/menu_render.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/moby_anim.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/moby_attach.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/moby_render.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/moby_spawn.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/occlusion.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/play_camera.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/rac2.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/rac2_level.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/rac2_play.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/reactive_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/saves.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/scene_render.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/screen_tint.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/shrub_billboard.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/shrub_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/sky_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/tesla_render.rs` | changed since supplied snapshot |

| `crates/rc-engine/src/tfrag_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/tie_lod.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/tie_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/travel_render.rs` | Weapon Evolution implementation or integration |

| `crates/rc-engine/src/upgrade_wave_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-engine/src/wrench_swing_render.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-extract/data/scus_972_68.tsv` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-extract/src/archive.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-extract/src/build_db.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-extract/src/extract.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-extract/src/identify.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/examples/overlay_recognition_perf.rs` | Weapon Evolution implementation or integration |

| `crates/rc-formats/examples/rac2_probe.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/gadget.rs` | Weapon Evolution implementation or integration |

| `crates/rc-formats/src/level.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/level_overlay.rs` | Weapon Evolution implementation or integration |

| `crates/rc-formats/src/lib.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/moby.rs` | Weapon Evolution implementation or integration |

| `crates/rc-formats/src/moby_anim.rs` | changed since supplied snapshot |

| `crates/rc-formats/src/rac2/armor.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/boot.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/front_end.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/gameplay.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/gui_sprite.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/level.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/mod.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/rac2/toc.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/shrub.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/test_data.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/src/tie.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/tests/formats/main.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-formats/tests/formats/rac2_disc.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/Cargo.toml` | changed since supplied snapshot |

| `crates/rc-game/src/audio.rs` | v40 combat correction or audio diagnosis |

| `crates/rc-game/src/audio/class_sounds.rs` | v40 combat correction or audio diagnosis |

| `crates/rc-game/src/audio/import.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/audio/music.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/audio/scene.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/audio/voices.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/cheats.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/cinematic.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/follow_camera.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/follow_camera/level.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/game_state.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/air.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/anim.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/blaster.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/bodies.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/bodies/giant.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/boots.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/combat_legs.rs` | added since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/combat_strafe.rs` | added since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/comet.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/damage.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/devastator.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/fx.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/gloves.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |

| `crates/rc-game/src/hero/glow.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/ground.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/guns.rs` | changed since supplied snapshot |

| `crates/rc-game/src/hero/idle.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/items.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/jump.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/melee.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/morph_ray.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/pda.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/physics.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/pyro_lava.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/pyrocitor.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/ryno.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/states.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/suck_cannon.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/surface.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hero/taunter.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/tesla.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/visibomb.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/walk.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/walloper.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hero/weapons.rs` | Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |

| `crates/rc-game/src/hero/worn.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hud.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/hud/calls.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/hud/meters.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/hud/xp_bar.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/lib.rs` | Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |

| `crates/rc-game/src/map.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/memcard.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/freeze.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/mod.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/overlay_tests.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/pause.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/pause/gadgets.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/pause/pages.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/pause/planet_select.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/pause/port.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/pause/saves.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/pause/shortcuts.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/pause/tests.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/pause/weapon_xp.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/quick_select.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/vendor.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/menus/vendor/reload_all.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/vendor/reload_tests.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/menus/vendor/tests.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/amoeboid.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/blaster_shot.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/bolt.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/bomb.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; original mod integration backup exists in supplied snapshot |

| `crates/rc-game/src/moby_update/classes/bomb/visibomb_grenade.rs` | Weapon Evolution implementation or integration |

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

| `crates/rc-game/src/moby_update/classes/gold_bolt.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/gunship.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/item_offer.rs` | Weapon Evolution implementation or integration |

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

| `crates/rc-game/src/moby_update/classes/units/eudora_story.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/eudora_walker.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/fleet_crew.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/fleet_small.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/fleet_turret.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/flying_biter.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/gaspar_cannon.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/gemlik_fighter.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/gemlik_launcher.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/gemlik_robot.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/gemlik_ship.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/gemlik_tower.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/gemlik_turret.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/grind_mine.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/hop_gunner.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/horny_toad.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/hoven_burrower.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/hoven_copter.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/hoven_drone.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/hoven_fall.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/hoven_gunner.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/hoven_mine.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/hover_ship.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/hover_zapper.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/kalebo_mine.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/kalebo_mine_drone.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/kalebo_race.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/kalebo_trooper.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/kerwan_bystander.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/kerwan_hound.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/kerwan_layer.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/kerwan_transport.rs` | v40 combat correction or audio diagnosis |

| `crates/rc-game/src/moby_update/classes/units/kerwan_trooper.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/light_fixture.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/linked_mover.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/linked_slider.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/oltanis_arcs.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/oltanis_carrier.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/oltanis_drone.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/oltanis_sentry.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/oltanis_small.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/oltanis_story.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/oltanis_zapper.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/orxon_brawler.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/orxon_drone.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/orxon_flyers.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/orxon_lava.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/orxon_story.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

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

| `crates/rc-game/src/moby_update/classes/units/quartu_small.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/qwark_ship.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/qwark_ship_parts.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/rilgar_biter.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/rilgar_flame_tank.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/rilgar_story.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/rilgar_trail_rider.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/rolling_mine.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/ship_fighter.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/umbris_beast.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/umbris_lobber.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/umbris_mines.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/umbris_swamp.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/umbris_turret.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/classes/units/veldin_beamer.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/veldin_boss.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/classes/units/veldin_hopper.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/units/veldin_tank.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/vendor.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/classes/visibomb.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/creature.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/creature/attack.rs` | changed since supplied snapshot |

| `crates/rc-game/src/moby_update/creature/damage.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/moby_update/creature/duplication_tests.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/moby_update/creature/flame.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

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

| `crates/rc-game/src/pad.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/particles.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/particles/type12.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/particles/type32.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/particles/type60.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/particles/type70.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/scene_actors.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/targeting.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/src/tick.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/travel/cards.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/travel/space.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/water/sea.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/src/weapon_evo.rs` | changed since supplied snapshot; Weapon Evolution implementation or integration; Weapon Evolution configuration or module |

| `crates/rc-game/src/weapon_evo_ammo.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/weapon_evo_bolt_chain.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_controls.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_controls_tests.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_discovery.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_gemlik_test.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_gold_shop.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_golden.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_golden/layout.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_golden/locations.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_golden/tests.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_legacy_gold.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_location_test.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_motion.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_notice.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/weapon_evo_population.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_release.ini` | added since supplied snapshot; Weapon Evolution configuration or module |

| `crates/rc-game/src/weapon_evo_save.rs` | added since supplied snapshot; Weapon Evolution implementation or integration; Weapon Evolution configuration or module |

| `crates/rc-game/src/weapon_evo_upgrade_tests.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_upgrade_wave.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_v76_tests.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_v77_tests.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_world.rs` | Weapon Evolution implementation, integration or documentation |

| `crates/rc-game/src/weapon_evo_wrench.rs` | Weapon Evolution implementation or integration |

| `crates/rc-game/src/weapon_reach.rs` | added since supplied snapshot; Weapon Evolution implementation or integration |

| `crates/rc-game/tests/capture_pickup_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/classes/orxon_gemlik_enemies.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/controls_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/encounter_effects_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/golden_render_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/hero/hero_surfaces.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/lava_render_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/local_world_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/nanotech_vendor_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/pursuit_api.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/ui/gemlik_generalisation.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/ui/pause_pages_novalis.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/weapons/hero_doom.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/weapons/hero_gameplay_novalis.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game/tests/weapons/hero_gloves.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/Cargo.toml` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/examples/rac2_coll.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/camera.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/front_end.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/gui.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/air.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/ground.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/jump.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/physics.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/registry.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/states.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/hero/walk.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `crates/rc-game2/src/lib.rs` | ReRAC 0.3.6-alpha supplied base synchronization |

| `docs/mod/AMMO_UNLOCK_MODE.md` | Weapon Evolution implementation or integration |

| `docs/mod/BUILD_LAYOUT.md` | Weapon Evolution implementation or integration |

| `docs/mod/CONFIG_REFERENCE.md` | Weapon Evolution implementation or integration |

| `docs/mod/CREDITS.md` | Weapon Evolution implementation or integration |

| `docs/mod/CURRENT_IMPLEMENTATION.md` | Weapon Evolution implementation or integration |

| `docs/mod/DEVELOPER_GUIDE.md` | Weapon Evolution implementation or integration |

| `docs/mod/FLAME_MINE_MODEL_SCALE.md` | Weapon Evolution implementation or integration |

| `docs/mod/LAUNCHER_RESTORE_V58_1.md` | Weapon Evolution implementation or integration |

| `docs/mod/LAUNCHER_V55.md` | Weapon Evolution implementation or integration |

| `docs/mod/LOCAL_TEST_KIT.md` | Weapon Evolution implementation or integration |

| `docs/mod/MINE_AMMO_UI_V57.md` | Weapon Evolution implementation or integration |

| `docs/mod/MODELS_AND_STATIC_PAGES_V58.md` | Weapon Evolution implementation or integration |

| `docs/mod/PUBLISHING.md` | Weapon Evolution implementation or integration |

| `docs/mod/SAVED_CONTROLS.md` | Weapon Evolution implementation, integration or documentation |

| `docs/mod/VERIFICATION.md` | Weapon Evolution implementation or integration |

| `docs/mod/VISIBOMB_GRENADES_V53.md` | Weapon Evolution implementation or integration |

| `docs/mod/WRENCH_AND_PROJECTILES_V54.md` | Weapon Evolution implementation or integration |

| `docs/mod/WRENCH_TUNING_V56.md` | Weapon Evolution implementation or integration |

| `docs/releases/v0.3.3.md` | Weapon Evolution implementation or integration |

| `docs/releases/v0.3.md` | Weapon Evolution implementation or integration |

| `docs/weapon-evolution/V81_SAVED_CONTROLS.md` | Weapon Evolution implementation, integration or documentation |

| `docs/weapon-evolution/V82_GADGET_SHORTCUTS.md` | Weapon Evolution implementation, integration or documentation |

| `launcher/BUILD_FINAL_WINDOWS_EXE.bat` | Weapon Evolution implementation, integration or documentation |

| `launcher/CREA_LAUNCHER_EXE.bat` | Weapon Evolution implementation, integration or documentation |

| `launcher/Editor.bat` | Weapon Evolution implementation, integration or documentation |

| `launcher/LEGGIMI.txt` | Weapon Evolution implementation, integration or documentation |

| `launcher/build_launcher_exe.py` | Weapon Evolution implementation or integration |

| `launcher/editabile.py` | Weapon Evolution implementation or integration |

| `launcher/final_release.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/i18n.py` | Weapon Evolution implementation or integration |

| `launcher/icon_assets.py` | Weapon Evolution implementation or integration |

| `launcher/launcher.ico` | Weapon Evolution implementation or integration |

| `launcher/launcher.py` | Weapon Evolution implementation or integration |

| `launcher/launcher_core.py` | Weapon Evolution implementation or integration |

| `launcher/migrate_ini.py` | Weapon Evolution implementation or integration |

| `launcher/schema.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_additive_nanotech.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_conversion.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_eight_grades.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_encounter_ui.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_experimental.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_intervals.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_launcher.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_nanotech_ui.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_ngplus_ui.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_pursuit_ui.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_release_behaviour.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_release_publish.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_release_tools.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_saved_controls.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/tests/test_standalone.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v44.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v45.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v46.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v48.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v51.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v52.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v53.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v54.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v55.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v56.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v57.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v58.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_v58_1.py` | Weapon Evolution implementation or integration |

| `launcher/tests/test_wrench_trail.py` | Weapon Evolution implementation, integration or documentation |

| `launcher/weapons.default.ini` | Weapon Evolution implementation or integration |

| `launcher/weapons.example.ini` | Weapon Evolution implementation or integration |

| `launcher/windows_entry.py` | Weapon Evolution implementation, integration or documentation |

| `mod/development.json` | Weapon Evolution implementation, integration or documentation |

| `mod/provenance.json` | Weapon Evolution implementation or integration |

| `mod/release.json` | Weapon Evolution implementation or integration |

| `tools/ghidra/README.md` | Weapon Evolution implementation, integration or documentation |

| `tools/ghidra/scripts/crossgame_diff.py` | Weapon Evolution implementation, integration or documentation |

| `tools/ghidra/scripts/export_decomp.py` | Weapon Evolution implementation, integration or documentation |

| `tools/ghidra/scripts/import_rc2.py` | Weapon Evolution implementation, integration or documentation |

| `tools/ghidra/scripts/overlay_to_elf.py` | Weapon Evolution implementation, integration or documentation |

| `tools/mod/check_source.py` | Weapon Evolution implementation or integration |

| `tools/mod/create_local_test_kit.py` | Weapon Evolution implementation or integration |

| `tools/mod/generate_config_reference.py` | Weapon Evolution implementation, integration or documentation |

| `tools/mod/local_test_update.py` | Weapon Evolution implementation or integration |

| `tools/mod/package_release.py` | Weapon Evolution implementation or integration |

| `tools/mod/publish_release.py` | Weapon Evolution implementation or integration |

| `tools/mod/refresh_source_map.py` | Weapon Evolution implementation or integration |

| `tools/mod/setup_msvc.bat` | Weapon Evolution implementation or integration |
