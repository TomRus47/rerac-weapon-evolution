//! Per-mine snapshots isolate deployed effects from the held weapon and grade.
use super::*;
const TAG_AT: usize = 0x80;
const FLAGS: usize = 0x84;
const LIFE: usize = 0x88;
const RANGE: usize = 0x8c;
const ROTATION: usize = 0x90;
const DPS: usize = 0x94;
const ANGLE: usize = 0x98;
const CORE: usize = 0x9c;
const OUTER: usize = 0xa0;
const BURN: usize = 0xa4;
const EXPLOSION: usize = 0xa8;
const LIGHT: usize = 0xac;
const HP: usize = 0xb0;
const TAG: u32 = 0x5745_4635;
const LIGHT_SLOT: usize = 0xb4;
const LIGHT_GENERATION: usize = 0xb8;
const LEN: usize = 0xbc;

pub fn snapshot(m: &mut crate::moby_runtime::Moby) {
    m.pvars.resize(LEN.max(m.pvars.len()), 0);
    p::set_u32(&mut m.pvars, TAG_AT, TAG);
    p::set_i32(&mut m.pvars, LIGHT_SLOT, -1);
    p::set_u32(
        &mut m.pvars,
        FLAGS,
        u32::from(crate::weapon_evo::extra_flag(
            17,
            "flame_mine_enabled",
            false,
        )),
    );
    let value = |key, native| crate::weapon_evo::extra(17, key, native);
    p::set_i32(
        &mut m.pvars,
        LIFE,
        crate::hero::physics::ticks((value("mine_lifetime_seconds", 30.0) * 60.0).round() as i32),
    );
    p::set_ff(&mut m.pvars, RANGE, value("mine_flame_range", 8.0));
    p::set_ff(
        &mut m.pvars,
        ROTATION,
        value("mine_flame_rotation_degrees_per_second", 90.0).to_radians() / 60.0,
    );
    p::set_ff(
        &mut m.pvars,
        DPS,
        value("mine_flame_damage_per_second", 100.0),
    );
    p::set_ff(
        &mut m.pvars,
        HP,
        value("damage_hp", 300.0 * value("damage_multiplier", 1.0)),
    );
    p::set_ff(&mut m.pvars, ANGLE, m.rotation[2]);
    for (off, key) in [
        (CORE, "flame_core_color"),
        (OUTER, "flame_outer_color"),
        (BURN, "burn_flame_color"),
        (EXPLOSION, "explosion_color"),
        (LIGHT, "light_color"),
    ] {
        p::set_i32(
            &mut m.pvars,
            off,
            crate::weapon_evo::extra_color(17, key).map_or(-1, |rgb| rgb as i32),
        );
    }
}
fn valid(m: &crate::moby_runtime::Moby) -> bool {
    m.pvars.len() >= LEN && p::u32(&m.pvars, TAG_AT) == TAG
}
fn color(m: &crate::moby_runtime::Moby, off: usize) -> Option<u32> {
    if !valid(m) {
        return None;
    }
    let value = p::i32(&m.pvars, off);
    (value >= 0).then_some(value as u32)
}
pub fn burn_color(m: &crate::moby_runtime::Moby) -> Option<u32> {
    color(m, BURN)
}
pub(super) fn blast_color(m: &crate::moby_runtime::Moby) -> Option<u32> {
    color(m, EXPLOSION)
}
pub(super) fn light_color(m: &crate::moby_runtime::Moby) -> Option<u32> {
    color(m, LIGHT)
}
pub(super) fn flame_colors(m: &crate::moby_runtime::Moby) -> [Option<u32>; 2] {
    [color(m, CORE), color(m, OUTER)]
}

pub(super) fn retire(w: &mut World, id: MobyId) {
    release_loop(w, id);
    if let Some(slot) = owned_light(w, id) {
        w.svc.point_lights.free(slot);
    }
    w.delete_moby(id);
}
/// Returns true when the experimental path or retirement owns this tick.
pub(super) fn update(w: &mut World, id: MobyId) -> bool {
    if !valid(w.m(id)) {
        snapshot(w.mm(id));
    }
    if c::pi32(w, id, pv::BORN) < 0 || c::dec_timer_pvar_i32(w, id, LIFE) != 0 {
        retire(w, id);
        return true;
    }
    if p::u32(&w.m(id).pvars, FLAGS) == 0 {
        let hp = c::pf(w, id, HP);
        w.svc.hits.real_hp.set_source_damage_hp(id, 17, hp);
        return false;
    }
    if w.m(id).cmd != 0 || w.get_hit(id, 1, false).is_some() {
        retire(w, id);
        return true;
    }
    if w.m(id).state & LANDED == 0 {
        flight(w,id);
        return true;
    }
    // Landed experimental mines keep the class collision volume, just like a
    // native landed mine. It is independent of our own swept terrain probes.
    w.mm(id).has_collision = w.classes.info(CLASS).is_some_and(|info| info.has_collision);
    let voice = c::pi32(w, id, pv::SOUND);
    if !w.sound_alive(voice, id) {
        let voice = w.play_sound_as(
            crate::hero::pyrocitor::LOOP_SOUND,
            4,
            id,
            crate::hero::pyrocitor::CLASS,
        );
        c::set_pi32(w, id, pv::SOUND, voice);
    }
    let angle = c::add_rot(c::pf(w, id, ANGLE), c::pf(w, id, ROTATION));
    c::set_pf(w, id, ANGLE, angle);
    let mut start = c::pos(w, id);
    start[2] += 0.12;
    let range = c::pf(w, id, RANGE);
    let dps = c::pf(w, id, DPS);
    w.svc.hits.real_hp.set_source_damage_hp(id, 17, dps);
    let colors = flame_colors(w.m(id));
    let point = crate::point_lights::PointLight {
        color: light_color(w.m(id)).map_or([1.0, 0.5, 0.1], crate::weapon_evo::rgb_float),
        intensity: 0.5,
        pos: [start[0], start[1], start[2]],
        radius: range.min(12.0),
    };
    if let Some(slot) = owned_light(w, id) {
        w.svc.point_lights.set(slot, point);
    } else if let Some(slot) = w.svc.point_lights.alloc(point, 0.0) {
        c::set_pi32(w, id, LIGHT_SLOT, slot as i32);
        let generation = w.svc.point_lights.generation[slot];
        p::set_u32(&mut w.mm(id).pvars, LIGHT_GENERATION, generation);
    }
    for arm in 0..4 {
        let yaw = angle + arm as f32 * std::f32::consts::FRAC_PI_2;
        let direction = [yaw.cos(), yaw.sin(), 0.0, 0.0];
        let far = c::add(start, c::scale(direction, range));
        let end = w
            .coll_line(pvq(start), pvq(far), 2, Some(id))
            .map_or(far, |h| [h.point[0], h.point[1], h.point[2], 1.0]);
        let length = c::dist3(start, end);
        let speed = length / 18.0 + crate::particles::type12::DECEL * DT2 * 9.0;
        let vel = c::scale(direction, speed);
        for flags in [0x84, 0x85] {
            let draws = crate::particles::type12::Draws::draw(w.rng, flags);
            if let Some(sys) = w.particles.as_deref_mut() {
                crate::particles::type12::spawn_colored(
                    sys, length, start, vel, flags, 0, &draws, colors,
                );
            }
        }
        let middle = c::scale(c::add(start, end), 0.5);
        let candidates =
            w.sphere_mobys_list(Pf::f(length * 0.5 + 1.0), pvq(middle), 0, Some(id), None);
        for target in candidates {
            if Some(target) == w.hero_moby
                || w.m(target).o_class == CLASS
                || w.m(target).is_deleted()
            {
                continue;
            }
            let point = w.m(target).position;
            let height = tg::aim_height(w.m(target)).unwrap_or(0.3).max(0.3);
            let point = [
                point[0],
                point[1],
                start[2].clamp(point[2], point[2] + height),
                1.0,
            ];
            if capsule_contains(start, end, point, 0.65) {
                let tmpl = HitTemplate {
                    dir: pvq([direction[0], direction[1], 1.0, f32::from_bits(0x45af_df66)]),
                    attacker: Some(id),
                    flags: 0x10000,
                    b18: 5,
                    b19: 0,
                    h1a: CLASS as u16,
                    damage: Pf::f(dps / 6000.0),
                    w20: 1,
                };
                w.deliver_hit(target, &tmpl);
            }
        }
    }
    true
}
const BODY_RADIUS: f32 = 0.25;
const LANDED_HEIGHT: f32 = 0.4;

/// Test both the centre and the lower hull, with the native two-sided terrain
/// mask. A missing class collision primitive must never let the mine tunnel.
fn flight(w: &mut World, id: MobyId) {
    w.mm(id).has_collision = false;
    let old = c::pos(w,id);
    let mut vel = c::pv4(w,id,pv::VEL);
    let new = c::add(old,vel);
    let flags = 0x12; // two-sided world / platform triangles, no owner primitives
    let centre = w.coll_line(pvq(old),pvq(new),flags,Some(id));
    let bottom_old = c::add(old,[0.0,0.0,-BODY_RADIUS,0.0]);
    let bottom_new = c::add(new,[0.0,0.0,-BODY_RADIUS,0.0]);
    let bottom = w.coll_line(pvq(bottom_old),pvq(bottom_new),flags,Some(id));
    let hit = match (centre,bottom) {
        (Some(a),Some(b)) => {
            let time = |h: &crate::collision_query::CollOutput, from: V|
                c::dot3(c::sub([h.point[0],h.point[1],h.point[2],0.0],from),vel);
            if time(&a,old)<=time(&b,bottom_old) {Some(a)} else {Some(b)}
        },
        (a,b) => a.or(b),
    };
    if let Some(hit) = hit {
        if hit.surface_id()==0 {retire(w,id);return;}
        let mut normal = c::set_len3([hit.normal[0],hit.normal[1],hit.normal[2],0.0],1.0);
        if c::dot3(normal,vel)>0.0 {normal=c::scale(normal,-1.0);}
        let point=[hit.point[0],hit.point[1],hit.point[2],1.0];
        if normal[2]>=0.5 && vel[2]<=0.0 {
            c::set_pos(w,id,c::add(point,c::scale(normal,LANDED_HEIGHT)));
            w.mm(id).state=OUT|LANDED;
            c::set_pv4(w,id,pv::VEL,[0.0;4]);
            w.mm(id).has_collision=w.classes.info(CLASS).is_some_and(|info|info.has_collision);
            orient(w,id,normal);
        } else {
            // A wall/ceiling redirects the mine; it does not activate a fire
            // cross in midair or allow the next position to enter the surface.
            c::set_pos(w,id,c::add(point,c::scale(normal,BODY_RADIUS+0.02)));
            vel=c::scale(reflect(vel,normal),0.5);
            vel[2]-=DT2*9.8;
            c::set_pv4(w,id,pv::VEL,vel);
        }
    } else {
        // A sphere at the next centre catches a grazing face missed by the two
        // segment probes. The downward support probe uses the same terrain mask.
        if let Some(hit)=w.coll_sphere(pvq(new),Pf::f(BODY_RADIUS),flags,Some(id)) {
            let mut normal=c::set_len3([hit.normal[0],hit.normal[1],hit.normal[2],0.0],1.0);
            if c::dot3(normal,vel)>0.0 {normal=c::scale(normal,-1.0);}
            if normal[2]>=0.5 && vel[2]<=0.0 && hit.surface_id()!=0 {
                let point=[hit.point[0],hit.point[1],hit.point[2],1.0];
                c::set_pos(w,id,c::add(point,c::scale(normal,LANDED_HEIGHT)));
                w.mm(id).state=OUT|LANDED;
                c::set_pv4(w,id,pv::VEL,[0.0;4]);
                w.mm(id).has_collision=w.classes.info(CLASS).is_some_and(|info|info.has_collision);
                orient(w,id,normal);
                return;
            }
            if hit.surface_id()==0 {retire(w,id);return;}
            let point=[hit.point[0],hit.point[1],hit.point[2],1.0];
            c::set_pos(w,id,c::add(point,c::scale(normal,BODY_RADIUS+0.02)));
            vel=c::scale(reflect(vel,normal),0.5);vel[2]-=DT2*9.8;
            c::set_pv4(w,id,pv::VEL,vel);
            return;
        }
        c::set_pos(w,id,new);
        vel[2]-=DT2*9.8;
        c::set_pv4(w,id,pv::VEL,vel);
    }
    if !c::projectile::in_world(c::pos(w,id)) {retire(w,id);}
}

fn capsule_contains(a: V, b: V, p: V, r: f32) -> bool {
    let d = c::sub(b, a);
    let len2 = c::dot3(d, d);
    let t = if len2 > 0.0 {
        (c::dot3(c::sub(p, a), d) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    c::dist3(p, c::add(a, c::scale(d, t))) <= r
}

fn owned_light(w: &World, id: MobyId) -> Option<usize> {
    let slot = usize::try_from(c::pi32(w, id, LIGHT_SLOT)).ok()?;
    (slot < 8
        && w.svc.point_lights.slots[slot].is_some()
        && w.svc.point_lights.generation[slot] == p::u32(&w.m(id).pvars, LIGHT_GENERATION))
    .then_some(slot)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dropped_mine(z:f32,vz:f32)->(crate::moby_update::classes::bomb_water::tests::Bench,MobyId) {
        use crate::moby_update::classes::bomb_water::tests::{Bench,pool};
        let mut b=Bench::new(pool(8.0,-100.0),&[CLASS],[40.,40.,20.]);
        b.ct.classes.get_mut(&CLASS).unwrap().0.has_collision=true;
        let id=b.create(CLASS);
        init_held(&mut b.table.mobys[id],&mut b.rng,[20.,20.,z],0);
        p::set_v4f(&mut b.table.mobys[id].pvars,pv::VEL,[0.02,0.,vz,0.]);
        release(&mut b.table,id,&mut b.rng,[20.,20.,z]);
        p::set_u32(&mut b.table.mobys[id].pvars,FLAGS,1);
        p::set_i32(&mut b.table.mobys[id].pvars,LIFE,600);
        (b,id)
    }
    #[test]
    fn experimental_mine_lands_above_ground_and_activates_without_native_seek() {
        let (mut b,id)=dropped_mine(13.,0.03);
        for _ in 0..180 {b.tick(None);if b.table.mobys[id].state&LANDED!=0 {break;}}
        let m=&b.table.mobys[id];assert!(!m.is_deleted());assert_ne!(m.state&LANDED,0);
        assert!(m.position[2]>8.2 && m.position[2]<8.5,"mine at {:?}",m.position);
        assert!(m.has_collision);let landed=m.position;
        for _ in 0..60 {b.tick(None);}
        assert_eq!(b.table.mobys[id].position,landed);
        assert_eq!(b.table.mobys[id].state&SEEKING,0);
        assert!(b.svc.point_lights.active().next().is_some(),"landed flame mine did not activate");
        assert!(b.svc.sounds.iter().any(|s|s.sound_class==crate::hero::pyrocitor::CLASS));
    }
    #[test]
    fn fast_downward_mine_does_not_tunnel_through_a_floor() {
        let (mut b,id)=dropped_mine(12.,-10.);
        b.tick(None);
        assert!(!b.table.mobys[id].is_deleted());
        assert_ne!(b.table.mobys[id].state&LANDED,0);
        assert!(b.table.mobys[id].position[2]>=8.2);
    }
    #[test]
    fn flight_detection_does_not_require_a_class_collision_blob() {
        let (mut b,id)=dropped_mine(9.,-2.);
        b.ct.classes.get_mut(&CLASS).unwrap().0.has_collision=false;
        b.tick(None);b.tick(None);
        assert_ne!(b.table.mobys[id].state&LANDED,0);
        assert!(b.svc.point_lights.active().next().is_some());
    }

    #[test]
    fn independent_cross_arms_share_a_source_but_not_other_mines() {
        assert!(capsule_contains(
            [0.0; 4],
            [8.0, 0.0, 0.0, 0.0],
            [4.0, 0.5, 0.0, 0.0],
            0.65
        ));
        assert!(!capsule_contains(
            [0.0; 4],
            [8.0, 0.0, 0.0, 0.0],
            [4.0, 2.0, 0.0, 0.0],
            0.65
        ));
        let mut table = MobyTable::new(
            vec![crate::moby_runtime::Moby {
                o_class: 577,
                state: 1,
                ..Default::default()
            }],
            0,
        );
        let mut events = crate::moby_update::real_hp::Events::default();
        events.enabled = true;
        events.set_source_damage_hp(10, 17, 120.0);
        events.set_source_damage_hp(11, 17, 240.0);
        let t = HitTemplate {
            attacker: Some(10),
            h1a: CLASS as u16,
            b18: 5,
            b19: 0,
            flags: 0x10000,
            damage: Pf::ONE,
            ..Default::default()
        };
        for _ in 0..4 {
            events.queue(&table, 0, &t, false, [Pf::ZERO; 4], 0);
        }
        let mut other = t;
        other.attacker = Some(11);
        events.queue(&table, 0, &other, false, [Pf::ZERO; 4], 0);
        let a = events.take(0, u32::MAX).unwrap();
        let b = events.take(0, u32::MAX).unwrap();
        assert!((a.damage.to_f32() * 100.0 - 2.0).abs() < 0.0001);
        assert!((b.damage.to_f32() * 100.0 - 4.0).abs() < 0.0001);
        assert_eq!(a.evolution.unwrap().weapon, 17);
        assert!(events.take(0, u32::MAX).is_none());
        table.mobys[0].state = 0xfd;
    }
}
