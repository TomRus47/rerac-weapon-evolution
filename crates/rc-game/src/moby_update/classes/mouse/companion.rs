//! Drone Device ownership is snapshotted on summon, independent of head equipment.
//! The native mouse flight, model, joints and disappearance remain shared.
use super::*;

const TAG: u32 = 0x5745_4d35;
const TAG_AT: usize = MOUSE_LEN;
const BORN: usize = TAG_AT + 4;
const SIZE: usize = TAG_AT + 8;
const LIFE: usize = TAG_AT + 12;
const BUDGET: usize = TAG_AT + 16;
const INTERVAL: usize = TAG_AT + 20;
const RANGE: usize = TAG_AT + 24;
const DAMAGE: usize = TAG_AT + 28;
const COLOR: usize = TAG_AT + 32;
const LEN: usize = TAG_AT + 36;
const SHOT_TAG: usize = 0x20;
const SHOT_LEFT: usize = 0x24;
const SHOT_HP: usize = 0x28;
const SHOT_COLOR: usize = 0x2c;
const SHOT_LEN: usize = 0x30;

pub(super) fn custom(m: &crate::moby_runtime::Moby) -> bool {
    m.pvars.len() >= LEN && p::u32(&m.pvars, TAG_AT) == TAG
}

/// Check the shared native/experimental singleton; stale deleted slots do not
/// block future summons, including after a checkpoint reload.
pub fn active(w: &mut World) -> bool {
    let valid = w.svc.mouse.summoned.is_some_and(|id| {
        w.table
            .mobys
            .get(id)
            .is_some_and(|m| m.o_class == 1818 && !m.is_deleted() && (2..=6).contains(&m.state))
    });
    if !valid {
        w.svc.mouse.summoned = None;
    }
    valid
}

pub fn summon(w: &mut World, yaw: f32) -> bool {
    if active(w) || w.classes.info(1818).is_none() || w.classes.info(SHOT_CLASS).is_none() {
        return false;
    }
    let Some(id) = w.create_moby(1818) else {
        return false;
    };
    let at = hero_feet(w);
    let frame = w.counter as u32;
    let value = |key, native| crate::weapon_evo::extra(24, key, native);
    let life = (value("sand_mouse_lifetime_seconds", 90.0) * 60.0).round() as i32;
    let life = w.ticks(life);
    let color =
        crate::weapon_evo::extra_color(24, "sand_mouse_projectile_color").map_or(-1, |c| c as i32);
    let (light, ambient) = w
        .hero_moby
        .map(|h| (w.m(h).light, w.m(h).ambient))
        .unwrap_or((0, [0x80; 4]));
    {
        let m = w.mm(id);
        m.pvars.resize(LEN, 0);
        p::set_u32(&mut m.pvars, TAG_AT, TAG);
        p::set_u32(&mut m.pvars, BORN, frame);
        p::set_ff(&mut m.pvars, SIZE, m.scale);
        p::set_i32(&mut m.pvars, LIFE, life);
        p::set_i32(
            &mut m.pvars,
            BUDGET,
            value("sand_mouse_max_shots", 100.0).round() as i32,
        );
        p::set_i32(
            &mut m.pvars,
            INTERVAL,
            value("sand_mouse_fire_interval_ticks", 8.0).round() as i32,
        );
        p::set_ff(&mut m.pvars, RANGE, value("sand_mouse_shot_range", 24.0));
        p::set_ff(
            &mut m.pvars,
            DAMAGE,
            value("sand_mouse_shot_damage_hp", 50.0),
        );
        p::set_i32(&mut m.pvars, COLOR, color);
        p::set_i32(&mut m.pvars, pvo::HOUSE, -1);
        p::set_i32(&mut m.pvars, pvo::VOICE, -1);
        m.position = [
            at[0] + yaw.cos() * 2.0,
            at[1] + yaw.sin() * 2.0,
            at[2] + 2.0,
            1.0,
        ];
        m.rotation = [0.0, 0.0, yaw, 0.0];
        m.scale *= 0.01;
        m.state = 5;
        m.cmd = 2;
        m.mode &=
            !(mode::HIDDEN | mode::NO_UPDATE | mode::NO_ANIM | mode::KEEP_MATRIX | mode::KEEP_ROWS);
        m.visible = 1;
        m.draw_dist = 0xff;
        m.update_dist = 0xff;
        m.alpha = 0x80;
        m.has_collision = false;
        m.light = light;
        m.ambient = ambient;
    }
    seed_walker(w, id);
    blend(w, id, 1, 0);
    w.svc.mouse.summoned = Some(id);
    true
}

pub(super) fn update(w: &mut World, id: MobyId) {
    if w.m(id).state == 6 {
        vanish(w, id);
        if w.m(id).state == 7 {
            if w.svc.mouse.summoned == Some(id) {
                w.svc.mouse.summoned = None;
            }
            w.delete_moby(id);
        }
        return;
    }
    let done = c::dec_timer_pvar_i32(w, id, LIFE) != 0
        || c::pi32(w, id, pvo::SHOTS) >= c::pi32(w, id, BUDGET);
    if done {
        c::set_pi16(w, id, pvo::HOLD, w.ticks(k::VANISH) as i16);
        voice_release(w, id);
        w.mm(id).state = 6;
        return;
    }
    let elapsed = (w.counter as u32).wrapping_sub(p::u32(&w.m(id).pvars, BORN));
    let growth = (elapsed as f32 / w.ticks(30).max(1) as f32).clamp(0.0, 1.0);
    w.mm(id).scale = c::pf(w, id, SIZE) * (0.01 + 0.99 * growth);
    target_hover(w, id);
    fly(w, id);
    if looped(w, id) {
        flap(w, id);
    }
    voice_keep(w, id);
    if growth >= 1.0 {
        shoot(w, id);
    }
    // The ordinary callback setup lives in mouse_update, before this branch.
    crate::shadows::blob(w, 0.3 * growth, id);
    jets(w, id);
}

pub(super) fn range(w: &World, id: MobyId) -> f32 {
    if custom(w.m(id)) {
        c::pf(w, id, RANGE)
    } else {
        k::RANGE
    }
}
pub(super) fn interval(w: &World, id: MobyId) -> i32 {
    if custom(w.m(id)) {
        c::pi32(w, id, INTERVAL)
    } else {
        k::COOLDOWN
    }
}

pub(super) fn configure_shot(w: &mut World, mouse: MobyId, shot: MobyId) {
    if !custom(w.m(mouse)) {
        return;
    }
    let (range, hp, rgb) = (
        c::pf(w, mouse, RANGE),
        c::pf(w, mouse, DAMAGE),
        c::pi32(w, mouse, COLOR),
    );
    let m = w.mm(shot);
    m.pvars.resize(SHOT_LEN.max(m.pvars.len()), 0);
    p::set_u32(&mut m.pvars, SHOT_TAG, TAG);
    p::set_ff(&mut m.pvars, SHOT_LEFT, range);
    p::set_ff(&mut m.pvars, SHOT_HP, hp);
    p::set_i32(&mut m.pvars, SHOT_COLOR, rgb);
    m.draw_dist = 0xff;
    if rgb >= 0 {
        m.glow = crate::weapon_evo::tint_rgba(m.glow, Some(rgb as u32));
        let [r, g, b] = crate::weapon_evo::rgb_float(rgb as u32);
        m.ambient = [
            (r * 128.0) as u8,
            (g * 128.0) as u8,
            (b * 128.0) as u8,
            m.ambient[3],
        ];
    }
    w.svc.hits.real_hp.set_source_damage_hp(shot, 24, hp);
}

pub fn owned_shot(m: &crate::moby_runtime::Moby) -> bool {
    m.o_class == SHOT_CLASS && m.pvars.len() >= SHOT_LEN && p::u32(&m.pvars, SHOT_TAG) == TAG
}

pub(super) fn shot_step(w: &mut World, id: MobyId) {
    let left = c::pf(w, id, SHOT_LEFT);
    if left <= 0.0 {
        return w.delete_moby(id);
    }
    let old = c::pos(w, id);
    let step = crate::moby_update::creature::fx::polar(
        (DT * 40.0).min(left),
        c::yaw(w, id),
        -w.m(id).rotation[1],
    );
    let new = c::add(old, step);
    let mut dir = c::set_len3(step, 1.0);
    dir[3]=f32::from_bits(0x45af_df66);
    let hp = c::pf(w, id, SHOT_HP);
    w.svc.hits.real_hp.set_source_damage_hp(id, 24, hp);
    let tmpl = HitTemplate {
        dir: pv(dir),
        attacker: Some(id),
        flags: 0x10001,
        b18: 1,
        b19: 1,
        h1a: SHOT_CLASS as u16,
        damage: crate::moby_update::services::pf(hp / 100.0),
        w20: 1,
    };
    let mouse = usize::try_from(c::pi32(w, id, 0x10)).ok();
    let hit = crate::moby_update::services::line_hit_in(
        w.table,
        w.svc,
        w.classes,
        w.coll,
        pv(old),
        pv(new),
        0,
        mouse,
        &tmpl,
    );
    w.mm(id).position = hit
        .as_ref()
        .map_or(new, |h| [h.point[0], h.point[1], h.point[2], 1.0]);
    c::set_pf(w, id, SHOT_LEFT, (left - c::len3(step)).max(0.0));
    let class_scale = w
        .classes
        .info(SHOT_CLASS)
        .map_or(w.m(id).scale, |i| i.scale);
    w.mm(id).scale = (w.m(id).scale + SPEED * 0.21 * class_scale).min(class_scale);
    if hit.is_some()
        || left <= c::len3(step)
        || !crate::moby_update::creature::projectile::in_world(new)
    {
        w.delete_moby(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_runtime::{ClassInfo, Moby, MobyTable};
    #[test]
    fn summon_is_singleton_grows_and_survives_summoner_equipment_changes() {
        let hero = crate::hero::Hero::spawn([100.0, 100.0, 50.0], 0.0);
        let mut rng = crate::rng::Rng::new();
        let mut classes = crate::moby_update::ClassTable::default();
        for (class, update) in [(1818, MOUSE_FN), (SHOT_CLASS, SHOT_FN)] {
            classes.classes.insert(
                class,
                (
                    ClassInfo {
                        scale: 2.0,
                        update_fn: Some(update),
                        ..Default::default()
                    },
                    None,
                ),
            );
        }
        let mut table = MobyTable::new(vec![], 8);
        let mut svc = crate::moby_update::Services::new();
        let mut w = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 0);
        assert!(summon(&mut w, 0.0));
        let id = w.svc.mouse.summoned.unwrap();
        assert!((w.m(id).scale - 0.02).abs() < 0.00001);
        assert_eq!(w.m(id).position, [102.0, 100.0, 52.0, 1.0]);
        assert!(!summon(&mut w, 0.0));
        // Neither owning nor wearing the head item is needed by the new branch.
        assert_eq!(w.hero.head_slot.id, 0);
        w.counter = 30;
        update(&mut w, id);
        assert!((w.m(id).scale - 2.0).abs() < 0.00001);
        assert_eq!(w.m(id).state, 5);
        c::set_pi32(&mut w, id, pvo::SHOTS, 100);
        update(&mut w, id);
        assert_eq!(w.m(id).state, 6);
        assert!(
            active(&mut w),
            "retiring companion still owns the singleton"
        );
        for _ in 0..35 {
            if !w.m(id).is_deleted() {
                update(&mut w, id);
            }
        }
        assert!(w.m(id).is_deleted());
        assert!(!active(&mut w));
    }
    #[test]
    fn native_active_mouse_blocks_the_drone_variant_and_stale_slots_do_not() {
        let hero = crate::hero::Hero::new();
        let mut rng = crate::rng::Rng::new();
        let classes = crate::moby_update::ClassTable::default();
        let mut svc = crate::moby_update::Services::new();
        let mouse = Moby {
            o_class: 1818,
            state: 5,
            ..Default::default()
        };
        let mut table = MobyTable::new(vec![mouse], 2);
        svc.mouse.summoned = Some(0);
        let mut w = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 0);
        assert!(active(&mut w));
        assert!(!summon(&mut w, 0.0));
        w.delete_moby(0);
        assert!(!active(&mut w));
    }
    #[test]
    fn shot_owner_tag_does_not_reclassify_native_mouse_shots() {
        let mut shot = Moby {
            o_class: SHOT_CLASS,
            pvars: vec![0; SHOT_LEN],
            ..Default::default()
        };
        assert!(!owned_shot(&shot));
        p::set_u32(&mut shot.pvars, SHOT_TAG, TAG);
        assert!(owned_shot(&shot));
        shot.o_class = 305;
        assert!(!owned_shot(&shot));
    }
}
