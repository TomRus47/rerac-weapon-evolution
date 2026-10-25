//! Encounter-local simulation clocks. Native collision sweeps run on every
//! substep, so faster enemies/shots never teleport through collision geometry.
//! Range adapters name documented pvar fields rather than guessing at bytes.
//! No clocks are serialized and no balance values are written back to presets.
use std::collections::HashMap;
use crate::moby_runtime::{Moby, MobyId};
use crate::moby_update::services::{pvar as p, World};

#[derive(Clone, Debug, Default)]
pub struct Motion {
    clocks: HashMap<MobyId, Clock>,
}
#[derive(Clone, Debug)]
struct Clock { class:i16, remainder:f32, range_applied:bool, sample_tick:Option<u64> }
impl Motion {
    pub fn forget(&mut self,id:MobyId) { self.clocks.remove(&id); }
    fn steps(&mut self,id:MobyId,class:i16,speed:f32)->(usize,bool) {
        let c=self.clocks.entry(id).or_insert(Clock{class,remainder:0.,range_applied:false,sample_tick:None});
        if c.class!=class {*c=Clock{class,remainder:0.,range_applied:false,sample_tick:None};}
        c.remainder+=speed;
        let n=c.remainder.floor() as usize;
        c.remainder-=n as f32;
        let first=!c.range_applied;
        c.range_applied=true;
        (n.min(16),first)
    }
}
/// Native periodic displacement probes must not compare the second substep
/// against a sample written by the first substep in the same world tick.
pub fn periodic_sample(w:&mut World,id:MobyId)->bool {
    if w.svc.bomb_evo.world_settings.enemy_speed_multiplier<=1. {return true;}
    let class=w.m(id).o_class;let tick=w.counter;
    let c=w.svc.motion.clocks.entry(id).or_insert(Clock{class,remainder:0.,range_applied:false,sample_tick:None});
    if c.class!=class {*c=Clock{class,remainder:0.,range_applied:false,sample_tick:None};}
    if c.sample_tick==Some(tick) {return false;}
    c.sample_tick=Some(tick);true
}
/// Hostile flight actors, including destructible shots, all boss projectiles,
/// rolling bombs and radial attacks. Friendly ship/cannon/helper shots are not
/// included. Class 435 switches between scenery bomber and hostile missile.
pub fn projectile(m:&Moby)->bool {
    matches!(m.o_class,41|49|50|82|83|184|248|409|425|564|568|624|627|628|681|686|696|697|698|700|722|819|848|880|882|922|934|983|1017|1046|1049|1208|1233|1238|1257|1292|1368|1634)
        || m.o_class==435 && matches!(m.state,1|2|5)
}
pub fn enemy(m:&Moby)->bool {
    !projectile(m) && (crate::moby_update::real_hp::donor_class(m.o_class)
        || matches!(m.o_class,1126|1454|1394|1274|1584|733))
}
/// Hoven's turret defense keeps its authored flight speeds while mounted.
/// The live rider state and battle flag are checked every tick. No settings,
/// projectile velocities, range lifetimes or clock remainders are overwritten.
fn hoven_turret_native_speed(w:&World,id:MobyId)->bool {
    use crate::moby_update::classes::units::{hoven_turret,gun_shot};
    if w.svc.level!=12 || w.hero.state!=0x32 || w.svc.units.word(hoven_turret::lw::ACTIVE)==0 {return false;}
    match w.m(id).o_class {
        326|336|409=>true,
        184=>w.m(id).pvars.get(gun_shot::pvo::OWNER..gun_shot::pvo::OWNER+4)
            .and_then(|raw|usize::try_from(i32::from_le_bytes(raw.try_into().unwrap())-1).ok())
            .and_then(|owner|w.table.mobys.get(owner)).is_some_and(|m|matches!(m.o_class,1274|1278)),
        _=>false,
    }
}
/// Story projectiles use authored targets and timing, not enemy combat
/// balance. Identify the exact role/owner, never exempt a whole shared hostile
/// projectile family across other levels.
fn story_flight(w:&World,id:MobyId)->bool {
    let m=w.m(id);
    match (w.svc.level,m.o_class) {
        (14,922)=>true, // Oltanis's statue missile; class has no combat role.
        (14,921)=>m.pvars.len()>=0xa8 && p::i32(&m.pvars,0x8c)>0,
        (1,688)=>m.pvars.len()>=0x174 && p::i32(&m.pvars,0x170)>=0,
        (8,435)=>matches!(m.state,0|3|4) && m.pvars.len()>=0x98,
        (1,686)=>m.pvars.get(0x30..0x34)
            .and_then(|b|usize::try_from(i32::from_le_bytes(b.try_into().unwrap())).ok())
            .and_then(|owner|w.table.mobys.get(owner))
            .is_some_and(|ship|ship.o_class==688 && ship.pvars.len()>=0x174
                && p::i32(&ship.pvars,0x170)>=0),
        _=>false,
    }
}
/// A native distance cutoff (not a collision/world boundary).
pub fn range(w:&World,id:MobyId,native:f32)->f32 {
    if projectile(w.m(id)) && !story_flight(w,id) {native*w.svc.bomb_evo.world_settings.enemy_projectile_range_multiplier} else {native}
}
fn field_f32(m:&mut Moby,offset:usize,k:f32) {
    if m.pvars.len()>=offset+4 { let v=p::ff(&m.pvars,offset);if v.is_finite(){p::set_ff(&mut m.pvars,offset,v*k);} }
}
fn timer32(m:&mut Moby,offset:usize,k:f32) {
    if m.pvars.len()>=offset+4 { let v=p::i32(&m.pvars,offset);if v>0{p::set_i32(&mut m.pvars,offset,((v as f64*k as f64).round().clamp(1.,i32::MAX as f64)) as i32);} }
}
fn timer16(m:&mut Moby,offset:usize,k:f32) {
    if m.pvars.len()>=offset+2 { let v=p::i16(&m.pvars,offset);if v>0{p::set_i16(&mut m.pvars,offset,(v as f32*k).round().clamp(1.,i16::MAX as f32) as i16);} }
}
fn horizontal(m:&mut Moby,offset:usize,k:f32) {field_f32(m,offset,k);field_f32(m,offset+4,k);}
/// Timed straight shots gain lifetime; ballistic shots gain horizontal reach
/// while preserving their gravity and impact/fuse behavior. The speed clock is
/// independent: a 2x-speed shot traverses the same trajectory in half the time.
fn extend_range(m:&mut Moby,k:f32) {
    if k==1. {return;}
    match m.o_class {
        41|425 => timer16(m,0x10,k),
        49|409|848 => timer32(m,0x14,k),
        50 => timer32(m,0x20,k),
        82 => timer32(m,0x28,k),
        83 => timer32(m,0x70,k),
        184 => {timer16(m,0x1c,k);timer16(m,0x1e,k);},
        934 => timer32(m,0x38,k),
        1017 => {timer32(m,0x14,k);timer32(m,0x18,k);},
        1238 => {timer32(m,0x2c,k);field_f32(m,0x28,k);},
        1233 => field_f32(m,0x28,k),
        819|1292 => timer32(m,0x18,k),
        1368 => timer32(m,0x38,k),
        248 => horizontal(m,0x10,k),
        568 => {horizontal(m,0x180,k);timer32(m,0x194,k);},
        983 => timer32(m,0x34,k),
        1634 => {timer16(m,0x18,k);timer16(m,0x1a,k);},
        624 => field_f32(m,0x28,k),
        1046 => field_f32(m,0x04,k),
        627|686|722|882|1049|1208|1257 => horizontal(m,0,k),
        _ => {}, // Distance tests and continuously aimed beams use range().
    }
}
/// The neutral path does exactly one native update and allocates nothing. A
/// non-gameplay scene always uses the native clock, including boss cutaways.
pub fn run(w:&mut World,id:MobyId,native:fn(&mut World,MobyId)) {
    let s=w.svc.bomb_evo.world_settings;
    if w.svc.game_mode!=0 || w.m(id).state>=0x80 || story_flight(w,id) {
        native(w,id);return;
    }
    let shot=projectile(w.m(id));
    let speed=if hoven_turret_native_speed(w,id) {1.}
        else if shot{s.enemy_projectile_speed_multiplier}else if enemy(w.m(id)) {s.enemy_speed_multiplier}else{1.};
    if speed==1. && (!shot || s.enemy_projectile_range_multiplier==1.) {native(w,id);return;}
    // Capture is driven once per hero tick. Repeating an enemy's carried
    // update consumes its one-tick pull lease twice, releasing it every frame.
    // Fired cannon ammunition is player-owned and uses this clock as well.
    if crate::moby_update::creature::react::cannon_controls(w,id) {
        w.svc.motion.forget(id);
        native(w,id);return;
    }
    let class=w.m(id).o_class;
    let (n,first)=w.svc.motion.steps(id,class,speed);
    if shot && first {
        extend_range(w.mm(id),s.enemy_projectile_range_multiplier);
        // Attached native glows must last as long as their extended flight.
        // They still die immediately with their owning moby.
        let factor=s.enemy_projectile_range_multiplier/speed;
        if factor!=1. {if let Some(sys)=w.particles.as_deref_mut() {
            use crate::particles::{rec,type26};
            for record in &mut sys.pool.recs {
                if type26::anchor_of(record).is_some_and(|(owner,_)|owner==id) {
                    let life=rec::u32(record,0x28);
                    let scaled=(life as f32*factor).round().clamp(1.,32767.) as u32;
                    rec::set_u32(record,0x28,scaled);
                    let timer=rec::i16(record,0x24);
                    rec::set_i16(record,0x24,(timer as f32*factor).round().clamp(1.,32767.) as i16);
                }
            }
        }}
    }
    // Initialization must happen once even at fractional speeds. It does not
    // spend a second native tick or repeat an init-only spawn on fast clocks.
    if !shot && w.m(id).state==0 {native(w,id);return;}
    for _ in 0..n {
        if w.m(id).state>=0x80 {break;}
        native(w,id);
        // A class can enter a cutaway mid-substep. Never run it again as combat.
        if w.svc.game_mode!=0 {break;}
        if crate::moby_update::creature::react::cannon_controls(w,id) {
            w.svc.motion.forget(id);break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn native_collision_projectile_runs_separate_speed_and_range_clocks() {
        use crate::moby_runtime::{ClassInfo,MobyTable};
        use crate::moby_update::{services::{Services,ClassTable},classes::path_enemy,scheduler};
        use crate::{hero::Hero,rng::Rng};
        let simulate=|speed:f32,reach:f32| {
            let mut classes=ClassTable::default();
            classes.classes.insert(722,(ClassInfo{update_fn:scheduler::port_update_fn(722),scale:1.,..Default::default()},None));
            let mut table=MobyTable::new(vec![Moby::init_instance(0,459,None)],32);
            let hero=Hero::new();let mut rng=Rng::new();let mut svc=Services::new();
            svc.bomb_evo.world_settings.enemy_projectile_speed_multiplier=speed;
            svc.bomb_evo.world_settings.enemy_projectile_range_multiplier=reach;
            let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
            let id=path_enemy::spawn_shot(&mut w,0,[10.,10.,10.,0.],[0.2,0.,0.1,0.],100).unwrap();
            scheduler::run_moby(&mut w,id);
            (w.m(id).position,p::i32(&w.m(id).pvars,0x14))
        };
        let (native,life)=simulate(1.,1.);let (fast,fast_life)=simulate(2.,1.);
        assert!((native[0]-10.2).abs()<0.0001);assert!((fast[0]-10.4).abs()<0.0001);
        assert_eq!((life,fast_life),(99,98));
        let (long,long_life)=simulate(1.,2.);assert!((long[0]-10.4).abs()<0.0001);
        assert_eq!(long_life,99,"ballistic range preserves the native fuse");
    }
    #[test] fn every_registered_hostile_shot_family_has_a_motion_policy() {
        use crate::moby_update::classes::units as u;
        let families:&[&[i16]]=&[&u::explosive_tank::FIREBALL_CLASSES,&u::hop_gunner::SHOT_CLASSES,
            &u::gemlik_turret::SHOT_CLASSES,&u::popup_turret::SHOT_CLASSES,&u::umbris_lobber::SHOT_CLASSES,
            &u::aridia_cannon::SHELL_CLASSES,&u::hover_ship::FLAME_CLASSES,&u::veldin_tank::SHELL_CLASSES,
            &u::dropship::SHOT_CLASSES,&u::drop_trooper::SHOT_CLASSES,&u::veldin_shots::LOB_CLASSES,
            &u::veldin_shots::RING_CLASSES,&u::veldin_shots::BEAM_CLASSES,&u::veldin_shots::AURA_CLASSES,
            &u::quartu_small::BOMB_CLASSES,&u::quartu_guard::SHOT_CLASSES,&u::oltanis_carrier::MISSILE_CLASSES,
            &u::gemlik_launcher::MISSILE_CLASSES,&u::orxon_drone::SHOT_CLASSES,&u::batalia_tank::SHELL_CLASSES,
            &u::batalia_grenadier::SHOT_CLASSES,&u::umbris_beast_fx::RING_CLASSES,&u::umbris_beast_fx::GLOB_CLASSES,
            &u::umbris_turret::SHOT_CLASSES,&u::qwark_ship_parts::MISSILE_CLASSES,&u::qwark_ship_parts::MINE_CLASSES,
            &u::fleet_turret::SHOT_CLASSES,&u::hoven_drone::SHOT_CLASSES,&u::gun_shot::CLASSES,&u::fighter_shot::CLASSES];
        for family in families {for &class in *family {assert!(projectile(&Moby::init_instance(1,class,None)),"missing class {class}");}}
    }
    #[test] fn hoven_turret_speed_is_scoped_and_restored_live_for_existing_projectiles() {
        use crate::{hero::Hero,moby_runtime::MobyTable,rng::Rng};
        use crate::moby_update::{services::{Services,ClassTable},classes::units::hoven_turret};
        fn step(w:&mut World,id:MobyId){w.mm(id).position[0]+=1.;}
        let mut mobys:Vec<_>=[326,336,409,184,1278,184,255,459,722].into_iter().enumerate().map(|(id,class)| {
            let mut m=Moby::init_instance(id as u32,class,None);m.state=1;m.pvars=vec![0;0x40];m
        }).collect();
        p::set_i32(&mut mobys[3].pvars,0x18,5);p::set_i32(&mut mobys[5].pvars,0x18,7);
        let mut table=MobyTable::new(mobys,4);let mut hero=Hero::new();let mut rng=Rng::new();
        let classes=ClassTable::default();let mut svc=Services::new();svc.level=12;
        svc.bomb_evo.world_settings.enemy_speed_multiplier=2.5;
        svc.bomb_evo.world_settings.enemy_projectile_speed_multiplier=3.;
        svc.units.set_word(hoven_turret::lw::ACTIVE,1);
        for mounted in [false,true,false] {
            hero.state=if mounted {0x32}else{0};
            let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
            let before:Vec<_>=w.table.mobys.iter().map(|m|m.position[0]).collect();
            for counter in 0..20 {w.counter=counter;for id in [0,1,2,3,5,7,8] {run(&mut w,id,step);}}
            for id in [0,2,3] {let expected=if mounted {20.}else if id==0 {50.}else{60.};assert_eq!(w.m(id).position[0]-before[id],expected,"id {id}, mounted {mounted}");}
            assert_eq!(w.m(5).position[0]-before[5],60.,"unrelated gun shots keep their modifier");
            assert_eq!(w.m(7).position[0]-before[7],50.);assert_eq!(w.m(8).position[0]-before[8],60.);
            assert_eq!(w.svc.bomb_evo.world_settings.enemy_projectile_speed_multiplier,3.);
        }
        hero.state=0x32;
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
        w.svc.level=11;assert!(!hoven_turret_native_speed(&w,0));
        w.svc.level=12;w.svc.units.set_word(hoven_turret::lw::ACTIVE,0);assert!(!hoven_turret_native_speed(&w,0));
        w.svc.units.set_word(hoven_turret::lw::ACTIVE,1);
        w.svc.bomb_evo.world_settings.enemy_speed_multiplier=0.5;
        w.svc.bomb_evo.world_settings.enemy_projectile_speed_multiplier=0.5;
        for id in [0,2,3] {let x=w.m(id).position[0];run(&mut w,id,step);assert_eq!(w.m(id).position[0]-x,1.);}
    }
    #[test] fn fractional_clocks_are_bounded_and_reset_on_reuse() {
        let mut s=Motion::default();let mut n=0;
        for _ in 0..100 {n+=s.steps(2,44,1.25).0;}
        assert_eq!(n,125);s.forget(2);assert_eq!(s.steps(2,44,1.25),(1,true));
        assert_eq!(s.steps(2,45,2.),(2,true));
        let mut s=Motion::default();assert_eq!((0..10).map(|_|s.steps(1,44,0.5).0).sum::<usize>(),5);
    }
    #[test] fn projectiles_are_not_also_accelerated_as_enemies_and_all_bosses_are_included() {
        for class in [41,49,50,82,83,184,248,409,425,564,568,624,627,628,681,686,696,697,698,700,722,819,848,880,882,922,934,983,1017,1046,1049,1208,1233,1238,1257,1292,1368,1634] {
            let m=Moby::init_instance(1,class,None);assert!(projectile(&m),"{class}");assert!(!enemy(&m));
        }
        for class in [1051,1106,388,1422,459,427,44,1126,1454] {assert!(enemy(&Moby::init_instance(1,class,None)));}
        for class in [0,10,11,13,14,15,16,74,121,153,172,305,457,479,806,1009,1633,1818] {
            let m=Moby::init_instance(1,class,None);assert!(!projectile(&m));assert!(!enemy(&m),"{class}");
        }
    }
    #[test] fn documented_range_fields_are_typed_and_do_not_change_damage_or_owner() {
        let mut m=Moby::init_instance(1,1238,None);m.pvars=vec![0;0x40];
        p::set_i32(&mut m.pvars,0x20,17);p::set_ff(&mut m.pvars,0x28,30.);p::set_i32(&mut m.pvars,0x2c,60);
        extend_range(&mut m,2.);assert_eq!(p::ff(&m.pvars,0x28),60.);assert_eq!(p::i32(&m.pvars,0x2c),120);assert_eq!(p::i32(&m.pvars,0x20),17);
        m.o_class=248;p::set_ff(&mut m.pvars,0x10,2.);p::set_ff(&mut m.pvars,0x14,-3.);p::set_ff(&mut m.pvars,0x18,4.);
        extend_range(&mut m,2.);assert_eq!(p::v4f(&m.pvars,0x10)[..3],[4.,-6.,4.]);
    }
}

#[cfg(test)]
mod v80_story_tests {
    use super::*;
    #[test]
    fn story_shells_use_native_clocks_and_ranges_while_shared_hostile_shots_stay_scaled() {
        use crate::{hero::Hero,moby_runtime::MobyTable,rng::Rng,moby_update::{Services,ClassTable}};
        let mut ship=Moby::zeroed();ship.o_class=688;ship.pvars=vec![0;0x174];
        p::set_i32(&mut ship.pvars,0x170,9);
        let mut shell=Moby::zeroed();shell.o_class=686;shell.state=1;shell.pvars=vec![0;0x40];
        p::set_i32(&mut shell.pvars,0x30,0);p::set_ff(&mut shell.pvars,0,1.);
        let mut table=MobyTable::new(vec![ship,shell],0);let hero=Hero::new();let mut rng=Rng::new();let classes=ClassTable::default();let mut svc=Services::new();
        svc.level=1;svc.bomb_evo.world_settings.enemy_projectile_speed_multiplier=2.;
        svc.bomb_evo.world_settings.enemy_projectile_range_multiplier=2.;
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
        fn move_once(w:&mut World,id:MobyId){let dx=p::ff(&w.m(id).pvars,0);w.mm(id).position[0]+=dx;}
        assert!(story_flight(&w,0),"the bridge flyby owner retains its native shot/camera timing");
        assert!(story_flight(&w,1));assert_eq!(range(&w,1,10.),10.);
        run(&mut w,1,move_once);assert_eq!(w.m(1).position[0],1.);assert_eq!(p::ff(&w.m(1).pvars,0),1.);
        p::set_i32(&mut w.mm(0).pvars,0x170,-1);
        assert!(!story_flight(&w,0));
        assert!(!story_flight(&w,1));assert_eq!(range(&w,1,10.),20.);
        run(&mut w,1,move_once);assert_eq!(w.m(1).position[0],5.);
        w.svc.level=14;w.mm(1).o_class=922;assert!(story_flight(&w,1));assert_eq!(range(&w,1,10.),10.);
        w.mm(0).o_class=921;
        p::set_i32(&mut w.mm(0).pvars,0x8c,1);assert!(story_flight(&w,0));
        p::set_i32(&mut w.mm(0).pvars,0x8c,0);assert!(!story_flight(&w,0));
        w.svc.level=8;w.mm(1).o_class=435;w.mm(1).pvars.resize(0x98,0);
        for state in [0,3,4]{w.mm(1).state=state;assert!(story_flight(&w,1));}
        for state in [1,2,5]{w.mm(1).state=state;assert!(!story_flight(&w,1));assert_eq!(range(&w,1,10.),20.);}
    }
}
