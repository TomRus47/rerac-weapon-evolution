//! Target acquisition `0x274b78(range, moby, &out)`: whom a creature goes for this tick.
//!
//! **Level copies** (G-ENM-009, 2026-09-29): level03 `0x24e830`, level04 `0x252e18`, level07 `0x2886d8`, level12
//! `0x278ba8` (cluster d001715051e4) are this function: their decompiled code is identical to `0x274b78` but for the
//! two decoy lists' addresses (`0x1b0cb0` / `0x1dd580` on level 01, `0x1b0930` / `0x1d1680` on level 03: `%lo`
//! operands the cluster hash does not mask), so [`acquire`] serves them.
//!
//! * Ratchet (`0x1413d0`) by default, **no target** while his movement group (`0x1413dc`) is 0x18 or his state
//!   (`0x1413d4`) is 0x72 (the cutscene hold): kind 2 then, and every field of the record is zero.
//! * A decoy nearer than `range` (the **xy** distance, `fun_001f9b80` = `VecDistance2` 0x221398; the port read it as
//!   3-D until 2026-09-28, hero_gameplay.md §18) wins over him (kind 1): the mobys of list `0x1b0cb0`
//!   of class 0xcb / 0x76c in state 3, then the 20 slots of `0x1dd580` of class 0x10e with `+0xbc ≠ 0`; each closer
//!   one replaces the previous and shrinks the range. The port has no writer of those two gadget lists, so it scans
//!   the moby table in index order for the same classes and states (the same candidates; only the order of equally
//!   distant decoys could differ). None exist on Novalis.
//! * Weapon Evolution may replace the range for Glove decoys with item 25's
//!   per-grade `lure_range`. Gold chickens keep the caller's original range.
//! * The record: +0x00 the target's position (moby +0x10), +0x10 its Euler (+0x40), +0x20 the aim point and +0x30
//!   the body point: for Ratchet `0x13f410` / `0x13f420` (the hero block's shadow and body points), for a decoy its
//!   position 0.6 up (both); +0x40 the moby, +0x44 the kind.

use super::V;
use crate::moby_runtime::MobyId;
use crate::moby_update::services::World;

/// The target record (`out` of 0x274b78).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Target {
    pub pos: V,
    pub rot: V,
    pub aim: V,
    pub body: V,
    /// +0x40 (None: no target; every vector is then zero).
    pub moby: Option<MobyId>,
    /// +0x44: 0 Ratchet, 1 a decoy, 2 none (Ratchet in state 0x72 / group 0x18).
    pub kind: u32,
}

/// Decoy classes of list `0x1b0cb0` (need state 3) and of `0x1dd580` (need `+0xbc ≠ 0`).
pub const DECOY_STATE3: [i16; 2] = [0xcb, 0x76c];
pub const DECOY_CMD: i16 = 0x10e;

/// `0x274b78(range, id)`.
pub fn acquire(w: &World, id: MobyId, range: f32) -> Target { acquire_in(w, id, range, None) }

/// `0x274df8(range, id, &out, cuboids, 0, points, count)` with a polygon region (spline `region`, its points as a 2-D
/// polygon, [`super::region::point_in_polygon`]): as [`acquire`], but Ratchet (his feet, 0x13f3d0) and every decoy
/// must be inside the region; Ratchet outside gives kind 2 and no target.
pub fn acquire_in(w: &World, id: MobyId, range: f32, region: Option<usize>) -> Target { acquire_with(w, id, range, &[], region) }

/// `0x274df8(range, id, &out, cuboids, n, points, count)` in full: with a cuboid list (`n ≥ 1`) the region is not
/// tested; instead Ratchet and every decoy are refused when inside one of the cuboids `cuboids[0 .. n − 1]`
/// (`PointInCuboid` 0x274820: the loop stops at `n − 1`, so the last cuboid of the list is never tested and a
/// one-cuboid list excludes nothing). Without a list (`n < 1`), [`acquire_in`]'s region test. First consumer of the
/// list form: the pod launchers 1885 (`units::pod_launcher`, a list of one).
pub fn acquire_with(w: &World, id: MobyId, range: f32, cuboids: &[i32], region: Option<usize>) -> Target {
    let glove_range = crate::weapon_evo::extra(25, "lure_range", range);
    acquire_ranges(w, id, range, glove_range, cuboids, region)
}

/// Separate eligibility radii, then pick the nearest eligible lure across both
/// lists. A Glove upgrade must not extend the native Gold chicken detection range.
fn acquire_ranges(w: &World, id: MobyId, range: f32, glove_range: f32, cuboids: &[i32], region: Option<usize>) -> Target {
    let me = super::pos(w, id);
    let mut nearest = f32::INFINITY;
    let (mut best, mut kind) = (w.hero_moby, 0u32);
    let tested = &cuboids[..cuboids.len().saturating_sub(1)];
    let inside = |p: V| {
        if cuboids.is_empty() { return region.is_none_or(|r| super::region::point_in_polygon(w, r, p)); }
        !tested.iter().any(|&c| crate::moby_update::triggers::point_in_cuboid(&w.svc.volumes, [p[0], p[1], p[2]], c))
    };
    if w.hero.group == 0x18 || w.hero.state == 0x72 || !inside(w.hero.pos.map(|x| f32::from_bits(x.0))) {
        best = None;
        kind = 2;
    }
    let mut consider = |cand: MobyId, limit: f32, best: &mut Option<MobyId>, kind: &mut u32| {
        let d = super::dist2(me, w.m(cand).position);
        if d < limit && d < nearest && inside(w.m(cand).position) {
            nearest = d;
            *best = Some(cand);
            *kind = 1;
        }
    };
    for (i, m) in w.table.mobys.iter().enumerate() {
        if m.state == crate::moby_runtime::state::END { break; }
        if DECOY_STATE3.contains(&m.o_class) && m.state == 3 { consider(i, glove_range, &mut best, &mut kind); }
    }
    for (i, m) in w.table.mobys.iter().enumerate() {
        if m.state == crate::moby_runtime::state::END { break; }
        if m.o_class == DECOY_CMD && m.state < 0xfd && m.cmd != 0 { consider(i, range, &mut best, &mut kind); }
    }
    let Some(t) = best else { return Target { kind, ..Target::default() } };
    // A living enemy switches from Ratchet to a Glove decoy.
    // The target API has &World, so XP is posted to a queue and processed
    // on the gameplay thread after the tick (one award per target lifetime).
    if kind == 1 && DECOY_STATE3.contains(&w.m(t).o_class) {
        if let Some(hp) = super::damage::special_health(w, id) {
            let frac = crate::weapon_evo::extra(25, "xp_attract_fraction", 0.25);
            w.svc.bomb_evo.enqueue_special(25, id, hp, frac);
        }
    }
    let m = w.m(t);
    let pos = m.position;
    let rot = m.rotation;
    if Some(t) == w.hero_moby {
        let f = |v: crate::hero::physics::V4| v.map(|x| f32::from_bits(x.0));
        return Target { pos, rot, aim: f(w.hero.shadow_point), body: f(w.hero.body_point), moby: Some(t), kind };
    }
    let mut body = pos;
    body[2] += 0.6;
    Target { pos, rot, aim: body, body, moby: Some(t), kind }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_update::classes::bomb_water::tests::Bench;

    fn bench() -> (Bench, MobyId) {
        let mut b = Bench::new(crate::hero::testkit::floor(0.0, 0, 12, 0, 12),
            &[577, 0xcb, 0x76c, 0x10e], [40.0, 40.0, 0.0]);
        let enemy = b.create(577);
        b.table.mobys[enemy].position = [0.0, 0.0, 0.0, 1.0];
        b.table.mobys[enemy].state = 0;
        (b, enemy)
    }

    fn lure(b: &mut Bench, class: i16, distance: f32) -> MobyId {
        let id = b.create(class);
        let m = &mut b.table.mobys[id];
        m.position = [distance, 0.0, 0.0, 1.0];
        m.state = if class == DECOY_CMD { 1 } else { 3 };
        m.cmd = if class == DECOY_CMD { 1 } else { 0 };
        id
    }

    fn pick(b: &mut Bench, enemy: MobyId, native: f32, glove: f32, cuboids: &[i32], region: Option<usize>) -> Target {
        let mut w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, b.counter);
        w.hero_moby = Some(0);
        acquire_ranges(&w, enemy, native, glove, cuboids, region)
    }

    #[test]
    fn glove_radius_can_extend_or_reduce_native_detection_and_zero_disables_it() {
        for class in DECOY_STATE3 {
            let (mut b, enemy) = bench();
            let decoy = lure(&mut b, class, 20.0);
            assert_eq!(pick(&mut b, enemy, 5.0, 30.0, &[], None).moby, Some(decoy));
            assert_eq!(pick(&mut b, enemy, 100.0, 10.0, &[], None).moby, Some(0));
            assert_eq!(pick(&mut b, enemy, 100.0, 0.0, &[], None).moby, Some(0));
            assert_eq!(pick(&mut b, enemy, 100.0, 20.0, &[], None).moby, Some(0), "native strict boundary");
            b.table.mobys[decoy].position = [4.0, 0.0, 500.0, 1.0];
            assert_eq!(pick(&mut b, enemy, 5.0, 5.0, &[], None).moby, Some(decoy), "distance stays horizontal");
        }
    }

    #[test]
    fn gold_chickens_keep_native_radius_and_nearest_eligible_lure_wins() {
        let (mut b, enemy) = bench();
        let decoy = lure(&mut b, 0xcb, 20.0);
        let chicken = lure(&mut b, DECOY_CMD, 7.0);
        assert_eq!(pick(&mut b, enemy, 5.0, 30.0, &[], None).moby, Some(decoy), "chicken outside its native radius");
        b.table.mobys[chicken].position[0] = 4.0;
        assert_eq!(pick(&mut b, enemy, 5.0, 30.0, &[], None).moby, Some(chicken));
        assert_eq!(pick(&mut b, enemy, 5.0, 0.0, &[], None).moby, Some(chicken));
        b.table.mobys[chicken].cmd = 0;
        assert_eq!(pick(&mut b, enemy, 5.0, 0.0, &[], None).moby, Some(0));
    }

    #[test]
    fn native_ranges_keep_list_order_ties_and_only_standing_decoys() {
        let (mut b, enemy) = bench();
        let first = lure(&mut b, 0xcb, 8.0);
        let second = lure(&mut b, 0x76c, 8.0);
        let chicken = lure(&mut b, DECOY_CMD, 8.0);
        assert_eq!(pick(&mut b, enemy, 10.0, 10.0, &[], None).moby, Some(first));
        b.table.mobys[first].state = 2;
        assert_eq!(pick(&mut b, enemy, 10.0, 10.0, &[], None).moby, Some(second));
        b.table.mobys[second].state = 5;
        assert_eq!(pick(&mut b, enemy, 10.0, 10.0, &[], None).moby, Some(chicken));
        b.table.mobys[chicken].cmd = 0;
        assert_eq!(pick(&mut b, enemy, 10.0, 10.0, &[], None).moby, Some(0));
    }

    #[test]
    fn expanded_radius_respects_exclusion_volumes_and_region_tests() {
        use rc_formats::volumes::{Shape, Volumes};
        let (mut b, enemy) = bench();
        let decoy = lure(&mut b, 0xcb, 8.0);
        let shape = Shape {
            matrix: [[2.0, 0.0, 0.0, 0.0], [0.0, 2.0, 0.0, 0.0], [0.0, 0.0, 2.0, 0.0], [8.0, 0.0, 0.0, 1.0]],
            inverse: [[0.5, 0.0, 0.0, 0.0], [0.0, 0.5, 0.0, 0.0], [0.0, 0.0, 0.5, 0.0]],
            ..Default::default()
        };
        b.svc.volumes = std::sync::Arc::new(Volumes { cuboids: vec![shape], ..Default::default() });
        assert_eq!(pick(&mut b, enemy, 2.0, 100.0, &[], None).moby, Some(decoy));
        assert_eq!(pick(&mut b, enemy, 2.0, 100.0, &[0, -1], None).moby, Some(0));
        let outside = pick(&mut b, enemy, 2.0, 100.0, &[], Some(999));
        assert_eq!((outside.moby, outside.kind), (None, 2));
    }
}
