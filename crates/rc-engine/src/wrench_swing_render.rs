//! A short filled ribbon between successive authored wrench-head poses.
//! This renderer is cosmetic: no hit records, lights, actors or RNG are created.
use crate::fx_draw::{FxAssets, FxGroup, FxSlots, PrimBuf, SOLID_FX};
use bevy::prelude::*;
#[derive(Clone, Copy)]
struct Sample {
    hand: [f32; 3],
    tip: [f32; 3],
    born: u64,
    style: rc_game::weapon_evo::wrench::TrailStyle,
}
#[derive(Resource, Default)]
struct Swing {
    last: Option<u64>,
    samples: Vec<Sample>,
    slots: FxSlots,
}
pub struct WrenchSwingPlugin;
impl Plugin for WrenchSwingPlugin {
    fn build(&self, app: &mut App) {
        if !crate::gameplay::enabled() {
            return;
        }
        app.add_systems(
            crate::level_switch::LevelUnload,
            crate::level_switch::remove::<Swing>,
        )
        .add_systems(Update, draw);
    }
}
#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    play: Option<Res<crate::gameplay::Play>>,
    mut swing: Option<ResMut<Swing>>,
    level: Res<crate::Level>,
    fog: Option<Res<crate::game_camera::GameFog>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<crate::fx_draw::FxPrimMaterial>>,
    mut vis: Query<&mut Visibility>,
) {
    let Some(p) = play else {
        return;
    };
    let Some(ref mut s) = swing else {
        commands.insert_resource(Swing::default());
        return;
    };
    let h = &p.game.hero;
    let now = p.game.counter;
    let active = h.health > 0
        && !p.death_pending
        && h.items.slot.id == 8
        && matches!(h.state, 0x13 | 0x14 | 0x70 | 0x2b);
    if !active || s.last.is_some_and(|n| now < n) {
        s.samples.clear();
    }
    if active && s.last != Some(now) {
        let hand = [0, 1, 2].map(|i| h.melee.hand[i].to_f32());
        let tip = [0, 1, 2].map(|i| h.melee.tip[i].to_f32());
        let length = (0..3).map(|i| (tip[i] - hand[i]).powi(2)).sum::<f32>();
        if length.is_finite() && (0.05..16.).contains(&length) {
            s.samples.push(Sample {
                hand,
                tip,
                born: now,
                style: rc_game::weapon_evo::wrench::TrailStyle::capture(&p.svc.bomb_evo, h.melee.tuning),
            });
        }
    }
    s.last = Some(now);
    s.samples.retain(|n| now.saturating_sub(n.born) < 7);
    let mut prims = PrimBuf::default();
    for pair in s.samples.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let d = (0..3).map(|i| (a.tip[i] - b.tip[i]).powi(2)).sum::<f32>();
        if !(0.0001..9.).contains(&d) {
            continue;
        }
        // Reject discontinuities in native space above, before scaling. A large
        // attack radius must not make a valid swing fail the teleport guard.
        let outer = |n: Sample| n.style.edge(n.hand, n.tip);
        let inner = |n: Sample| {
            let edge = outer(n);
            std::array::from_fn(|i| n.hand[i] + (edge[i] - n.hand[i]) * 0.45)
        };
        let alpha = (64 * (7 - now.saturating_sub(b.born).min(7)) / 7) as u32;
        prims.quad(
            [inner(a), inner(b), outer(a), outer(b)],
            [[0.5, 0.5]; 4],
            [alpha << 24 | b.style.color; 4],
        );
    }
    let groups = if prims.is_empty() {
        vec![]
    } else {
        vec![FxGroup {
            fx: SOLID_FX,
            additive: false,
            subtract: false,
            prims,
        }]
    };
    let fog = fog
        .map(|f| f.uniform)
        .unwrap_or_else(|| crate::game_camera::TfragFog::new(&level.0.fog));
    let mut assets = FxAssets {
        meshes: &mut meshes,
        images: &mut images,
        materials: &mut materials,
        fx: level
            .0
            .particles
            .textures
            .as_ref()
            .map(|t| t.fx_textures.as_slice()),
        fog,
    };
    s.slots.show(
        &mut commands,
        &mut vis,
        &mut assets,
        groups,
        crate::fx_draw::LIST1_BIAS + 53.,
        "Wrench swing trail",
    );
}
