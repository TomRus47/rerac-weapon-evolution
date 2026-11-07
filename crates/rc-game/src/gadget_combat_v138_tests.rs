//! Capture borrows physical collision, never native damage/XP or slot lifetime.
use super::*;
use super::v134_tests::{actor,classes,hero};
use crate::{hero::{anim::RecordingAnim,items::ItemData},moby_update::{Services,ClassTable,services::{SoundSink,SoundEvent}},pad::PadState};
#[derive(Default)]
struct Voices {plays:Vec<SoundEvent>,stops:Vec<(i32,MobyId)>}
impl SoundSink for Voices {
    fn play_class_sound(&mut self,event:&SoundEvent,_:&mut Rng)->i32{self.plays.push(*event);7}
    fn release(&mut self,slot:i32,id:MobyId){self.stops.push((slot,id));}
}
struct Sink {svc:Services,classes:ClassTable,voices:Voices}
impl Sink {fn new()->Self{
    let mut svc=Services::new();svc.bomb_evo.gadget_settings.drill_enabled=true;
    svc.bomb_evo.gadget_settings.drill_dps=0.;svc.bomb_evo.gadget_settings.drill_spin=0.;
    Self{svc,classes:classes(),voices:Voices::default()}
}}
impl HitSink for Sink {
    fn sphere(&mut self,_:&mut MobyTable,_:Pf,_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<usize>{None}
    fn line(&mut self,_:&mut MobyTable,_:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>>{None}
    fn world(&mut self,t:&mut MobyTable,h:&Hero,r:&mut Rng,c:u64,f:&mut dyn FnMut(&mut World))->bool{
        let mut w=World::new(t,h,r,&self.classes,&mut self.svc,c);w.sound=Some(&mut self.voices);f(&mut w);true
    }
}
fn tick(h:&mut Hero,t:&mut MobyTable,s:&mut Sink,frame:i32,fire:bool){
    let data=ItemData::default();let pad=PadState{held:if fire{button::CIRCLE}else{0},..Default::default()};
    let env=ItemEnv{data:&data,pad:&pad,frame,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};
    item_tick(h,t,&mut RecordingAnim::default(),&env,s,&mut Rng::new());
}
fn table()->MobyTable{MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],4)}
#[test]
fn physical_capture_is_temporary_and_the_motor_only_runs_with_a_living_target(){
    let mut h=hero(26);let mut t=table();let mut s=Sink::new();t.mobys[1].coll_disable=0x12;
    for frame in 1..=20{tick(&mut h,&mut t,&mut s,frame,true);assert!(h.gadget_combat.holding);assert!(!t.mobys[1].has_collision);}
    assert_eq!(s.voices.plays.len(),1);assert!(s.voices.stops.is_empty());assert_eq!(t.mobys[1].coll_disable,0x12);
    tick(&mut h,&mut t,&mut s,21,false);assert!(t.mobys[1].has_collision);assert_eq!(t.mobys[1].coll_disable,0x12);
    assert_eq!(s.voices.stops,[(7,0)]);assert!(s.svc.gadget_combat.drill_collision.is_none());
    // Continuing to hold in empty space must neither restart nor retain a motor.
    t.mobys[1].position[0]=150.;for frame in 22..26{tick(&mut h,&mut t,&mut s,frame,true);}
    assert_eq!(s.voices.plays.len(),1);assert_eq!(s.voices.stops.len(),1);
}
#[test]
fn empty_fire_never_starts_audio_and_already_absent_collision_stays_absent(){
    let mut h=hero(26);let mut t=table();let mut s=Sink::new();t.mobys[1].position[0]=150.;
    tick(&mut h,&mut t,&mut s,1,true);assert!(s.voices.plays.is_empty());assert!(!h.gadget_combat.holding);
    t.mobys[1].position[0]=103.;t.mobys[1].has_collision=false;tick(&mut h,&mut t,&mut s,2,true);
    assert!(h.gadget_combat.holding);tick(&mut h,&mut t,&mut s,3,false);assert!(!t.mobys[1].has_collision);
}
#[test]
fn defeat_stops_the_motor_without_reenabling_a_corpse_or_recycled_slot(){
    for recycled in [false,true]{
        let mut h=hero(26);let mut t=table();let mut s=Sink::new();tick(&mut h,&mut t,&mut s,1,true);
        if recycled{s.svc.gadget_lifetimes.spawned(1);t.mobys[1].has_collision=false;}
        else{crate::moby_update::services::pvar::set_ff(&mut t.mobys[1].pvars,0x20,0.);}
        tick(&mut h,&mut t,&mut s,2,true);assert!(!h.gadget_combat.holding);assert!(!t.mobys[1].has_collision);
        assert_eq!(s.voices.plays.len(),1);assert_eq!(s.voices.stops,[(7,0)]);
    }
}
#[test]
fn jump_unequip_disabled_mode_and_ratchet_death_restore_a_living_victim(){
    for cancel in 0..4{
        let mut h=hero(26);let mut t=table();let mut s=Sink::new();tick(&mut h,&mut t,&mut s,1,true);
        match cancel{0=>{h.air_ticks=1;h.group=2;},1=>h.items.slot.id=12,2=>s.svc.bomb_evo.gadget_settings.drill_enabled=false,_=>h.health=0}
        tick(&mut h,&mut t,&mut s,2,true);assert!(t.mobys[1].has_collision);assert_eq!(s.voices.stops,[(7,0)]);
    }
}
#[test]
fn world_cleanup_stops_a_dead_targets_voice_before_the_next_item_update(){
    let mut h=hero(26);let mut t=table();let mut s=Sink::new();tick(&mut h,&mut t,&mut s,1,true);
    crate::moby_update::services::pvar::set_ff(&mut t.mobys[1].pvars,0x20,0.);
    let mut rng=Rng::new();let mut w=World::new(&mut t,&h,&mut rng,&s.classes,&mut s.svc,2);w.sound=Some(&mut s.voices);world_tick(&mut w);
    assert!(w.svc.gadget_combat.drill.is_none());assert!(w.svc.gadget_combat.sound.is_none());drop(w);
    assert_eq!(s.voices.stops,[(7,0)]);assert!(!t.mobys[1].has_collision);
}
fn body_query(h:&Hero,t:&mut MobyTable,s:&mut Sink)->bool{
    use crate::collision_query::QueryFlags;
    let mut raw=[0;8];raw[0]=1|4<<8|0x8002<<16;raw[6]=716.8f32.to_bits();raw[7]=1024f32.to_bits();
    s.svc.coll_classes=std::sync::Arc::new(std::collections::HashMap::from([(459,rc_formats::moby_collision::MobyCollision{
        joint_counts:[0,0],prims:vec![rc_formats::moby_collision::MobyCollPrim{raw}],vertices:vec![],faces:vec![]})]));
    for m in &mut t.mobys{std::sync::Arc::make_mut(&mut s.svc.grid).register(m);}
    let mut rng=Rng::new();let w=World::new(t,h,&mut rng,&s.classes,&mut s.svc,30);
    let m=w.m(1);let point=add(pos(m),[0.3,0.,0.7]);
    w.coll_sphere(from_f32x3(point),Pf::f(0.5),QueryFlags::TWO_SIDED.0,Some(0)).is_some()
}
#[test]
fn real_body_collision_queries_skip_capture_and_hit_it_again_after_release(){
    let mut h=hero(26);let mut t=table();let mut s=Sink::new();
    t.mobys[1].rows[3]=[0.,0.,716.8,2048.];crate::moby_update::scheduler::rebuild_matrix(&mut t.mobys[1],None);
    assert!(body_query(&h,&mut t,&mut s));tick(&mut h,&mut t,&mut s,1,true);
    assert!(!body_query(&h,&mut t,&mut s));tick(&mut h,&mut t,&mut s,2,false);assert!(body_query(&h,&mut t,&mut s));
}
#[test]
fn collisionless_capture_still_debits_native_hp_and_nanotech_xp_once(){
    let mut h=hero(26);let mut t=table();let mut s=Sink::new();
    s.svc.bomb_evo.enabled=true;s.svc.bomb_evo.real_hp=true;s.svc.bomb_evo.xp_per_damage=0.1;
    s.svc.bomb_evo.world_settings.enemy_damage_by_health=true;s.svc.bomb_evo.world_settings.nanotech_xp_per_upgrade=100000.;
    s.svc.bomb_evo.gadget_settings.drill_dps=60.;
    for frame in 1..=60{
        tick(&mut h,&mut t,&mut s,frame,true);assert!(!t.mobys[1].has_collision);
        let mut rng=Rng::new();let mut w=World::new(&mut t,&h,&mut rng,&s.classes,&mut s.svc,frame as u64);
        assert!(!creature::damage::frozen_damage(&mut w,1));
    }
    assert!((crate::moby_update::services::pvar::ff(&t.mobys[1].pvars,0x20)-2.4).abs()<0.001);
    assert!((s.svc.bomb_evo.nanotech.xp-6.).abs()<0.001);
}
#[test]
fn each_ice_detonation_plays_one_native_hoven_one_shot_at_the_impact(){
    let h=hero(22);let mut t=table();let mut s=Sink::new();let mut rng=Rng::new();
    s.svc.gadget_combat.ice.push(Ice{pos:[103.,100.,10.],vel:[1.,0.,0.],travel:0.,age:0,
        settings:Settings{ice_range:0.001,ice_gravity:0.,freeze_seconds:2.,..Default::default()}});
    let count=t.mobys.len();let mut w=World::new(&mut t,&h,&mut rng,&s.classes,&mut s.svc,1);w.sound=Some(&mut s.voices);
    world_tick(&mut w);assert_eq!(w.svc.gadget_combat.frost_bursts.len(),1);let centre=w.svc.gadget_combat.frost_bursts[0].centre;
    assert!(frozen(&w,1));w.counter=2;world_tick(&mut w);drop(w);
    assert_eq!(s.voices.plays.len(),1);let e=s.voices.plays[0];assert_eq!(e.sound_class,crate::hero::damage::ICE_CLASS);
    assert_eq!((e.index,e.flags),(0,0));assert_eq!(e.pos,centre);assert!(s.voices.stops.is_empty());assert_eq!(t.mobys.len(),count);
}
