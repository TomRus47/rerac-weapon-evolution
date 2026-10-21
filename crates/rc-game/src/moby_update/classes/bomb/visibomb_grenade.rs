//! Replacement Visibomb grenades reuse native bomb flight, spin and explosions.
//! Extended pvars capture this shot's geometry/colors/HP and item-13 ownership.
use super::*;
use crate::moby_runtime::Moby;
const TAG_AT:usize=0x80;
const MAGIC:u32=0x56475233;
const RADIUS:usize=0x84;
const DAMAGE:usize=0x88;
const BLAST_COLOR:usize=0x8c;
const MODEL_COLOR:usize=0x90;
const ORIGIN:usize=0xa0;
const RANGE:usize=0xb0;
const AGE:usize=0xb4;
const LEN:usize=0xb8;
/// Quiet cleanup only; grenades never explode merely because this expires.
pub const FUSE:i32=1200;
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Settings {
    pub size_percent:f32,pub arc_height:f32,pub max_range:f32,pub fire_interval:i32,
    pub damage_hp:f32,pub explosion_radius:f32,pub explosion_color:Option<u32>,pub color:Option<u32>,
}
impl Default for Settings {
    fn default()->Self {Self {size_percent:250.,arc_height:3.,max_range:12.,fire_interval:30,
        damage_hp:600.,explosion_radius:1.,explosion_color:None,color:None}}
}
impl Settings {
    pub fn current()->Self {
        let get=|key,native|crate::weapon_evo::extra(13,key,native);
        Self {size_percent:get("grenade_size_percent",250.),arc_height:get("grenade_arc_height",3.),
            max_range:get("grenade_max_range",12.),fire_interval:get("grenade_fire_interval_ticks",30.) as i32,
            damage_hp:get("grenade_damage_hp",600.),explosion_radius:get("grenade_explosion_radius",1.),
            explosion_color:crate::weapon_evo::extra_color(13,"grenade_explosion_color"),
            color:crate::weapon_evo::extra_color(13,"grenade_color")}
    }
}
#[derive(Clone,Copy,Debug)]
pub struct Shot {pub from:[f32;3],pub velocity:[f32;3],pub range:f32}
/// Apex above the higher endpoint; native gravity is 11 units/s².
pub fn solve(from:[f32;3],mut target:[f32;3],height:f32,range:f32)->Shot {
    let dx=target[0]-from[0];let dy=target[1]-from[1];let distance=dx.hypot(dy);
    if distance>range {target[0]=from[0]+dx*range/distance;target[1]=from[1]+dy*range/distance;}
    let apex=from[2].max(target[2])+height;
    let up=(22.*(apex-from[2])).sqrt();let time=up/11.+(2.*(apex-target[2])/11.).sqrt();
    Shot {from,velocity:[(target[0]-from[0])/time*dt(),(target[1]-from[1])/time*dt(),up*dt()],range}
}
/// Swept circle boundary, shared by preview and actual flight.
pub fn limited_step(origin:[f32;3],from:[f32;3],next:[f32;3],range:f32)->([f32;3],bool) {
    let d=[from[0]-origin[0],from[1]-origin[1]];let n=[next[0]-origin[0],next[1]-origin[1]];
    if n[0]*n[0]+n[1]*n[1]<=range*range {return (next,false);}
    let v=[next[0]-from[0],next[1]-from[1]];let a=v[0]*v[0]+v[1]*v[1];
    let b=2.*(d[0]*v[0]+d[1]*v[1]);let c=d[0]*d[0]+d[1]*d[1]-range*range;
    if a<=0.000000000001 {return (next,true);}
    let t=if a>0. {((-b+(b*b-4.*a*c).max(0.).sqrt())/(2.*a)).clamp(0.,1.)}else{0.};
    (std::array::from_fn(|i|from[i]+(next[i]-from[i])*t),true)
}
pub fn is_grenade(m:&Moby)->bool {m.o_class==BOMB_CLASS && m.pvars.len()>=LEN && p::u32(&m.pvars,TAG_AT)==MAGIC}
pub(super) fn radius(m:&Moby)->f32 {p::ff(&m.pvars,RADIUS)}
pub(super) fn damage(m:&Moby)->f32 {p::ff(&m.pvars,DAMAGE)/100.}
pub(super) fn blast_color(m:&Moby)->Option<u32> {let c=p::u32(&m.pvars,BLAST_COLOR);(c!=u32::MAX).then_some(c)}
pub fn launch(w:&mut World,shot:Shot,settings:Settings)->Option<MobyId> {
    if w.classes.info(BOMB_CLASS).is_none_or(|info|info.scale<=0.) {return None;}
    let id=w.create_moby(BOMB_CLASS)?;
    init_held(&mut w.table.mobys[id],w.rng,shot.from,shot.velocity[1].atan2(shot.velocity[0]));
    let m=w.mm(id);m.pvars.resize(LEN,0);
    p::set_u32(&mut m.pvars,TAG_AT,MAGIC);p::set_ff(&mut m.pvars,RADIUS,settings.explosion_radius);
    p::set_ff(&mut m.pvars,DAMAGE,settings.damage_hp);
    p::set_u32(&mut m.pvars,BLAST_COLOR,settings.explosion_color.unwrap_or(u32::MAX));
    p::set_u32(&mut m.pvars,MODEL_COLOR,settings.color.unwrap_or(u32::MAX));
    p::set_v4f(&mut m.pvars,ORIGIN,[shot.from[0],shot.from[1],shot.from[2],1.]);p::set_ff(&mut m.pvars,RANGE,shot.range);
    p::set_v4f(&mut m.pvars,pv::VEL,[shot.velocity[0],shot.velocity[1],shot.velocity[2],0.]);
    m.scale*=settings.size_percent/100.;release(m,shot.from,None);
    w.svc.hits.real_hp.set_source_damage_hp(id,13,settings.damage_hp);
    apply_color(w,id);w.build_matrix(id);
    if let Some(hero)=w.hero_moby {
        let pos=w.m(hero).position;
        if let Some(hit)=w.coll_line(pv4([pos[0],pos[1],shot.from[2],1.]),pv4([shot.from[0],shot.from[1],shot.from[2],1.]),0,Some(hero)) {
            release_hit(w.mm(id),[hit.point[0],hit.point[1],hit.point[2]]);
        }
    }
    Some(id)
}
fn apply_color(w:&mut World,id:MobyId) {
    let color=p::u32(&w.m(id).pvars,MODEL_COLOR);if color==u32::MAX {return;}
    let rgb=crate::weapon_evo::rgb_float(color).map(|c|(c*127.).round() as u8);
    let m=w.mm(id);m.ambient=[rgb[0],rgb[1],rgb[2],0];m.glow=0xff000000|color;
}
/// Segment/unit-box intersection catches thin death cuboids between ticks.
fn segment_box(a:[f32;3],b:[f32;3])->bool {
    let (mut enter,mut leave)=(0.0f32,1.0f32);
    for i in 0..3 {
        let d=b[i]-a[i];
        if d.abs()<0.000001 {if a[i].abs()>1. {return false;}}
        else {let t0=(-1.-a[i])/d;let t1=(1.-a[i])/d;enter=enter.max(t0.min(t1));leave=leave.min(t0.max(t1));if enter>leave {return false;}}
    }
    true
}
fn kill_cuboids(w:&World)->Vec<i32> {
    let mut indices:Vec<_>=w.table.mobys.iter().filter(|m|m.o_class==1039 && !m.is_deleted() && m.pvars.len()>=256)
        .flat_map(|m|(0..64).map(move |i|p::i32(&m.pvars,i*4))).filter(|&i|i>=0).collect();
    indices.sort_unstable();indices.dedup();indices
}
fn void_step(w:&World,from:[f32;3],to:[f32;3],cuboids:&[i32])->bool {
    if to[2]<w.svc.death_z || !(2.0..=1021.).contains(&to[0]) || !(2.0..=1021.).contains(&to[1]) {return true;}
    cuboids.iter().any(|&i|w.svc.volumes.shape(rc_formats::volumes::ShapeKind::Cuboid,i)
        .is_some_and(|shape|segment_box(shape.local(from),shape.local(to))))
}

fn pit_contact(w:&World,from:[f32;3],to:[f32;3])->bool {
    w.coll_line(pv4([from[0],from[1],from[2],1.]),pv4([to[0],to[1],to[2],1.]),0x10,w.hero_moby)
        .is_some_and(|h|h.moby.is_none() && matches!(h.surface_id(),8|12))
}
pub(super) fn update(w:&mut World,id:MobyId) {
    if w.m(id).state==EXPLODING {super::exploding(w,id);return;}
    let age=p::i32(&w.m(id).pvars,AGE);
    let from=w.m(id).position;let velocity=p::v4f(&w.m(id).pvars,pv::VEL);let origin=p::v4f(&w.m(id).pvars,ORIGIN);
    let from=[from[0],from[1],from[2]];
    let next=std::array::from_fn(|i|from[i]+velocity[i]);
    let (end,expired)=limited_step([origin[0],origin[1],origin[2]],from,next,p::ff(&w.m(id).pvars,RANGE));
    if age>=FUSE || void_step(w,from,end,&kill_cuboids(w)) || pit_contact(w,from,end) {w.delete_moby(id);return;}
    if w.m(id).cmd!=0 {super::explode(w,id,[0.;4],[0.,0.,1.,0.]);return;}
    p::set_i32(&mut w.mm(id).pvars,AGE,age+1);
    // Native bomb flight still owns collision, gravity, spin and contact blast.
    // Its short fuse is not used for replacement-grenade detonation.
    p::set_i16(&mut w.mm(id).pvars,pv::LIFE,(300-age.min(11)) as i16);
    if expired {p::set_v4f(&mut w.mm(id).pvars,pv::VEL,[end[0]-from[0],end[1]-from[1],end[2]-from[2],0.]);}
    super::fly(w,id);
    if !w.m(id).is_deleted() && w.m(id).state==FLYING {
        if expired {let mut v=p::v4f(&w.m(id).pvars,pv::VEL);v[0]=0.;v[1]=0.;p::set_v4f(&mut w.mm(id).pvars,pv::VEL,v);}
        apply_color(w,id);
    }
}
/// Only a real contact earns a marker. A range boundary in empty air is not
/// terrain: stop horizontal travel there and keep integrating the fall.
pub fn preview(w:&mut World,shot:Shot) {
    if w.svc.game_mode!=0 {return;}
    let mut from=shot.from;let mut velocity=shot.velocity;
    let cuboids=kill_cuboids(w);
    for _ in 0..FUSE {
        let next=std::array::from_fn(|i|from[i]+velocity[i]);let (end,expired)=limited_step(shot.from,from,next,shot.range);
        if void_step(w,from,end,&cuboids) {return;}
        if let Some(hit)=w.coll_line(pv4([from[0],from[1],from[2],1.]),pv4([end[0],end[1],end[2],1.]),0x10,w.hero_moby) {
            if hit.moby.is_none() && matches!(hit.surface_id(),8|12) {return;}
            let point=[hit.point[0],hit.point[1],hit.point[2]];let normal=[hit.normal[0],hit.normal[1],hit.normal[2]];
            let camera=w.camera.map(Pf::to_f32);let point=crate::targeting::pull_toward([camera[0],camera[1],camera[2]],point);
            w.svc.reticles.register(w.counter,crate::targeting::Reticle{point,normal,style:&crate::targeting::BOMB_RETICLE});return;
        }
        from=end;if expired {velocity[0]=0.;velocity[1]=0.;}velocity[2]-=dt2()*11.;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_update::classes::bomb_water::tests::{Bench,pool};
    #[test]
    fn marker_uses_ground_below_hero_and_range_does_not_make_a_fake_surface() {
        let mut b=Bench::new(pool(8.,-100.),&[],[40.,40.,30.]);
        let shot=solve([20.,20.,30.],[28.,20.,8.],3.,8.);
        let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,1);w.coll=Some(&b.coll);w.camera=b.camera;
        preview(&mut w,shot);
        let marker=w.svc.reticles.of_tick(1).first().expect("missing ground marker");
        assert!(marker.point[2]>=8. && marker.point[2]<9.,"marker {:?}",marker.point);
        let id=launch(&mut w,shot,Settings::default()).unwrap();drop(w);
        for _ in 0..600 {b.tick(None);if b.table.mobys[id].state==EXPLODING {break;}}
        assert_eq!(b.table.mobys[id].state,EXPLODING);assert!((b.table.mobys[id].position[2]-8.).abs()<0.1);
    }
    #[test]
    fn water_contact_explodes_while_death_plane_removes_without_a_blast() {
        for water in [true,false] {
            let mut b=Bench::new(pool(8.,if water {10.}else{-100.}),&[],[40.,40.,30.]);
            if !water {b.svc.death_z=12.;}
            let shot=Shot{from:[20.,20.,13.],velocity:[0.,0.,-1.],range:12.};
            let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,1);
            let id=launch(&mut w,shot,Settings::default()).unwrap();drop(w);
            for _ in 0..10 {b.tick(None);if b.table.mobys[id].is_deleted() || b.table.mobys[id].state==EXPLODING {break;}}
            if water {assert_eq!(b.table.mobys[id].state,EXPLODING);assert!((b.table.mobys[id].position[2]-10.).abs()<0.1);}
            else {assert!(b.table.mobys[id].is_deleted());assert_eq!(b.alive(FIREBALL_CLASS),0);}
        }
        assert!(segment_box([-2.,0.,0.],[2.,0.,0.]));
        assert!(!segment_box([-2.,2.,0.],[2.,2.,0.]));
    }
    #[test]
    fn native_fuse_does_not_explode_a_replacement_grenade_in_empty_air() {
        let mut b=Bench::new(pool(8.,-100.),&[],[40.,40.,30.]);
        let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,1);
        let id=launch(&mut w,Shot{from:[20.,20.,900.],velocity:[0.;3],range:12.},Settings::default()).unwrap();drop(w);
        for _ in 0..305 {b.tick(None);}
        assert_eq!(b.table.mobys[id].state,FLYING);assert!(b.table.mobys[id].position[2]>100.);
    }
    #[test]
    fn ballistic_arc_and_swept_horizontal_range() {
        let shot=solve([20.,20.,10.],[32.,20.,8.],3.,12.);let t=12./(shot.velocity[0]/dt());
        assert!((10.+shot.velocity[2]/dt()*t-5.5*t*t-8.).abs()<0.0001);
        let (point,expired)=limited_step([20.,20.,10.],[31.,20.,11.],[34.,20.,8.],12.);
        assert!(expired);assert_eq!(point,[32.,20.,10.]);
        let (fall,expired)=limited_step([0.;3],[12.000001,0.,20.],[12.000001,0.,19.],12.);
        assert!(expired);assert_eq!(fall[2],19.);
    }
    fn launched()->(Bench,usize) {
        let mut b=Bench::new(pool(8.,-100.),&[],[40.,40.,20.]);
        let shot=solve([20.,20.,11.],[28.,20.,8.],3.,8.);
        let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,1);
        let id=launch(&mut w,shot,Settings{damage_hp:725.,explosion_radius:1.6,color:Some(0x0011ff),explosion_color:Some(0xff3300),..Default::default()}).unwrap();(b,id)
    }
    #[test]
    fn independent_size_colors_damage_and_native_spin() {
        let (mut b,id)=launched();let m=&b.table.mobys[id];assert!(is_grenade(m));
        assert_eq!(m.scale,2.5);assert_eq!(m.glow,0xff0011ff);assert_eq!(radius(m),1.6);
        assert_eq!(damage(m),7.25);assert_eq!(blast_color(m),Some(0xff3300));let rotation=m.rotation;
        b.tick(None);assert_ne!(b.table.mobys[id].rotation,rotation);assert_eq!(damage(&b.table.mobys[id]),7.25);
    }
    #[test]
    fn range_stops_horizontal_motion_without_air_detonation() {
        let (mut b,id)=launched();p::set_ff(&mut b.table.mobys[id].pvars,RANGE,0.01);b.tick(None);
        assert_eq!(b.table.mobys[id].state,FLYING);let pos=b.table.mobys[id].position;
        assert!((pos[0]-20.).hypot(pos[1]-20.)<=0.0101);assert!(b.svc.visibomb.missile.is_none());
    }
    #[test]
    fn only_explosions_award_visibomb_damage_once_per_instance() {
        let (mut b,id)=launched();let target=b.table.mobys.len();b.table.mobys.push(Moby{o_class:577,state:1,..Default::default()});
        let second=b.table.mobys.len();b.table.mobys.push(b.table.mobys[id].clone());
        let events=&mut b.svc.hits.real_hp;events.enabled=true;events.set_source_damage_hp(second,13,325.);
        let t=HitTemplate{attacker:Some(id),h1a:BOMB_CLASS as u16,b18:3,b19:2,flags:0x830000,damage:Pf::f(99.),..Default::default()};
        assert!(events.queue(&b.table,target,&t,true,[Pf::ZERO;4],0));assert!(events.take(target,u32::MAX).is_none());
        for _ in 0..3 {events.queue(&b.table,target,&t,false,[Pf::ZERO;4],0);}
        let first=events.take(target,u32::MAX).unwrap();let a=first.evolution.unwrap();assert_eq!(a.weapon,13);
        assert_eq!(first.damage.to_f32(),7.25);assert!(events.resolve_once(target,a));assert!(events.take(target,u32::MAX).is_none());
        events.queue(&b.table,target,&HitTemplate{attacker:Some(second),..t},false,[Pf::ZERO;4],0);
        let next=events.take(target,u32::MAX).unwrap();assert_eq!(next.damage.to_f32(),3.25);
        assert_ne!(next.evolution.unwrap().instance,a.instance);assert_eq!(next.evolution.unwrap().weapon,13);
    }
}
