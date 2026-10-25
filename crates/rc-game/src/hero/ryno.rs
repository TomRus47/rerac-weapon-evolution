//! **The R.Y.N.O.** (item 23, class 454; level01 item update `0x2e4e60`, its search `0x2e4bb8`, the missile's spawn
//! `0x2e5738`; read from the decompiler output and the disassembly). No weapon-check case: its update fires. A
//! persistent-arm weapon (def +0x30 = 1; standing sequence 81, no moving arm layer). The missiles are class 457
//! (`crate::moby_update::classes::ryno_missile`).
//!
//! **Pvars** ([`Ryno`]): +0x00 the target, +0x04 the salvo's shot count, +0x05 the barrel (the class's joint lists
//! 0..8), +0x06 the shot timer, +0x08 the lock timer, +0x0c the target's aim height, +0x10 the marker's wait, +0x12
//! the next slot of +0x18.. (the salvo's seven past targets), +0x14 the target's health left for this salvo.
//!
//! **Every tick in the item's states 2 / 3** (ready / aiming): the lock timer steps; a target it outlived, or a dead
//! one, is dropped. **The search** [`search`] (`0x2e4bb8(97.3°, 97.3°, 80, barrel point, camera angles, none)`) from
//! the barrel's point with the camera's yaw / elevation; a live, drawn (+0x31) current target is kept, else the found
//! one is taken: the lock timer `ticks(20)`, the health left = its record's `+0x00` (1 without a record), the aim
//! height its record's `+0x10`. With a target, the marker's wait out and ammo: the green marker (FX 0x23, colour
//! 0xff0fff0f) at its aim point, size `|0.9 − d/150·0.65|`.
//!
//! **States** (+0x20): 0 → 1 (target none, the swap lock 0x1403fc clear, the marker's wait 0); 1 → 2 when the draw
//! wraps; **2**: the look stance 1 becomes 0x1e; ○ pressed (not 0x1413fc) with one ammo (`0x249450`) → **4**, the swap
//! lock 2 (no item swap during the salvo), the shot timer and counters 0, the weapon drawn (`0x22ee08`), the past
//! targets cleared; **4** (the salvo): every `ticks(9)` one missile from the next barrel (0..8) — along Ratchet's facing
//! (in 0x1e the camera's) plus 3× his displacement and platform motion, the weapon drawn again, `0x2e5738(aim height,
//! yaw, pitch, item, barrel point, target)` — seven in all (one ammo for the salvo), then the swap lock clear, the
//! weapon put away, state **5** with the shot timer and the marker's wait `ticks(60)`. Each missile takes 1 off the
//! target's health left; at 0 (or with the target gone) the target joins the past targets and a new one is searched
//! (`0x2e4bb8(π, π, 100, the item's position, the camera angles, the past targets)`), else a past one is picked at
//! random (`rand() & (n + 1)` must be 0, then `rand() % n`); **5**: after the shot timer, target none, the count 0,
//! back to 2. **State 3** (0x2e5264, ported 2026-09-28): the red crosshair (FX 0x23, 0xff0f0fff, size 1) at the screen
//! centre every tick; with L1 or L2 held ○ fires the salvo as state 2 does ([`start_salvo`], the same code in the game),
//! released → state 2. No code in the level overlays writes 3 to the item's +0x20 (every writer of the R.Y.N.O.'s state
//! stores 0 / 1 / 2 / 4 / 5): the state is unreachable in play, kept for completeness and tested by setting it. The
//! prologue's lock / marker upkeep also runs in state 3 (states 2 / 3). The shot statistics (0x141738..) are not ported
//! (G-SAV-009).
//!
//! **Native.** Standard `f32`; the camera angles are the camera's yaw and its elevation (0x167250's `(x, −y, z)`: y
//! is the pitch, positive down) from its forward row [M]. The item is not a table moby in the port: the missiles'
//! owner is Ratchet's moby, and the second search starts from the item's position (moby +0x10, `HeroItemsAttach`'s).

use super::guns::{self, add3, len3, scale3, sub3};
use super::items::{HitSink, ItemEnv};
use super::physics::{to_f32x3, ticks};
use super::Hero;
use crate::moby_runtime::{mode, MobyId, MobyTable};
use crate::moby_update::creature::{atan, diff_rots};
use crate::rng::Rng;
use crate::targeting::{self, Marker};

pub const RYNO: i32 = 23;
pub const CLASS: i16 = 454;
pub const MARKER_FX: usize = 0x23;
/// The first search's cones and range (`0x3fd95d40` = 1.6982 rad, 80) and the salvo's (π, 100).
pub const CONE: f32 = 1.698_161_6;
pub const RANGE: f32 = 80.0;
pub const SALVO_RANGE: f32 = 100.0;
/// Missiles per salvo and the barrels.
pub const SALVO: u8 = 7;
pub const BARRELS: u8 = 9;

/// The R.Y.N.O.'s pvars (item moby +0x78).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ryno {
    /// +0x00: the target.
    pub target: Option<MobyId>,
    /// +0x04 / +0x05: the salvo's shot count, the barrel.
    pub count: u8,
    pub barrel: u8,
    /// +0x06: the shot timer (s16); +0x08 the lock timer.
    pub shot_timer: i16,
    pub lock_timer: i32,
    /// +0x0c: the target's aim height; +0x14 its health left for this salvo.
    pub aim_height: f32,
    pub left: f32,
    /// +0x10: the marker's wait (s16).
    pub marker_wait: i16,
    /// +0x12 / +0x18..: the past targets of the salvo (seven slots, the next one to write).
    pub next: i16,
    pub past: [Option<MobyId>; 7],
}

fn dead(table: &MobyTable, id: MobyId) -> bool { table.mobys.get(id).is_none_or(|m| m.state == 0xfe || m.state == 0xfd) }

/// `0x2e4bb8(cone_yaw, cone_pitch, range, from, angles, exclude)`: the R.Y.N.O.'s target (module doc). Walks the run
/// list (here: the target list, the same mobys in its order, plus the ones the draw distance keeps): a moby with a draw
/// distance (+0x32), drawn (+0x31, only without an exclusion list) or not excluded, with a record whose health is not
/// negative, targetable, a creature (class type 5), within `range` (2D); its aim point 0.1 above the record's height;
/// inside `cone_yaw` of `angles.yaw` and `cone_pitch` of `angles.elevation`; score `d/5` within 5, else
/// `yaw²·pitch²·d + d`; the lowest score with a clear line (flags 2) wins.
#[allow(clippy::too_many_arguments)]
pub fn search(table: &MobyTable, list: &[MobyId], cone_yaw: f32, cone_pitch: f32, range: f32, from: [f32; 3], yaw: f32, elevation: f32, exclude: Option<&[Option<MobyId>; 7]>, class_type: &dyn Fn(i16) -> Option<u8>, mut clear: impl FnMut([f32; 3], [f32; 3], MobyId) -> bool) -> Option<MobyId> {
    search_from_hero(table, list, cone_yaw, cone_pitch, range, from, from, yaw, elevation, exclude, class_type, &mut clear)
}

#[allow(clippy::too_many_arguments)]
fn search_from_hero(table: &MobyTable, list: &[MobyId], cone_yaw: f32, cone_pitch: f32, range: f32, from: [f32; 3], hero: [f32; 3], yaw: f32, elevation: f32, exclude: Option<&[Option<MobyId>; 7]>, class_type: &dyn Fn(i16) -> Option<u8>, mut clear: impl FnMut([f32; 3], [f32; 3], MobyId) -> bool) -> Option<MobyId> {
    let mut best = 1e9f32;
    let mut out = None;
    for &id in list {
        let Some(m) = table.mobys.get(id) else { continue };
        if m.draw_dist == 0 { continue; }
        match exclude {
            None => { if m.visible == 0 { continue; } }
            Some(x) => { if x.contains(&Some(id)) { continue; } }
        }
        let Some(health) = targeting::record_health(m) else { continue };
        if health < 0.0 || m.mode & mode::TARGETABLE == 0 || !m.has_class || class_type(m.o_class) != Some(5) { continue; }
        let d = ((m.position[0] - from[0]).powi(2) + (m.position[1] - from[1]).powi(2)).sqrt();
        if !(d < range) { continue; }
        let p = [m.position[0], m.position[1], m.position[2] + targeting::aim_height(m).unwrap_or(0.0) + 0.1];
        if !targeting::reach::missile_allows(RYNO, m, from, hero, p) { continue; }
        let dy = diff_rots(yaw, atan(p[0] - from[0], p[1] - from[1]));
        if !(dy * dy < cone_yaw * cone_yaw) { continue; }
        let dp = diff_rots(elevation, atan(d, p[2] - from[2]));
        if cone_pitch * cone_pitch <= dp * dp { continue; }
        let score = if 5.0 < d { dy * dy * dp * dp * d + d } else { d / 5.0 };
        if score < best && clear(from, p, id) {
            best = score;
            out = Some(id);
        }
    }
    out
}

/// The camera's yaw and elevation (0x167258 and −0x167254).
pub fn camera_angles(env: &ItemEnv) -> (f32, f32) {
    match env.camera {
        Some((_, f)) => (atan(f[0], f[1]), atan((f[0] * f[0] + f[1] * f[1]).sqrt(), f[2])),
        None => (0.0, 0.0),
    }
}

/// The new target's lock: the lock timer, the health left and the aim height from its record.
fn lock(r: &mut Ryno, table: &MobyTable, t: Option<MobyId>) {
    r.target = t;
    r.lock_timer = ticks(20);
    r.left = 1.0;
    r.aim_height = 0.0;
    if let Some(m) = t.and_then(|t| table.mobys.get(t)) {
        if let Some(rec) = targeting::record(m) {
            r.aim_height = f32::from_le_bytes(m.pvars[rec + 0x10..rec + 0x14].try_into().unwrap());
            r.left = f32::from_le_bytes(m.pvars[rec..rec + 4].try_into().unwrap());
        }
    }
}

/// States 2 and 3 on ○ (the same code twice in `0x2e4e60`): with one ammo (`0x249450`), the shot statistics
/// 0x141738.. (not kept: G-SAV-009), the swap lock 2, state 4, the shot timer, barrel and count 0, the weapon drawn
/// (`0x22ee08`), the past targets cleared.
fn start_salvo(hero: &mut Hero, id: i32) {
    if !hero.weapons.use_ammo(id, 1) { return; }
    hero.items.slot.swap = 2;
    if let Some(it) = hero.items.slot.item.as_mut() { it.mstate = 4; }
    let r = &mut hero.weapons.ryno;
    r.shot_timer = 0;
    r.barrel = 0;
    r.count = 0;
    r.past = [None; 7];
    r.next = 0;
    hero.weapons.pending_draw = true;
}

/// `0x2e4e60`, the R.Y.N.O.'s update (from the slot loop with the slot ready).
pub fn update(hero: &mut Hero, table: &mut MobyTable, _anim: &dyn super::anim::AnimCtl, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng) {
    let Some(item) = hero.items.slot.item.as_ref() else { return };
    let mstate = item.mstate;
    let wrapped = item.anim.flags & 2 != 0;
    let item_pos = item.position;
    let id = hero.items.slot.id;
    let tick = env.frame as u64;
    if super::combat_strafe::active(hero) && hero.items.f13fc == 0 {
        hero.weapons.markers.register(tick, Marker { size: 1.0, angle: 0.0, rgba: super::blaster::MARKER_RED, at: None, fx: MARKER_FX });
    }
    let coll = env.coll;
    // The class types of the listed mobys (class header +0x46), read once.
    let types: Vec<(i16, Option<u8>)> = env.targets.iter().filter_map(|&t| table.mobys.get(t)).map(|m| (m.o_class, hits.class_type(m.o_class))).collect();
    let class_type = |c: i16| types.iter().find(|x| x.0 == c).and_then(|x| x.1);
    let clear = |a: [f32; 3], b: [f32; 3], _m: MobyId| coll.is_none_or(|c| super::physics::line_world(c, super::physics::from_f32x3(a), super::physics::from_f32x3(b), 2).is_none());
    if mstate.wrapping_sub(2) < 2 {
        let r = &mut hero.weapons.ryno;
        if !guns::dec(&mut r.lock_timer) {
            if r.target.is_some_and(|t| dead(table, t)) { r.target = None; }
            if r.target.is_none() { r.aim_height = 0.0; }
        } else {
            r.target = None;
            r.aim_height = 0.0;
        }
        let (yaw, elev) = camera_angles(env);
        let muzzle = guns::item_point(hero, env, hero.weapons.ryno.barrel as usize);
        let limit = targeting::reach::missile_envelope(RYNO, 0,
            crate::weapon_evo::extra(23, "missile_speed_multiplier", 1.0),
            crate::weapon_evo::extra(23, "missile_range_multiplier", 1.0));
        let found = search_from_hero(table, env.targets, CONE, CONE, limit.0.min(limit.1), muzzle, to_f32x3(hero.pos), yaw, elev, None, &class_type, clear);
        let r = &mut hero.weapons.ryno;
        let keep = r.target.is_some_and(|t| !dead(table, t) && table.mobys[t].visible != 0
            && targeting::reach::missile_allows(RYNO, &table.mobys[t], muzzle, to_f32x3(hero.pos),
                [table.mobys[t].position[0], table.mobys[t].position[1], table.mobys[t].position[2] + r.aim_height]));
        if !keep { lock(r, table, found); }
        if let Some(t) = r.target {
            if guns::dec16(&mut r.marker_wait) && hero.weapons.has_ammo(id) != 0 {
                let r = &mut hero.weapons.ryno;
                if dead(table, t) {
                    r.target = None;
                    r.aim_height = 0.0;
                } else {
                    let cam = env.camera.map_or(muzzle, |c| c.0);
                    let m = &table.mobys[t];
                    let d = len3(sub3([m.position[0], m.position[1], m.position[2]], cam));
                    let size = (0.9 - (d / 150.0) * 0.65).abs();
                    let at = [m.position[0], m.position[1], m.position[2] + r.aim_height];
                    hero.weapons.markers.register(tick, Marker { size, angle: 0.0, rgba: super::blaster::MARKER_GREEN, at: Some(at), fx: MARKER_FX });
                }
            }
        }
    }
    if mstate >= 2 && (crate::weapon_evo::extra_flag(RYNO, "continuous_missiles_enabled", false)
        || crate::weapon_evo::extra_flag(RYNO, "shotgun_enabled", false)) {
        experimental_fire(hero, table, env, hits, rng);
        return;
    }
    let set_state = |hero: &mut Hero, s: u8| if let Some(it) = hero.items.slot.item.as_mut() { it.mstate = s; };
    match mstate {
        0 => {
            let r = &mut hero.weapons.ryno;
            r.target = None;
            hero.items.slot.swap = 0;
            r.marker_wait = 0;
            set_state(hero, 1);
        }
        1 => { if wrapped { set_state(hero, 2); } }
        2 => {
            if hero.state == 1 { hero.weapons.deferred = Some(0x1e); }
            if env.pad.pressed & hero.items.slot.fire_mask == 0 || hero.items.f13fc != 0 { return; }
            start_salvo(hero, id);
        }
        3 => {
            // The first-person variant (0x2e5264): the red crosshair at the screen centre (`FUN_0020fb60(1.0, 0, 90.0,
            // item, 0xff0f0fff, none, 0x23, −1, 4)`) every tick; while L1 or L2 is held (0x13cae0 & 5) ○ fires the
            // salvo as state 2 does; without them, back to state 2.
            hero.weapons.markers.register(tick, Marker { size: 1.0, angle: 0.0, rgba: super::blaster::MARKER_RED, at: None, fx: MARKER_FX });
            if env.pad.held & (crate::pad::button::L1 | crate::pad::button::L2) == 0 {
                set_state(hero, 2);
                return;
            }
            if env.pad.pressed & hero.items.slot.fire_mask == 0 || hero.items.f13fc != 0 { return; }
            start_salvo(hero, id);
        }
        4 => {
            if !guns::dec16(&mut hero.weapons.ryno.shot_timer) { return; }
            {
                let r = &mut hero.weapons.ryno;
                r.shot_timer = ticks(crate::weapon_evo::extra(23, "fire_interval_ticks", 9.0).round() as i32) as i16;
                r.barrel = (r.barrel + 1) % BARRELS;
                r.count += 1;
            }
            let fwd = match (hero.state == 0x1e, env.camera) {
                (true, Some((_, f))) => f,
                _ => to_f32x3(hero.aim_rows[0]),
            };
            let motion = scale3(add3(to_f32x3(hero.disp), to_f32x3(hero.plat_applied)), 3.0);
            let fwd = if super::combat_strafe::active(hero) {
                let muzzle = guns::item_point(hero, env, hero.weapons.ryno.barrel as usize);
                guns::combat_direction(hero, env, muzzle, 100.0)
            } else { fwd };
            let dir = if super::combat_strafe::active(hero) { fwd } else { add3(fwd, motion) };
            let (yaw, pitch) = guns::aim_angles(dir);
            let muzzle = guns::item_point(hero, env, hero.weapons.ryno.barrel as usize);
            hero.weapons.pending_draw = true;
            if hero.weapons.ryno.target.is_some_and(|t| table.mobys.get(t).is_none_or(|m|
                !targeting::reach::missile_allows(RYNO, m, muzzle, to_f32x3(hero.pos),
                    [m.position[0], m.position[1], m.position[2] + hero.weapons.ryno.aim_height]))) {
                hero.weapons.ryno.target = None;
            }
            let (aimh, target) = (hero.weapons.ryno.aim_height, hero.weapons.ryno.target);
            crate::moby_update::classes::ryno_missile::spawn(hero, table, env, hits, rng, aimh, yaw, pitch, muzzle, target);
            if (crate::weapon_evo::extra(23, "missiles_per_salvo", SALVO as f32).round() as u8).clamp(1, 36) <= hero.weapons.ryno.count {
                hero.items.slot.swap = 0;
                super::weapons::put_away(hero);
                set_state(hero, 5);
                let r = &mut hero.weapons.ryno;
                r.shot_timer = ticks(60) as i16;
                r.marker_wait = ticks(60) as i16;
            }
            let r = &mut hero.weapons.ryno;
            r.left -= 1.0;
            if 0.0 < r.left && r.target.is_some_and(|t| !dead(table, t)) { return; }
            let k = r.next.clamp(0, 6) as usize;
            r.past[k] = r.target;
            r.next = (r.next + 1) % 7;
            let (yaw, elev) = camera_angles(env);
            let past = r.past;
            let limit = targeting::reach::missile_envelope(RYNO, 0,
                crate::weapon_evo::extra(23, "missile_speed_multiplier", 1.0),
                crate::weapon_evo::extra(23, "missile_range_multiplier", 1.0));
            let found = search_from_hero(table, env.targets, std::f32::consts::PI, std::f32::consts::PI, limit.0.min(limit.1), item_pos, to_f32x3(hero.pos), yaw, elev, Some(&past), &class_type, clear);
            let r = &mut hero.weapons.ryno;
            let pick = match found {
                Some(t) => Some(t),
                None if r.next < 1 => None,
                None => {
                    if rng.rand() & (r.next as i32 + 1) != 0 {
                        None
                    } else {
                        let n = r.next as i32;
                        past[(rng.rand() % n) as usize]
                    }
                }
            };
            lock(r, table, pick);
        }
        5 => {
            if !guns::dec16(&mut hero.weapons.ryno.shot_timer) { return; }
            hero.weapons.ryno.target = None;
            hero.weapons.ryno.count = 0;
            set_state(hero, 2);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_runtime::Moby;

    fn creature(pos: [f32; 3], health: f32) -> Moby {
        let mut m = Moby { position: [pos[0], pos[1], pos[2], 1.0], mode: mode::TARGETABLE | 0x20, pvars: vec![0; 0x80], has_class: true, draw_dist: 0x40, visible: 1, o_class: 577, ..Moby::default() };
        m.pvars[0..4].copy_from_slice(&0x20u32.to_le_bytes());
        m.pvars[0x20..0x24].copy_from_slice(&health.to_le_bytes());
        m.pvars[0x30..0x34].copy_from_slice(&0.4f32.to_le_bytes());
        m
    }

    /// 0x2e4bb8: creatures only, alive, in the cones and range; the lowest score wins (close ones by distance / 5);
    /// the exclusion list replaces the drawn test.
    #[test]
    fn search_scores_and_filters() {
        let ty = |c: i16| if c == 577 { Some(5) } else { Some(0) };
        let from = [0.0, 0.0, 0.0];
        let t = MobyTable::new(vec![creature([10.0, 1.0, 0.0], 3.0), creature([4.0, 0.0, 0.0], 3.0), creature([90.0, 0.0, 0.0], 3.0)], 0);
        assert_eq!(search(&t, &[0, 1, 2], CONE, CONE, RANGE, from, 0.0, 0.0, None, &ty, |_, _, _| true), Some(1));
        let x = [Some(1), None, None, None, None, None, None];
        assert_eq!(search(&t, &[0, 1, 2], CONE, CONE, RANGE, from, 0.0, 0.0, Some(&x), &ty, |_, _, _| true), Some(0));
        let mut dead = creature([4.0, 0.0, 0.0], -1.0);
        dead.visible = 1;
        let t2 = MobyTable::new(vec![dead], 0);
        assert_eq!(search(&t2, &[0], CONE, CONE, RANGE, from, 0.0, 0.0, None, &ty, |_, _, _| true), None);
        // Behind (180°) is outside 97°; the salvo's π cone takes it.
        let t3 = MobyTable::new(vec![creature([-10.0, 0.0, 0.0], 3.0)], 0);
        assert_eq!(search(&t3, &[0], CONE, CONE, RANGE, from, 0.0, 0.0, None, &ty, |_, _, _| true), None);
        assert_eq!(search(&t3, &[0], std::f32::consts::PI + 0.01, std::f32::consts::PI + 0.01, SALVO_RANGE, from, 0.0, 0.0, None, &ty, |_, _, _| true), Some(0));
    }
}

/// Experimental modes take complete ownership of firing: neither falls through
/// to the native seven-missile salvo. Only one ammo clock is advanced.
fn experimental_fire(hero: &mut Hero, table: &mut MobyTable, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng) {
    experimental_fire_mode(hero, table, env, hits, rng,
        crate::weapon_evo::extra_flag(RYNO, "shotgun_enabled", false));
}

fn experimental_fire_mode(hero: &mut Hero, table: &mut MobyTable, env: &ItemEnv,
    hits: &mut dyn HitSink, rng: &mut Rng, shotgun: bool) {
    hero.items.slot.swap = 0;
    if let Some(item) = &mut hero.items.slot.item { item.mstate = 2; }
    let ready = guns::dec16(&mut hero.weapons.ryno.shot_timer);
    // Release the persistent R.Y.N.O. firing stance even while its next shot
    // is cooling down. The native salvo normally does this in state 4; both
    // experiments bypass that state and must own the matching cleanup.
    // Keep the cooldown running so tapping fire cannot bypass the fire rate.
    if env.pad.held & hero.items.slot.fire_mask == 0 || hero.items.f13fc != 0
        || hero.weapons.has_ammo(RYNO) == 0 {
        super::weapons::put_away(hero);
        return;
    }
    if !ready { return; }
    // Admit a whole volley before charging ammo or queuing any effects. Slot
    // exhaustion must never silently remove the last rows of a 7 x 5 grid.
    let reserved = if shotgun {
        let Some(ids) = reserve_shotgun(table, env, hits) else { return };
        Some(ids)
    } else { None };
    if !hero.weapons.use_ammo(RYNO, 1) {
        if let Some(ids) = reserved { for id in ids { hits.delete_moby(table, id, env.frame as u64); } }
        return;
    }
    hero.weapons.pending_draw = true;
    hero.weapons.ryno.shot_timer = ticks(crate::weapon_evo::extra(RYNO, "fire_interval_ticks", 9.0).round().max(1.0) as i32) as i16;
    let fwd = match (super::combat_strafe::active(hero) || hero.state == 0x1e, env.camera) {
        (true, Some((_, f))) => f,
        _ => to_f32x3(hero.moby_rows[0]),
    };
    let yaw = atan(fwd[0], fwd[1]);
    let pitch = -atan(fwd[0].hypot(fwd[1]), fwd[2]);
    if let Some(reserved) = reserved {
        use crate::moby_update::classes::blaster_shot::{spawn_ryno, RynoPellet};
        let value = |key, default| crate::weapon_evo::extra(RYNO, key, default);
        let profile = RynoPellet { hp: value("shotgun_damage_hp", 100.0), speed: value("shotgun_speed", 40.0),
            range: value("shotgun_range", 30.0), size: value("shotgun_size_percent", 100.0) / 100.0,
            colors: ["shotgun_projectile_color", "shotgun_trail_color", "shotgun_impact_color"].map(|k| crate::weapon_evo::extra_color(RYNO, k)) };
        let horizontal = value("shotgun_horizontal_spread_degrees",
            value("shotgun_spread_degrees",12.0)).to_radians();
        let vertical = value("shotgun_vertical_spread_degrees",3.0).to_radians();
        let muzzles: [[f32;3];9] = std::array::from_fn(|barrel|guns::item_point(hero,env,barrel));
        let centre=scale3(muzzles.iter().copied().fold([0.0;3],add3),1.0/9.0);
        let up_hint = env.camera_up.unwrap_or_else(||to_f32x3(hero.moby_rows[2]));
        let (forward,right,up)=shotgun_basis(fwd,up_hint);
        let mut half=[0.0f32;2];
        for muzzle in muzzles {
            let delta=sub3(muzzle,centre);
            half[0]=half[0].max(guns::dot3(delta,right).abs());
            half[1]=half[1].max(guns::dot3(delta,up).abs());
        }
        // New columns/rows interpolate the weapon's muzzle plane, with minimum
        // spacing scaled to pellet size so a zero-spread volley stays a grid.
        half[0]=half[0].max(0.6*profile.size);
        half[1]=half[1].max(0.4*profile.size);
        for row in 0..5 {
            for column in 0..7 {
                let cell=shotgun_cell(column,row);
                let muzzle=add3(centre,add3(scale3(right,half[0]*cell[0]),scale3(up,half[1]*cell[1])));
                let dir=shotgun_direction(forward,right,up,cell,horizontal,vertical);
                let aim=(atan(dir[0],dir[1]),-atan(dir[0].hypot(dir[1]),dir[2]));
                spawn_ryno(reserved[row * 7 + column],hero,table,env,hits,rng,aim,muzzle,profile);
            }
        }
        let at = guns::item_point(hero, env, 4);
        hero.fx.item_voices.push(crate::hero::packs::SoundCmd::MobySound { moby: env.hero_moby,
            o_class: crate::moby_update::classes::ryno_missile::CLASS, pos: at, index: 1, flags: 0 });
    } else {
        let barrel = hero.weapons.ryno.barrel;
        hero.weapons.ryno.barrel = (barrel + 1) % BARRELS;
        let muzzle = guns::item_point(hero, env, barrel as usize);
        let (height, target) = (hero.weapons.ryno.aim_height, hero.weapons.ryno.target);
        crate::moby_update::classes::ryno_missile::spawn(hero, table, env, hits, rng, height, yaw, pitch, muzzle, target);
    }
}

/// Reservation has no muzzle probes, particles or ammo side effects. The
/// preflight prevents churning nearly-full tables; rollback also covers a hit
/// sink that rejects allocation despite available native slots.
fn reserve_shotgun(table: &mut MobyTable, env: &ItemEnv, hits: &mut dyn HitSink) -> Option<[MobyId; 35]> {
    if table.available_dynamic_slots(env.frame as u64) < 35 { return None; }
    let mut ids = [0; 35];
    for n in 0..ids.len() {
        match hits.create_moby(table, crate::moby_update::classes::blaster_shot::CLASS, env.frame as u64) {
            Some(id) => ids[n] = id,
            None => {
                for &id in &ids[..n] { hits.delete_moby(table, id, env.frame as u64); }
                return None;
            }
        }
    }
    Some(ids)
}

/// Symmetric 7-column / 5-row coordinates. The exact centre has no spread.
fn shotgun_cell(column: usize,row: usize)->[f32;2] {
    [(column as f32-3.0)/3.0,(row as f32-2.0)/2.0]
}
fn shotgun_basis(forward:[f32;3],up_hint:[f32;3])->([f32;3],[f32;3],[f32;3]) {
    let forward=guns::with_len(forward,1.0);
    let right=guns::with_len(guns::cross3(forward,up_hint),1.0);
    let up=guns::with_len(guns::cross3(right,forward),1.0);
    (forward,right,up)
}
fn shotgun_direction(forward:[f32;3],right:[f32;3],up:[f32;3],cell:[f32;2],horizontal:f32,vertical:f32)->[f32;3] {
    guns::with_len(add3(forward,add3(scale3(right,(horizontal*cell[0]).tan()),
        scale3(up,(vertical*cell[1]).tan()))),1.0)
}

#[cfg(test)]
mod shotgun_tests {
    use super::*;
    use crate::ps2v::Pf;
    fn firing_pose() -> Hero {
        use super::super::items::HandItem;
        let class = rc_formats::moby_anim::MobyAnimClass {
            joint_count: 0, skeleton: vec![], rest: vec![], parent_word: vec![], sequences: vec![],
        };
        let mut hero = Hero::spawn([410., 410., 100.], 0.);
        hero.items.slot.id = RYNO;
        hero.items.slot.fire_mask = crate::pad::button::CIRCLE;
        hero.items.slot.item = Some(HandItem {
            o_class: CLASS, mstate: 4, anim: rc_formats::moby_anim::AnimState::spawn(&class),
            snapshot: None, scale: 1., position: [0.; 3], rows: [[0; 4]; 3],
            hit_timer: 0, flight: Default::default(),
        });
        hero.f13f8 = 1; hero.f13fa = 1;
        hero.weapons.arm_item = RYNO;
        hero.weapons.uses_ammo[RYNO as usize] = true;
        hero.weapons.ammo[RYNO as usize] = 10;
        hero.weapons.ryno.shot_timer = 12;
        hero.items.slot.swap = 2;
        hero
    }

    fn experimental_tick(hero: &mut Hero, held: bool) {
        let data = super::super::items::ItemData::default();
        let mut pad = crate::pad::PadState::default();
        if held { pad.held = crate::pad::button::CIRCLE; }
        let env = ItemEnv { mobys: None, data: &data, pad: &pad, frame: 0, hero_moby: 0,
            coll: None, camera: None, camera_up: None, targets: &[] };
        experimental_fire(hero, &mut MobyTable::new(vec![], 0), &env,
            &mut super::super::items::NoHits, &mut Rng::new());
    }

    #[test]
    fn release_during_cooldown_clears_firing_pose_without_resetting_fire_rate() {
        let mut hero = firing_pose();
        // A queued draw from the last shot must not restore the firing pose.
        hero.weapons.pending_draw = true;
        experimental_tick(&mut hero, false);
        assert_eq!(hero.f13f8, 0); assert_eq!(hero.f13fa, 0);
        assert!(hero.weapons.pending_idle); assert!(!hero.weapons.pending_draw);
        assert_eq!(hero.items.slot.item.as_ref().unwrap().mstate, 2);
        assert_eq!(hero.items.slot.swap, 0);
        assert_eq!(hero.weapons.ryno.shot_timer, 11);
        experimental_tick(&mut hero, true);
        assert_eq!(hero.weapons.ryno.shot_timer, 10);
        assert!(!hero.weapons.pending_draw);
        assert_eq!(hero.weapons.ammo[RYNO as usize], 10);
    }

    #[test]
    fn blocked_or_empty_experimental_fire_releases_the_pose_even_if_held() {
        for empty in [false, true] {
            let mut hero = firing_pose();
            if empty { hero.weapons.ammo[RYNO as usize] = 0; }
            else { hero.items.f13fc = 1; }
            experimental_tick(&mut hero, true);
            assert_eq!(hero.f13f8, 0); assert_eq!(hero.f13fa, 0);
            assert!(hero.weapons.pending_idle);
            assert_eq!(hero.weapons.ryno.shot_timer, 11);
        }
    }

    #[derive(Default)]
    struct Allocator { calls: usize, fail_at: Option<usize> }
    impl HitSink for Allocator {
        fn sphere(&mut self, _: &mut MobyTable, _: Pf, _: [Pf; 4], _: u32,
            _: Option<MobyId>, _: &crate::moby_update::services::HitTemplate) -> Option<MobyId> { None }
        fn line(&mut self, _: &mut MobyTable, _: [Pf; 4], _: [Pf; 4], _: u32,
            _: Option<MobyId>, _: &crate::moby_update::services::HitTemplate) -> Option<Option<MobyId>> { None }
        fn create_moby(&mut self, table: &mut MobyTable, class: i16, tick: u64) -> Option<MobyId> {
            let n = self.calls; self.calls += 1;
            if self.fail_at == Some(n) { return None; }
            // Missing native update entry is intentional: experimental pellets
            // must install their own update and clear NO_UPDATE.
            table.create(class, Some(&crate::moby_runtime::ClassInfo { scale: 1.0, ..Default::default() }), tick)
        }
    }

    fn shotgun_tick(hero: &mut Hero, table: &mut MobyTable, allocator: &mut Allocator, frame: i32) {
        let data = super::super::items::ItemData::default();
        let mut pad = crate::pad::PadState::default(); pad.held = crate::pad::button::CIRCLE;
        let env = ItemEnv { mobys: None, data: &data, pad: &pad, frame, hero_moby: 0,
            coll: None, camera: Some(([0.; 3], [1., 0., 0.])), camera_up: Some([0., 0., 1.]), targets: &[] };
        hero.state = 0x1e;
        hero.items.slot.item.as_mut().unwrap().position = [410., 410., 100.];
        experimental_fire_mode(hero, table, &env, allocator, &mut Rng::new(), true);
    }

    #[test]
    fn full_volley_is_admitted_before_ammo_and_all_slots_keep_updating() {
        for capacity in [0, 34, 35, 70] {
            let mut hero = firing_pose(); hero.weapons.ryno.shot_timer = 0;
            let mut table = MobyTable::new(vec![crate::moby_runtime::Moby::zeroed()], capacity);
            let mut allocator = Allocator::default();
            shotgun_tick(&mut hero, &mut table, &mut allocator, 10);
            let expected = if capacity < 35 { 0 } else { 35 };
            let live: Vec<_> = table.mobys.iter().filter(|m| crate::moby_update::classes::blaster_shot::is_ryno_pellet(m) && !m.is_deleted()).collect();
            assert_eq!(live.len(), expected);
            assert_eq!(allocator.calls, expected);
            assert_eq!(hero.weapons.ammo[RYNO as usize], if expected == 0 { 10 } else { 9 });
            assert_eq!(hero.weapons.pending_draw, expected != 0);
            assert_eq!(hero.fx.item_voices.len(), usize::from(expected != 0));
            for pellet in live {
                assert_eq!(pellet.update_fn, Some(crate::moby_update::classes::blaster_shot::UPDATE_FN));
                assert_eq!(pellet.mode & crate::moby_runtime::mode::NO_UPDATE, 0);
                assert!(crate::moby_update::scheduler::is_active(pellet, [Pf::f(9000.); 4]));
            }
        }
    }

    #[test]
    fn every_pellet_gets_a_head_before_optional_trails_under_particle_pressure() {
        let mut hero = firing_pose(); hero.weapons.ryno.shot_timer = 0;
        let mut table = MobyTable::new(vec![crate::moby_runtime::Moby::zeroed()], 105);
        let mut allocator = Allocator::default();
        for frame in 10..13 {
            hero.weapons.ryno.shot_timer = 0;
            shotgun_tick(&mut hero, &mut table, &mut allocator, frame);
        }
        let mut particles = crate::particles::Particles::new(None, Vec::new());
        for _ in 0..700 { particles.create_part(1).unwrap(); }
        super::super::fx::create_particles(&hero, &mut particles);
        let heads = particles.pool.recs.iter().filter(|r| r[0] == 26 && r[1] & crate::particles::FLAG_DEAD == 0).count();
        assert_eq!(heads, 105, "all rows remain visible before spending room on tails");
        assert_eq!(particles.pool.count as usize, crate::particles::POOL_RECORDS / 2);
        // The soft budget applies only to the shotgun's optional tails.
        let native = super::super::fx::PartSpawn::Trail72 { moby: 999, slot: 0,
            pos: [0.; 4], draws: crate::particles::type72::Draws::draw(&mut Rng::new()),
            size: 1., rgba: 0x7fff_ffff, pellet: false };
        super::super::fx::create_one(&mut particles, &native, 0);
        assert_eq!(particles.pool.count as usize, crate::particles::POOL_RECORDS / 2 + 1);
        // Exercise the actual same-frame particle update, not just allocation:
        // a new projectile was born after the moby anchor snapshot was built.
        particles.update_parts(&mut Rng::new());
        assert_eq!(particles.pool.recs.iter().filter(|r| r[0] == 26 && r[1] & crate::particles::FLAG_DEAD == 0).count(), 105);
    }

    #[test]
    fn failed_reservation_rolls_back_without_charge_or_partial_effects() {
        let mut hero = firing_pose(); hero.weapons.ryno.shot_timer = 0;
        let mut table = MobyTable::new(vec![crate::moby_runtime::Moby::zeroed()], 35);
        let mut allocator = Allocator { fail_at: Some(17), ..Default::default() };
        shotgun_tick(&mut hero, &mut table, &mut allocator, 10);
        assert_eq!(hero.weapons.ammo[RYNO as usize], 10);
        assert!(hero.fx.parts.is_empty()); assert!(hero.fx.item_voices.is_empty());
        assert_eq!(table.available_dynamic_slots(10), 18);
        assert_eq!(table.available_dynamic_slots(12), 35);
        allocator.fail_at = None;
        shotgun_tick(&mut hero, &mut table, &mut allocator, 11);
        assert_eq!(hero.weapons.ammo[RYNO as usize], 10);
        shotgun_tick(&mut hero, &mut table, &mut allocator, 12);
        assert_eq!(hero.weapons.ammo[RYNO as usize], 9);
        assert_eq!(table.available_dynamic_slots(12), 0);
    }

    #[test]
    fn grid_has_35_unique_origins_even_with_both_spreads_zero() {
        let (f,r,u)=shotgun_basis([1.,0.,0.],[0.,0.,1.]);
        let mut origins=Vec::new();
        for row in 0..5 {for column in 0..7 {
            let cell=shotgun_cell(column,row);
            let origin=add3(scale3(r,cell[0]*0.6),scale3(u,cell[1]*0.4));
            assert!(!origins.contains(&origin));origins.push(origin);
            assert_eq!(shotgun_direction(f,r,u,cell,0.,0.),f);
        }}
        assert_eq!(origins.len(),35);assert_eq!(shotgun_cell(3,2),[0.,0.]);
    }
    #[test]
    fn independent_axes_scale_outward_and_preserve_the_centre() {
        let (f,r,u)=shotgun_basis([1.,0.,0.],[0.,0.,1.]);
        let h=30f32.to_radians();let v=3f32.to_radians();
        assert_eq!(shotgun_direction(f,r,u,shotgun_cell(3,2),h,v),f);
        for row in 0..5 {for column in 0..7 {
            let cell=shotgun_cell(column,row);let d=shotgun_direction(f,r,u,cell,h,v);
            let x=guns::dot3(d,f);let y=guns::dot3(d,r);let z=guns::dot3(d,u);
            assert!((y.atan2(x)-h*cell[0]).abs()<0.00001);
            assert!((z.atan2(x)-v*cell[1]).abs()<0.00001);
            assert!(d[0]>0.);
            if column==3 {assert_eq!(y,0.);}
            if row==2 {assert_eq!(z,0.);}
        }}
    }
    #[test]
    fn grid_stays_perpendicular_when_aiming_steeply_up_or_down() {
        for angle in [-85f32,0.,85.] {
            let a=angle.to_radians();let (f,r,u)=shotgun_basis([a.cos(),0.,a.sin()],[0.,0.,1.]);
            assert!(guns::dot3(f,r).abs()<0.00001);assert!(guns::dot3(f,u).abs()<0.00001);
            assert!(guns::dot3(r,u).abs()<0.00001);
            assert_eq!(shotgun_direction(f,r,u,[0.,0.],1.,1.),f);
        }
    }
}
