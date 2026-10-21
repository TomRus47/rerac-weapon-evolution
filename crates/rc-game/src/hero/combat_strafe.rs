//! Optional, native-physics third-person combat movement, held on L2.
//! Body yaw follows the camera; locomotion and collision remain in their native states.
use super::{Hero, physics};
use crate::ps2v::Pf;
use crate::pad::{button, PadState};
use std::sync::OnceLock;

pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("RC_COMBAT_STRAFE").is_ok_and(|s| s == "1"))
}

fn eligible(hero: &Hero) -> bool {
    // delete_hand clears the outgoing id for a few ticks before create_hand.
    // Keep L2 movement while the pending item is another weapon, including wrench.
    let weapon = crate::weapon_evo::WEAPON_IDS.contains(&hero.items.slot.id)
        || (hero.items.slot.id == 0 && hero.items.slot.state == 0
            && crate::weapon_evo::WEAPON_IDS.contains(&hero.items.target));
    hero.mode == 0 && hero.health > 0 && hero.frozen == 0 && hero.f13ff == 0
        && hero.gravity_mode == 0 && hero.f658 == 0
        && weapon
        && matches!(hero.state, 0 | 1 | 2 | 3 | 4 | 6 | 7 | 8 | 9 | 0xb | 0xe | 0x13 | 0x14 | 0x15 | 0x1e | 0x23 | 0x2f | 0x73)
}

pub fn active(hero: &Hero) -> bool { hero.combat_strafe && eligible(hero) }

/// Locomotion keeps its own world direction until the body has caught up.
pub fn separate_motion(hero: &Hero) -> bool {
    if hero.state==0x13 && !crate::weapon_evo::extra_flag(8,"moving_attacks_enabled",true) {return false;}
    active(hero) || (hero.combat_release && eligible(hero)
        && matches!(hero.state, 0 | 2 | 3 | 4 | 6 | 7 | 8 | 9 | 0xb | 0xe | 0x13 | 0x23 | 0x2f | 0x73))
}

pub fn configure(hero: &mut Hero, pad: &PadState, yaw: Pf, enabled: bool) {
    let on = enabled && eligible(hero) && pad.held & button::L2 != 0 && pad.held & button::L1 == 0;
    if on && !hero.combat_strafe {
        hero.combat_move_yaw = if physics::len2(hero.eff) > Pf::f(0.001) {
            crate::pad::fast_arctan(hero.eff[0], hero.eff[1])
        } else { yaw };
    }
    if hero.combat_strafe && !on && eligible(hero) && pad.held & button::L1 == 0 {
        // Use resolved displacement, not the facing or requested speed: a wall must
        // not inject momentum and backwards movement must remain backwards.
        let speed = physics::len2(hero.eff);
        hero.combat_release = speed > Pf::f(0.001);
        hero.speed = speed;
        hero.fwd_speed = speed;
        if hero.combat_release {
            hero.combat_move_yaw = crate::pad::fast_arctan(hero.eff[0], hero.eff[1]);
            hero.target_yaw = hero.combat_move_yaw;
            hero.yaw_vel = Pf::ZERO;
            hero.yaw_residual = Pf::ZERO;
            hero.sharp_turn = 0;
        }
    } else if on || !eligible(hero) || pad.held & button::L1 != 0 {
        hero.combat_release = false;
    } else if hero.combat_release {
        let error = physics::fast_subtract_rotations(hero.combat_move_yaw, hero.rot[2]).abs();
        if error < Pf::f(0.01) || physics::len2(hero.eff) < Pf::f(0.0001) {
            // The final angular remainder is less than one degree.
            if error < Pf::f(0.01) { hero.rot[2] = hero.combat_move_yaw; }
            hero.combat_release = false;
        }
    }
    hero.combat_strafe = on;
    hero.combat_yaw = yaw;
}

pub fn face_camera(hero: &mut Hero) {
    if !active(hero) { return; }
    hero.rot[2] = hero.combat_yaw;
    hero.yaw_vel = Pf::ZERO;
    hero.yaw_residual = Pf::ZERO;
    hero.sharp_turn = 0;
    hero.turn_to_target = 0;
}

/// A bounded, eased turn: no angular snap, overshoot or dependence on run speed.
/// Runs once per physics tick, after the native state has selected its target.
pub fn release_facing(hero: &mut Hero) {
    if !hero.combat_release || !separate_motion(hero) { return; }
    let error = physics::fast_subtract_rotations(hero.combat_move_yaw, hero.rot[2]);
    let step = (error.to_f32() * 0.38).clamp(-0.3926991, 0.3926991);
    hero.rot[2] = physics::fast_add_rotations(hero.rot[2], Pf::f(step));
    hero.yaw_vel = Pf::ZERO;
    hero.yaw_residual = Pf::ZERO;
    hero.sharp_turn = 0;
    hero.turn_to_target = 0;
}

/// Keep the physical pad intact for menus, gadgets and L1. While locked, consume
/// L2 and turn R2 into an alternative fire button instead of native crouch.
#[cfg(test)]
pub fn gameplay_pad(pad: &PadState, active: bool) -> PadState {
    gameplay_pad_for_weapon(pad, active, 15)
}

pub fn gameplay_pad_for_weapon(pad: &PadState, active: bool, item: i32) -> PadState {
    let mut out = pad.clone();
    if active {
        let fire = if item == 8 { button::SQUARE } else { button::CIRCLE };
        let consumed = button::L2 | button::R2 | fire;
        let alias = |mask: u32| (mask & !(button::L2 | button::R2))
            | if mask & button::R2 != 0 { fire } else { 0 };
        out.held = alias(pad.held);
        out.prev_held = alias(pad.prev_held);
        out.held_store = alias(pad.held_store);
        // Treat Circle/R2 as one logical button. Releasing either while the
        // other is held must not stop a beam or start another grenade throw.
        out.pressed = (pad.pressed & !consumed) | (out.held & !out.prev_held & fire);
        out.released = (pad.released & !consumed) | (out.prev_held & !out.held & fire);
        out.raw = alias(pad.raw);
        out.prev_raw = alias(pad.prev_raw);
        out.raw_pressed = (pad.raw_pressed & !consumed) | (out.raw & !out.prev_held & fire);
        out.raw_released = (pad.raw_released & !consumed) | (out.prev_raw & !out.raw & fire);
        out.held_unmirrored = alias(pad.held_unmirrored);
        out.pressed_unmirrored = (pad.pressed_unmirrored & !consumed)
            | (out.held_unmirrored & !out.prev_held & fire);
        out.released_unmirrored = (pad.released_unmirrored & !consumed)
            | (out.prev_held & !out.held_unmirrored & fire);
        for mask in &mut out.hist_pressed { *mask &= !(button::L2 | button::R2); }
        if out.hist_count > 0 {
            let newest = (out.hist_index as usize + crate::pad::HISTORY - 1) % crate::pad::HISTORY;
            out.hist_pressed[newest] = (out.hist_pressed[newest] & !fire) | (out.pressed & fire);
        }
        let pressure = if item == 8 { 7 } else { 5 };
        out.pressure[pressure] = out.pressure[pressure].max(out.pressure[11]);
        out.analog_copy[pressure + 4] = out.analog_copy[pressure + 4].max(out.analog_copy[15]);
        out.pressure[10] = Pf::ZERO;
        out.pressure[11] = Pf::ZERO;
        out.analog_copy[14] = Pf::ZERO;
        out.analog_copy[15] = Pf::ZERO;
        out.nothing_held = out.held == 0;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::{anim::RecordingAnim, testkit};
    use crate::pad::PadInput;
    use crate::rng::Rng;
    use crate::moby_runtime::Moby;

    fn armed() -> Hero {
        let mut h = Hero::spawn([40.0, 40.0, 10.0], 0.0);
        h.items.slot.id = 15;
        h
    }
    fn pad(input: PadInput) -> PadState {
        let mut p = PadState::default();
        p.update(Some(&input.bytes()), false);
        p
    }
    #[test]
    fn l2_only_and_safe_states() {
        let l2 = pad(PadInput::neutral().press(button::L2));
        let mut h = armed();
        configure(&mut h, &l2, Pf::ZERO, true);
        assert!(active(&h));
        configure(&mut h, &pad(PadInput::neutral().press(button::L1 | button::L2)), Pf::ZERO, true);
        assert!(!active(&h));
        for state in [0x16, 0x18, 0x1d, 0x28, 0x32, 0x33, 0x3d, 0x72] {
            h.state = state;
            configure(&mut h, &l2, Pf::ZERO, true);
            assert!(!active(&h), "special state {state:#x}");
        }
        h.state = 0;
        h.items.slot.id = 8;
        configure(&mut h, &l2, Pf::ZERO, true);
        assert!(active(&h), "wrench supports L2");
        h.items.slot.id = 15;
        configure(&mut h, &l2, Pf::ZERO, false);
        assert!(!active(&h));
    }
    #[test]
    fn consumes_l2_without_mutating_fire_jump_or_real_input() {
        let original = pad(PadInput::neutral().press(button::L2 | button::CIRCLE | button::CROSS).stick(1.0, 0.0));
        let filtered = gameplay_pad(&original, true);
        assert_eq!(filtered.held & button::L2, 0);
        assert_eq!(original.held & button::L2, button::L2);
        assert_eq!(filtered.held & (button::CIRCLE | button::CROSS), button::CIRCLE | button::CROSS);
        assert_eq!(filtered.lx, original.lx);
        assert_eq!(filtered.ly, original.ly);
        assert_eq!(gameplay_pad(&original, false).held, original.held);
    }

    #[test]
    fn r2_is_alternative_fire_only_in_combat_and_does_not_crouch() {
        let original = pad(PadInput::neutral().press(button::L2 | button::R2 | button::CROSS));
        for &item in &crate::weapon_evo::WEAPON_IDS {
            let filtered = gameplay_pad_for_weapon(&original, true, item);
            let fire = if item == 8 { button::SQUARE } else { button::CIRCLE };
            assert_eq!(filtered.held & fire, fire, "item {item}");
            assert_eq!(filtered.pressed & fire, fire);
            assert_eq!(filtered.held & (button::L2 | button::R2), 0);
            assert_eq!(filtered.held & button::CROSS, button::CROSS);
            assert_eq!(filtered.pressure[11], Pf::ZERO);
            assert_eq!(filtered.analog_copy[15], Pf::ZERO);
            assert_eq!(format!("{filtered:?}"), format!("{:?}", gameplay_pad_for_weapon(&filtered, true, item)),
                "hero and item updates can both filter their pad");
        }
        let native = gameplay_pad(&original, false);
        assert_eq!(format!("{native:?}"), format!("{original:?}"));
        assert_eq!(original.held & button::R2, button::R2);
    }

    #[test]
    fn circle_and_r2_share_one_press_hold_and_release() {
        let variants = [0, button::CIRCLE, button::R2, button::CIRCLE | button::R2];
        for previous in variants {
            for current in variants {
                let mut physical = PadState::default();
                physical.update(Some(&PadInput::neutral().press(button::L2 | previous).bytes()), false);
                physical.update(Some(&PadInput::neutral().press(button::L2 | current).bytes()), false);
                let logical = gameplay_pad(&physical, true);
                let held = if current != 0 { button::CIRCLE } else { 0 };
                let pressed = if current != 0 && previous == 0 { button::CIRCLE } else { 0 };
                let released = if current == 0 && previous != 0 { button::CIRCLE } else { 0 };
                assert_eq!(logical.held & button::CIRCLE, held);
                assert_eq!(logical.pressed & button::CIRCLE, pressed, "{previous:#x} -> {current:#x}");
                assert_eq!(logical.released & button::CIRCLE, released);
                assert_eq!(logical.raw_pressed & button::CIRCLE, pressed);
                assert_eq!(logical.raw_released & button::CIRCLE, released);
                assert_eq!(logical.pressed_unmirrored & button::CIRCLE, pressed);
                assert_eq!(logical.released_unmirrored & button::CIRCLE, released);
            }
        }
    }
    fn run(input: PadInput, ticks: usize) -> Hero {
        let coll = testkit::floor(10.0, 6, 14, 6, 14);
        let mut h = armed();
        let mut moby = Moby::zeroed();
        let mut pad = PadState::default();
        let mut anim = RecordingAnim::default();
        let mut rng = Rng::new();
        let (rows, yaw) = testkit::cam_x();
        for _ in 0..ticks {
            pad.update(Some(&input.bytes()), false);
            let env = physics::Env { coll: &coll, pad: &pad, cam_yaw: yaw, cam_rows: rows,
                mirror: false, death_z: Pf::f(-50.0), mobys: None, hero_moby: None, water: None, world: None };
            let result = crate::hero::hero_update_with_combat_mode(&mut h, &mut moby, &env,
                &mut anim, &mut rng, &mut crate::hero::NoHeroSounds, true);
            assert_eq!(result, crate::hero::HeroTick::Ran);
        }
        h
    }
    #[test]
    fn sideways_and_backwards_motion_do_not_turn_ratchet() {
        let side = run(PadInput::neutral().press(button::L2).stick(1.0, 0.0), 90);
        assert!((side.rot[2].to_f32()).abs() < 0.001);
        assert!((side.pos[0].to_f32() - 40.0).abs() < 0.6, "sideways x drift {:?}", side.position());
        assert!((side.pos[1].to_f32() - 40.0).abs() > 1.0, "did not move sideways {:?}", side.position());
        let back = run(PadInput::neutral().press(button::L2).stick(0.0, 1.0), 90);
        assert!(back.pos[0].to_f32() < 39.0, "did not move backwards {:?}", back.position());
        assert!(back.rot[2].to_f32().abs() < 0.001);
    }
    #[test]
    fn sideways_jump_keeps_facing_and_horizontal_motion() {
        let h = run(PadInput::neutral().press(button::L2 | button::CROSS).stick(1.0, 0.0), 16);
        assert!(matches!(h.group, 2 | 4 | 5), "jump group {}", h.group);
        assert!(h.pos[2].to_f32() > 10.05, "did not jump {:?}", h.position());
        assert!(h.rot[2].to_f32().abs() < 0.001);
        assert!((h.pos[0].to_f32() - 40.0).abs() < 0.8);
    }
    #[test]
    fn camera_does_not_enter_first_person_or_orbit_from_sideways_motion() {
        use crate::follow_camera::{Camera, CamInput, CameraOptions};
        let coll = testkit::floor(10.0, 4, 18, 4, 18);
        let mut h = armed();
        let p = pad(PadInput::neutral().press(button::L2));
        configure(&mut h, &p, Pf::ZERO, true);
        let mut camera = Camera::new(&CamInput { hero: &h, pad: &p, coll: &coll, mobys: None, hero_moby: None }, CameraOptions::default());
        for _ in 0..120 { camera.update(&CamInput { hero: &h, pad: &p, coll: &coll, mobys: None, hero_moby: None }); }
        let yaw = camera.out.yaw();
        for _ in 0..90 {
            h.pos[1] = h.pos[1] + Pf::f(0.03);
            camera.update(&CamInput { hero: &h, pad: &p, coll: &coll, mobys: None, hero_moby: None });
        }
        assert!(!camera.first_person.active);
        let error = physics::fast_subtract_rotations(camera.out.yaw(), yaw).to_f32().abs();
        assert!(error < 0.08, "camera drifted {error} radians");
    }


    struct Sim {
        h: Hero,
        moby: Moby,
        pad: PadState,
        anim: RecordingAnim,
        rng: Rng,
        coll: rc_formats::collision::Collision,
    }
    impl Sim {
        fn new(item: i32) -> Self {
            let mut h = armed();
            h.items.slot.id = item;
            let mut sim = Self { h, moby: Moby::zeroed(), pad: PadState::default(), anim: RecordingAnim::default(),
                rng: Rng::new(), coll: testkit::floor(10.0, 2, 24, 2, 24) };
            // The native ground probe clamps its lower endpoint to z=0. A floor
            // at z=0 falls in cell z=-1 and is never crossed by that ray.
            // Settle on an interior, positive-Z floor before any movement test.
            sim.step(PadInput::neutral());
            sim.step(PadInput::neutral());
            assert_eq!(sim.h.air_ticks, 0, "fixture is not grounded: {:?}", sim.h.position());
            assert!((sim.h.pos[2].to_f32() - 10.0).abs() < 0.02);
            sim
        }
        fn ground_run(&mut self, input: PadInput, ticks: usize) {
            for _ in 0..ticks {
                self.step(input);
                assert_eq!(self.h.air_ticks, 0, "run lost ground: state {}, position {:?}", self.h.state, self.h.position());
            }
        }
        fn step(&mut self, input: PadInput) {
            self.pad.update(Some(&input.bytes()), false);
            let (rows, yaw) = testkit::cam_x();
            let env = physics::Env { coll: &self.coll, pad: &self.pad, cam_yaw: yaw, cam_rows: rows,
                mirror: false, death_z: Pf::f(-50.0), mobys: None, hero_moby: None, water: None, world: None };
            assert_eq!(crate::hero::hero_update_with_combat_mode(&mut self.h, &mut self.moby, &env,
                &mut self.anim, &mut self.rng, &mut crate::hero::NoHeroSounds, true), crate::hero::HeroTick::Ran);
        }
    }
    #[test]
    fn release_has_no_forward_kick_or_speed_gain_and_turns_promptly() {
        for item in [8, 15] {
            for (sx, sy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.7, 0.7)] {
                let mut sim = Sim::new(item);
                sim.ground_run(PadInput::neutral().press(button::L2).stick(sx, sy), 90);
                let velocity = sim.h.eff;
                let speed = physics::len2(velocity).to_f32();
                assert!(speed > 0.04);
                let direction = crate::pad::fast_arctan(velocity[0], velocity[1]);
                for tick in 0..22 {
                    let old_yaw = sim.h.rot[2];
                    sim.step(PadInput::neutral());
                    let v = sim.h.eff;
                    let current = physics::len2(v).to_f32();
                    assert!(current <= speed + 0.002, "release accelerated: {current} > {speed}");
                    let dot = (v[0] * velocity[0] + v[1] * velocity[1]).to_f32();
                    assert!(dot >= current * speed * 0.98, "release reversed or redirected movement at tick {tick}");
                    let turn = physics::fast_subtract_rotations(sim.h.rot[2], old_yaw).abs().to_f32();
                    assert!(turn <= 0.394, "angular snap {turn}");
                }
                assert!(physics::fast_subtract_rotations(direction, sim.h.rot[2]).abs().to_f32() < 0.04,
                    "body did not catch movement within 22 ticks");
                for _ in 0..70 { sim.step(PadInput::neutral()); }
                assert!(physics::len2(sim.h.eff).to_f32() < 0.003, "residual speed never stopped");
            }
        }
    }
    #[test]
    fn release_with_direction_held_and_reenter_l2_keep_moving_backwards() {
        let mut sim = Sim::new(8);
        sim.ground_run(PadInput::neutral().press(button::L2).stick(0.0, 1.0), 90);
        for _ in 0..18 {
            sim.step(PadInput::neutral().stick(0.0, 1.0));
            assert!(sim.h.eff[0].to_f32() < -0.04);
        }
        sim.step(PadInput::neutral().press(button::L2).stick(0.0, 1.0));
        assert!(active(&sim.h));
        assert!(!sim.h.combat_release);
        assert!(sim.h.eff[0].to_f32() < -0.04);
    }
    #[test]
    fn lateral_jump_and_double_jump_use_native_flips_without_braking() {
        for sx in [-1.0, 1.0] {
            let mut sim = Sim::new(8);
            sim.ground_run(PadInput::neutral().press(button::L2).stick(sx, 0.0), 80);
            sim.step(PadInput::neutral().press(button::L2 | button::CROSS).stick(sx, 0.0));
            assert_eq!(sim.h.group, 4);
            assert!(matches!(sim.anim.v.seq_b, 0x1c | 0x1d), "native side flip not selected");
            for _ in 0..18 { sim.step(PadInput::neutral().press(button::L2).stick(sx, 0.0)); }
            let speed = physics::len2(sim.h.eff).to_f32();
            sim.step(PadInput::neutral().press(button::L2 | button::CROSS).stick(sx, 0.0));
            assert_eq!(sim.h.state, 0xe, "second press should keep native double-jump physics");
            assert!(matches!(sim.anim.v.seq_b, 0x1c | 0x1d));
            for _ in 0..8 {
                sim.step(PadInput::neutral().press(button::L2).stick(sx, 0.0));
                assert!(physics::len2(sim.h.eff).to_f32() >= speed * 0.85, "double jump braked lateral movement");
                assert!(sim.h.eff[0].abs().to_f32() < 0.003, "jump injected forward velocity");
            }
            // Release L2 in mid-air: keep horizontal momentum and finish the same animation.
            let side = sim.h.eff[1].to_f32();
            sim.step(PadInput::neutral());
            assert!(sim.h.eff[1].to_f32() * side > 0.0);
            assert!(matches!(sim.anim.v.seq_b, 0x1c | 0x1d));
        }
    }
    #[test]
    fn wrench_ground_combo_moves_sideways_without_the_native_forward_lunge() {
        let mut sim = Sim::new(8);
        sim.ground_run(PadInput::neutral().press(button::L2).stick(1.0, 0.0), 80);
        let (rows, yaw) = testkit::cam_x();
        let env = physics::Env { coll: &sim.coll, pad: &sim.pad, cam_yaw: yaw, cam_rows: rows,
            mirror: false, death_z: Pf::f(-50.0), mobys: None, hero_moby: None, water: None, world: None };
        sim.h.set_state(&mut crate::hero::states::Ctx { env: &env, anim: &mut sim.anim, rng: &mut sim.rng, voice: None }, 0x13, true);
        for _ in 0..8 {
            sim.step(PadInput::neutral().press(button::L2).stick(1.0, 0.0));
            assert!(active(&sim.h));
            assert!(sim.h.eff[0].abs().to_f32() < 0.003);
            assert!(sim.h.eff[1].abs().to_f32() > 0.04);
            assert!(sim.h.rot[2].abs().to_f32() < 0.003);
        }
    }
    #[test]
    fn no_resolved_motion_on_release_cannot_generate_a_push() {
        let mut h = armed();
        h.state = 2;
        h.group = 1;
        h.combat_strafe = true;
        h.speed = Pf::f(0.095);
        h.eff = physics::V0; // movement blocked by collision last tick
        configure(&mut h, &PadState::default(), Pf::ZERO, true);
        assert_eq!(h.speed, Pf::ZERO);
        assert!(!h.combat_release);
        assert!(!separate_motion(&h));
    }
    #[test]
    fn native_control_and_special_state_cancel_the_handoff() {
        let mut h = armed();
        configure(&mut h, &pad(PadInput::neutral().press(button::L2)), Pf::ZERO, false);
        assert!(!active(&h));
        h.combat_release = true;
        h.state = 0x16; // damage retains its native knockback
        configure(&mut h, &PadState::default(), Pf::ZERO, true);
        assert!(!separate_motion(&h));
        assert!(!h.combat_release);
    }

    #[test]
    fn locked_movement_survives_the_empty_hand_gap_for_every_weapon() {
        for &next in &crate::weapon_evo::WEAPON_IDS {
            for (sx, sy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                let mut sim = Sim::new(15);
                let input = PadInput::neutral().press(button::L2).stick(sx, sy);
                sim.ground_run(input, 30);
                let velocity = sim.h.eff;
                let speed = physics::len2(velocity).to_f32();
                assert!(speed > 0.04);
                sim.h.items.target = next;
                sim.h.items.slot.state = 3;
                sim.step(input);
                // delete_hand -> create_after gate -> create_hand: actual slot fields.
                sim.h.items.slot.id = 0;
                sim.h.items.slot.state = 0;
                sim.h.items.slot.item = None;
                for _ in 0..4 {
                    sim.step(input);
                    assert!(active(&sim.h), "lost lock while drawing item {next}");
                    assert!(!matches!(sim.h.state, 1 | 0x1e));
                    let current = physics::len2(sim.h.eff).to_f32();
                    assert!(current >= speed * 0.9, "weapon draw braked movement, item {next}");
                    let dot = (sim.h.eff[0] * velocity[0] + sim.h.eff[1] * velocity[1]).to_f32();
                    assert!(dot > current * speed * 0.98, "weapon draw redirected movement");
                }
                sim.h.items.slot.id = next;
                sim.h.items.slot.state = 2;
                sim.step(input);
                assert!(active(&sim.h));
                assert!(physics::len2(sim.h.eff).to_f32() >= speed * 0.9);
            }
        }
    }

    #[test]
    fn empty_hand_gap_does_not_lock_gadgets_or_override_l1() {
        let mut h = armed();
        h.items.slot.id = 0; h.items.slot.state = 0;
        let l2 = pad(PadInput::neutral().press(button::L2));
        for target in [0, 0x1a, 0x1f] {
            h.items.target = target;
            configure(&mut h, &l2, Pf::ZERO, true);
            assert!(!active(&h));
        }
        h.items.target = 16;
        configure(&mut h, &l2, Pf::ZERO, true);
        assert!(active(&h));
        configure(&mut h, &pad(PadInput::neutral().press(button::L1 | button::L2)), Pf::ZERO, true);
        assert!(!active(&h));
    }

    #[test]
    fn jump_fixture_floor_is_crossed_before_the_probe_endpoint() {
        use crate::collision_query::{coll_line, QueryFlags};
        let floor = testkit::floor(10.0, 2, 24, 2, 24);
        let hit = coll_line(&floor, [40.0, 40.0, 11.0], [40.0, 40.0, 0.0], QueryFlags(2))
            .expect("positive-Z floor must be crossed by the native ground ray");
        assert!((hit.point[2] - 10.0).abs() < 0.001);
        let boundary = testkit::floor(0.0, 2, 24, 2, 24);
        assert!(coll_line(&boundary, [40.0, 40.0, 1.0], [40.0, 40.0, 0.0], QueryFlags(2)).is_none(),
            "a floor at the ray endpoint cannot support this fixture");
    }

}
