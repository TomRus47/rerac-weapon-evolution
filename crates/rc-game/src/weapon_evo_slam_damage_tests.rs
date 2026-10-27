//! Collision-to-native-HP contract for the experimental wrench blast.
use super::*;
use crate::{hero::{melee,items::{ItemData,HitSink}},moby_runtime::{Moby,ClassInfo},moby_update::{services::{World,pvar},ClassTable,Services},rng::Rng,pad::PadState};
use rc_formats::moby_collision::{MobyCollision,MobyCollPrim};
struct Hits {svc:Services,classes:ClassTable,ground:f32}
impl HitSink for Hits {
    fn sphere(&mut self,_:&mut MobyTable,_:Pf,_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<usize>{None}
    fn line(&mut self,_:&mut MobyTable,_:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>>{None}
    fn probe(&mut self,_:&mut MobyTable,a:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>)->Option<Option<[f32;3]>>{Some(Some([a[0].to_f32(),a[1].to_f32(),self.ground]))}
    fn world(&mut self,t:&mut MobyTable,h:&Hero,rng:&mut Rng,n:u64,f:&mut dyn FnMut(&mut World))->bool{let mut w=World::new(t,h,rng,&self.classes,&mut self.svc,n);f(&mut w);true}
}
#[test]
fn actual_blast_debits_enemy_hp_once_and_awards_only_wrench_xp(){
    for z in [10.0f32,0.0]{
        let mut h=Hero::new();h.state=0x14;h.air_ticks=1;h.pos=[Pf::f(100.),Pf::f(100.),Pf::f(z+2.),Pf::ONE];h.ground_z=Pf::ZERO;h.rot[2]=Pf::ZERO;
        h.melee.falling=Pf::ONE;h.melee.slam_landing_ticks=5;h.melee.attack_serial=7;h.melee.slam_blast=SlamBlast{enabled:true,damage_hp:200.,radius:0.5,..Default::default()};
        let mut enemy=Moby{o_class:577,index:1,uid_hi:65536,has_class:true,has_collision:true,state:1,mode:0x4020,scale:1.,position:[100.9,100.,z+0.2,1.],pvars:vec![0;0x238],..Default::default()};
        enemy.bsphere=[100.9*1024.,100.*1024.,(z+0.2)*1024.,1024.];enemy.rows=[[1.,0.,0.,0.],[0.,1.,0.,0.],[0.,0.,1.,0.],[0.;4]];
        pvar::set_i32(&mut enemy.pvars,0,0x20);pvar::set_ff(&mut enemy.pvars,0x20,5.);
        let mut box_=enemy.clone();box_.o_class=511;box_.index=2;box_.uid_hi=131072;box_.position[1]+=0.3;box_.bsphere[1]+=0.3*1024.;
        let mut distant=enemy.clone();distant.index=3;distant.uid_hi=196608;distant.position[0]+=5.;distant.bsphere[0]+=5.*1024.;
        let mut table=MobyTable::new(vec![Moby{o_class:0,has_class:true,state:1,..Default::default()},enemy,box_,distant],0);
        let mut hits=Hits{svc:Services::new(),classes:ClassTable::default(),ground:z};
        hits.svc.bomb_evo=crate::weapon_evo::Evolution::parse(include_str!("weapon_evo_release.ini")).unwrap();
        // The regression is independent of the author's private embedded balance.
        hits.svc.bomb_evo.enabled=true;hits.svc.bomb_evo.real_hp=true;
        hits.svc.bomb_evo.xp_per_damage=0.12;hits.svc.bomb_evo.world_settings=Default::default();
        let wrench=hits.svc.bomb_evo.extras.iter_mut().find(|t|t.id==8).unwrap();
        wrench.enabled=true;wrench.xp_required=[1000.;7];
        hits.classes.classes.insert(577,(ClassInfo{ty:5,has_collision:true,scale:1.,..Default::default()},None));
        let mut raw=[0u32;8];raw[0]=1|(0x8003<<16);raw[7]=0.3f32.to_bits();let blob=MobyCollision{prims:vec![MobyCollPrim{raw}],..Default::default()};
        hits.svc.set_moby_collision(vec![(577,blob.clone()),(511,blob)]);hits.svc.build_grid(&mut table);
        let data=ItemData::default();let pad=PadState::default();let env=ItemEnv{data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};let mut rng=Rng::new();
        melee::jump_attack_shockwave(&mut h,&mut table,&env,&mut hits,&mut rng);
        let visual=h.melee.slam_visual.expect("damage produces one matching visual snapshot");
        assert_eq!(visual.radius,h.melee.slam_blast.radius);
        assert!((visual.centre[0]-100.9).abs()<0.0001);
        assert!((visual.centre[2]-(z+0.05)).abs()<0.0001);
        assert_eq!(visual.born,1);
        assert!(hits.svc.fx.part_spawns.is_empty(),"no unbounded bomb particles");
        melee::jump_attack_shockwave(&mut h,&mut table,&env,&mut hits,&mut rng);
        assert_eq!(h.melee.slam_visual,Some(visual));
        let mut w=World::new(&mut table,&h,&mut rng,&hits.classes,&mut hits.svc,1);
        crate::moby_update::classes::critter::update(&mut w,1);
        assert_eq!(pvar::ff(&w.m(1).pvars,0x20),3.,"z={z}: configured 200 HP must be removed");
        assert!((w.svc.bomb_evo.extras.iter().find(|t|t.id==8).unwrap().xp-24.).abs()<0.01);
        assert_eq!(w.svc.bomb_evo.xp,0.);assert_eq!(pvar::ff(&w.m(3).pvars,0x20),5.);
        let hit=w.get_hit(2,u32::MAX,false).expect("crate receives native radial bomb hit");assert_eq!((hit.h2a,hit.damage),(121,Pf::f(2.)));
    }
}

#[test]
fn optional_collision_sink_preserves_headless_slam_contract(){
    let mut h=Hero::new();h.state=0x14;h.air_ticks=1;h.melee.falling=Pf::ONE;h.melee.slam_landing_ticks=5;
    h.melee.slam_blast=SlamBlast{enabled:true,..Default::default()};
    let data=ItemData::default();let pad=PadState::default();let env=ItemEnv{data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};
    let mut table=MobyTable::new(vec![Moby::default()],0);
    melee::jump_attack_shockwave(&mut h,&mut table,&env,&mut crate::hero::items::NoHits,&mut Rng::new());
    assert!(h.melee.slam_blast_emitted);
}
