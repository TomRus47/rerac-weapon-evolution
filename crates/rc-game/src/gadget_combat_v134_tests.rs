//! Regressions use real matrix revisions, nonzero world coordinates and floor collision.
use super::*;
use crate::hero::{physics,anim::RecordingAnim,items::ItemData,testkit};
use crate::moby_runtime::{Moby,ClassInfo};
use crate::moby_update::{ClassTable,Services,scheduler};
use crate::pad::{PadState,PadInput};
pub(super) fn actor(id:u32,class:i16,p:V)->Moby{
    let mut m=Moby::init_instance(id,class,Some(&ClassInfo{ty:5,scale:1.,has_collision:true,..Default::default()}));
    m.state=1;m.mode=0x4020;m.position=[p[0],p[1],p[2],1.];m.pvars=vec![0;0x238];m.update_dist=255;
    crate::moby_update::services::pvar::set_i32(&mut m.pvars,0,0x20);
    crate::moby_update::services::pvar::set_ff(&mut m.pvars,0x20,3.);m
}
pub(super) fn classes()->ClassTable{
    let mut c=ClassTable::default();for class in [0,459,577,660,1106]{c.classes.insert(class,(ClassInfo{ty:5,scale:1.,has_collision:true,..Default::default()},None));}c
}
pub(super) fn hero(item:i32)->Hero{
    let mut h=Hero::new();h.pos=from_f32x3([100.,100.,10.]);h.body_point=from_f32x3([100.,100.,10.7]);
    if item==26{h.cap_moby=Some(1);h.cap_hit=1;} // Simulated native body contact. 
    h.ground_z=Pf::f(10.);h.ground_point=h.pos;h.items.slot.id=item;h.items.slot.state=2;
    h.items.slot.item=Some(crate::hero::items::HandItem{o_class:match item{12=>208,26=>188,_=>1251},
        mstate:2,anim:Moby::default().anim,snapshot:None,scale:1.,position:[100.,100.,9.8],rows:[[0;4];3],hit_timer:0,flight:Default::default()});h
}
pub(super) struct Sink{pub(super) svc:Services,pub(super) classes:ClassTable}
impl HitSink for Sink{
    fn sphere(&mut self,_:&mut MobyTable,_:Pf,_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<usize>{None}
    fn line(&mut self,_:&mut MobyTable,_:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>>{None}
    fn world(&mut self,table:&mut MobyTable,h:&Hero,rng:&mut Rng,tick:u64,f:&mut dyn FnMut(&mut World))->bool{
        f(&mut World::new(table,h,rng,&self.classes,&mut self.svc,tick));true
    }
}
pub(super) fn fire(h:&mut Hero,table:&mut MobyTable,sink:&mut Sink,frame:i32,anim:&mut dyn AnimCtl){
    if h.items.slot.id==22&&frame==1{
        let data=ItemData::default();let pad=PadState::default();
        let env=ItemEnv{data:&data,pad:&pad,frame:0,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};
        item_tick(h,table,anim,&env,sink,&mut Rng::new());
    }
    let data=ItemData::default();let pad=PadState{held:button::CIRCLE,pressed:if frame==1{button::CIRCLE}else{0},..Default::default()};
    let env=ItemEnv{data:&data,pad:&pad,frame,hero_moby:0,coll:None,mobys:None,camera:Some(([100.,100.,11.],[1.,0.,0.])),camera_up:None,targets:&[]};
    item_tick(h,table,anim,&env,sink,&mut Rng::new());
}
#[test]
fn matrix_revision_is_not_a_lifetime_but_same_class_slot_reuse_is(){
    let h=hero(26);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.])],1);
    let mut svc=Services::new();let c=classes();let mut rng=Rng::new();let mut w=World::new(&mut table,&h,&mut rng,&c,&mut svc,1);
    let id=w.create_moby(459).unwrap();let lease=Lease::of(&w,id);
    for _ in 0..300{scheduler::rebuild_matrix(w.mm(id),None);assert!(lease.valid(&w));}
    w.delete_moby(id);assert!(!lease.valid(&w));w.counter=3;
    assert_eq!(w.create_moby(459),Some(id));assert!(!lease.same(&w));
}
#[test]
fn living_novalis_trooper_turns_for_180_frames_without_one_projectile_copy(){
    let mut h=hero(26);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],8);
    let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.drill_enabled=true;
    sink.svc.bomb_evo.gadget_settings.drill_dps=0.;let mut anim=RecordingAnim::default();let mut rng=Rng::new();
    for frame in 1..=180{
        if frame>1{
            let mut w=World::new(&mut table,&h,&mut rng,&sink.classes,&mut sink.svc,frame as u64);
            world_tick(&mut w);assert!(frozen(&w,1));
            // Real scheduler: animation/AI pause, but native matrix revision advances.
            scheduler::run_moby(&mut w,1);
        }
        fire(&mut h,&mut table,&mut sink,frame,&mut anim);
        assert!(h.gadget_combat.holding,"lost capture on frame {frame}");
        assert_eq!(table.mobys.iter().filter(|m|m.state<0x80).count(),2);
    }
    assert!(table.mobys[1].uid_hi&65535>100);
}
#[test]
fn a_confirmed_death_does_not_allocate_a_replacement_enemy(){
    let mut h=hero(26);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],8);
    let original_count=table.mobys.len();
    let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.drill_enabled=true;
    sink.svc.bomb_evo.gadget_settings.drill_dps=0.;let mut anim=RecordingAnim::default();
    fire(&mut h,&mut table,&mut sink,1,&mut anim);
    crate::moby_update::services::pvar::set_ff(&mut table.mobys[1].pvars,0x20,0.);
    table.delete(1,1);
    for frame in 2..=100{fire(&mut h,&mut table,&mut sink,frame,&mut anim);}
    assert_eq!(table.mobys.len(),original_count);
    assert_eq!(table.mobys.iter().filter(|m|m.state<0x80).count(),1);
    assert!(sink.svc.gadget_combat.drill.is_none());
}
#[test]
fn losing_or_replacing_a_captured_slot_does_not_invent_a_kill(){
    let mut h=hero(26);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],8);
    let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.drill_enabled=true;
    let mut anim=RecordingAnim::default();fire(&mut h,&mut table,&mut sink,1,&mut anim);
    sink.svc.gadget_lifetimes.spawned(1);fire(&mut h,&mut table,&mut sink,2,&mut anim);
    assert!(sink.svc.gadget_combat.drill.is_none());assert!(!h.gadget_combat.holding);
}
#[test]
fn bad_hydro_nozzle_is_above_real_floor_and_the_landing_freezes_novalis_enemies(){
    let mut h=hero(22);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[106.,100.,10.]),actor(2,577,[107.,100.,10.])],8);
    let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.ice_enabled=true;
    let mut anim=RecordingAnim::default();fire(&mut h,&mut table,&mut sink,1,&mut anim);
    let start=sink.svc.gadget_combat.ice[0].pos;assert!(start[2]>=10.6);assert!(start[0]>100.);
    let floor=testkit::floor(10.,20,40,20,40);
    assert!(crate::collision_query::coll_line(&floor,[100.,100.,11.],[100.,100.,9.],crate::collision_query::QueryFlags(2)).is_some());
    let mut rng=Rng::new();let mut hit=false;
    for frame in 2..=90{
        let mut w=World::new(&mut table,&h,&mut rng,&sink.classes,&mut sink.svc,frame);w.coll=Some(&floor);
        world_tick(&mut w);
        if !w.svc.gadget_combat.frost_bursts.is_empty(){
            let burst=w.svc.gadget_combat.frost_bursts[0];assert!((burst.centre[2]-10.).abs()<0.05);
            assert_eq!(burst.radius,6.);assert!(burst.centre[0]>103.);
            assert!(frozen(&w,1)&&frozen(&w,2));
            for _ in 0..4{scheduler::rebuild_matrix(w.mm(1),None);assert!(frozen(&w,1));}
            hit=true;break;
        }
    }
    assert!(hit,"crystal never reached the real floor");
}
#[test]
fn l1_look_physics_cannot_brake_the_cables_vertical_speed(){
    let mut h=hero(12);h.state=0x1e;h.f13f5=1;h.owned.set(12,true);
    h.gadget_combat.cable_pose=2;h.gadget_combat.grapple_step=Some([0.,0.,0.2]);
    let floor=testkit::floor(10.,20,40,20,40);let mut pad=PadState::default();let mut anim=RecordingAnim::default();let mut rng=Rng::new();
    let mut moby=actor(0,0,[100.,100.,10.]);let(rows,yaw)=testkit::cam_x();
    pad.update(Some(&PadInput::neutral().press(button::L1|button::CIRCLE).bytes()),false);
    for _ in 0..20{
        let env=physics::Env{coll:&floor,pad:&pad,cam_yaw:yaw,cam_rows:rows,mirror:false,death_z:Pf::ZERO,mobys:None,hero_moby:None,water:None,world:None};
        crate::hero::hero_update_with_combat_mode(&mut h,&mut moby,&env,&mut anim,&mut rng,&mut crate::hero::NoHeroSounds,false);
    }
    assert!((h.pos[2].to_f32()-14.).abs()<0.08,"L1 stole vertical travel: {:?}",h.pos);
    pad.update(Some(&PadInput::neutral().press(button::L1).bytes()),false);assert_eq!(grapple_step(&h,&pad),None);
}
#[test]
fn custom_cable_selects_native_launch_and_pull_poses_without_a_hang_state(){
    let mut h=hero(12);let mut anim=RecordingAnim::default();let mut rng=Rng::new();
    cable_animation(&mut h,&mut anim,&mut rng,1);assert_eq!(anim.v.seq_b,0x2f);
    h.gadget_combat.cable_pose=1;cable_animation(&mut h,&mut anim,&mut rng,2);assert_eq!(anim.v.seq_b,0x2f);
    assert_eq!(h.group,0);assert!(!matches!(h.state,0x26|0x2c));
}

#[test]
fn swingshot_pulls_a_novalis_trooper_through_real_matrix_revisions(){
    let mut h=hero(12);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[108.,100.,10.])],4);
    let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.swing_enabled=true;
    let mut anim=RecordingAnim::default();let mut rng=Rng::new();
    for frame in 1..=45{
        let mut w=World::new(&mut table,&h,&mut rng,&sink.classes,&mut sink.svc,frame as u64);
        world_tick(&mut w);scheduler::rebuild_matrix(w.mm(1),None);drop(w);
        fire(&mut h,&mut table,&mut sink,frame,&mut anim);
        if frame<5{assert!(h.gadget_combat.rope.is_some());}
    }
    assert!(table.mobys[1].position[0]<104.,"trooper was not pulled: {:?}",table.mobys[1].position);
    let w=World::new(&mut table,&h,&mut rng,&sink.classes,&mut sink.svc,45);
    assert!(frozen(&w,1));assert_eq!(w.table.mobys.iter().filter(|m|m.state<0x80).count(),2);
}
