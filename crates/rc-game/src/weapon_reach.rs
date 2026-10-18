//! Reach shared by automatic sights and the actual projectile/beam simulation.
use crate::moby_runtime::Moby;

const DT: f32 = 1.0 / 60.0;

/// Blaster collisions run for life-1 steps: the expiry step is discarded.
pub(crate) fn blaster_flight(gold: i32, range: f32, speed: f32) -> (i32, f32) {
    let life = ((crate::hero::physics::ticks(35) + gold * 10) as f32 * range)
        .round().clamp(0.0, i16::MAX as f32) as i32;
    (life, 40.0 * DT * speed.max(0.0))
}

pub(crate) fn blaster(gold: i32) -> (i32, f32) {
    blaster_flight(gold, crate::weapon_evo::extra(15, "projectile_range_multiplier", 1.0),
        crate::weapon_evo::extra(15, "projectile_speed_multiplier", 1.0))
}

pub(crate) fn blaster_range(gold: i32) -> f32 {
    let (life, step) = blaster(gold);
    (life - 1).max(0) as f32 * step
}

pub(crate) fn contains(from: [f32; 3], point: [f32; 3], range: f32) -> bool {
    range > 0.0 && (0..3).map(|k| (point[k] - from[k]).powi(2)).sum::<f32>() <= range * range
}

/// The missile update accelerates before moving, then expires after collision.
fn accelerated_distance(ticks: i32, initial: f32, top: f32) -> f32 {
    let (mut speed, mut distance) = (initial, 0.0);
    for _ in 0..ticks {
        speed += (top - speed) * 0.1;
        distance += speed;
    }
    distance
}

pub(crate) fn missile_envelope(item: i32, target_speed: u8, speed: f32, range: f32) -> (f32, f32) {
    let speed = speed.clamp(0.1, 10.0);
    let range = range.clamp(0.1, 10.0);
    if item == 11 {
        let top = if target_speed == 0 { 20.0 } else { target_speed as f32 };
        (accelerated_distance(crate::hero::physics::ticks(300), 20.0 * DT / 3.0 * speed, top * DT * speed),
            (if top > 20.0 { 140.0 } else { 70.0 }) * range)
    } else {
        (accelerated_distance(crate::hero::physics::ticks(250), 0.0, 20.0 * DT * speed), 200.0 * range)
    }
}

/// Radial despawn is measured from Ratchet; lifetime is measured from the muzzle.
/// Keep both tests: a slow missile can run out of time before its radial limit.
pub(crate) fn missile_allows(item: i32, target: &Moby, from: [f32; 3], hero: [f32; 3], point: [f32; 3]) -> bool {
    let target_speed = if item == 11 {
        super::record(target).map_or(0, |r| target.pvars[r + 0xc])
    } else { 0 };
    let (travel, radial) = missile_envelope(item, target_speed,
        crate::weapon_evo::extra(item, "missile_speed_multiplier", 1.0),
        crate::weapon_evo::extra(item, "missile_range_multiplier", 1.0));
    contains(from, point, travel) && (point[0] - hero[0]).hypot(point[1] - hero[1]) <= radial
}

pub(crate) fn beam_range(item: i32, gold: bool) -> f32 {
    if item == 19 {
        (if gold { 18.0 } else { 12.0 }) * crate::weapon_evo::extra(19, "target_range_multiplier", 1.0)
    } else {
        8.0 * crate::weapon_evo::extra(21, "beam_range_multiplier", 1.0)
    }
}

/// Centre-snapping a ground reticle must not move it beyond the ballistic arc.
/// Test the centre's height, rather than using the weapon's flat-ground range.
pub(crate) fn arc_can_reach(s: &super::ArcStart, point: [f32; 3]) -> bool {
    let mut vel = s.vel;
    if s.gold { vel = [vel[0] * 1.5, vel[1] * 1.5, vel[2] / 1.5]; }
    let delta = [point[0] - s.from[0], point[1] - s.from[1], point[2] - s.from[2]];
    let speed = vel[0].hypot(vel[1]);
    if speed <= 0.0 { return delta[0].hypot(delta[1]) <= 0.01; }
    let time = delta[0].hypot(delta[1]) / speed;
    if time > s.budget.max(0) as f32 * 10.0 { return false; }
    let height = s.from[2] + vel[2] * time - s.gravity * time * (time - 1.0) * 0.5;
    // A descending arc can hit the ground below the aim point, not above its envelope.
    point[2] <= height + 0.05
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blaster_sight_accounts_for_life_speed_gold_and_expiry() {
        for (gold, range, speed) in [(0, 0.75, 0.75), (0, 0.9, 0.9), (0, 1.0, 1.0),
                                    (0, 1.2, 1.2), (1, 1.5, 1.4)] {
            let (life, step) = blaster_flight(gold, range, speed);
            let distance = (life - 1) as f32 * step;
            assert!(contains([0.0; 3], [distance - 0.001, 0.0, 0.0], distance));
            assert!(!contains([0.0; 3], [distance + 0.001, 0.0, 0.0], distance));
            assert!(!contains([0.0; 3], [distance, 0.0, 1.0], distance));
        }
        assert!(!contains([0.0; 3], [0.0; 3], 0.0));
        assert_eq!(blaster_flight(0, 1.0, 0.0).1, 0.0);
    }

    #[test]
    fn missiles_respect_both_lifetime_and_radial_distance() {
        let (travel, radial) = missile_envelope(11, 0, 1.0, 1.0);
        assert!((97.9..98.1).contains(&travel)); assert_eq!(radial, 70.0);
        let (slow, far) = missile_envelope(11, 0, 0.1, 3.0);
        assert!(slow < 10.0); assert_eq!(far, 210.0);
        let (ryno, radius) = missile_envelope(23, 0, 1.0, 1.0);
        assert!((80.2..80.4).contains(&ryno)); assert_eq!(radius, 200.0);
        assert_eq!(missile_envelope(11, 40, 1.0, 1.0).1, 140.0);
    }

    #[test]
    fn decoy_snap_cannot_reach_an_object_beyond_its_landing_arc() {
        let s = super::super::ArcStart { from: [0.0, 0.0, 1.0], vel: [0.1, 0.0, 0.1],
            budget: 300, gravity: 0.01, first: [0.0; 3], first_ignore: None,
            ignore: None, own: [None; 2], gold: false,
            prims: super::super::Prims::Snap { max_rise: None }, first_test: false };
        assert!(arc_can_reach(&s, [1.0, 0.0, 0.0]));
        assert!(!arc_can_reach(&s, [10.0, 0.0, 0.0]));
        assert!(arc_can_reach(&s, [1.0, 0.0, 1.0]));
        assert!(!arc_can_reach(&s, [1.0, 0.0, 3.0]));
    }
}
