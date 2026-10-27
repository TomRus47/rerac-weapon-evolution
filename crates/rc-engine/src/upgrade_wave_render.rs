//! Filled, translucent bursts for earned weapon and Nanotech upgrades. They use
//! renderer-owned meshes only: no explosion actors, hitboxes or point lights.
//! Slam bursts use the damage query's captured centre and maximum radius.
use crate::fx_draw::{FxAssets, FxGroup, FxSlots, PrimBuf};
use bevy::prelude::*;
use rc_game::weapon_evo::upgrade_wave::State;

#[derive(Resource, Default)]
struct Waves {
    state: State,
    slots: FxSlots,
}
pub struct UpgradeWavePlugin;
impl Plugin for UpgradeWavePlugin {
    fn build(&self, app: &mut App) {
        if !crate::gameplay::enabled() {
            return;
        }
        app.add_systems(
            crate::level_switch::LevelUnload,
            crate::level_switch::remove::<Waves>,
        )
        .add_systems(Update, draw);
    }
}
#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    play: Option<Res<crate::gameplay::Play>>,
    mut waves: Option<ResMut<Waves>>,
    level: Res<crate::Level>,
    fog: Option<Res<crate::game_camera::GameFog>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<crate::fx_draw::FxPrimMaterial>>,
    mut vis: Query<&mut Visibility>,
) {
    let Some(play) = play else {
        return;
    };
    let Some(ref mut gfx) = waves else {
        commands.insert_resource(Waves::default());
        return;
    };
    let now = play.game.counter;
    let mut levels =
        std::array::from_fn(|i| play.svc.bomb_evo.weapon_level(i as i32).unwrap_or(0) as u8);
    if play.svc.bomb_evo.world_settings.enemy_damage_by_health {
        levels[0]=(play.svc.bomb_evo.nanotech.bonus+1).min(255) as u8;
    }
    let position = play.game.hero.pos.map(|n| n.to_f32());
    gfx.state.update(
        now,
        !play.death_pending && play.game.hero.health > 0,
        levels,
        [position[0], position[1], position[2]],
    );
    let mut prims = PrimBuf::default();
    let upgrades=gfx.state.waves.iter().filter_map(|&wave|
        wave.appearance(now).map(|(radius,rgba)|(wave.origin,radius,rgba)));
    let slam=play.game.hero.melee.slam_visual.filter(|_| !play.death_pending && play.game.hero.health>0)
        .and_then(|visual|visual.appearance(now).map(|(radius,rgba)|(visual.centre,radius,rgba)));
    for (origin,radius,rgba) in upgrades.chain(slam) {
        // Filled, evenly colored spherical surface. A neutral texel preserves
        // exact grade RGB; regular alpha blending keeps the world visible.
        let point = |latitude: f32, longitude: f32| {
            let c = latitude.cos();
            [
                origin[0] + radius * c * longitude.cos(),
                origin[1] + radius * c * longitude.sin(),
                origin[2] + radius * latitude.sin(),
            ]
        };
        for row in 0..12 {
            for col in 0..24 {
                let lat = [row as f32, (row + 1) as f32]
                    .map(|n| -std::f32::consts::FRAC_PI_2 + n * std::f32::consts::PI / 12.);
                let lon = [col as f32, (col + 1) as f32].map(|n| n * std::f32::consts::TAU / 24.);
                prims.quad(
                    [
                        point(lat[0], lon[0]),
                        point(lat[0], lon[1]),
                        point(lat[1], lon[0]),
                        point(lat[1], lon[1]),
                    ],
                    [[0.5, 0.5]; 4],
                    [rgba; 4],
                );
            }
        }
    }
    let groups = if prims.is_empty() {
        Vec::new()
    } else {
        vec![FxGroup {
            fx: crate::fx_draw::SOLID_FX,
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
    gfx.slots.show(
        &mut commands,
        &mut vis,
        &mut assets,
        groups,
        crate::fx_draw::LIST1_BIAS + 52.,
        "Upgrade and wrench blast visuals",
    );
}
