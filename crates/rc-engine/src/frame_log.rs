//! `RC_FRAME_LOG=1`: a line for every slow frame (over 25 ms), to find where a stall comes from: the frame time,
//! the gameplay ticks it ran, Ratchet's position, and how many entities, meshes, moby materials and images exist and
//! how many appeared since the last frame. A stall with many new entities or materials points at first-show spawns
//! and pipeline compiles; one with none points elsewhere (the tick, the renderer). Off by default.
//! Reports are capped at one per second. CPU data sizes are retained source
//! bytes, not GPU memory or an attribution of the stall to a particular stage.

use bevy::prelude::*;

/// Frames slower than this are printed.
const SLOW_MS: f32 = 25.0;

pub struct FrameLogPlugin;

#[derive(Resource, Default)]
struct FrameClock(Option<std::time::Instant>);

fn begin(mut clock: ResMut<FrameClock>) { clock.0 = Some(std::time::Instant::now()); }

impl Plugin for FrameLogPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var("RC_FRAME_LOG").is_ok_and(|v| v.trim() == "1") {
            println!("frame log: frames over {SLOW_MS} ms are printed (RC_FRAME_LOG=1)");
            app.init_resource::<FrameClock>()
                .add_systems(First, begin)
                .add_systems(Last, log);
        }
    }
}

#[derive(Default)]
struct LastFrame {
    counter: Option<u64>,
    counts: [usize; 6],
    report: Option<std::time::Instant>,
}

#[allow(clippy::too_many_arguments)]
fn log(
    mut clock: Local<Option<std::time::Instant>>,
    play: Option<Res<crate::gameplay::Play>>,
    entities: Query<Entity>,
    meshes: Res<Assets<Mesh>>,
    mats: Res<Assets<crate::moby_render::MobyMaterial>>,
    images: Res<Assets<Image>>,
    fx: Res<Assets<crate::fx_draw::FxPrimMaterial>>,
    buffers: Res<Assets<bevy::render::storage::ShaderBuffer>>,
    parts: Option<Res<crate::particle_render::ParticleSim>>,
    mode: Option<Res<crate::menu_render::MenuMode>>,
    frame: Res<FrameClock>,
    mut last: Local<LastFrame>,
) {
    // The wall clock: Bevy's clocks advance by whole game ticks (crate::frame_pace).
    let now = std::time::Instant::now();
    let ms = clock.replace(now).map_or(0.0, |t| (now - t).as_secs_f32() * 1000.0);
    let counter = play.as_ref().map(|p| p.game.counter);
    let counts = [entities.iter().count(), meshes.len(), mats.len(), images.len(), fx.len(), buffers.len()];
    // Keep a sustained stall from generating disk/console work every frame.
    if ms > SLOW_MS && last.report.is_none_or(|t| now.duration_since(t).as_secs_f32() >= 1.) {
        last.report = Some(now);
        let ticks = counter.zip(last.counter).map_or(0, |(a, b)| a.saturating_sub(b));
        let at = play.as_ref().map(|p| p.game.hero.position()).unwrap_or_default();
        let d = |k: usize| counts[k] as i64 - last.counts[k] as i64;
        println!(
            "frame log: {ms:.1} ms, {ticks} ticks (tick {}), Ratchet at {:.1}, {:.1}, {:.1}; entities {} ({:+}), meshes {} ({:+}), moby materials {} ({:+}), images {} ({:+}), FX materials {} ({:+}), buffers {} ({:+}); level {:?}, mode {:?}, active mobys {}, particles {}; image CPU data {:.1} MiB, buffer CPU data {:.1} MiB, app elapsed {:.1} ms",
            counter.unwrap_or(0), at[0], at[1], at[2], counts[0], d(0), counts[1], d(1), counts[2], d(2), counts[3], d(3), counts[4], d(4), counts[5], d(5),
            crate::level_load::level_index(), mode.as_ref().map(|m| m.state.mode),
            play.as_ref().map_or(0, |p| p.game.mobys.mobys.iter().filter(|m| m.state < 0x80 && m.o_class != 0).count()),
            parts.as_ref().map_or(0, |p| p.sys.pool.count),
            images.iter().map(|(_, i)| i.data.as_ref().map_or(0, Vec::len)).sum::<usize>() as f64 / 1048576.,
            buffers.iter().map(|(_, b)| b.data.as_ref().map_or(0, Vec::len)).sum::<usize>() as f64 / 1048576.,
            frame.0.map_or(0., |t| now.duration_since(t).as_secs_f64() * 1000.)
        );
    }
    last.counter = counter;
    last.counts = counts;
}
