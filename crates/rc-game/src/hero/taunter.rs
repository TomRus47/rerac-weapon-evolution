//! **The Taunter** (item 14, class 175; level01 item update `0x2ccb78`, the lure `0x2cc830`; read from the decompiler
//! output). No weapon-check case, no arm, no ammo: ○ plays one of its whistles and, while it sounds, lures the
//! creatures in front of Ratchet. The lure is the one weapon input the creatures share besides hits: the damage record's
//! +0x18 ([`crate::moby_update::creature::react::lure`]), which 577 (240-tick alert), the amoeboids (alert) and the
//! trooper 459 (600-tick alert) read and clear every tick.
//!
//! **The update** (pvars [`Taunter`]; item +0x20): +0x08 the lure range starts at 30. Key B on sequence 2 (put away) →
//! state 3. State 0 → 1 (no sound; the rings reset); 1 → 2 when a sequence wraps. **State 2**: while the item is on its
//! whistle sequence 3 (not 0x1413fc) the rings run and +0x18 counts the ticks. With **no whistle sounding** (the last
//! one's slot +0x04 is not alive): ○ pressed, or ○ held for more than 16 ticks, not 0x1413fc → +0x10 =
//! `ticks(rand_range(15, 45))` (pressed only), sequence 3 (3 ticks), the next whistle (class sound +0x00, flags 1) —
//! the whistles cycle while the next sound def's near / far distances are both 3.0 ([`Taunter::cycle`], from the class's
//! sound defs), else from 0 — then **the lure** ([`lure`]); nothing pressed → ○ held counts +0x0c (0 without it), and
//! the item goes back to sequence 1 (5 ticks). **While a whistle sounds**: +0x0c counts; ○ let go after 30 ticks of
//! whistle → stop (sequence 1, the sound released, +0x04 −1); else (not 0x1413fc) the lure again every 4th frame.
//!
//! **The lure** `0x2cc830(range, taunter)` over the target list 0x1abe80 (mobys with a damage record): within `range`
//! and within 55° of Ratchet's facing (seen from Ratchet), the record's +0x18 = the Taunter unless a nearer lure holds
//! it; the crates 500 / 501 / 505 / 511 within 12 in the same cone count, and the `randi(max(+0x14, 1))`-th of them
//! gets a hit (`0x26eaa8`: damage 1, flags 0x10000, from Ratchet's feet, 1.5 along taunter → crate). The Mine Glove's
//! mines (list 0x1b0c30: pvar +0x78 = the Taunter) are the second loop (within the reach in xy and 55° of Ratchet's
//! facing; `crate::moby_update::classes::mine` triples its seek reach while lured).
//!
//! **Native / inferred.** Standard `f32`. [L] The hand item is not a table moby: the lure's position and the lure moby
//! are Ratchet's (the whistle sits in his hand; the classes only test the field for non-zero); the whistle's
//! aliveness is the flush's of the previous tick ([`super::fx::HeroFx::item_loop_alive`]); the whistle has its own
//! channel ([`super::fx::LOOP_ITEM`]: on channel 0 the Pyrocitor's `item_gone` released it a tick after it started, so
//! each whistle and its lures lasted one tick). **The rings** ([`Rings`], `0x2cd000` + draw callback `0x2cd1d0`):
//! while the whistle sequence plays, a ring every 10 ticks leaves the horn (the item's joint list 0) along −row 1 at
//! 20 u/s for 30 ticks, growing from 0.3 to 10 and fading from alpha 0x14 (FX 8, colour 0x7f5050, additive: faint by
//! the game's constants). Not ported: the stats 0x1416f0...

use super::items::{HitSink, ItemEnv};
use super::packs::SoundCmd;
use super::physics::{ticks, to_f32x3};
use super::Hero;
use crate::moby_runtime::{MobyId, MobyTable};
use crate::moby_update::creature::{self as c, attack, diff_rots, react};
use crate::moby_update::services::World;
use crate::rng::Rng;
use crate::ps2v::Pf;
use rc_formats::moby_anim;

/// The Taunter (item 14) and its class.
pub const TAUNTER: i32 = 14;
pub const CLASS: i16 = 175;
/// The lure's start range (+0x08: 30) and the crates' reach (12), the cone (55°).
pub const RANGE: f32 = 30.0;
pub const CRATE_REACH: f32 = 12.0;
pub const CONE: f32 = 0.959_931_1;
/// The crates the whistle knocks (`uVar1 − 500 < 2`, 0x1f9, 0x1ff).
pub const CRATES: [i16; 4] = [500, 501, 505, 511];
/// The item loop channel of the whistle ([`super::fx::HeroFx::item_loops`]).
pub const CHANNEL: usize = super::fx::LOOP_ITEM;

/// The Taunter's pvars (item moby +0x78).
#[derive(Clone, Debug, PartialEq)]
pub struct Taunter {
    /// +0x00: the next whistle (class sound index); +0x08: the lure range; +0x0c: ○ held / whistle ticks; +0x10: a
    /// timer the pressed whistle sets (`rand_range(15, 45)` ticks, read by nothing else in the update); +0x14: the
    /// crates counted by the last lure; +0x18: the whistle-sequence ticks.
    pub next: i32,
    pub range: f32,
    pub held: i32,
    pub t10: i32,
    pub crates: i32,
    pub on_seq: i32,
    /// +0x04 ≥ 0: a whistle's slot is kept (set by a whistle, −1 again at the stop).
    pub slot: bool,
    /// Whether the class's sound def `i` is a whistle that the next press may follow (near and far both 3.0; the
    /// engine fills it from the level's sound bank; empty: every whistle is def 0).
    pub cycle: Vec<bool>,
    /// Lures made (the port's count) and the mobys lured by the last one.
    pub lures: u32,
    pub lured: Vec<MobyId>,
    /// The sound-wave rings (`0x2cd000`, [`Rings`]).
    pub rings: Rings,
}

/// One ring (0x1db8c0 + 0xc·i): +0x00 its distance out, +0x04 (s16) its ticks left, +0x06 (s16) its alpha, +0x08 on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ring {
    pub dist: f32,
    pub left: i16,
    pub alpha: i16,
    pub on: bool,
    /// Visual settings captured at birth, so a level-up cannot distort live rings.
    pub speed: f32,
    pub inv_life: f32,
    pub size_multiplier: f32,
    pub rgb: u32,
}

/// The rings' globals (`0x2cd000` / draw callback `0x2cd1d0`; gp−0x5720.. the constants below).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rings {
    pub rings: [Ring; 10],
    /// 0x1614fc: clear them at the next step (the update's state 0); 0x161500 the spawn timer; 0x161504 paused (set
    /// every update, cleared while the whistle sequence plays); 0x161508 a ring's life (`trunc(10 / (20·dt))`) and
    /// 0x16150c its inverse.
    pub reset: bool,
    pub timer: i32,
    pub paused: bool,
    pub life: i32,
    pub inv_life: f32,
    /// 0x161510: the horn (the item's joint list 0) and the axis they leave along (−the item's row 1, `+0xd0`).
    pub mouth: [f32; 3],
    pub dir: [f32; 3],
    /// Ratchet's up (−0x13f5e0) for the rings' frame.
    pub up: [f32; 3],
    /// The tick the draw callback was registered for.
    pub drawn: Option<u64>,
}

/// gp−0x5720 spawn interval, −0x571c reach, −0x5718 speed (u/s), −0x5714 alpha, −0x5710 colour, −0x570c / −0x5708
/// the size at birth / at the end.
pub const RING_EVERY: i32 = 10;
pub const RING_REACH: f32 = 10.0;
pub const RING_SPEED: f32 = 20.0;
pub const RING_ALPHA: i16 = 0x14;
pub const RING_RGB: u32 = 0x7f_5050;
pub const RING_SIZE: [f32; 2] = [0.3, 10.0];
/// One ring quad: corners, ST, colours.
pub type RingQuad = ([[f32; 3]; 4], [[f32; 2]; 4], [u32; 4]);

/// The rings' texture (`GetEffectTex(8)`).
pub const RING_FX: usize = 8;

#[derive(Clone, Copy)]
struct WaveVisual {
    speed: f32,
    size: f32,
    reach: f32,
    interval: i32,
    alpha: i16,
    rgb: u32,
}
impl WaveVisual {
    fn configured() -> Self {
        use crate::weapon_evo::{extra, extra_color};
        Self {
            speed: RING_SPEED * extra(TAUNTER, "wave_speed_multiplier", 1.0).clamp(0.1, 8.0),
            size: extra(TAUNTER, "wave_size_multiplier", 1.0).clamp(0.1, 8.0),
            reach: RING_REACH * extra(TAUNTER, "wave_range_multiplier", 1.0).clamp(0.1, 8.0),
            interval: extra(TAUNTER, "wave_interval_ticks", RING_EVERY as f32).round().clamp(1.0, 600.0) as i32,
            alpha: extra(TAUNTER, "wave_alpha", RING_ALPHA as f32).round().clamp(0.0, 128.0) as i16,
            rgb: extra_color(TAUNTER, "wave_color").unwrap_or(RING_RGB),
        }
    }
}

impl Rings {
    /// `0x2cd000`'s state part: the reset, a new ring every `ticks(10)` while not paused, each ring out at 20 u/s until
    /// its life runs out.
    fn step(&mut self) { self.step_visual(WaveVisual::configured()); }

    fn step_visual(&mut self, look: WaveVisual) {
        if self.reset {
            for r in &mut self.rings { r.on = false; }
            self.reset = false;
            self.timer = 0;
        }
        self.life = ((look.reach / (look.speed * (1.0 / 60.0))) as i32).clamp(1, 30_000);
        self.inv_life = 1.0 / self.life as f32;
        if !self.paused && super::guns::dec(&mut self.timer) {
            if let Some(r) = self.rings.iter_mut().find(|r| !r.on) {
                self.timer = ticks(look.interval);
                *r = Ring { dist: 0.0, left: self.life as i16, alpha: look.alpha, on: true,
                    speed: look.speed, inv_life: self.inv_life, size_multiplier: look.size, rgb: look.rgb };
            }
        }
        for r in self.rings.iter_mut().filter(|r| r.on) {
            if super::guns::dec16(&mut r.left) { r.on = false; } else { r.dist += r.speed * (1.0 / 60.0); }
        }
    }

    /// `0x2cd1d0`: one camera-independent quad per ring (FX 8, additive): centred `dist` along the axis, in the frame
    /// (axis, side = unit(axis × up), side × axis), half-size from 0.3 at birth to 10 at the end, alpha 20 → 0.
    pub fn quads(&self) -> Vec<RingQuad> {
        use super::guns::{add3, cross3, scale3, with_len};
        let mut out = Vec::new();
        let side = with_len(cross3(self.dir, self.up), 1.0);
        let up = cross3(side, self.dir);
        for r in self.rings.iter().filter(|r| r.on) {
            let c = add3(self.mouth, scale3(self.dir, r.dist));
            let k = (r.left as f32 * r.inv_life).clamp(0.0, 1.0);
            let size = (RING_SIZE[1] + (RING_SIZE[0] - RING_SIZE[1]) * k) * r.size_multiplier;
            let a = (r.alpha as f32 * k) as u32;
            let rgba = a << 24 | r.rgb;
            let corner = |y: f32, z: f32| add3(c, add3(scale3(side, y * size), scale3(up, z * size)));
            out.push(([corner(-1.0, 1.0), corner(-1.0, -1.0), corner(1.0, 1.0), corner(1.0, -1.0)], [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]], [rgba; 4]));
        }
        out
    }
}

impl Default for Taunter {
    fn default() -> Self { Taunter { next: 0, range: 0.0, held: 0, t10: 0, crates: 0, on_seq: 0, slot: false, cycle: Vec::new(), lures: 0, lured: Vec::new(), rings: Rings::default() } }
}

/// [`Taunter::cycle`] from the class's sound defs (`class +0x28`, 0x20 each): def `i` may follow its predecessor when
/// `trunc(near·10) == 30` and `trunc(far·10) == 30`.
pub fn cycle_of(defs: &[rc_formats::sound_bank::SoundDef]) -> Vec<bool> {
    defs.iter().map(|d| (d.near * 10.0) as i32 == 30 && (d.far * 10.0) as i32 == 30).collect()
}

/// `0x2ccb78`, the Taunter's update (from the slot loop with the slot ready): the whistle, then (not put away) the rings.
pub fn update(hero: &mut Hero, table: &mut MobyTable, anim: &dyn super::anim::AnimCtl, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng) {
    hero.weapons.reactive.taunter.rings.paused = true;
    whistle(hero, table, anim, env, hits, rng);
    if hero.items.slot.state != 3 {
        let mouth = super::guns::item_point(hero, env, 0);
        let t = &mut hero.weapons.reactive.taunter.rings;
        t.step();
        t.mouth = mouth;
        if let Some(it) = hero.items.slot.item.as_ref() {
            let r1 = [f32::from_bits(it.rows[1][0]), f32::from_bits(it.rows[1][1]), f32::from_bits(it.rows[1][2])];
            t.dir = super::guns::with_len(r1, -1.0);
        }
        let g = super::physics::to_f32x3(hero.gravity_dir);
        t.up = [-g[0], -g[1], -g[2]];
        t.drawn = Some(env.frame as u64);
    }
}

fn whistle(hero: &mut Hero, table: &mut MobyTable, _anim: &dyn super::anim::AnimCtl, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng) {
    let Some(class) = hero.items.slot.item.as_ref().and_then(|m| env.data.class(m.o_class)).cloned() else { return };
    let tick = env.frame as u64;
    if hero.weapons.reactive.taunter.range == 0.0 { hero.weapons.reactive.taunter.range = RANGE; }
    let blend = |hero: &mut Hero, seq: u8, n: i32| {
        if let Some(it) = hero.items.slot.item.as_mut() {
            if it.anim.seq_b != seq { moby_anim::set_sequence(&mut it.anim, &class.anim, seq, 0, ticks(n), &mut it.snapshot); }
        }
    };
    let Some(it) = hero.items.slot.item.as_mut() else { return };
    if it.anim.seq_b == 2 { it.mstate = 3; }
    if it.mstate == 0 {
        hero.fx.item_voices.push(SoundCmd::ItemRelease { n: CHANNEL });
        it.mstate = 1;
        hero.weapons.reactive.taunter.slot = false;
        hero.weapons.reactive.taunter.rings.reset = true;
    }
    match it.mstate {
        1 => {
            if it.anim.flags & 2 != 0 {
                it.mstate = 2;
                hero.weapons.reactive.taunter.held = 0;
            }
            return;
        }
        2 => {}
        _ => return,
    }
    let fc = hero.items.f13fc != 0;
    let on_whistle = it.anim.seq_b == 3 && !fc;
    let t = &mut hero.weapons.reactive.taunter;
    if on_whistle { t.on_seq += 1; } else { t.on_seq = 0; }
    if on_whistle { t.rings.paused = false; }
    let mask = hero.items.slot.fire_mask;
    let pressed = env.pad.pressed & mask != 0;
    let held = env.pad.held & mask != 0;
    let sounding = hero.fx.item_loop_alive[CHANNEL];
    if !sounding {
        if (pressed || (held && 0x10 < t.held)) && !fc {
            if pressed { t.t10 = ticks(rng.rand_range(0xf, 0x2d)); }
            blend(hero, 3, 3);
            let t = &mut hero.weapons.reactive.taunter;
            let index = t.next;
            hero.fx.item_voices.push(SoundCmd::ItemLoop { n: CHANNEL, index, flags: 1 });
            t.slot = true;
            let n = t.next + 1;
            t.next = if t.cycle.get(n as usize).copied().unwrap_or(false) { n } else { 0 };
            do_lure(hero, table, env, hits, rng, tick);
            hero.weapons.reactive.taunter.held = 0;
            return;
        }
        if hero.weapons.reactive.taunter.slot {
            // The last whistle ended: the stop.
            stop(hero, &class);
            return;
        }
        let t = &mut hero.weapons.reactive.taunter;
        t.held = if held && !fc { t.held + 1 } else { 0 };
        blend(hero, 1, 5);
        return;
    }
    t.held += 1;
    if !held && ticks(0x1e) < t.on_seq {
        stop(hero, &class);
        return;
    }
    if fc {
        stop(hero, &class);
        return;
    }
    if env.frame & 3 == 0 { do_lure(hero, table, env, hits, rng, tick); }
}

fn stop(hero: &mut Hero, class: &super::items::ItemClass) {
    if let Some(it) = hero.items.slot.item.as_mut() {
        if it.anim.seq_b != 1 { moby_anim::set_sequence(&mut it.anim, &class.anim, 1, 0, ticks(5), &mut it.snapshot); }
    }
    hero.fx.item_voices.push(SoundCmd::ItemRelease { n: CHANNEL });
    hero.weapons.reactive.taunter.held = 0;
    hero.weapons.reactive.taunter.slot = false;
}

fn do_lure(hero: &mut Hero, table: &mut MobyTable, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng, tick: u64) {
    let from = hero.items.slot.item.as_ref().map_or(to_f32x3(hero.pos), |it| it.position);
    let range = crate::weapon_evo::extra(14, "lure_range", hero.weapons.reactive.taunter.range);
    let mut crates = hero.weapons.reactive.taunter.crates;
    let targets = env.targets.to_vec();
    let mut lured = Vec::new();
    let mut t10 = None;
    hits.world(table, &*hero, rng, tick, &mut |w| { (lured, t10) = lure(w, &targets, from, range, &mut crates); });
    let t = &mut hero.weapons.reactive.taunter;
    if let Some(x) = t10 { t.t10 = x; }
    t.crates = crates;
    t.lures += 1;
    t.lured = lured;
}

/// `0x2cc830(range, taunter)` (module doc) with the Taunter at `from` (the lure moby: Ratchet's, [L]); `crates` is its
/// pvar +0x14. Returns the lured mobys and the crate hit's `ticks(rand_range(5, 45))` for the Taunter's +0x10.
pub fn lure(w: &mut World, list: &[MobyId], from: [f32; 3], range: f32, crates: &mut i32) -> (Vec<MobyId>, Option<i32>) {
    let Some(me) = w.hero_moby else { return (Vec::new(), None) };
    let pick = w.rng.randi((*crates).max(1));
    *crates = 0;
    let hp = w.hero.pos.map(|x| f32::from_bits(x.0));
    let hyaw = f32::from_bits(w.hero.rot[2].0);
    let at: c::V = [from[0], from[1], from[2], 0.0];
    let cone = crate::weapon_evo::extra(14, "lure_cone_degrees", CONE.to_degrees())
        .clamp(1.0, 180.0).to_radians();
    let mut out = Vec::new();
    let mut t10 = None;
    for &m in list {
        if m >= w.table.mobys.len() { continue; }
        let Some(slot) = react::lure(w, m) else { continue };
        let p = c::pos(w, m);
        let d = c::dist3(at, p);
        let facing = || diff_rots(hyaw, c::atan(p[0] - hp[0], p[1] - hp[1])) < cone;
        if d < CRATE_REACH && CRATES.contains(&w.m(m).o_class) && facing() {
            if *crates == pick {
                let dir = c::set_len3(c::sub(p, at), 1.5);
                attack::hit_moby(w, m, me, 1.0, 0x10000, [hp[0], hp[1], hp[2], 0.0], dir);
                let n = w.rng.rand_range(5, 0x2d);
                t10 = Some(w.ticks(n));
            }
            *crates += 1;
        }
        if d < range && facing() {
            let cur = c::pi32(w, m, slot);
            // The current lure's distance (the only lure is the Taunter in Ratchet's hand).
            let nearer = cur == 0 || d < c::dist3(at, p);
            if nearer {
                c::set_pi32(w, m, slot, me as i32 + 1);
                out.push(m);
                // XP for a newly lured living enemy, once per enemy and session.
                let is_enemy = w.classes.info(w.m(m).o_class).is_some_and(|k| k.ty == 5)
                    || react::table(w, m).is_some();
                if let Some(enemy_hp) = c::damage::special_health(w, m).filter(|_| is_enemy) {
                    let frac = crate::weapon_evo::extra(14, "xp_attract_fraction", 1.0 / 3.0);
                    w.svc.bomb_evo.award_special_once(14, m, enemy_hp, frac);
                    // Taunter's actual stun: a percentage roll once per cooldown,
                    // then a temporary suspension of this creature's AI update.
                    // Its native animation/collision keeps running throughout.
                    let stun_range = crate::weapon_evo::extra(14, "stun_range", 0.0);
                    let duration = crate::weapon_evo::extra(14, "stun_duration_ticks", 0.0)
                        .round().clamp(0.0, 600.0) as u64;
                    let efficiency = crate::weapon_evo::extra(14, "stun_efficiency_percent", 100.0)
                        .clamp(0.0, 100.0);
                    let moby_class = w.m(m).o_class;
                    let now = w.counter;
                    if stun_range > 0.0 && d < stun_range && duration > 0 && efficiency > 0.0
                        && w.svc.evo_stuns.can_attempt(m, moby_class, now)
                    {
                        let success = w.rng.randf(0.0, 100.0) < efficiency;
                        if w.svc.evo_stuns.attempt(m, moby_class, now, duration, success) {
                            // One native, damage-free recoil, only at the moment
                            // the real stun starts (not every four ticks).
                            let distance=crate::weapon_evo::extra(14,"stun_launch_distance",0.0);
                            if distance>0.0 {
                                let mut delta=c::sub(p,at);delta[2]=0.0;delta[3]=0.0;
                                let delta=c::set_len3(delta,distance);
                                w.svc.evo_stuns.launch(m,delta,(duration as u32).min(20).max(1));
                            }
                            let push = c::set_len3(c::sub(p, at), 0.25);
                            attack::hit_moby(w, m, me, 0.0, 0x10000, [hp[0], hp[1], hp[2], 0.0], push);
                        }
                    }
                    // V5 offensive aura is separately ranged from attraction and stun.
                    let hit_range = crate::weapon_evo::extra(14, "damage_range", 0.0);
                    let dps = crate::weapon_evo::extra(14, "damage_per_second", 0.0) / if w.svc.bomb_evo.real_hp_enabled() { 100.0 } else { 1.0 };
                    if hit_range > 0.0 && d < hit_range && dps > 0.0 && w.counter % 60 == 0 {
                        let push = c::set_len3(c::sub(p, at), 0.1);
                        if w.svc.bomb_evo.real_hp_enabled() && crate::moby_update::real_hp::donor(w.m(m)) {
                            let template=crate::moby_update::services::HitTemplate {
                                dir:push.map(Pf::f),attacker:Some(me),flags:0x10000,h1a:CLASS as u16,
                                damage:Pf::f(dps),w20:1,..Default::default()
                            };
                            w.deliver_hit(m,&template);
                        } else {
                            let dealt = dps.min(crate::targeting::record_health(w.m(m)).unwrap_or(dps).max(0.0));
                            attack::hit_moby(w, m, me, dps, 0x10000, [hp[0], hp[1], hp[2], 0.0], push);
                            w.svc.bomb_evo.award_extra_damage(14, dealt);
                        }
                    }
                }
            }
        }
    }
    // The second loop: the Mine Glove's mines (list 0x1b0c30: the table's mines, not deleted) within `range` (xy) and
    // 55° of Ratchet's facing are lured (+0x78; `classes::mine` triples its reach and clears it each tick).
    use crate::moby_update::classes::mine;
    let mines: Vec<MobyId> = w.table.mobys.iter().enumerate().filter(|(_, x)| x.o_class == mine::CLASS && x.state < 0x80).map(|(i, _)| i).collect();
    for m in mines {
        let p = c::pos(w, m);
        if !(c::dist2(at, p) < range) { continue; }
        if diff_rots(hyaw, c::atan(p[0] - hp[0], p[1] - hp[1])) < cone {
            if w.m(m).pvars.len() < mine::pv::SIZE { w.mm(m).pvars.resize(mine::pv::SIZE, 0); }
            c::set_pi32(w, m, mine::pv::LURED, me as i32 + 1);
            out.push(m);
        }
    }
    (out, t10)
}

#[cfg(test)]
mod evolution_visual_tests {
    use super::*;
    fn native() -> WaveVisual {
        WaveVisual { speed: RING_SPEED, size: 1.0, reach: RING_REACH,
            interval: RING_EVERY, alpha: RING_ALPHA, rgb: RING_RGB }
    }
    fn rings() -> Rings {
        Rings { reset: true, dir: [1.0, 0.0, 0.0], up: [0.0, 0.0, 1.0], ..Default::default() }
    }
    #[test]
    fn waves_change_speed_size_colour_without_changing_live_rings() {
        let mut a = rings();
        let mut b = rings();
        a.step_visual(native());
        b.step_visual(WaveVisual { speed: RING_SPEED * 2.0, size: 3.0, rgb: 0xff0000,
            alpha: 80, ..native() });
        assert!((b.rings[0].dist - a.rings[0].dist * 2.0).abs() < 1e-6);
        assert!(b.life < a.life);
        let before = b.rings[0];
        b.paused = true;
        b.step_visual(native());
        assert_eq!(b.rings[0].rgb, before.rgb);
        assert_eq!(b.rings[0].size_multiplier, 3.0);
        assert!((b.rings[0].dist - before.dist - before.speed / 60.0).abs() < 1e-6);
        let q = b.quads();
        assert_eq!(q.len(), 1);
        assert_eq!(q[0].2[0] & 0xffffff, 0xff0000);
        let k = b.rings[0].left as f32 * b.rings[0].inv_life;
        let size = (RING_SIZE[1] + (RING_SIZE[0] - RING_SIZE[1]) * k) * 3.0;
        assert!((q[0].0[0][1].abs() - size).abs() < 1e-5);
        for _ in 0..100 { b.step_visual(native()); }
        assert!(b.quads().is_empty());
    }
    #[test]
    fn dense_waves_stay_within_native_pool() {
        let mut r = rings();
        let look = WaveVisual { interval: 1, reach: 80.0, speed: 2.0, ..native() };
        for _ in 0..3000 { r.step_visual(look); }
        assert!(r.quads().len() <= 10);
        assert!(r.rings.iter().all(|r| r.dist.is_finite()));
    }
}
