//! Native weapon pickup meshes for Gemlik and seeded Gold Bolt replacements.
//! Models use the pause-menu orientation table and ready pose, not a guessed
//! ninety-degree correction. The owning pickup hides its mesh and glow together.
use crate::moby_render::{self, ExtraMobys, MobyMaterial};
use bevy::camera::visibility::VisibilitySystems;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use rc_formats::moby_anim::{self, AnimState, MobyAnimClass};
use rc_game::weapon_evo::{discovery, pickup_visuals};
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
    extra: Option<ExtraMobys>,
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
        crate::reward_render_probe::announce();
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
    mut reward_materials: ResMut<Assets<crate::reward_model_render::RewardMaterial>>,
    fog: Option<Res<crate::game_camera::GameFog>>,
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
    let mixed=play.svc.bomb_evo.randomizer_mode;
    let mut wanted: Vec<_> = play.svc.golden.rewards.iter().map(|r| r.item).collect();
    if mixed {
        wanted.extend(play.svc.randomizer.anchors.iter().map(|a|1000+a.source));
    }
    if !random_mode && !mixed && play.svc.bomb_evo.hide_ryno && play.svc.discovery.has_location() {
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
    if mixed {
        if let Some(data)=name_table.as_ref(){
            for item in 2..37 {
                let name=crate::gameplay::golden_item_name(item,data.item_defs_addr,&root,crate::level_load::level_index(),&play.svc.interact.messages);
                if !name.is_empty(){play.svc.gold_shop.names.insert(item,name);}
            }
        }
    }
    for key in wanted {
        let item=if key>=1000{play.svc.bomb_evo.progress().randomizer.reward(key-1000).unwrap_or(0) as usize}else{key};
        if let Some(data) = name_table.as_ref().filter(|_|item<37) {
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
        let override_class=if key>=1000 {
            match item {65..=82|48..=53|96.. =>Some(rc_game::weapon_evo::randomizer::display_class(item as u16,-1)),_=>None}
        } else {None};
        let native_class = override_class.unwrap_or_else(||play
            .game
            .item_data
            .as_ref().filter(|_|item<37)
            .map(|d| d.def(item as i32).o_class)
            .filter(|&c| c > 0)
            .unwrap_or(if item == discovery::ITEM {
                discovery::MODEL_CLASS
            } else {
                -1
            }));
        // Flag-only rewards use a textured ammo-crate display, not a native actor.
        let native_class=if key>=1000 && native_class<=0{511}else{native_class};
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
        let sequence=pickup_visuals::pose_sequence(class_id,sequence);
        moby_anim::hard_cut(&mut state, &animation, sequence, 0);
        state.t = 0.;
        state.speed = 0.;
        let pose = moby_anim::evaluate_with_snapshot(&animation, &state, None);
        // Mixed rewards are frozen display meshes, not initialized native actors.
        // Fit the geometry AFTER skinning and compact it before any GPU upload.
        let prepared=if key>=1000 {
            match rc_game::weapon_evo::pickup_mesh::prepare(class,&pose) {
                Ok(prepared)=>Some(prepared),
                Err(error)=>{
                    eprintln!("world reward: source {} class {class_id}: invalid display geometry: {error}; not marked ready",key-1000);
                    continue;
                }
            }
        }else{None};
        if let Some(prepared)=&prepared {
            println!("reward bounded mesh: key={key} reward={item} class={class_id} posed_extent={} bind_fallback={} world_extent={}",
                prepared.posed_extent,prepared.bind_fallback,rc_game::weapon_evo::pickup_mesh::WORLD_EXTENT);
        }
        let class=prepared.as_ref().map_or(class,|prepared|&prepared.class);
        let pose=if prepared.is_some(){vec![moby_anim::IDENTITY]}else{pose};
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
            if item == 24 || item>=37 { Some(0.) } else { angle(4) },
            if item == 24 || item>=37 { Some(0.) } else { angle(5) },
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
        let fit = if key>=1000 {
            let Some(fit)=display::world_scale_multiplier(extent) else{
                eprintln!("world reward: source {} has invalid model extent; pickup disabled",key-1000);continue;
            };fit
        }else{display::scale_multiplier(item, extent)};
        let display_scale = scale * fit;
        let display_centre = std::array::from_fn(|k| {
            if lo[k].is_finite() && hi[k].is_finite() {
                (lo[k] + hi[k]) * 0.5 * fit
            } else {
                0.
            }
        });
        println!("reward render mesh: key={key} reward={item} class={class_id} native_scale={scale} fit={fit} extent={extent} centre={display_centre:?}");
        let (extra,entities)=if key>=1000 {
            let fog=fog.as_ref().map(|f|f.uniform)
                .unwrap_or_else(||crate::game_camera::TfragFog::new(&level.0.fog));
            match crate::reward_model_render::spawn(&mut commands,&level.0,class,fog,
                &mut meshes,&mut images,&mut reward_materials){
                Ok(entities)=>(None,entities),
                Err(error)=>{eprintln!("world reward: source {} standalone model failed: {error}",key-1000);continue;}
            }
        }else{
            let mut extra=ExtraMobys::new(
                &level.0,vec![0;moby_render::EXTRA_RECORD_SIZE],palette,&mut buffers,
            );
            let entities=extra.spawn(
                &mut commands,&level.0,class,0,Transform::IDENTITY,
                "Weapon Evolution world reward",&mut meshes,&mut images,&mut materials,
            );
            (Some(extra),entities)
        };
        if entities.is_empty() {
            continue;
        }
        for &entity in &entities {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        if key>=1000 {
            play.svc.randomizer.ready.insert(key-1000);
        } else if random_mode {
            play.svc.golden.set_ready(item);
        } else if item == discovery::ITEM && play.svc.bomb_evo.hide_ryno {
            play.svc.discovery.set_ready();
        }
        play.svc.gold_shop.ready.insert(item);
        parts.push(RewardPart {
            item:key,
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
    mut probe_frames: Local<u32>,
) {
    let options=crate::reward_render_probe::options();
    let (Some(mut gfx), Some(play)) = (gfx, play) else {
        return;
    };
    let alive = !play.death_pending && play.game.hero.health > 0;
    let mut visuals = play
        .svc
        .golden
        .visuals(&play.svc.bomb_evo.golden_progress(), alive);
    visuals.extend(play.svc.randomizer.visuals(play.svc.bomb_evo.progress().randomizer,alive));
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
                visibility.set_if_neq(if visual.is_some() && options.models {
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
            // Native VU trig accepts bounded angles, not an elapsed-turn clock.
            // This applies to both standalone rewards and native gold exhibits.
            let euler = pickup_visuals::world_euler(part.class_id,[part.roll, part.pitch,yaw],yaw);
            let rows = pickup_visuals::display_rows(euler);
            let at = if displayed || part.item>=1000 {
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
                if displayed || part.item>=1000 {
                    part.display_scale
                } else {
                    part.scale
                },
                at,
            );
            if let Some(extra)=&mut part.extra {
                extra.set_gold(&mut commands,0,part.class_id,displayed);
                let record=moby_render::extra_record(&model,None,0);
                crate::asset_write::set_buffer(&mut buffers,&extra.instances,&record);
            }
            let transform = Transform::from_matrix(model);
            for &entity in &part.entities {
                if let Ok(mut t) = transforms.get_mut(entity) {
                    t.set_if_neq(transform);
                }
            }
            if displayed || !options.halos {
                continue;
            }
            let Some(halo)=pickup_visuals::halo(visual.position,play.game.camera.out.pos_f32()) else{continue;};
            let mut glow=crate::fx_draw::PrimBuf::default();
            for k in 0..pickup_visuals::HALO_SEGMENTS {
                glow.strip([
                    (halo.centre,[0.5,0.5],pickup_visuals::HALO_CENTRE_RGBA),
                    (halo.rim[k],[0.5,0.5],pickup_visuals::HALO_EDGE_RGBA),
                    (halo.rim[(k+1)%pickup_visuals::HALO_SEGMENTS],[0.5,0.5],pickup_visuals::HALO_EDGE_RGBA),
                ]);
            }
            // Source-over, not additive: overlapping halos remain bounded.
            // Use a solid texel and vertex alpha, never a level-dependent FX
            // texture whose dark RGB corners can acquire the blue fog colour.
            groups.push(crate::fx_draw::FxGroup {
                fx:crate::fx_draw::SOLID_FX,additive:false,subtract:false,prims:glow,
            });
        }
    }
    *probe_frames=probe_frames.wrapping_add(1);
    if (*probe_frames).is_multiple_of(300) {
        println!("reward render probe frame: level={} mode={} candidates={} halos={} models_enabled={}",play.svc.level,options.name,visuals.len(),groups.len(),options.models);
    }
    let mut fog = fog
        .map(|f| f.uniform)
        .unwrap_or_else(|| crate::game_camera::TfragFog::new(&level.0.fog));
    // Glow colour must not become the level fog colour at long distance.
    fog.color.w=0.;
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
    visuals.extend(play.svc.randomizer.visuals(play.svc.bomb_evo.progress().randomizer,alive));
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
        let visible = crate::reward_render_probe::options().models
            && (in_shop || visuals.iter().any(|(item, _)| *item == part.item));
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
