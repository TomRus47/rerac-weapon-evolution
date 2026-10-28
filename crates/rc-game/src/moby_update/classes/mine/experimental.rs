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
const SIZE_PERCENT: usize = 0xbc;
const BASE_SCALE: usize = 0xc0;
const MODEL: usize = 0xc4;
const BODY_COLOR: usize = 0xc8;
const MAX_ACTIVE: usize = 0xcc;
const LEN: usize = 0xd0;

pub fn snapshot(m: &mut crate::moby_runtime::Moby) {
    snapshot_configured(m, |key,native|crate::weapon_evo::extra(17,key,native), |key|crate::weapon_evo::extra_color(17,key));
}
fn snapshot_configured(m:&mut crate::moby_runtime::Moby,value:impl Fn(&str,f32)->f32,color:impl Fn(&str)->Option<u32>) {
    let base_scale=if valid(m) {p::ff(&m.pvars,BASE_SCALE)} else {m.scale};
    m.pvars.resize(LEN.max(m.pvars.len()), 0);
    p::set_ff(&mut m.pvars,BASE_SCALE,base_scale);
    p::set_u32(&mut m.pvars, TAG_AT, TAG);
    let model=value("flame_mine_model",0.) as u32;
    p::set_u32(&mut m.pvars,MODEL,model);
    let body=match model {1=>"mine_bomb_color",2=>"mine_sphere_color",_=>"mine_model_color"};
    p::set_i32(&mut m.pvars,BODY_COLOR,color(body).map_or(-1,|v|v as i32));
    p::set_i32(&mut m.pvars, LIGHT_SLOT, -1);
    let fire_enabled=value("flame_mine_enabled",0.)!=0.;
    let standard_max=value("max_active_mines",8.);
    let standard_lifetime=value("mine_lifetime_seconds",30.);
    // The registry already reflects the saved R1 mode, including per-grade
    // switch limits. Missing overrides preserve old profiles' standard values.
    let (maximum,lifetime)=if fire_enabled {
        (value("flame_mine_max_active_mines",standard_max),
         value("flame_mine_lifetime_seconds",standard_lifetime))
    } else {(standard_max,standard_lifetime)};
    p::set_i32(&mut m.pvars,MAX_ACTIVE,maximum.round().clamp(1.,32.) as i32);
    p::set_u32(
        &mut m.pvars,
        FLAGS,
        u32::from(fire_enabled),
    );
    let size=if p::u32(&m.pvars,FLAGS)!=0 {value("flame_mine_size_percent",150.)} else {100.};
    p::set_ff(&mut m.pvars,SIZE_PERCENT,size);
    m.scale=base_scale*size/100.;
    p::set_i32(
        &mut m.pvars,
        LIFE,
        crate::hero::physics::ticks((lifetime * 60.0).round() as i32),
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
            color(match (model,key) {
                (1,"flame_core_color")=>"mine_bomb_core_color", (1,"flame_outer_color")=>"mine_bomb_outer_color", (1,"burn_flame_color")=>"mine_bomb_burn_color",
                (2,"flame_core_color")=>"mine_sphere_core_color", (2,"flame_outer_color")=>"mine_sphere_outer_color", (2,"burn_flame_color")=>"mine_sphere_burn_color", _=>key,
            }).map_or(-1, |rgb| rgb as i32),
        );
    }
}
/// The incoming mine's launch-time mode chooses the existing shared mine budget.
pub(super) fn max_active(m:&crate::moby_runtime::Moby)->usize {
    if valid(m) {p::i32(&m.pvars,MAX_ACTIVE).clamp(1,32) as usize} else {8}
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

pub fn body_color(m:&crate::moby_runtime::Moby)->Option<u32> {
    visual_class(m).and_then(|_|color(m,BODY_COLOR))
}
pub fn visual_class(m:&crate::moby_runtime::Moby)->Option<i16> {
    if m.o_class!=CLASS || !valid(m) || p::u32(&m.pvars,FLAGS)==0 {return None;}
    Some(match p::u32(&m.pvars,MODEL) {1=>super::super::bomb::BOMB_CLASS,2=>super::super::doom_canister::CLASS,_=>CLASS})
}

/// Apply the captured percentage to the selected model's own native scale.
/// Alternate meshes must not inherit the physical mine's held/release scaling.
/// Native mines retain their existing deployed size and collision behavior.
pub fn visual_scale(m:&crate::moby_runtime::Moby,model_native_scale:f32)->Option<f32> {
    let selected=visual_class(m)?;
    Some(if selected==CLASS {m.scale} else {model_native_scale*p::ff(&m.pvars,SIZE_PERCENT)/100.})
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
    if let Some(color)=color(w.m(id),BODY_COLOR) {
        let rgb=crate::weapon_evo::rgb_float(color).map(|v|(v*127.).round() as u8);
        let m=w.mm(id);m.ambient=[rgb[0],rgb[1],rgb[2],0];m.glow=0xff000000|color;
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
    w.mm(id).rotation[2]=angle;
    w.build_matrix(id);
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
    let size=c::pf(w,id,SIZE_PERCENT).max(1.)/100.;
    let body_radius=BODY_RADIUS*size;let landed_height=LANDED_HEIGHT*size;
    w.mm(id).has_collision = false;
    let old = c::pos(w,id);
    let mut vel = c::pv4(w,id,pv::VEL);
    let new = c::add(old,vel);
    let flags = 0x12; // two-sided world / platform triangles, no owner primitives
    let centre = w.coll_line(pvq(old),pvq(new),flags,Some(id));
    let bottom_old = c::add(old,[0.0,0.0,-body_radius,0.0]);
    let bottom_new = c::add(new,[0.0,0.0,-body_radius,0.0]);
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
        let normal = contact_normal(&hit, old);
        if departing_surface(w, id, &hit, normal, new, vel, body_radius) { return; }
        let point=[hit.point[0],hit.point[1],hit.point[2],1.0];
        if normal[2]>=0.5 && vel[2]<=0.0 {
            c::set_pos(w,id,c::add(point,c::scale(normal,landed_height)));
            w.mm(id).state=OUT|LANDED;
            c::set_pv4(w,id,pv::VEL,[0.0;4]);
            w.mm(id).has_collision=w.classes.info(CLASS).is_some_and(|info|info.has_collision);
            orient(w,id,normal);
        } else {
            // A wall/ceiling redirects the mine; it does not activate a fire
            // cross in midair or allow the next position to enter the surface.
            c::set_pos(w,id,c::add(point,c::scale(normal,body_radius+0.02)));
            vel=c::scale(reflect(vel,normal),0.5);
            vel[2]-=DT2*9.8;
            c::set_pv4(w,id,pv::VEL,vel);
        }
    } else {
        // A sphere at the next centre catches a grazing face missed by the two
        // segment probes. The downward support probe uses the same terrain mask.
        if let Some(hit)=w.coll_sphere(pvq(new),Pf::f(body_radius),flags,Some(id)) {
            let normal = contact_normal(&hit, old);
            if hit.surface_id() == 0 { retire(w, id); return; }
            if departing_surface(w, id, &hit, normal, new, vel, body_radius) { return; }
            if normal[2]>=0.5 && vel[2]<=0.0 {
                let point=[hit.point[0],hit.point[1],hit.point[2],1.0];
                c::set_pos(w,id,c::add(point,c::scale(normal,landed_height)));
                w.mm(id).state=OUT|LANDED;
                c::set_pv4(w,id,pv::VEL,[0.0;4]);
                w.mm(id).has_collision=w.classes.info(CLASS).is_some_and(|info|info.has_collision);
                orient(w,id,normal);
                return;
            }
            let point=[hit.point[0],hit.point[1],hit.point[2],1.0];
            c::set_pos(w,id,c::add(point,c::scale(normal,body_radius+0.02)));
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

/// Choose the side occupied by the previous centre, rather than reversing the
/// face normal to oppose velocity. A large mine can initially overlap a floor
/// while rising: that is separation, not a downward bounce through the floor.
fn contact_normal(hit: &crate::collision_query::CollOutput, old: V) -> V {
    let normal = c::set_len3([hit.normal[0], hit.normal[1], hit.normal[2], 0.0], 1.0);
    let point = [hit.point[0], hit.point[1], hit.point[2], 1.0];
    if c::dot3(normal, c::sub(old, point)) < 0.0 { c::scale(normal, -1.0) } else { normal }
}

/// Resolve residual hull penetration while preserving outward launch velocity.
/// Incoming contacts continue through the existing landing/bounce path.
fn departing_surface(w: &mut World, id: MobyId, hit: &crate::collision_query::CollOutput,
                     normal: V, new: V, mut vel: V, radius: f32) -> bool {
    if c::dot3(normal, vel) <= 0.0 { return false; }
    let point = [hit.point[0], hit.point[1], hit.point[2], 1.0];
    let depth = (radius + 0.02 - c::dot3(c::sub(new, point), normal)).max(0.0);
    c::set_pos(w, id, c::add(new, c::scale(normal, depth)));
    vel[2] -= DT2 * 9.8;
    c::set_pv4(w, id, pv::VEL, vel);
    if !c::projectile::in_world(c::pos(w, id)) { retire(w, id); }
    true
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
    #[test]
    fn flame_limits_override_only_active_mode_and_are_captured_for_every_grade() {
        for grade in 1..=8 {
            for fire in [false,true] {
                let (mut b,id)=dropped_mine(12.,0.02);
                let standard_max=grade as f32;let alternate_max=grade as f32+12.;
                let standard_life=30.+grade as f32;let alternate_life=2.+grade as f32;
                snapshot_configured(&mut b.table.mobys[id],|key,native|match key {
                    "flame_mine_enabled"=>f32::from(u8::from(fire)),
                    "max_active_mines"=>standard_max,"mine_lifetime_seconds"=>standard_life,
                    "flame_mine_max_active_mines"=>{assert!(fire);alternate_max},
                    "flame_mine_lifetime_seconds"=>{assert!(fire);alternate_life},_=>native,
                },|_|None);
                let lifetime=crate::hero::physics::ticks(((if fire {alternate_life}else{standard_life})*60.) as i32);
                assert_eq!(max_active(&b.table.mobys[id]),if fire {alternate_max}else{standard_max} as usize);
                assert_eq!(p::i32(&b.table.mobys[id].pvars,LIFE),lifetime);
                b.tick(None);
                // The live registry may now be native again: no recapture occurs.
                assert_eq!(p::i32(&b.table.mobys[id].pvars,LIFE),lifetime-1);
                assert_eq!(max_active(&b.table.mobys[id]),if fire {alternate_max}else{standard_max} as usize);
            }
        }
    }
    #[test]
    fn missing_flame_overrides_inherit_the_standard_profile() {
        let (mut b,id)=dropped_mine(12.,0.02);
        snapshot_configured(&mut b.table.mobys[id],|key,native|match key {
            "flame_mine_enabled"=>1.,"max_active_mines"=>3.,"mine_lifetime_seconds"=>17.,_=>native,
        },|_|None);
        assert_eq!(max_active(&b.table.mobys[id]),3);
        assert_eq!(p::i32(&b.table.mobys[id].pvars,LIFE),crate::hero::physics::ticks(17*60));
    }
    #[test]
    fn every_flame_mine_model_uses_its_own_scale_and_the_captured_percentage_for_all_grades() {
        for model in 0..=2 {
            for percent in [50.,100.,150.,200.,500.] {
                let (mut b,id)=dropped_mine(12.,0.02);
                let base=b.table.mobys[id].scale;
                let config=|key:&str,native|match key {"flame_mine_enabled"=>1.,"flame_mine_model"=>model as f32,"flame_mine_size_percent"=>percent,_=>native};
                snapshot_configured(&mut b.table.mobys[id],config,|_|None);
                let selected_native_scale=[13.,29.,71.][model];
                let expected=if model==0 {base*percent/100.} else {selected_native_scale*percent/100.};
                assert!((visual_scale(&b.table.mobys[id],selected_native_scale).unwrap()-expected).abs()<0.00001,"model {model}, size {percent}");
                snapshot_configured(&mut b.table.mobys[id],config,|_|None);
                assert!((visual_scale(&b.table.mobys[id],selected_native_scale).unwrap()-expected).abs()<0.00001,"repeated snapshot compounded size");
                b.tick(None);
                assert!((visual_scale(&b.table.mobys[id],selected_native_scale).unwrap()-expected).abs()<0.00001,"flight lost captured size");
                assert_eq!(b.table.mobys[id].o_class,CLASS);
                assert_eq!(p::ff(&b.table.mobys[id].pvars,SIZE_PERCENT),percent);
            }
        }
        assert_eq!(visual_scale(&crate::moby_runtime::Moby::default(),29.),None);
    }

    #[test]
    fn visual_models_keep_native_mine_identity_and_capture_only_their_own_palette() {
        for model in 0..=2 {
            let (mut b,id)=dropped_mine(12.,0.02);
            let colors=|key:&str|match key {
                "mine_model_color"|"flame_core_color"|"flame_outer_color"|"burn_flame_color"=>Some(0xff0000),
                "mine_bomb_color"|"mine_bomb_core_color"|"mine_bomb_outer_color"|"mine_bomb_burn_color"=>Some(0x00ff00),
                "mine_sphere_color"|"mine_sphere_core_color"|"mine_sphere_outer_color"|"mine_sphere_burn_color"=>Some(0x0000ff),_=>None};
            snapshot_configured(&mut b.table.mobys[id],|key,native|match key {"flame_mine_enabled"=>1.,"flame_mine_model"=>model as f32,_=>native},colors);
            let expected=Some([0xff0000,0x00ff00,0x0000ff][model]);
            let m=&b.table.mobys[id];assert_eq!(m.o_class,CLASS);
            assert_eq!(body_color(m),expected);assert_eq!(flame_colors(m),[expected;2]);assert_eq!(burn_color(m),expected);
            assert_eq!(visual_class(m),Some([CLASS,crate::moby_update::classes::bomb::BOMB_CLASS,crate::moby_update::classes::doom_canister::CLASS][model]));
            b.tick(None);assert_eq!(b.table.mobys[id].glow,0xff000000|expected.unwrap());
            // A default color in the selected palette never borrows another model's red.
            snapshot_configured(&mut b.table.mobys[id],|key,native|match key {"flame_mine_enabled"=>1.,"flame_mine_model"=>model as f32,_=>native},|key|if key=="unused_color" {Some(0xff0000)}else{None});
            assert_eq!(flame_colors(&b.table.mobys[id]),[None;2]);
        }
    }
    #[test]
    fn double_size_mine_exits_initial_floor_overlap_without_bouncing_underground() {
        // Slow rise exercises the hull-sphere fallback; faster rise crosses
        // the floor with the bottom sweep. Both previously flipped downward.
        for vz in [0.005, 0.08] {
            let (mut b, id) = dropped_mine(8.45, vz);
            snapshot_configured(&mut b.table.mobys[id], |key, native| match key {
                "flame_mine_enabled" => 1.0,
                "flame_mine_size_percent" => 200.0,
                _ => native,
            }, |_| None);
            b.tick(None);
            assert!(!b.table.mobys[id].is_deleted());
            assert!(b.table.mobys[id].position[2] >= 8.49, "mine below floor: {:?}", b.table.mobys[id].position);
            assert!(p::v4f(&b.table.mobys[id].pvars, pv::VEL)[2] > 0.0, "launch was reversed");
            for _ in 0..180 {
                b.tick(None);
                if b.table.mobys[id].state & LANDED != 0 { break; }
            }
            assert!(!b.table.mobys[id].is_deleted());
            assert_ne!(b.table.mobys[id].state & LANDED, 0);
            assert!(b.table.mobys[id].position[2] >= 8.79);
        }
    }
    #[test]
    fn size_is_idempotent_and_body_tracks_flame_rotation() {
        let (mut b,id)=dropped_mine(12.,-10.);let base=b.table.mobys[id].scale;
        let config=|key:&str,native|match key {"flame_mine_enabled"=>1.,"flame_mine_size_percent"=>150.,"mine_flame_rotation_degrees_per_second"=>90.,_=>native};
        snapshot_configured(&mut b.table.mobys[id],config,|_|None);snapshot_configured(&mut b.table.mobys[id],config,|_|None);
        assert!((b.table.mobys[id].scale-base*1.5).abs()<0.000001);
        b.tick(None);b.tick(None);assert_ne!(b.table.mobys[id].state&LANDED,0);assert!(b.table.mobys[id].position[2]>=8.59);
        let before=b.table.mobys[id].rotation[2];b.tick(None);let m=&b.table.mobys[id];assert_eq!(m.rotation[2],p::ff(&m.pvars,ANGLE));
        assert!((c::add_rot(before,std::f32::consts::FRAC_PI_2/60.)-m.rotation[2]).abs()<0.0001);
        p::set_ff(&mut b.table.mobys[id].pvars,ROTATION,0.);let before=b.table.mobys[id].rotation[2];b.tick(None);assert_eq!(b.table.mobys[id].rotation[2],before);
    }
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

#[cfg(test)]
mod reload_tests {
    #[test]
    fn death_table_reset_detaches_flame_mine_lights_and_allows_safe_slot_reuse() {
        let mut svc=crate::moby_update::services::Services::new();
        let light=crate::point_lights::PointLight {color:[1.,0.5,0.1],intensity:0.5,pos:[20.,20.,9.],radius:8.};
        let mut lists=crate::point_lights::WorldLightLists::new(&crate::point_lights::WorldSpheres {ties:vec![[20.,20.,9.,1.]],..Default::default()});
        let spheres=crate::point_lights::WorldSpheres {ties:vec![[20.,20.,9.,1.]],..Default::default()};
        let slot=svc.point_lights.alloc(light,0.).unwrap();
        let generation=svc.point_lights.generation[slot];
        assert!(lists.update(&svc.point_lights,&spheres));
        assert_ne!(lists.ties[0],crate::point_lights::NO_LIGHTS);
        svc.reset_moby_transients();
        assert_eq!(svc.point_lights.active().count(),0);
        assert!(lists.update(&svc.point_lights,&spheres));
        assert_eq!(lists.ties[0],crate::point_lights::NO_LIGHTS);
        assert_eq!(svc.point_lights.generation[slot],generation);
        assert_eq!(svc.point_lights.alloc(light,0.),Some(slot));
        assert_ne!(svc.point_lights.generation[slot],generation);
        svc.reset_moby_transients();svc.reset_moby_transients();
        assert_eq!(svc.point_lights.active().count(),0);
    }
}
