//! Independent running keys on the two leg branches. Upper-body keys stay native.
use super::{combat_strafe, physics, Hero};
use rc_formats::moby_anim::{AnimState, JointYaw, MobyAnimClass, PoseLayer};

// Hero animation is updated at 60 Hz: 7 ticks into idle, 8 ticks out.
// Fifteen ticks halve the previous 30-tick crossover exactly.
const TURN_IN_TICKS: u8 = 7;
const TURN_OUT_TICKS: u8 = 8;
const TURN_TICKS: u8 = TURN_IN_TICKS + TURN_OUT_TICKS;

#[derive(Clone, Debug, Default)]
pub struct LegPose {
    checked: bool,
    joints: Vec<u8>,
    roots: Vec<u8>,
    pub yaws: Vec<JointYaw>,
    pub weight: f32,
    yaw: f32,
    phase: f32,
    seq: u8,
    frame_count: u8,
    clock_speed: f32,
    backward: bool,
    native_keys: bool,
    clean_running: bool,
    idle_joints: Vec<u8>,
    idle_weight: f32,
    turn_ticks: u8,
    turn_from_yaw: f32,
    turn_from_backward: bool,
}

fn wrap(a: f32) -> f32 {
    (a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

/// Relative facing is limited to the front semicircle. Rearward displacement
/// uses the diametrically opposite facing and reverses only the leg clock.
fn reverse_sector(angle: f32) -> bool {
    let angle = angle.rem_euclid(360.0);
    angle >= 200.0 - 0.0001 && angle < 340.0 - 0.0001
}

fn direction(relative: f32) -> (f32, bool) {
    // User coordinates: right=0, forward=90, left=180, backwards=270.
    // A 20-degree margin below each lateral direction still uses normal gait.
    let angle = (relative.to_degrees() + 90.0).rem_euclid(360.0);
    let backward = reverse_sector(angle);
    let yaw = if backward { wrap(relative + std::f32::consts::PI) } else { wrap(relative) };
    (yaw.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2), backward)
}

/// The switching thresholds are stick angles, even with a pitched camera.
/// With no direction held (or during release), use the real residual movement.
fn movement_direction(hero: &Hero) -> (f32, bool) {
    let heading = hero.eff[1].to_f32().atan2(hero.eff[0].to_f32());
    let relative = wrap(heading - hero.rot[2].to_f32());
    let (mut yaw, mut back) = direction(relative);
    if combat_strafe::active(hero) && crate::pad::len2(hero.stick[0], hero.stick[1]).to_f32() > 0.1 {
        let angle = (-hero.stick[1].to_f32()).atan2(hero.stick[0].to_f32()).to_degrees();
        back = reverse_sector(angle);
        yaw = if back { wrap(relative + std::f32::consts::PI) } else { relative };
        yaw = yaw.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
    }
    (yaw, back)
}

fn under(class: &MobyAnimClass, mut joint: usize, root: usize) -> bool {
    for _ in 0..class.joint_count {
        if joint == root { return true; }
        match class.parent(joint) {
            Some(p) if p < class.joint_count && p != joint => joint = p,
            _ => return false,
        }
    }
    false
}

/// Resolve branches from the loaded model's actual foot chains, never numeric
/// bone guesses. A malformed chain that includes an arm/head is rejected.
fn rig(class: &MobyAnimClass, chains: &[Vec<u8>]) -> Option<(Vec<u8>, Vec<u8>)> {
    let (left, right) = (chains.get(22)?, chains.get(23)?);
    let common = left.iter().zip(right).take_while(|(a, b)| a == b).count();
    if common == 0 { return None; }
    let roots = vec![*left.get(common)?, *right.get(common)?];
    if roots[0] == roots[1] { return None; }
    for &root in &roots {
        if root as usize >= class.joint_count { return None; }
        for list in [0, 1, 4] { // hands and head must remain on the native pose
            let tip = *chains.get(list)?.last()? as usize;
            if tip >= class.joint_count || under(class, tip, root as usize) { return None; }
        }
    }
    if !under(class, *left.last()? as usize, roots[0] as usize)
        || !under(class, *right.last()? as usize, roots[1] as usize) { return None; }
    let joints: Vec<u8> = (0..class.joint_count.min(256))
        .filter(|&j| roots.iter().any(|&r| under(class, j, r as usize)))
        .map(|j| j as u8).collect();
    (!joints.is_empty()).then_some((joints, roots))
}

/// Idle may settle the pelvis/body during the quarter-second direction change,
/// while the arm/face branches keep their weapon and look pose layers.
fn idle_mask(class: &MobyAnimClass, hero: &Hero) -> Vec<u8> {
    let foot = hero.fx.joints.hero.get(22);
    let mut protected = Vec::new();
    for (list, chain) in [(12usize, 0usize), (13, 1), (24, 4)] {
        let direct = hero.joint_targets.get(list).copied()
            .filter(|&j| j != 0xff && (j as usize) < class.joint_count && class.parent(j as usize).is_some());
        let fallback = hero.fx.joints.hero.get(chain).and_then(|hand| {
            let foot = foot?;
            let common = hand.iter().zip(foot).take_while(|(a, b)| a == b).count();
            hand.get(common).copied()
        });
        if let Some(joint) = direct.or(fallback) { protected.push(joint); }
    }
    (0..class.joint_count.min(256))
        .filter(|&j| !protected.iter().any(|&p| under(class, j, p as usize)))
        .map(|j| j as u8).collect()
}

/// Integrate a signed animation clock using the original per-key rates. Reverse
/// traversal uses the preceding interval's rate and wraps continuously at zero.
fn advance_phase(class: &MobyAnimClass, seq: u8, phase: &mut f32, speed: f32) {
    let Some(sequence) = class.sequence(seq) else { return; };
    let count = sequence.frames.len().min(sequence.header.frame_count as usize);
    if count < 2 || !speed.is_finite() || speed.abs() < 0.00001 { return; }
    let count = count as f32;
    *phase = phase.rem_euclid(count);
    let mut time: f32 = 1.0;
    for _ in 0..128 {
        if time <= 0.000001 { break; }
        let frac = phase.fract();
        let reverse = speed < 0.0;
        let (index, distance) = if reverse {
            ((phase.ceil() - 1.0).rem_euclid(count) as usize, if frac > 0.000001 { frac } else { 1.0 })
        } else {
            (phase.floor() as usize, 1.0 - frac)
        };
        let rate = if sequence.header.rate_override != 0.0 { sequence.header.rate_override }
            else { sequence.frames[index].header.rate };
        if !rate.is_finite() || rate <= 0.0 { break; }
        let step = speed.abs() * rate;
        let duration = distance / step;
        let used = time.min(duration);
        *phase = (*phase + if reverse { -step * used } else { step * used }).rem_euclid(count);
        time -= used;
    }
}

/// Limit pose cleanup to grounded running while L2 is actually held. Walking,
/// release inertia, weapon aiming, and other movement states keep native rules.
pub(super) fn locked_run(hero: &Hero) -> bool {
    combat_strafe::active(hero) && hero.state == 2 && hero.substate == 1 && hero.air_ticks == 0
}

/// Remove only lingering turn-lean rotations. Keep scales, translations, the
/// weapon-arm pitch and slope adjustments, including replacement-foot scaling.
pub fn clean_run_modifiers(hero: &mut Hero) {
    if !locked_run(hero) { return; }
    use super::idle::joint;
    let mut records = vec![joint::REC0, joint::NECK, joint::REC2];
    records.extend(joint::SECONDARY..joint::SECONDARY + 4);
    let flat = hero.ground_normal[0].abs().to_f32() < 0.001
        && hero.ground_normal[1].abs().to_f32() < 0.001;
    if flat { records.extend([joint::LEG_L, joint::LEG_R, joint::FOOT_L, joint::FOOT_R]); }
    for index in records {
        let record = &mut hero.idle.joints[index];
        record.cur = [0.0; 3];
        record.vel = [0.0; 3];
        record.target = [0.0; 3];
    }
}

impl LegPose {
    fn clear_pose(&mut self) {
        self.weight = 0.0;
        self.yaw = 0.0;
        self.phase = 0.0;
        self.clock_speed = 0.0;
        self.backward = false;
        self.native_keys = false;
        self.clean_running = false;
        self.turn_ticks = 0;
        self.idle_weight = 0.0;
        self.turn_from_yaw = 0.0;
        self.turn_from_backward = false;
        self.yaws.clear();
    }

    fn current_run(&mut self, hero: &Hero, class: &MobyAnimClass, base: &AnimState, speed: f32) {
        let seq: u8 = if hero.substate == 1 { 4 } else { 3 }; // native run / walk state
        let count = class.sequence(seq).map_or(0, |s| s.header.frame_count);
        if count == 0 { self.clear_pose(); return; }
        let (yaw, back) = movement_direction(hero);
        let native_phase = if base.seq_a == seq && base.seq_b == seq {
            (base.frame_a as f32 + base.t).rem_euclid(count as f32)
        } else if base.seq_b == seq {
            // A native snapshot is a blend with an earlier animation, not a
            // running key. Take only the current run's target key for the legs.
            (base.frame_b as f32).rem_euclid(count as f32)
        } else { 0.0 };
        let mode_changed = self.clean_running && self.backward != back;
        let entering = !self.clean_running || self.seq != seq;
        if mode_changed && self.turn_ticks == 0 && class.frame(0, 0).is_some() {
            self.turn_ticks = TURN_TICKS;
            self.turn_from_yaw = self.yaw;
            self.turn_from_backward = self.backward;
        }
        let stage = if self.turn_ticks > 0 { Some(TURN_TICKS - self.turn_ticks + 1) } else { None };
        let pace = if seq == 4 { (speed * 14.0).clamp(0.6, 2.2) } else { (speed * 157.0).clamp(0.6, 4.0) };
        self.seq = seq;
        self.frame_count = count;
        self.weight = 1.0;
        let target_yaw = if yaw.abs() < 0.00001 { 0.0 } else { yaw };
        let settling = stage.is_some_and(|tick| tick <= TURN_IN_TICKS);
        let idle_progress = stage.map_or(0.0, |tick| {
            if tick <= TURN_IN_TICKS {
                tick as f32 / TURN_IN_TICKS as f32
            } else {
                (TURN_TICKS - tick) as f32 / TURN_OUT_TICKS as f32
            }
        });
        // Smoothstep has zero slope at both endpoints: no sharp cut into or
        // out of the idle pose. The old gait keeps moving until fully hidden.
        self.idle_weight = idle_progress * idle_progress * (3.0 - 2.0 * idle_progress);
        self.yaw = if settling { self.turn_from_yaw } else { target_yaw } * (1.0 - self.idle_weight);
        let visual_back = if settling { self.turn_from_backward } else { back };
        self.clock_speed = if visual_back { -pace } else { pace };
        self.native_keys = !back && base.seq_a == seq && base.seq_b == seq
            && self.idle_weight == 0.0;
        self.phase = self.phase.rem_euclid(count as f32);
        if visual_back {
            if entering || stage == Some(TURN_IN_TICKS + 1) { self.phase = native_phase; }
            advance_phase(class, seq, &mut self.phase, self.clock_speed);
        } else {
            self.phase = native_phase;
        }
        self.backward = back;
        self.clean_running = true;
        if self.turn_ticks > 0 { self.turn_ticks -= 1; }
        self.yaws.clear();
        if self.yaw != 0.0 {
            self.yaws.extend(self.roots.iter().map(|&joint| JointYaw { joint, radians: self.yaw }));
        }
    }

    pub fn update(&mut self, hero: &Hero, class: &MobyAnimClass, base: &AnimState) {
        if !self.checked && hero.fx.joints.hero.len() >= 24 {
            self.checked = true;
            match rig(class, &hero.fx.joints.hero) {
                Some((joints, roots)) => {
                    self.joints = joints;
                    self.roots = roots;
                    self.idle_joints = idle_mask(class, hero);
                    if self.idle_joints.is_empty() { self.idle_joints.clone_from(&self.joints); }
                    eprintln!("combat legs: resolved roots {:?}, {} leg joints", self.roots, self.joints.len());
                }
                None => eprintln!("combat legs: unsafe or incomplete foot chains; keeping native pose"),
            }
        }
        if self.joints.is_empty() { return; }
        if hero.mode != 0 || hero.health <= 0 || hero.group == 7 || hero.f13f5 != 0 {
            self.clear_pose();
            return;
        }
        let speed = physics::len2(hero.eff).to_f32();
        let run_pose = combat_strafe::separate_motion(hero) && hero.state == 2 && hero.air_ticks == 0;
        if run_pose && speed > 0.001 {
            self.current_run(hero, class, base, speed);
            return;
        }
        if self.clean_running {
            // A jump, stop or new action must not receive a fading running pose.
            self.clear_pose();
            return;
        }
        let ground_run = matches!(hero.state, 0 | 2 | 3 | 0x13 | 0x23 | 0x2f | 0x73) && hero.air_ticks == 0;
        let wanted = combat_strafe::separate_motion(hero) && ground_run && speed > 0.001;
        let mut target_yaw = 0.0;
        let mut target_speed = 0.0;
        if wanted {
            let (yaw, back) = movement_direction(hero);
            target_yaw = yaw;
            self.backward = back;
            let seq = if speed > physics::DT.to_f32() * 1.5 { 4 } else { 3 };
            let count = class.sequence(seq).map_or(0, |s| s.header.frame_count);
            if count == 0 { self.weight = 0.0; self.yaws.clear(); return; }
            if self.weight == 0.0 && base.seq_b == seq {
                self.phase = (base.frame_a as f32 + base.t).rem_euclid(count as f32);
            } else if self.seq != seq {
                let old_count = class.sequence(self.seq).map_or(1, |s| s.header.frame_count.max(1));
                self.phase = self.phase / old_count as f32 * count as f32;
            }
            self.seq = seq;
            self.frame_count = count;
            let pace = if seq == 4 { (speed * 14.0).clamp(0.6, 2.2) } else { (speed * 157.0).clamp(0.6, 4.0) };
            target_speed = if back { -pace } else { pace };
        }
        self.weight = (self.weight + if wanted { 1.0 / 6.0 } else { -0.25 }).clamp(0.0, 1.0);
        // No wrap across the back semicircle: the hips always stay within +/-90.
        self.yaw += ((target_yaw - self.yaw) * 0.38).clamp(-0.2617994, 0.2617994);
        self.clock_speed += (target_speed - self.clock_speed).clamp(-0.35, 0.35);
        advance_phase(class, self.seq, &mut self.phase, self.clock_speed);
        self.yaws.clear();
        if self.weight > 0.0 {
            self.yaws.extend(self.roots.iter().map(|&joint| JointYaw { joint, radians: self.yaw * self.weight }));
        }
    }

    pub fn idle_layer(&self) -> Option<PoseLayer<'_>> {
        if self.idle_weight <= 0.0 || self.idle_joints.is_empty() { return None; }
        Some(PoseLayer { joints: &self.idle_joints, seq_a: 0, frame_a: 0,
            seq_b: 0, frame_b: 0, t: 0.0, weight: self.idle_weight, alt: None })
    }

    pub fn layer(&self) -> Option<PoseLayer<'_>> {
        if self.weight <= 0.0 || self.joints.is_empty() || self.native_keys { return None; }
        let frame = self.phase.floor() as u8;
        Some(PoseLayer { joints: &self.joints, seq_a: self.seq, frame_a: frame,
            seq_b: self.seq, frame_b: self.next_frame(frame), t: self.phase.fract(), weight: self.weight, alt: None })
    }

    fn next_frame(&self, frame: u8) -> u8 {
        (frame as u16 + 1).rem_euclid(self.frame_count.max(1) as u16) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_formats::moby_anim::{self, MobyFrame, MobyFrameHeader, MobySequence, MobySequenceHeader, IDENTITY};
    fn class() -> MobyAnimClass {
        let frames = (0..4).map(|k| {
            let mut quats = vec![[0i16, 0, 0, 32767]; 7];
            // The run keys animate a leg and also the arm. Only the leg mask may
            // receive the independent clock's keys.
            let angle = k as f32 * 0.12;
            let q = [0, ((angle * 0.5).sin() * 32767.0) as i16, 0, ((angle * 0.5).cos() * 32767.0) as i16];
            quats[1] = q;
            quats[5] = q;
            let payload = quats.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
            MobyFrame { header: MobyFrameHeader { rate: 1.0, ..Default::default() }, quats, scales: vec![], trans: vec![], payload }
        }).collect();
        let run = MobySequence { header: MobySequenceHeader { frame_count: 4, ..Default::default() }, frames, triggers: vec![] };
        MobyAnimClass { joint_count: 7, skeleton: vec![IDENTITY; 7],
            rest: vec![[0.0; 3], [0.0, 2.0, 0.0], [2.0, 0.0, -3.0], [0.0, -2.0, 0.0], [2.0, 0.0, -3.0], [1.0, 0.0, 1.0], [0.0, 0.0, 3.0]],
            parent_word: vec![0, 0x70000000, 0x70000040, 0x70000000, 0x700000c0, 0x70000000, 0x70000000],
            sequences: vec![Some(run.clone()), None, None, Some(run.clone()), Some(run)] }
    }
    fn chains() -> Vec<Vec<u8>> {
        let mut out = vec![vec![0]; 24];
        out[0] = vec![0, 5];
        out[1] = vec![0, 5];
        out[4] = vec![0, 6];
        out[22] = vec![0, 1, 2];
        out[23] = vec![0, 3, 4];
        out
    }
    fn base(class: &MobyAnimClass) -> AnimState {
        let mut s = AnimState::spawn(class);
        s.seq_a = 4; s.seq_b = 4; s.frame_a = 0; s.frame_b = 1; s.t = 0.0;
        s
    }
    #[test]
    fn all_eight_directions_fold_to_the_front_semicircle() {
        for (degrees, wanted, reverse) in [(0.0f32, 0.0f32, false), (-45.0, -45.0, false),
            (-90.0, -90.0, false), (-135.0, 45.0, true), (180.0, 0.0, true),
            (135.0, -45.0, true), (90.0, 90.0, false), (45.0, 45.0, false)] {
            let (yaw, back) = direction(degrees.to_radians());
            assert!((yaw.to_degrees() - wanted).abs() < 0.001, "{degrees}: {}", yaw.to_degrees());
            assert_eq!(back, reverse, "{degrees}");
        }
    }
    #[test]
    fn reverse_run_clock_wraps_and_obeys_key_rates() {
        let mut c = class();
        let mut phase = 0.0;
        advance_phase(&c, 4, &mut phase, -1.0);
        assert!((phase - 3.0).abs() < 0.0001);
        advance_phase(&c, 4, &mut phase, 1.0);
        assert!(phase.abs() < 0.0001);
        c.sequences[4].as_mut().unwrap().frames[3].header.rate = 0.5;
        advance_phase(&c, 4, &mut phase, -1.0);
        assert!((phase - 3.5).abs() < 0.0001, "previous interval rate, phase {phase}");
        let before = phase;
        advance_phase(&c, 4, &mut phase, 0.0);
        assert_eq!(phase, before);
    }
    #[test]
    fn rig_uses_leg_branches_and_rejects_an_upper_body_root() {
        let c = class();
        let (mask, roots) = rig(&c, &chains()).unwrap();
        assert_eq!(roots, [1, 3]);
        assert_eq!(mask, [1, 2, 3, 4]);
        let mut broken = chains();
        broken[22] = vec![0, 5]; // the hand is never a leg
        assert!(rig(&c, &broken).is_none());
        assert!(rig(&c, &[]).is_none());
    }
    #[test]
    fn leg_keys_and_yaw_leave_the_upper_body_bit_identical() {
        let c = class();
        let s = base(&c);
        let native = moby_anim::evaluate_posed(&c, &s, None, &[], &[]);
        let legs = [1, 2, 3, 4];
        let layer = PoseLayer { joints: &legs, seq_a: 4, frame_a: 2, seq_b: 4, frame_b: 3, t: 0.4, weight: 1.0, alt: None };
        let yaws = [JointYaw { joint: 1, radians: 0.7 }, JointYaw { joint: 3, radians: 0.7 }];
        let posed = moby_anim::evaluate_posed_yaw(&c, &s, None, &[layer], &[], &yaws);
        for j in [0, 5, 6] { assert_eq!(posed[j], native[j], "upper joint {j} changed"); }
        assert_ne!(posed[2], native[2]);
        // A hip rotation preserves its own pivot; children rotate around it.
        assert_eq!(posed[1][3], native[1][3]);
        let without_yaws = moby_anim::evaluate_posed_yaw(&c, &s, None, &[], &[], &[]);
        assert_eq!(native, without_yaws, "native evaluator must remain unchanged");
    }
    #[test]
    fn foot_attachment_chain_matches_the_rendered_leg_pose() {
        let c = class();
        let s = base(&c);
        let legs = [1, 2, 3, 4];
        let layer = PoseLayer { joints: &legs, seq_a: 4, frame_a: 1, seq_b: 4, frame_b: 2, t: 0.2, weight: 0.8, alt: None };
        let yaws = [JointYaw { joint: 1, radians: -0.9 }, JointYaw { joint: 3, radians: -0.9 }];
        let full = moby_anim::evaluate_posed_yaw(&c, &s, None, &[layer], &[], &yaws);
        let chains = chains();
        let selected = [chains[22].as_slice(), chains[23].as_slice(), chains[0].as_slice()];
        let attached = moby_anim::evaluate_chains_posed_yaw(&c, &s, None, &selected, &[layer], &[], &yaws);
        for (i, j) in [2, 4, 5].into_iter().enumerate() {
            for row in 0..4 { for axis in 0..3 {
                assert!((full[j][row][axis] - attached[i][row][axis]).abs() < 0.0001);
            } }
        }
    }
    #[test]
    fn backward_lower_body_clock_does_not_change_movement_and_fades_for_jump() {
        let c = class();
        let s = base(&c);
        let mut h = Hero::spawn([40.0, 40.0, 0.0], 0.0);
        h.set_joint_chains(chains());
        h.items.slot.id = 15; h.state = 2; h.group = 1; h.combat_strafe = true;
        // A walking-speed fixture avoids aliasing a full cycle of this four-key test clip.
        h.eff = [crate::ps2v::Pf::f(-0.014), crate::ps2v::Pf::f(-0.007), crate::ps2v::Pf::ZERO, crate::ps2v::Pf::ZERO];
        let velocity = h.eff;
        let position = h.pos;
        let mut pose = LegPose::default();
        for _ in 0..30 { pose.update(&h, &c, &s); }
        assert!(pose.backward && pose.clock_speed < 0.0);
        assert!(pose.yaw > 0.0 && pose.yaw < std::f32::consts::FRAC_PI_2);
        assert_eq!(h.eff, velocity); assert_eq!(h.pos, position);
        let phase = pose.phase;
        pose.update(&h, &c, &s);
        assert_ne!(pose.phase, phase);
        h.state = 7; h.group = 4; h.air_ticks = 1;
        for _ in 0..5 { pose.update(&h, &c, &s); }
        assert_eq!(pose.weight, 0.0);
        assert!(pose.layer().is_none() && pose.yaws.is_empty());
    }
    fn runner() -> Hero {
        let mut h = Hero::spawn([40.0, 40.0, 10.0], 0.0);
        h.set_joint_chains(chains());
        h.items.slot.id = 15;
        h.state = 2; h.substate = 1; h.group = 1; h.combat_strafe = true;
        h.eff = [crate::ps2v::Pf::f(0.08), crate::ps2v::Pf::ZERO, crate::ps2v::Pf::ZERO, crate::ps2v::Pf::ZERO];
        h
    }
    #[test]
    fn returning_forward_finishes_the_quarter_second_idle_then_restores_native_pose() {
        let c = class(); let mut s = base(&c); s.frame_a = 1; s.frame_b = 2; s.t = 0.375;
        let mut h = runner();
        h.eff[0] = crate::ps2v::Pf::f(-0.08);
        h.eff[1] = crate::ps2v::Pf::f(-0.04);
        let mut pose = LegPose::default();
        for _ in 0..12 { pose.update(&h, &c, &s); }
        assert!(pose.clock_speed < 0.0 && !pose.native_keys);
        h.eff[0] = crate::ps2v::Pf::f(0.08); h.eff[1] = crate::ps2v::Pf::ZERO;
        for _ in 0..TURN_TICKS { pose.update(&h, &c, &s); }
        assert_eq!(pose.yaw, 0.0);
        assert!(pose.yaws.is_empty() && pose.layer().is_none());
        assert!((pose.phase - 1.375).abs() < 0.00001 && pose.clock_speed > 0.0);
        let native = moby_anim::evaluate_posed(&c, &s, None, &[], &[]);
        let layers: Vec<_> = pose.layer().into_iter().collect();
        let clean = moby_anim::evaluate_posed_yaw(&c, &s, None, &layers, &[], &pose.yaws);
        assert_eq!(native, clean, "straight run must be the native pose, without old layers");
    }
    #[test]
    fn a_small_run_direction_change_has_no_old_angle_or_overshoot() {
        let c = class(); let s = base(&c); let mut h = runner(); let mut pose = LegPose::default();
        for angle in [1.3f32, 0.08, -0.06, 0.0, 0.025, 0.0] {
            h.eff[0] = crate::ps2v::Pf::f(angle.cos() * 0.08);
            h.eff[1] = crate::ps2v::Pf::f(angle.sin() * 0.08);
            pose.update(&h, &c, &s);
            assert!((pose.yaw - angle).abs() < 0.00001, "angle {angle}, got {}", pose.yaw);
            assert!(pose.layer().is_none(), "forward direction must use the same native run keys");
            for _ in 0..5 { pose.update(&h, &c, &s); }
            assert!((pose.yaw - angle).abs() < 0.00001, "yaw accumulated over time");
        }
    }
    #[test]
    fn forward_leg_phase_follows_the_body_clock_without_drift() {
        let c = class(); let mut s = base(&c); let mut h = runner(); let mut pose = LegPose::default();
        h.eff[1] = crate::ps2v::Pf::f(0.01);
        for tick in 0..40 {
            s.frame_a = (tick % 4) as u8; s.frame_b = ((tick + 1) % 4) as u8; s.t = 0.27;
            pose.update(&h, &c, &s);
            assert!((pose.phase - (s.frame_a as f32 + s.t)).abs() < 0.00001);
            assert!(pose.native_keys && pose.layer().is_none());
        }
    }
    #[test]
    fn reverse_running_clock_has_a_bounded_visual_idle_crossover() {
        let c = class(); let s = base(&c); let mut h = runner(); let mut pose = LegPose::default();
        pose.update(&h, &c, &s);
        h.eff[0] = crate::ps2v::Pf::f(-0.08);
        for tick in 1..=TURN_TICKS {
            pose.update(&h, &c, &s);
            if tick <= TURN_IN_TICKS {
                assert!(pose.clock_speed > 0.0, "the old gait keeps playing while settling");
            } else {
                assert!(pose.clock_speed < 0.0, "the reversed gait emerges after idle");
            }
        }
        assert!(pose.layer().is_some() && pose.idle_layer().is_none());
        assert_eq!(pose.yaw, 0.0);
        assert!(pose.phase > 0.0, "reverse running must be advancing after the crossover");
        h.eff[0] = crate::ps2v::Pf::f(0.08);
        for tick in 1..=TURN_TICKS {
            let previous_phase = pose.phase;
            pose.update(&h, &c, &s);
            if tick <= TURN_IN_TICKS {
                assert!(pose.clock_speed < 0.0);
                assert_ne!(pose.phase, previous_phase, "the outgoing reverse cycle must not freeze");
            } else {
                assert!(pose.clock_speed > 0.0);
            }
        }
        assert!(pose.clock_speed > 0.0 && pose.layer().is_none() && pose.idle_layer().is_none());
        assert_eq!(pose.phase, 0.0);
    }
    #[test]
    fn a_jump_or_stop_does_not_receive_the_previous_running_layer() {
        for state in [7, 3] {
            let c = class(); let s = base(&c); let mut h = runner(); let mut pose = LegPose::default();
            h.eff[0] = crate::ps2v::Pf::f(-0.08); h.eff[1] = crate::ps2v::Pf::f(-0.04);
            pose.update(&h, &c, &s); assert!(pose.layer().is_some());
            h.state = state;
            if state == 7 { h.group = 4; h.air_ticks = 1; }
            pose.update(&h, &c, &s);
            assert_eq!(pose.weight, 0.0);
            assert!(pose.layer().is_none() && pose.yaws.is_empty());
        }
    }
    #[test]
    fn cleanup_removes_old_lean_but_preserves_aim_and_boot_scaling() {
        use super::super::idle::joint;
        let mut h = runner();
        for k in [joint::REC0, joint::NECK, joint::REC2, joint::SECONDARY, joint::FOOT_L] {
            h.idle.joints[k].cur = [0.2, -0.1, 0.3];
            h.idle.joints[k].vel = [-0.07, 0.03, 0.1];
            h.idle.joints[k].target = [0.1; 3];
        }
        h.idle.joints[joint::ARM].cur[1] = 0.42;
        h.idle.joints[joint::FOOT_L].node_scale = 0.01;
        let velocity = h.eff;
        clean_run_modifiers(&mut h);
        assert_eq!(h.idle.joints[joint::REC0].cur, [0.0; 3]);
        assert_eq!(h.idle.joints[joint::NECK].vel, [0.0; 3]);
        assert_eq!(h.idle.joints[joint::FOOT_L].cur, [0.0; 3]);
        assert_eq!(h.idle.joints[joint::ARM].cur[1], 0.42);
        assert_eq!(h.idle.joints[joint::FOOT_L].node_scale, 0.01);
        assert_eq!(h.eff, velocity);
        h.combat_strafe = false; h.idle.joints[joint::REC0].cur = [0.4; 3];
        let before = h.idle.joints;
        clean_run_modifiers(&mut h);
        assert_eq!(h.idle.joints, before, "normal run stays native");
        h.combat_strafe = true; h.substate = 0;
        clean_run_modifiers(&mut h);
        assert_eq!(h.idle.joints, before, "walking stays native");
    }
    #[test]
    fn locked_run_discards_old_animation_snapshots_and_walking_keeps_them() {
        use super::super::anim::{AnimCtl, RatchetAnim};
        let c = class(); let mut h = runner();
        let dirty = |a: &mut RatchetAnim| {
            a.state.seq_a = moby_anim::SNAPSHOT_SEQ; a.state.seq_b = 4;
            a.state.frame_b = 2; a.state.t = 0.6;
            a.snapshot = Some(c.frame(0, 0).unwrap().clone());
        };
        let mut a = RatchetAnim::new(&c); dirty(&mut a);
        a.ctl(&c).combat_pose(&h);
        assert_eq!(a.state.seq_a, 4); assert_eq!(a.state.seq_b, 4);
        assert_eq!(a.state.frame_a, 2); assert_eq!(a.state.frame_b, 3);
        assert_eq!(a.state.t, 0.0); assert!(a.snapshot.is_none());
        dirty(&mut a); h.substate = 0;
        let state = a.state; let snap = a.snapshot.clone();
        a.ctl(&c).combat_pose(&h);
        assert_eq!(a.state, state); assert_eq!(a.snapshot, snap);
    }

    fn point_stick(h: &mut Hero, degrees: f32) {
        let relative = (degrees - 90.0).to_radians();
        h.eff[0] = crate::ps2v::Pf::f(relative.cos() * 0.08);
        h.eff[1] = crate::ps2v::Pf::f(relative.sin() * 0.08);
    }
    #[test]
    fn reverse_sector_starts_at_200_and_returns_to_normal_at_340() {
        for (degrees, back) in [(0.0f32, false), (90.0, false), (180.0, false),
            (190.0, false), (199.9, false), (200.0, true), (200.1, true),
            (270.0, true), (339.9, true), (340.0, false), (350.0, false), (359.9, false)] {
            let (_, reverse) = direction((degrees - 90.0).to_radians());
            assert_eq!(reverse, back, "screen angle {degrees}");
        }
        let (left, back) = direction(100.0f32.to_radians()); // screen=190
        assert!(!back && (left - std::f32::consts::FRAC_PI_2).abs() < 0.00001);
        let (right, back) = direction((-100.0f32).to_radians()); // screen=350
        assert!(!back && (right + std::f32::consts::FRAC_PI_2).abs() < 0.00001);
    }
    #[test]
    fn walking_below_the_lateral_axis_keeps_forward_playback() {
        let c = class(); let mut s = base(&c); s.seq_a = 3; s.seq_b = 3;
        let mut h = runner(); h.substate = 0; let mut pose = LegPose::default();
        for degrees in [0.0f32, 355.0, 345.0, 350.0, 0.0, 180.0, 190.0, 199.0] {
            point_stick(&mut h, degrees);
            pose.update(&h, &c, &s);
            assert!(!pose.backward && pose.clock_speed > 0.0, "angle {degrees}");
            assert_eq!(pose.seq, 3);
            assert!(pose.idle_layer().is_none(), "the margin must not cause unnecessary turn poses");
            assert!(pose.native_keys);
        }
    }
    #[test]
    fn idle_crossover_is_fifteen_ticks_and_never_changes_physics() {
        for walk in [false, true] {
            let c = class(); let mut s = base(&c); let mut h = runner();
            if walk { h.substate = 0; s.seq_a = 3; s.seq_b = 3; }
            let mut pose = LegPose::default(); pose.update(&h, &c, &s);
            point_stick(&mut h, 270.0);
            let before = (h.pos, h.eff, h.vel, h.speed, h.state, h.substate);
            let mut weights = Vec::new();
            for tick in 1..=15 {
                pose.update(&h, &c, &s);
                weights.push(pose.idle_weight);
                assert_eq!(pose.turn_ticks, 15 - tick);
                if tick < 15 { assert!(pose.idle_layer().is_some()); }
                assert_eq!((h.pos, h.eff, h.vel, h.speed, h.state, h.substate), before);
            }
            assert!(weights[0] > 0.0 && weights[0] < 0.06, "soft initial blend");
            assert_eq!(weights[6], 1.0, "idle is reached on tick seven");
            assert!(weights[13] > 0.0 && weights[13] < 0.06, "soft final blend");
            assert_eq!(weights[14], 0.0, "finished at one quarter of a second");
            for pair in weights[..7].windows(2) { assert!(pair[1] > pair[0]); }
            for pair in weights[6..].windows(2) { assert!(pair[1] < pair[0]); }
            assert_eq!(weights.iter().filter(|&&w| w == 1.0).count(), 1,
                "there is no hold at idle");
            assert_eq!(pose.turn_ticks, 0);
            assert!(pose.backward && pose.idle_layer().is_none());
            for _ in 0..8 { pose.update(&h, &c, &s); assert!(pose.idle_layer().is_none()); }
            point_stick(&mut h, 199.0);
            let return_motion = (h.pos, h.eff, h.vel, h.speed, h.state, h.substate);
            for _ in 0..15 {
                pose.update(&h, &c, &s);
                assert_eq!((h.pos, h.eff, h.vel, h.speed, h.state, h.substate), return_motion);
            }
            assert!(!pose.backward && pose.clock_speed > 0.0 && pose.idle_layer().is_none());
        }
    }
    #[test]
    fn rapid_boundary_changes_cannot_restart_idle_forever() {
        let c = class(); let s = base(&c); let mut h = runner(); let mut pose = LegPose::default();
        pose.update(&h, &c, &s);
        for i in 0..15 {
            let degrees = if i % 2 == 0 && i < 14 { 201.0f32 } else { 199.0 };
            point_stick(&mut h, degrees); pose.update(&h, &c, &s);
            assert_eq!(pose.turn_ticks, 14 - i as u8);
        }
        assert!(pose.idle_layer().is_none());
        assert!(!pose.backward);
    }
    #[test]
    fn idle_pose_excludes_hands_and_is_cancelled_when_jumping() {
        let c = class(); let s = base(&c); let mut h = runner(); let mut pose = LegPose::default();
        pose.update(&h, &c, &s); point_stick(&mut h, 270.0);
        for _ in 0..7 { pose.update(&h, &c, &s); }
        let idle = pose.idle_layer().expect("the middle tick reaches idle");
        assert_eq!(idle.seq_a, 0); assert_eq!(idle.frame_a, 0); assert_eq!(idle.weight, 1.0);
        assert!(!idle.joints.contains(&5) && !idle.joints.contains(&6), "hands/head keep their layers");
        assert!(idle.joints.contains(&0) && idle.joints.contains(&1));
        h.state = 7; h.group = 4; h.air_ticks = 1;
        pose.update(&h, &c, &s);
        assert!(pose.idle_layer().is_none() && pose.layer().is_none() && pose.yaws.is_empty());
    }

    #[test]
    fn thresholds_follow_the_analogue_stick_even_if_motion_is_still_diagonal() {
        let mut h = runner(); point_stick(&mut h, 225.0);
        for (angle, back) in [(199.0f32, false), (200.0, true), (270.0, true), (340.0, false), (350.0, false)] {
            let radians = angle.to_radians();
            h.stick = [crate::ps2v::Pf::f(radians.cos()), crate::ps2v::Pf::f(-radians.sin())];
            assert_eq!(movement_direction(&h).1, back, "stick angle {angle}");
        }
        h.stick = [crate::ps2v::Pf::ZERO; 2];
        assert!(movement_direction(&h).1, "neutral stick follows real backwards movement");
        h.combat_strafe = false; h.combat_release = true;
        h.stick = [crate::ps2v::Pf::ONE, crate::ps2v::Pf::ZERO];
        assert!(movement_direction(&h).1, "release pose follows real momentum");
    }

}
