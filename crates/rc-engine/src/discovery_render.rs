//! Native weapon pickup meshes for Gemlik and seeded Gold Bolt replacements.
//! Models use the pause-menu orientation table and ready pose, not a guessed
//! ninety-degree correction. The owning pickup hides its mesh and glow together.
use crate::moby_render::{self, ExtraMobys, MobyMaterial};
use bevy::camera::visibility::VisibilitySystems;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use rc_formats::moby_anim::{self, AnimState, MobyAnimClass};
use rc_game::weapon_evo::discovery;
#[path = "gold_display.rs"]
mod display;

#[derive(Resource)]
struct RewardGfx {
    parts: Vec<RewardPart>,
    glow: crate::fx_draw::FxSlots,
}
struct RewardPart {
    item: usize,
    pitch: f32,
    roll: f32,
    extra: ExtraMobys,
    entities: Vec<Entity>,
    scale: f32,
    display_scale: f32,
    display_centre: [f32; 3],
    class_id: i32,
}

pub struct DiscoveryRenderPlugin;
impl Plugin for DiscoveryRenderPlugin {
    fn build(&self, app: &mut App) {
        if !crate::gameplay::enabled() {
            return;
        }
        app.add_systems(
            crate::level_switch::LevelUnload,
            crate::level_switch::remove::<RewardGfx>,
        )
        .add_systems(Update, (setup, draw).chain())
        .add_systems(
            PostUpdate,
            restore_visibility
                .after(crate::moby_spawn::force_hidden)
                .before(VisibilitySystems::VisibilityPropagate),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn setup(
    mut attempted: Local<bool>,
    generation: Res<crate::level_switch::LevelGeneration>,
    mut commands: Commands,
    play: Option<ResMut<crate::gameplay::Play>>,
    level: Res<crate::Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<MobyMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    if generation.is_changed() {
        *attempted = false;
    }
    if *attempted {
        return;
    }
    let Some(mut play) = play else {
        return;
    };
    *attempted = true;
    let random_mode = play.svc.bomb_evo.golden_settings.enabled;
    let mut wanted: Vec<_> = play.svc.golden.rewards.iter().map(|r| r.item).collect();
    if !random_mode && play.svc.bomb_evo.hide_ryno && play.svc.discovery.has_location() {
        wanted.push(discovery::ITEM);
    }
    // Renderer setup may run before the first offer update. Preload the small
    // authored catalog directly, so scheduler order cannot leave all cases empty.
    if play.svc.counters.times_completed > 0 {
        match play.svc.level {
            1 => wanted.extend(
                rc_game::moby_update::classes::item_offer::GOLD_ITEMS
                    .iter()
                    .map(|&i| i as usize),
            ),
            13 => wanted.extend(
                rc_game::weapon_evo::gold_shop::GEMLIK
                    .iter()
                    .map(|p| p.item),
            ),
            _ => {}
        }
    }
    wanted.sort_unstable();
    wanted.dedup();
    if wanted.is_empty() {
        return;
    }
    let gadgets = match crate::moby_attach::load_blobs() {
        Ok((_, g)) => g,
        Err(e) => {
            eprintln!("world rewards: gadget models unavailable: {e:#}");
            return;
        }
    };
    let root = crate::level_load::extracted_root();
    let empty = rc_game::menus::Overlay::default();
    let overlay = crate::gameplay::menu_overlay_for(&root, crate::level_load::level_index())
        .unwrap_or(&empty);
    let name_table = crate::disc_source::read(&root, "boot/SCUS_971.99")
        .ok()
        .zip(
            crate::disc_source::level_file(&root, crate::level_load::level_index(), "overlay.bin")
                .ok(),
        )
        .and_then(|(elf, overlay)| rc_formats::save_game::ItemTables::load(&elf, &overlay).ok());
    let mut parts = Vec::new();
    for item in wanted {
        if let Some(data) = name_table.as_ref() {
            let name = crate::gameplay::golden_item_name(
                item,
                data.item_defs_addr,
                &root,
                crate::level_load::level_index(),
                &play.svc.interact.messages,
            );
            if !name.is_empty() {
                play.svc.gold_shop.names.insert(item, name);
            }
        }
        let native_class = play
            .game
            .item_data
            .as_ref()
            .map(|d| d.def(item as i32).o_class)
            .filter(|&c| c > 0)
            .unwrap_or(if item == discovery::ITEM {
                discovery::MODEL_CLASS
            } else {
                -1
            });
        // Drone Device's inventory definition names the Hologuise model. Its
        // actual shield drone is a resident level class, also used by the native
        // pause-menu preview. Never substitute the unrelated inventory mesh.
        let class_id = display::model_class(item, native_class);
        let loaded = if let Some(g) = gadgets.iter().find(|g| g.moby.o_class == class_id) {
            moby_anim::parse_sequences(&g.blob, &g.moby.class)
                .ok()
                .map(|seqs| (&g.moby, MobyAnimClass::new(&g.moby.class, seqs)))
        } else {
            level
                .0
                .mobys
                .classes
                .iter()
                .enumerate()
                .find(|(_, c)| c.o_class == class_id)
                .and_then(|(index, class)| {
                    level
                        .0
                        .mobys
                        .anim
                        .get(index)
                        .cloned()
                        .map(|anim| (class, anim))
                })
        };
        let Some((class, animation)) = loaded else {
            eprintln!("world reward: item {item} model {class_id} unavailable; pickup disabled");
            continue;
        };
        let mut state = AnimState::spawn(&animation);
        let sequence = if item == 24 {
            0
        } else {
            1.min(animation.sequences.len().saturating_sub(1) as u8)
        };
        moby_anim::hard_cut(&mut state, &animation, sequence, 0);
        state.t = 0.;
        state.speed = 0.;
        let pose = moby_anim::evaluate_with_snapshot(&animation, &state, None);
        let joints = (ExtraMobys::max_skinned_joint(class) as u32 + 1)
            .max(pose.len() as u32)
            .max(1);
        let mut palette = crate::moby_anim::identity_palette(joints);
        for (i, byte) in pose
            .iter()
            .flat_map(|row| row.iter().flatten().flat_map(|f| f.to_le_bytes()))
            .enumerate()
        {
            if let Some(target) = palette.get_mut(i) {
                *target = byte;
            }
        }
        let scale = class.class.header.scale;
        if !scale.is_finite() || scale <= 0. {
            continue;
        }
        // Native preview: table +0x20*item has authored X/Y rotations in words
        // 4/5; word 6/7 contain pivot offsets, not another Euler rotation.
        let angle = |word: u32| {
            overlay
                .u32(
                    overlay.at(rc_game::menus::pause::gadgets::PREVIEW_TABLE)
                        + 0x20 * item as u32
                        + word * 4,
                )
                .map(f32::from_bits)
                .filter(|f| f.is_finite())
        };
        let (Some(roll), Some(pitch)) = (
            if item == 24 { Some(0.) } else { angle(4) },
            if item == 24 { Some(0.) } else { angle(5) },
        ) else {
            eprintln!("world reward: item {item} preview orientation unavailable; pickup disabled");
            continue;
        };
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for mesh in class.class.high_lod.iter().chain(&class.class.metal) {
            for v in &mesh.vertices {
                let point = class.class.position(v);
                for k in 0..3 {
                    lo[k] = lo[k].min(point[k]);
                    hi[k] = hi[k].max(point[k]);
                }
            }
        }
        let extent = (0..3).map(|k| hi[k] - lo[k]).fold(0.0f32, f32::max);
        let fit = display::scale_multiplier(item, extent);
        let display_scale = scale * fit;
        let display_centre = std::array::from_fn(|k| {
            if lo[k].is_finite() && hi[k].is_finite() {
                (lo[k] + hi[k]) * 0.5 * fit
            } else {
                0.
            }
        });
        let mut extra = ExtraMobys::new(
            &level.0,
            vec![0; moby_render::EXTRA_RECORD_SIZE],
            palette,
            &mut buffers,
        );
        let entities = extra.spawn(
            &mut commands,
            &level.0,
            class,
            0,
            Transform::IDENTITY,
            "Weapon Evolution world reward",
            &mut meshes,
            &mut images,
            &mut materials,
        );
        if entities.is_empty() {
            continue;
        }
        for &entity in &entities {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        if random_mode {
            play.svc.golden.set_ready(item);
        } else if item == discovery::ITEM && play.svc.bomb_evo.hide_ryno {
            play.svc.discovery.set_ready();
        }
        play.svc.gold_shop.ready.insert(item);
        parts.push(RewardPart {
            item,
            pitch,
            roll,
            extra,
            entities,
            scale,
            display_scale,
            display_centre,
            class_id,
        });
    }
    commands.insert_resource(RewardGfx {
        parts,
        glow: Default::default(),
    });
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    gfx: Option<ResMut<RewardGfx>>,
    play: Option<Res<crate::gameplay::Play>>,
    level: Res<crate::Level>,
    fog: Option<Res<crate::game_camera::GameFog>>,
    mut vis: Query<&mut Visibility>,
    mut transforms: Query<&mut Transform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut fx_materials: ResMut<Assets<crate::fx_draw::FxPrimMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    let (Some(mut gfx), Some(play)) = (gfx, play) else {
        return;
    };
    let alive = !play.death_pending && play.game.hero.health > 0;
    let mut visuals = play
        .svc
        .golden
        .visuals(&play.svc.bomb_evo.golden_progress(), alive);
    if alive {
        if let Some(v) = play.svc.discovery.visual() {
            visuals.push((discovery::ITEM, v));
        }
    }
    let shop: Vec<_> = play
        .svc
        .gold_shop
        .displays
        .iter()
        .filter(|d| {
            play.svc.gold_shop.item_visible(
                &play.svc.bomb_evo,
                play.svc.counters.times_completed,
                &play.game.hero.owned.0,
                d.item,
            )
        })
        .copied()
        .collect();
    for d in &shop {
        visuals.push((
            d.item,
            discovery::Visual {
                position: d.position,
                yaw: d.yaw,
            },
        ));
    }
    let mut groups = Vec::new();
    for part in &mut gfx.parts {
        let visual = visuals
            .iter()
            .find(|(item, _)| *item == part.item)
            .map(|(_, v)| *v);
        for &entity in &part.entities {
            if let Ok(mut visibility) = vis.get_mut(entity) {
                visibility.set_if_neq(if visual.is_some() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
        if let Some(visual) = visual {
            let displayed = shop.iter().any(|d| d.item == part.item);
            // Quarter-turn only the stationary shop models. Keep the native
            // upright pose, selected scale and display-center anchor; world
            // pickups retain their own spinning orientation.
            let yaw = visual.yaw
                + if displayed {
                    std::f32::consts::FRAC_PI_2
                } else {
                    0.
                };
            let rows = rc_formats::moby_light::rotation_rows([part.roll, part.pitch, yaw]);
            let rows = rows.map(|r| [r[0], r[1], r[2]].map(f32::from_bits));
            let at = if displayed {
                std::array::from_fn(|i| {
                    visual.position[i]
                        - (0..3)
                            .map(|j| rows[j][i] * part.display_centre[j])
                            .sum::<f32>()
                })
            } else {
                visual.position
            };
            let model = moby_render::extra_model(
                rows,
                if displayed {
                    part.display_scale
                } else {
                    part.scale
                },
                at,
            );
            part.extra
                .set_gold(&mut commands, 0, part.class_id, displayed);
            let record = moby_render::extra_record(&model, None, 0);
            crate::asset_write::set_buffer(&mut buffers, &part.extra.instances, &record);
            let transform = Transform::from_matrix(model);
            for &entity in &part.entities {
                if let Ok(mut t) = transforms.get_mut(entity) {
                    t.set_if_neq(transform);
                }
            }
            if displayed {
                continue;
            }
            let mut glow = crate::fx_draw::PrimBuf::default();
            crate::fx_draw::glow_quad(
                &mut glow,
                2.,
                0.15,
                visual.position,
                0x4040_ffff,
                play.game.camera.out.pos_f32(),
            );
            groups.push(crate::fx_draw::FxGroup {
                fx: crate::fx_draw::GLOW_FX,
                additive: true,
                subtract: false,
                prims: glow,
            });
        }
    }
    let fog = fog
        .map(|f| f.uniform)
        .unwrap_or_else(|| crate::game_camera::TfragFog::new(&level.0.fog));
    let tex = level
        .0
        .particles
        .textures
        .as_ref()
        .map(|t| t.fx_textures.as_slice());
    let mut assets = crate::fx_draw::FxAssets {
        meshes: &mut meshes,
        images: &mut images,
        materials: &mut fx_materials,
        fx: tex,
        fog,
    };
    gfx.glow.show(
        &mut commands,
        &mut vis,
        &mut assets,
        groups,
        crate::fx_draw::LIST1_BIAS + 51.,
        "Weapon Evolution world reward glow",
    );
}

/// ExtraMobys has its own instance buffer, whose local slot 0 is unrelated to
/// placed instance 0. Loader/respawn queries can nevertheless apply SpawnHidden
/// through the shared MeshTag. Reassert this pickup's visibility after those
/// rules, before visibility propagation, and remove only its own stray marker.
/// Do not exempt Ratchet or any native placed actor from their hiding rules.
fn restore_visibility(
    mut commands: Commands,
    gfx: Option<Res<RewardGfx>>,
    play: Option<Res<crate::gameplay::Play>>,
    hidden: Query<(), With<crate::moby_spawn::SpawnHidden>>,
    mut visibility: Query<&mut Visibility>,
) {
    let Some(gfx) = gfx else {
        return;
    };
    let Some(play) = play else {
        return;
    };
    let alive = !play.death_pending && play.game.hero.health > 0;
    let mut visuals = play
        .svc
        .golden
        .visuals(&play.svc.bomb_evo.golden_progress(), alive);
    if alive {
        if let Some(v) = play.svc.discovery.visual() {
            visuals.push((discovery::ITEM, v));
        }
    }
    for part in &gfx.parts {
        let in_shop = play
            .svc
            .gold_shop
            .displays
            .iter()
            .any(|d| d.item == part.item)
            && play.svc.gold_shop.item_visible(
                &play.svc.bomb_evo,
                play.svc.counters.times_completed,
                &play.game.hero.owned.0,
                part.item,
            );
        let visible = in_shop || visuals.iter().any(|(item, _)| *item == part.item);
        for &entity in &part.entities {
            if hidden.contains(entity) {
                commands
                    .entity(entity)
                    .remove::<crate::moby_spawn::SpawnHidden>();
            }
            if let Ok(mut v) = visibility.get_mut(entity) {
                v.set_if_neq(if visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                });
            }
        }
    }
}
