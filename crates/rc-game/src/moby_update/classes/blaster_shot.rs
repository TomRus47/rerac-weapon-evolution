//! **The Blaster's shot** (class 305; level01 spawn `0x2e1cc8` from the Blaster's update, update `0x2e2170`, end
//! `0x2e2a18`; read from the decompiler output). The Blaster itself is `crate::hero::blaster`.
//!
//! **Pvars** (the game's layout): +0x00 the target's aim point (0: none), +0x10 / +0x14 the yaw / pitch turn rates,
//! +0x18 / +0x1c the yaw / pitch it was fired along, +0x20 the owner (Ratchet; the port keeps moby + 1), +0x24 its life
//! (s16) and +0x26 the life it started with (`ticks(35) + 10·gold`), +0x28 the gold ricochets, +0x2c.. the 17 trail
//! records (type 72; the port keeps record + 1).
//!
//! **Spawn** ([`spawn`], `0x2e1cc8(yaw, pitch, moby yaw, moby pitch, owner, muzzle, target)`): `CreateMoby(0x131)`;
//! update and draw distances 0x7f, visible, the owner's light word; rotation (0, moby pitch − π/2, moby yaw); at the
//! muzzle; the target's aim point (record height, 0.5 without one). A line from the owner's x / y at the muzzle's
//! height to the muzzle (flags 0, owner ignored) that hits: the shot starts at the hit, five impact sparks (type 27,
//! size 30000, 0x7f2f4f6f: 40·dt along its aim reflected off the face plus a random half, `randf(3, 6)·dt` long,
//! `rand_range(ticks(10), ticks(15))`) and [`end`] without sparks; the shot is deleted. Otherwise its glow (type 26,
//! size 200000, 0x2f4f7f7f, its life, on the moby) and the 17 trail sprites (type 72; size `randf(30000, 50000)` eased
//! to 16000 and alpha 128 → 64 along the trail by `0x26cc00(−1.5, 0, 1, 0, i/16)`).
//!
//! **Update** ([`update`]): the pitch without the model's −π/2; the aim: toward the target's aim point when there is
//! one (a spring turn `0x26cef0` with 2π·dt² / π·dt², at most 270°/s), else, after its first tick and with no
//! ricochet, toward the pvar aim (360°/s); moves 40 u/s along it; outside the positive octant or more than 64 from the
//! camera (2D) it ends. While its life runs, the path is tested (`CollLine_Fix(old, new, 0, owner, tmpl)`: push along
//! Ratchet → shot, 1, 5627.97, flags 0x10001, damage 0.25, type 1 / 1, class 305): a face other than water ends it at
//! the hit with five sparks (as the spawn's); water cuts its life to 3 ticks. Then the trail: the glow at the shot, the
//! 16 sprites behind it one step apart (0.15, less during its first `ticks(13)`: `0.15·age/ticks(13)`; each step 0.85
//! of the last) back along its path, jittered `randf_sym(0, 0.075)`; one tick in five a side spark (type 27: across
//! the path and the camera's up, turned `randf_sym(0, π)` about the path, `randf(0.5, 2)·dt`). Its life out → [`end`].
//!
//! **End** ([`end`], `0x2e2a18(shot, sparks)`): the trail records killed; with `sparks` five sparks (size 35000,
//! random directions, `randf(3, 6)·dt`); three smoke puffs (type 23: jitter 0.2, growth 1..1.02, size 140000, spin
//! ±1, rising `randf(0.01, 0.025)`, grey 0x606060 at alpha `rand_range(0x20, 0x80)` fading over
//! `ticks(rand_range(50, 80))`, half of them ALPHA 0x44). The callers then delete it.
//!
//! **The gold Blaster** (0x13e52f, item 15): a face hit (not surfaces 0, 1, 3, 4, 7, 0xb, 0xd) ricochets the shot off it
//! (reflected, 0.05 out, the turn rates 0, the Blaster's class sound 4) up to gold + 5 times; then [`end`] with sparks.
//! [L] The sound plays at the shot (the game plays it on the hand item). Native `f32`; the game's draws in its order.

use crate::hero::fx::PartSpawn;
use crate::hero::guns::{add3, len3, scale3, sub3, with_len};
use crate::moby_runtime::MobyId;
use crate::moby_update::creature::turn::spring_turn;
use crate::moby_update::creature::{add_rot, atan};
use crate::moby_update::services::{self as sv, pvar as p, HitTemplate, World};
use crate::particles::{rec, type23, type27};
use crate::ps2v::Pf;
use crate::targeting::polar;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub const UPDATE_FN: u32 = 0x2e2170;
pub const CLASSES: [i16; 1] = [305];
/// The Blaster's hand item class (0xa8).
const BLASTER_CLASS: i16 = 0xa8;
pub const CLASS: i16 = 305;
/// gp−0x5338 (0x1618c8): the shot's x rotation; −π/2 (0x1618cc) is the model's pitch offset.
const ROT_X: f32 = 0.0;
const PITCH_OFFSET: f32 = -FRAC_PI_2;
/// 0x1618a8 / 0x1618ac: the trail sizes' range; 0x1618b0 the trail's end size; 0x1618b4 the jitter; 0x1618b8 the step;
/// 0x1618bc the step's decay; 0x1618c0 the glow's size.
const TRAIL_LO: f32 = 30000.0;
const TRAIL_HI: f32 = 50000.0;
const TRAIL_END: f32 = 16000.0;
const JITTER: f32 = 0.075;
const STEP: f32 = 0.15;
const STEP_DECAY: f32 = 0.85;
const GLOW_SIZE: f32 = 200000.0;
/// Trail records (+0x2c..+0x6c).
pub const TRAIL: usize = 17;
const DT: f32 = 1.0 / 60.0;

pub mod pv {
    pub const TARGET: usize = 0x00;
    pub const YAW_VEL: usize = 0x10;
    pub const PITCH_VEL: usize = 0x14;
    pub const YAW: usize = 0x18;
    pub const PITCH: usize = 0x1c;
    pub const OWNER: usize = 0x20;
    pub const LIFE: usize = 0x24;
    pub const LIFE0: usize = 0x26;
    pub const BOUNCE: usize = 0x28;
    pub const TRAIL: usize = 0x2c;
    // Port-only colour snapshot, after the 17 native trail pointers (end 0x70).
    pub const PROJECTILE_RGB: usize = 0x70;
    pub const TRAIL_RGB: usize = 0x74;
    pub const IMPACT_RGB: usize = 0x78;
    pub const COLOR_TAG: usize = 0x7c;
}

/// Capture visual choices once per shot. None keeps the exact native RGBA;
/// the presence bit permits black (#000000) as a real configured colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ShotColors {
    projectile: Option<u32>,
    trail: Option<u32>,
    impact: Option<u32>,
}
impl ShotColors {
    const TAG: u32 = 0x4556_4f42;
    pub(crate) fn configured() -> Self {
        use crate::weapon_evo::extra_color;
        Self { projectile: extra_color(15, "projectile_color"),
            trail: extra_color(15, "trail_color"), impact: extra_color(15, "impact_color") }
    }
    fn store(self, shot: &mut crate::moby_runtime::Moby) {
        if shot.pvars.len() < 0x80 { shot.pvars.resize(0x80, 0); }
        for (offset, color) in [(pv::PROJECTILE_RGB, self.projectile),
            (pv::TRAIL_RGB, self.trail), (pv::IMPACT_RGB, self.impact)] {
            p::set_u32(&mut shot.pvars, offset, color.map_or(0, |rgb| 0x8000_0000 | rgb));
        }
        p::set_u32(&mut shot.pvars, pv::COLOR_TAG, Self::TAG);
        if shot.mode & crate::moby_runtime::mode::GLOW != 0 {
            shot.glow = self.projectile(shot.glow);
        }
    }
    fn of_shot(shot: &crate::moby_runtime::Moby) -> Self {
        if shot.o_class != CLASS || shot.pvars.len() < 0x80
            || p::u32(&shot.pvars, pv::COLOR_TAG) != Self::TAG { return Self::default(); }
        let read = |offset| {
            let value = p::u32(&shot.pvars, offset);
            (value & 0x8000_0000 != 0).then_some(value & 0x00ff_ffff)
        };
        Self { projectile: read(pv::PROJECTILE_RGB), trail: read(pv::TRAIL_RGB), impact: read(pv::IMPACT_RGB) }
    }
    pub(crate) fn projectile(self, rgba: u32) -> u32 { crate::weapon_evo::tint_rgba(rgba, self.projectile) }
    fn trail(self, rgba: u32) -> u32 { crate::weapon_evo::tint_rgba(rgba, self.trail) }
    fn impact(self, rgba: u32) -> u32 { crate::weapon_evo::tint_rgba(rgba, self.impact) }
}

/// `0x26cc00(−1.5, 0, 1, 0, t)`: the trail's size / alpha curve (0 at the shot, 1 at its end).
pub fn trail_curve(t: f32) -> f32 {
    let (a, b, c, d) = (-1.5f32, 0.0f32, 1.0f32, 0.0f32);
    let e = (d - c) - (a - b);
    e * t * t * t + ((a - b) - e) * t * t + (c - a) * t + b
}

/// `0x2e1cc8` from the Blaster's update (`crate::hero::blaster`): the shot, its glow and its trail (queued with the
/// hero's particle spawns), or — muzzle behind a face — its sparks and smoke and no shot.
#[allow(clippy::too_many_arguments)]
pub fn spawn(hero: &mut crate::hero::Hero, table: &mut crate::moby_runtime::MobyTable, env: &crate::hero::items::ItemEnv, hits: &mut dyn crate::hero::items::HitSink, rng: &mut crate::rng::Rng, aim: (f32, f32), rot: (f32, f32), muzzle: [f32; 3], owner_pos: [f32; 3], target: Option<MobyId>, gold: i32) -> Option<MobyId> {
    spawn_impl(hero, table, env, hits, rng, aim, rot, muzzle, owner_pos, target, gold, None, None)
}

#[derive(Clone, Copy, Debug)]
pub struct RynoPellet {
    pub hp: f32, pub speed: f32, pub range: f32, pub size: f32,
    pub colors: [Option<u32>; 3],
}
const RYNO_TAG: u32 = 0x5745_5035;
const PELLET_LEN: usize = 0x90;

pub fn is_ryno_pellet(m: &crate::moby_runtime::Moby) -> bool {
    m.o_class == CLASS && m.pvars.len() >= PELLET_LEN && p::u32(&m.pvars, 0x80) == RYNO_TAG
}

pub(crate) fn spawn_ryno(id: MobyId, hero: &mut crate::hero::Hero, table: &mut crate::moby_runtime::MobyTable,
    env: &crate::hero::items::ItemEnv, hits: &mut dyn crate::hero::items::HitSink, rng: &mut crate::rng::Rng,
    aim: (f32, f32), muzzle: [f32; 3], pellet: RynoPellet) -> Option<MobyId> {
    let owner = crate::hero::physics::to_f32x3(hero.pos);
    spawn_impl(hero, table, env, hits, rng, aim, aim, muzzle, owner, None, 0, Some(pellet), Some(id))
}

#[allow(clippy::too_many_arguments)]
fn spawn_impl(hero: &mut crate::hero::Hero, table: &mut crate::moby_runtime::MobyTable, env: &crate::hero::items::ItemEnv,
    hits: &mut dyn crate::hero::items::HitSink, rng: &mut crate::rng::Rng, aim: (f32,f32), rot: (f32,f32),
    muzzle: [f32;3], owner_pos: [f32;3], target: Option<MobyId>, gold: i32, pellet: Option<RynoPellet>, reserved: Option<MobyId>) -> Option<MobyId> {
    let id = match reserved { Some(id) => id, None => hits.create_moby(table, CLASS, env.frame as u64)? };
    let (light, ambient) = table.mobys.get(env.hero_moby).map_or((0, [0x40; 4]), |h| (h.light, h.ambient));
    let tp = target.and_then(|t| table.mobys.get(t)).map(crate::targeting::aim_point);
    let life = pellet.map_or_else(|| crate::targeting::reach::blaster(gold).0, |p| (p.range / p.speed * 60.0).ceil() as i32 + 1);
    let colors = pellet.map_or_else(ShotColors::configured, |p| ShotColors { projectile: p.colors[0], trail: p.colors[1], impact: p.colors[2] });
    let size_k = pellet.map_or(1.0, |p| p.size);
    {
        let m = &mut table.mobys[id];
        if m.pvars.len() < 0x80 { m.pvars.resize(0x80, 0); }
        p::set_i32(&mut m.pvars, pv::OWNER, env.hero_moby as i32 + 1);
        m.draw_dist = 0x7f;
        m.update_dist = 0x7f;
        m.visible = 1;
        m.light = light;
        m.ambient = ambient;
        m.rotation = [ROT_X, add_rot(rot.1, PITCH_OFFSET), rot.0, 0.0];
        m.position = [muzzle[0], muzzle[1], muzzle[2], m.position[3]];
        p::set_v4f(&mut m.pvars, pv::TARGET, tp.map_or([0.0; 4], |t| [t[0], t[1], t[2], 0.0]));
        p::set_ff(&mut m.pvars, pv::YAW, aim.0);
        p::set_ff(&mut m.pvars, pv::PITCH, aim.1);
        p::set_ff(&mut m.pvars, pv::YAW_VEL, 0.0);
        p::set_ff(&mut m.pvars, pv::PITCH_VEL, 0.0);
        p::set_i16(&mut m.pvars, pv::LIFE0, life as i16);
        p::set_i16(&mut m.pvars, pv::LIFE, life as i16);
        p::set_i32(&mut m.pvars, pv::BOUNCE, 0);
        colors.store(m);
        if let Some(pellet) = pellet {
            m.pvars.resize(PELLET_LEN, 0);
            p::set_u32(&mut m.pvars, 0x80, RYNO_TAG);
            p::set_ff(&mut m.pvars, 0x84, pellet.speed);
            p::set_ff(&mut m.pvars, 0x88, pellet.range);
            p::set_ff(&mut m.pvars, 0x8c, pellet.hp);
            m.scale *= pellet.size;
            // This port owns pellet movement even if the native class table
            // has no update entry for Blaster shots on this planet.
            m.update_fn = Some(UPDATE_FN);
            m.mode &= !crate::moby_runtime::mode::NO_UPDATE;
            m.update_dist = 0xff;
            m.draw_dist = 0xff;
        }
    }
    let from = [owner_pos[0], owner_pos[1], muzzle[2]];
    let f4 = |a: [f32; 3]| [Pf::f(a[0]), Pf::f(a[1]), Pf::f(a[2]), Pf::ZERO];
    let blocked = hits.probe_moby(table, f4(from), f4(muzzle), 0, Some(env.hero_moby)).flatten().or_else(|| {
        // No moby system: the world mesh alone.
        env.coll.and_then(|c| crate::hero::physics::line_world(c, f4(from), f4(muzzle), 0)).map(|o| crate::hero::items::Probe { moby: None, point: o.point, normal: o.normal, surface: o.surface_id() })
    });
    let at = |t: [f32; 3]| [t[0], t[1], t[2], 0.0];
    match blocked {
        None => {
            let glow_rng = *rng;
            let _ = rng.rand();
            hero.fx.parts.push(PartSpawn::Glow26 { size: GLOW_SIZE * size_k, moby: id, rgba: colors.projectile(0x2f4f_7f7f), life, at: muzzle, rng: glow_rng, pellet: pellet.is_some() });
            for i in 0..TRAIL {
                let draws = crate::particles::type72::Draws::draw(rng);
                let mut size = rng.randf(TRAIL_LO, TRAIL_HI);
                let f = trail_curve(i as f32 / 16.0);
                size += (TRAIL_END - size) * f;
                let a = (f * -64.0 + 128.0) as i32 as u32;
                hero.fx.parts.push(PartSpawn::Trail72 { moby: id, slot: i as u8, pos: at(muzzle), draws, size: size * size_k, rgba: colors.trail(a << 24 | 0xff_ffff), pellet: pellet.is_some() });
            }
            Some(id)
        }
        Some(h) => {
            let dir = polar(DT * 40.0, rot.0, -add_rot(rot.1, PITCH_OFFSET));
            for _ in 0..5 {
                let x = rng.randf(-1.0, 1.0);
                let y = rng.randf(-1.0, 1.0);
                let z = rng.randf(-1.0, 1.0);
                let r = crate::hero::guns::reflect(dir, h.normal);
                let v = add3(r, with_len([x, y, z], len3(r) * 0.5));
                let s = rng.randf(DT * 3.0, DT * 6.0);
                let v = with_len(v, s);
                let life = rng.rand_range(10, 15);
                let rot = rng.rand() as u8;
                hero.fx.parts.push(PartSpawn::Spark27 { size: 30000.0, pos: at(h.point), vel: [v[0], v[1], v[2], 0.0], rgba: colors.impact(0x7f2f_4f6f), life, rot });
            }
            // 0x2e2a18(shot, 0): no trail yet; the smoke.
            for _ in 0..3 {
                let alpha = rng.rand_range(0x20, 0x80) as u8;
                let spin = if rng.randi(2) != 0 { -1 } else { 1 };
                let vz = rng.randf(0.01, 0.025);
                let snap = *rng;
                for _ in 0..3 { let _ = rng.randf_sym(0.0, 0.2); }
                let _ = rng.randi(2);
                let _ = rng.randf(1.0, 1.02);
                let life = rng.rand_range(50, 80);
                let normal = rng.randi(2) != 0;
                hero.fx.parts.push(PartSpawn::Puff23 { jitter: 0.2, lo: 1.0, hi: 1.02, size: 140000.0, pos: at(h.point), spin, vel: [0.0, 0.0, vz, 1.0], rgba: (alpha as u32) << 24 | 0x60_6060, life, alpha, normal, rng: snap });
            }
            hits.delete_moby(table, id, env.frame as u64);
            None
        }
    }
}

fn owner(w: &World, id: MobyId) -> Option<MobyId> { (p::i32(&w.m(id).pvars, pv::OWNER) as usize).checked_sub(1) }

/// The shot's trail records: the ones the particle hook linked since its last update become its pointers.
fn take_links(w: &mut World, id: MobyId) {
    let Some(sys) = w.particles.as_deref_mut() else { return };
    for k in 0..TRAIL {
        if let Some(i) = sys.links.remove(&(id, k as u8)) { p::set_i32(&mut w.table.mobys[id].pvars, pv::TRAIL + 4 * k, i as i32 + 1); }
    }
}

/// A trail pointer's record (`None`: none, or no longer the shot's type-72 record).
fn trail_rec(w: &World, id: MobyId, k: usize) -> Option<usize> {
    let i = (p::i32(&w.m(id).pvars, pv::TRAIL + 4 * k) as usize).checked_sub(1)?;
    let sys = w.particles.as_deref()?;
    let r = sys.pool.recs.get(i)?;
    (r[0] == crate::particles::type72::TYPE && r[1] & crate::particles::FLAG_DEAD == 0).then_some(i)
}

/// Five impact sparks at `pos`: `dir` reflected off `normal` plus a random half of its length (type 27, size 30000).
/// The same loop is inline in the Gemlik turret's shot 1238 (level13 `0x306300`, `units::gemlik_turret`).
pub(crate) fn impact_sparks(w: &mut World, pos: [f32; 3], dir: [f32; 3], normal: [f32; 3]) {
    // Turrets share this helper; their sparks must keep the original colours.
    impact_sparks_colored(w, pos, dir, normal, ShotColors::default());
}

fn impact_sparks_colored(w: &mut World, pos: [f32; 3], dir: [f32; 3], normal: [f32; 3], colors: ShotColors) {
    for _ in 0..5 {
        let x = w.rng.randf(-1.0, 1.0);
        let y = w.rng.randf(-1.0, 1.0);
        let z = w.rng.randf(-1.0, 1.0);
        let r = crate::hero::guns::reflect(dir, normal);
        let v = add3(r, with_len([x, y, z], len3(r) * 0.5));
        let s = w.rng.randf(DT * 3.0, DT * 6.0);
        let v = with_len(v, s);
        let life = w.rng.rand_range(w.ticks(10), w.ticks(15));
        part27(w, 30000.0, pos, v, colors.impact(0x7f2f_4f6f), life);
    }
}

fn part27(w: &mut World, size: f32, pos: [f32; 3], vel: [f32; 3], rgba: u32, life: i32) {
    let Some(sys) = w.particles.as_deref_mut() else { return };
    type27::spawn_rng(sys, w.rng, size, [pos[0], pos[1], pos[2], 0.0], [vel[0], vel[1], vel[2], 0.0], rgba, life);
}

/// `0x2e2a18(shot, sparks)` (module doc).
pub fn end(w: &mut World, id: MobyId, sparks: bool) {
    let colors = ShotColors::of_shot(w.m(id));
    for k in 0..TRAIL {
        if let Some(i) = trail_rec(w, id, k) { if let Some(sys) = w.particles.as_deref_mut() { sys.kill_part(i); } }
        p::set_i32(&mut w.table.mobys[id].pvars, pv::TRAIL + 4 * k, 0);
    }
    let pos = { let q = w.m(id).position; [q[0], q[1], q[2]] };
    if sparks {
        for _ in 0..5 {
            let x = w.rng.randf(-1.0, 1.0);
            let y = w.rng.randf(-1.0, 1.0);
            let z = w.rng.randf(-1.0, 1.0);
            let s = w.rng.randf(DT * 3.0, DT * 6.0);
            let v = with_len([x, y, z], s);
            let life = w.rng.rand_range(w.ticks(10), w.ticks(15));
            part27(w, 35000.0, pos, v, colors.impact(0x7f2f_4f6f), life);
        }
    }
    for _ in 0..3 {
        let alpha = w.rng.rand_range(0x20, 0x80) as u8;
        let spin = if w.rng.randi(2) != 0 { -1 } else { 1 };
        let vz = w.rng.randf(0.01, 0.025);
        let Some(sys) = w.particles.as_deref_mut() else { continue };
        let Some(i) = type23::spawn(sys, w.rng, 0.2, 1.0, 1.02, 140000.0, [pos[0], pos[1], pos[2], 0.0], spin, [0.0, 0.0, vz, 1.0], (alpha as u32) << 24 | 0x60_6060) else { continue };
        let life = w.rng.rand_range(50, 80);
        let normal = w.rng.randi(2) != 0;
        let r = &mut sys.pool.recs[i];
        rec::set_i16(r, 0xa, life as i16);
        rec::set_u32(r, 0x24, 2);
        r[0x2a] = alpha;
        r[0x2b] = life as u8;
        if normal { r[3] = 0x44; }
    }
}

fn finish(w: &mut World, id: MobyId) {
    end(w, id, false);
    w.delete_moby(id);
}

/// `0x2e2170`.
pub fn update(w: &mut World, id: MobyId) {
    if is_ryno_pellet(w.m(id)) { return update_ryno(w, id); }
    if w.m(id).pvars.len() < 0x80 { w.mm(id).pvars.resize(0x80, 0); }
    take_links(w, id);
    let (dt, dt2) = (DT, DT * DT);
    let pos0 = { let q = w.m(id).position; [q[0], q[1], q[2]] };
    // The pitch without the model's offset.
    let mut pitch = add_rot(w.m(id).rotation[1], -PITCH_OFFSET);
    let mut yaw = w.m(id).rotation[2];
    let pvars = w.m(id).pvars.clone();
    let tgt = p::v4f(&pvars, pv::TARGET);
    let tgt = [tgt[0], tgt[1], tgt[2]];
    // `c.lt.s` then its negation: a NaN length homes.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    let homing = !(len3(tgt) < 0.1);
    let (dy, dp) = if homing {
        let d = sub3(tgt, pos0);
        (atan(d[0], d[1]), -atan((d[0] * d[0] + d[1] * d[1]).sqrt(), d[2]))
    } else {
        let life = p::i16(&pvars, pv::LIFE) as i32;
        if life < w.ticks(35) - w.ticks(1) && p::i32(&pvars, pv::BOUNCE) == 0 { (p::ff(&pvars, pv::YAW), p::ff(&pvars, pv::PITCH)) } else { (yaw, pitch) }
    };
    let max = if homing { dt * 4.712_389 } else { dt * TAU };
    let (mut vy, mut vp) = (p::ff(&pvars, pv::YAW_VEL), p::ff(&pvars, pv::PITCH_VEL));
    yaw = spring_turn(yaw, dy, dt2 * TAU, dt2 * PI, max, &mut vy);
    pitch = spring_turn(pitch, dp, dt2 * TAU, dt2 * PI, max, &mut vp);
    let step = polar(crate::targeting::reach::blaster(0).1, yaw, -pitch);
    let pos = add3(pos0, step);
    {
        let m = w.mm(id);
        m.rotation[2] = yaw;
        m.rotation[1] = pitch;
        p::set_ff(&mut m.pvars, pv::YAW_VEL, vy);
        p::set_ff(&mut m.pvars, pv::PITCH_VEL, vp);
        m.position = [pos[0], pos[1], pos[2], m.position[3]];
    }
    if pos[0] < 0.0 || pos[1] < 0.0 || pos[2] < 0.0 {
        finish(w, id);
        return;
    }
    let cam = [w.camera[0].to_f32(), w.camera[1].to_f32()];
    if ((pos[0] - cam[0]).powi(2) + (pos[1] - cam[1]).powi(2)).sqrt() > 64.0 {
        finish(w, id);
        return;
    }
    let hero = crate::hero::physics::to_f32x3(w.hero.pos);
    let n = with_len(sub3(pos, hero), 1.0);
    let tmpl = HitTemplate { dir: [Pf::f(n[0]), Pf::f(n[1]), Pf::ONE, Pf::b(0x45af_df66)], attacker: Some(id), flags: 0x1_0001, b18: 1, b19: 1, h1a: CLASS as u16, damage: Pf::f(0.25 * crate::weapon_evo::extra(15, "damage_multiplier", 1.0)), w20: 1 };
    let mut life = p::i16(&w.m(id).pvars, pv::LIFE);
    let out = sv::fast_dec_timer_s16(&mut life) != 0;
    p::set_i16(&mut w.mm(id).pvars, pv::LIFE, life);
    if out {
        finish(w, id);
        return;
    }
    let f4 = |a: [f32; 3]| [Pf::f(a[0]), Pf::f(a[1]), Pf::f(a[2]), Pf::ZERO];
    let ignore = owner(w, id);
    if let Some(h) = sv::line_hit_in(w.table, w.svc, w.classes, w.coll, f4(pos0), f4(pos), 0, ignore, &tmpl) {
        if h.surface_id() != 0 {
            {
                let m = w.mm(id);
                m.position = [h.point[0], h.point[1], h.point[2], m.position[3]];
            }
            let colors = ShotColors::of_shot(w.m(id));
            impact_sparks_colored(w, h.point, step, h.normal, colors);
            // The gold Blaster (0x13e52f, item 15): the shot ricochets up to gold + 5 times, never off surfaces 0, 1,
            // 3, 4, 7, 0xb, 0xd; once it stops, it ends with sparks.
            let gold = w.hero.weapons.gold.get(15).copied().unwrap_or(0);
            if gold == 0 {
                finish(w, id);
                return;
            }
            let mut n = p::i32(&w.m(id).pvars, pv::BOUNCE) + 1;
            if matches!(h.surface_id(), 0 | 1 | 3 | 4 | 7 | 0xb | 0xd) { n = 1000; }
            p::set_i32(&mut w.mm(id).pvars, pv::BOUNCE, n);
            if gold as i32 + 5 < n {
                end(w, id, true);
                w.delete_moby(id);
                return;
            }
            let nn = with_len(h.normal, 1.0);
            let r = crate::hero::guns::reflect(step, nn);
            {
                let m = w.mm(id);
                m.rotation[2] = atan(r[0], r[1]);
                m.rotation[1] = -atan((r[0] * r[0] + r[1] * r[1]).sqrt(), r[2]);
                p::set_ff(&mut m.pvars, pv::YAW_VEL, 0.0);
                p::set_ff(&mut m.pvars, pv::PITCH_VEL, 0.0);
                m.position = [h.point[0] + nn[0] * 0.05, h.point[1] + nn[1] * 0.05, h.point[2] + nn[2] * 0.05, m.position[3]];
            }
            // The Blaster's class sound 4 (on the hand item 0xa8 in the game; at the shot here [L]).
            if w.hero.items.slot.item.as_ref().is_some_and(|m| m.o_class == BLASTER_CLASS) { w.play_sound_as(4, 0, id, BLASTER_CLASS); }
        } else {
            // Water: its life cut to 3 ticks.
            let t3 = w.ticks(3) as i16;
            let m = w.mm(id);
            if t3 < p::i16(&m.pvars, pv::LIFE) { p::set_i16(&mut m.pvars, pv::LIFE, t3); }
        }
    }
    // The model's pitch offset back.
    { let m = w.mm(id); m.rotation[1] = add_rot(m.rotation[1], PITCH_OFFSET); }
    trail(w, id, pos0);
}

/// The trail's placement and the side spark (the tail of `0x2e2170`).
fn trail(w: &mut World, id: MobyId, old: [f32; 3]) {
    let pos = { let q = w.m(id).position; [q[0], q[1], q[2]] };
    let (life0, life) = (p::i16(&w.m(id).pvars, pv::LIFE0) as i32, p::i16(&w.m(id).pvars, pv::LIFE) as i32);
    let age = life0 - life;
    let t13 = w.ticks(13);
    let l = if age < t13 { age as f32 / t13 as f32 * STEP } else { STEP };
    let mut dir = with_len(sub3(old, pos), l);
    let mut at = pos;
    if let Some(i) = trail_rec(w, id, 0) { if let Some(sys) = w.particles.as_deref_mut() { rec::set_v3(&mut sys.pool.recs[i], 0x10, at); } }
    for k in 1..TRAIL {
        let rec_i = trail_rec(w, id, k);
        if rec_i.is_some() {
            let p = add3(at, dir);
            at = p;
            let j = [w.rng.randf_sym(0.0, JITTER), w.rng.randf_sym(0.0, JITTER), w.rng.randf_sym(0.0, JITTER)];
            if let (Some(i), Some(sys)) = (rec_i, w.particles.as_deref_mut()) { rec::set_v3(&mut sys.pool.recs[i], 0x10, add3(p, j)); }
            dir = scale3(dir, STEP_DECAY);
        }
    }
    if w.rng.randi(5) != 0 { return; }
    // The camera's up row 0x167470 (the view's rows: world axis j's camera components; camera y is down). Without
    // the view (headless worlds) z up [L].
    let up = w.view.map_or([0.0, 0.0, 1.0], |v| [0, 1, 2].map(|j| -f32::from_bits(v.rows[j][1])));
    let mut v = crate::hero::guns::cross3(dir, up);
    let a = w.rng.randf_sym(0.0, PI);
    v = rotate(v, a, dir);
    let n = len3(v);
    if n == 0.0 { return; }
    let s = w.rng.randf(DT * 0.5, DT + DT);
    let v = scale3(v, s / n);
    let life = w.rng.rand_range(w.ticks(10), w.ticks(15));
    let colors = ShotColors::of_shot(w.m(id));
    part27(w, 30000.0, pos, v, colors.trail(0x7f2f_4f6f), life);
}

/// `FUN_00274ac8(θ, out, v, axis)`: `v` turned by θ about `axis` (right-handed; below 1e-5 unchanged).
pub fn rotate(v: [f32; 3], t: f32, axis: [f32; 3]) -> [f32; 3] {
    if t.abs() < 1e-5 { return v; }
    let a = with_len(axis, 1.0);
    let (s, c) = t.sin_cos();
    let k = crate::hero::guns::cross3(a, v);
    let d = crate::hero::guns::dot3(a, v) * (1.0 - c);
    [v[0] * c + k[0] * s + a[0] * d, v[1] * c + k[1] * s + a[1] * d, v[2] * c + k[2] * s + a[2] * d]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trail_curve_runs_from_0_to_1() {
        assert_eq!(trail_curve(0.0), 0.0);
        assert!((trail_curve(1.0) - 1.0).abs() < 1e-6);
        assert!((trail_curve(0.5) - (0.0625 - 0.5 + 1.25)).abs() < 1e-6);
    }

    #[test]
    fn color_snapshot_keeps_native_pvars_and_black_is_distinct_from_default() {
        let mut shot = crate::moby_runtime::Moby::zeroed();
        shot.o_class = CLASS;
        shot.pvars = vec![0x5a; 0x80];
        shot.position = [10.0, 20.0, 30.0, 1.0];
        shot.mode = crate::moby_runtime::mode::GLOW;
        shot.glow = 0x7f40_607f;
        let native = shot.clone();
        let colors = ShotColors { projectile: Some(0), trail: Some(0x00ff_ffff), impact: None };
        colors.store(&mut shot);
        assert_eq!(ShotColors::of_shot(&shot), colors);
        assert_eq!(&shot.pvars[..0x70], &native.pvars[..0x70], "all native fields and trail links stay intact");
        assert_eq!(shot.position, native.position);
        assert_eq!(shot.mode, native.mode);
        assert_eq!(shot.glow, native.glow & 0xff00_0000, "black is a real colour");
        let mut new_shot = native;
        ShotColors { projectile: Some(0xff), trail: None, impact: Some(0xff0000) }.store(&mut new_shot);
        assert_eq!(ShotColors::of_shot(&shot), colors, "a later shot cannot recolour one in flight");
        shot.o_class = 1238;
        assert_eq!(ShotColors::of_shot(&shot), ShotColors::default(), "enemy shots cannot borrow Blaster colours");
        shot.o_class = CLASS;
        p::set_u32(&mut shot.pvars, pv::COLOR_TAG, 0);
        assert_eq!(ShotColors::of_shot(&shot), ShotColors::default(), "unmarked native shots keep native colours");
    }

    #[test]
    fn default_colors_preserve_rgba_and_custom_colors_preserve_alpha() {
        let native = ShotColors::default();
        let colors = ShotColors { projectile: Some(0xff), trail: Some(0xff00), impact: Some(0xff0000) };
        for alpha in 0..=128u32 {
            let rgba = alpha << 24 | 0x4f_7f7f;
            assert_eq!(native.projectile(rgba), rgba);
            assert_eq!(native.trail(rgba), rgba);
            assert_eq!(native.impact(rgba), rgba);
            for tinted in [colors.projectile(rgba), colors.trail(rgba), colors.impact(rgba)] {
                assert_eq!(tinted >> 24, alpha);
            }
            assert_eq!(colors.projectile(rgba) & 0xffff00, 0);
            assert_eq!(colors.trail(rgba) & 0xff00ff, 0);
            assert_eq!(colors.impact(rgba) & 0x00ffff, 0);
        }
    }

    fn impact_records(colors: ShotColors, mode: u8) -> (Vec<crate::particles::Record>, u32) {
        use crate::moby_runtime::{Moby, MobyTable};
        use crate::moby_update::services::{ClassTable, Services};
        let mut shot = Moby::zeroed();
        shot.o_class = CLASS; shot.position = [10.0, 20.0, 30.0, 1.0];
        colors.store(&mut shot);
        let mut table = MobyTable::new(vec![shot], 0);
        let hero = crate::hero::Hero::new();
        let classes = ClassTable::default();
        let mut services = Services::new();
        let mut rng = crate::rng::Rng::new();
        let mut particles = crate::particles::Particles::new(None, Vec::new());
        {
            let mut world = World::new(&mut table, &hero, &mut rng, &classes, &mut services, 0);
            world.particles = Some(&mut particles);
            if mode == 2 { end(&mut world, 0, true); }
            else if mode == 1 {
                let snapshot = ShotColors::of_shot(world.m(0));
                impact_sparks_colored(&mut world, [10.0, 20.0, 30.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], snapshot);
            } else {
                impact_sparks(&mut world, [10.0, 20.0, 30.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
            }
        }
        let records = particles.pool.recs.iter()
            .filter(|r| r[0] != 0 && r[1] & crate::particles::FLAG_DEAD == 0).copied().collect();
        (records, rng.state)
    }

    #[test]
    fn impacts_and_end_recolor_only_sparks_preserving_smoke_rng_and_motion() {
        let custom = ShotColors { impact: Some(0xff0000), ..Default::default() };
        for mode in [1, 2] {
            let (native, native_rng) = impact_records(ShotColors::default(), mode);
            let (colored, colored_rng) = impact_records(custom, mode);
            assert_eq!(native_rng, colored_rng);
            assert_eq!(native.len(), colored.len());
            assert_eq!(native.iter().filter(|r| r[0] == type27::TYPE).count(), 5);
            for (a, b) in native.iter().zip(&colored) {
                if a[0] == type27::TYPE {
                    assert_eq!(&a[..4], &b[..4]);
                    assert_eq!(&a[7..], &b[7..], "alpha, motion, lifetime and size remain native");
                    assert_eq!(rec::u32(b, 4) & 0xffff, 0, "blue sparks have no red/green channels");
                    assert!(rec::u32(b, 4) & 0xff0000 != 0);
                } else { assert_eq!(a, b, "impact smoke remains native"); }
            }
            if mode == 2 { assert_eq!(native.iter().filter(|r| r[0] == 23).count(), 3); }
        }
        assert_eq!(impact_records(custom, 0), impact_records(ShotColors::default(), 0),
            "the shared turret effect must remain byte-identical");
    }
}

/// Straight independent pellets retain the native model, glow, trail and impact.
fn update_ryno(w: &mut World, id: MobyId) {
    take_links(w, id);
    let pvs = &w.m(id).pvars;
    let (speed, left, hp) = (p::ff(pvs, 0x84), p::ff(pvs, 0x88), p::ff(pvs, 0x8c));
    let old4 = w.m(id).position;
    let old = [old4[0], old4[1], old4[2]];
    let step = polar((speed * DT).min(left), p::ff(pvs, pv::YAW), -p::ff(pvs, pv::PITCH));
    let pos = add3(old, step);
    w.svc.hits.real_hp.set_source_damage_hp(id, 23, hp);
    let tmpl = HitTemplate { dir: [Pf::f(step[0]), Pf::f(step[1]), Pf::ONE, Pf::b(0x45af_df66)],
        attacker: Some(id), flags: 0x10001, b18: 1, b19: 1, h1a: CLASS as u16, damage: Pf::f(hp / 100.0), w20: 1 };
    let f4 = |a: [f32;3]| [Pf::f(a[0]),Pf::f(a[1]),Pf::f(a[2]),Pf::ONE];
    let owner = owner(w, id);
    let hit = sv::line_hit_in(w.table, w.svc, w.classes, w.coll, f4(old), f4(pos), 0, owner, &tmpl);
    let new = hit.as_ref().map_or(pos, |h| h.point);
    w.mm(id).position = [new[0], new[1], new[2], 1.0];
    p::set_ff(&mut w.mm(id).pvars, 0x88, (left - len3(step)).max(0.0));
    let life = p::i16(&w.m(id).pvars, pv::LIFE).saturating_sub(1);
    p::set_i16(&mut w.mm(id).pvars, pv::LIFE, life);
    if life <= 0 || hit.is_some() || left <= len3(step) || !crate::moby_update::creature::projectile::in_world(w.m(id).position) {
        end(w, id, hit.is_some());
        w.delete_moby(id);
    } else { trail(w, id, old); }
}


#[cfg(test)]
mod ryno_lifetime_tests {
    use super::*;
    use crate::moby_runtime::{Moby, MobyTable};
    use crate::moby_update::services::{ClassTable, Services};

    #[test]
    fn stationary_range_residue_expires_and_releases_dynamic_slot() {
        let mut table = MobyTable::new(Vec::new(), 1);
        let id = table.create(CLASS, None, 0).unwrap();
        let shot = &mut table.mobys[id];
        shot.position = [410., 410., 100., 1.]; shot.pvars.resize(PELLET_LEN, 0);
        p::set_u32(&mut shot.pvars, 0x80, RYNO_TAG);
        p::set_ff(&mut shot.pvars, 0x84, 40.);
        // Length calculation underflows although the residual is positive.
        p::set_ff(&mut shot.pvars, 0x88, f32::MIN_POSITIVE);
        p::set_i16(&mut shot.pvars, pv::LIFE0, 2);
        p::set_i16(&mut shot.pvars, pv::LIFE, 2);
        let hero = crate::hero::Hero::new();
        let classes = ClassTable::default(); let mut services = Services::new(); let mut rng = crate::rng::Rng::new();
        update_ryno(&mut World::new(&mut table, &hero, &mut rng, &classes, &mut services, 10), id);
        assert!(!table.mobys[id].is_deleted());
        assert_eq!(table.mobys[id].position[..3], [410., 410., 100.]);
        update_ryno(&mut World::new(&mut table, &hero, &mut rng, &classes, &mut services, 11), id);
        assert!(table.mobys[id].is_deleted());
        assert_eq!(table.available_dynamic_slots(12), 0);
        assert_eq!(table.available_dynamic_slots(13), 1);
    }

    #[test]
    fn ordinary_range_endpoint_deletes_without_waiting_for_lifetime() {
        let mut shot = Moby::zeroed(); shot.o_class = CLASS; shot.position = [410., 410., 100., 1.];
        shot.pvars.resize(PELLET_LEN, 0); p::set_u32(&mut shot.pvars, 0x80, RYNO_TAG);
        p::set_ff(&mut shot.pvars, 0x84, 60.); p::set_ff(&mut shot.pvars, 0x88, 0.5);
        p::set_i16(&mut shot.pvars, pv::LIFE, 90);
        let mut table = MobyTable::new(vec![shot], 0); let hero = crate::hero::Hero::new();
        let classes = ClassTable::default(); let mut services = Services::new(); let mut rng = crate::rng::Rng::new();
        update_ryno(&mut World::new(&mut table, &hero, &mut rng, &classes, &mut services, 0), 0);
        assert!(table.mobys[0].is_deleted());
    }
}
