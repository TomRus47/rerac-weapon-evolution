//! Real airborne palette recovery and native capsule contact, not radius-only capture.
use super::*;
use super::v134_tests::{actor,classes,hero,fire,Sink};
use super::v136_tests::swing_class;
use crate::{hero::{anim::{RatchetAnim,RecordingAnim},physics,testkit},moby_update::{Services,scheduler},pad::PadState};
use rc_formats::moby_anim::{self,SNAPSHOT_SEQ};
#[test]
fn releasing_the_cable_in_air_selects_a_decoded_fall_pose_until_landing(){
    let class=swing_class();let mut a=RatchetAnim::new(&class);let mut anim=a.ctl(&class);let mut h=hero(12);let mut rng=Rng::new();
    h.state=6;h.group=2;h.air_ticks=10;h.gadget_combat.cable_pose=2;
    apply_hero_pose(&mut h,&mut anim,&mut rng);assert_eq!(anim.view().seq_b,0x2f);
    prepare_hero_input(&mut h,&mut PadState::default());
    for _ in 0..25{apply_hero_pose(&mut h,&mut anim,&mut rng);anim.advance(Pf::ONE);
        assert_eq!(anim.view().seq_b,0xb);assert!(anim.pose_frame().is_some());
        assert_ne!(moby_anim::evaluate_posed(&class,&anim.a.state,anim.a.snapshot.as_ref(),&[],&[])[0],moby_anim::IDENTITY);
    }
}
#[test]
fn an_already_selected_native_fall_with_a_missing_source_snapshot_is_repaired(){
    let class=swing_class();let mut a=RatchetAnim::new(&class);let mut anim=a.ctl(&class);let mut h=hero(12);
    h.state=6;h.group=2;h.air_ticks=3;h.gadget_combat.pose_sequence=Some(0x2f);h.gadget_combat.pose_item=12;
    anim.a.state.seq_a=SNAPSHOT_SEQ;anim.a.state.seq_b=0xb;anim.a.snapshot=None;
    assert_eq!(moby_anim::evaluate_posed(&class,&anim.a.state,None,&[],&[])[0],moby_anim::IDENTITY);
    apply_hero_pose(&mut h,&mut anim,&mut Rng::new());assert!(anim.pose_frame().is_some());
    assert_eq!(anim.view().seq_b,0xb);assert_ne!(moby_anim::evaluate_posed(&class,&anim.a.state,anim.a.snapshot.as_ref(),&[],&[])[0],moby_anim::IDENTITY);
}
#[test]
fn native_hero_updates_keep_a_falling_body_and_clank_after_midair_release(){
    let class=swing_class();let mut a=RatchetAnim::new(&class);let mut anim=a.ctl(&class);let mut h=hero(12);
    h.pos=from_f32x3([100.,100.,15.]);h.state=6;h.group=2;h.air_ticks=4;h.height=Pf::f(5.);h.substate=1;
    h.group_gravity=physics::DT2*Pf::f(24.);h.gadget_combat.cable_pose=2;h.gadget_combat.cable_latched=true;let mut rng=Rng::new();
    apply_hero_pose(&mut h,&mut anim,&mut rng);
    let mut chains=vec![Vec::new();crate::hero::fx::BACK_LIST+1];chains[crate::hero::fx::BACK_LIST]=vec![0];h.fx.joints.hero=chains.into();
    let floor=testkit::floor(10.,20,40,20,40);let(rows,yaw)=testkit::cam_x();let pad=PadState::default();let mut m=actor(0,0,[100.,100.,15.]);
    for _ in 0..15{
        let env=physics::Env{coll:&floor,pad:&pad,cam_yaw:yaw,cam_rows:rows,mirror:false,death_z:Pf::ZERO,mobys:None,hero_moby:None,water:None,world:None};
        crate::hero::hero_update_with_combat_mode(&mut h,&mut m,&env,&mut anim,&mut rng,&mut crate::hero::NoHeroSounds,false);
        assert!(h.air_ticks>0);assert_eq!(anim.view().seq_b,0xb);assert!(anim.pose_frame().is_some());
        let p=anim.eval_chains(&[&[0]])[0][3];let expected=crate::hero::fx::joint_world_point(&[m.rows[0],m.rows[1],m.rows[2]],m.position,m.scale,p);
        for k in 0..3{assert!((h.fx.back.position[k]-expected[k]).abs()<0.0001);}
    }
    assert!(h.pos[2].to_f32()<15.,"release must retain native falling physics");
}
#[test]
fn nearby_or_distant_enemies_without_a_native_contact_cannot_be_captured(){
    for (x,stale) in [(100.8,None),(100.8,Some(1)),(103.,None),(115.,None)]{
        let mut h=hero(26);h.cap_moby=stale;h.cap_hit=0;h.wall_moby=None;
        let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[x,100.,10.])],4);
        let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings.drill_enabled=true;s.svc.bomb_evo.gadget_settings.drill_range=20.;
        fire(&mut h,&mut t,&mut s,1,&mut RecordingAnim::default());
        assert!(!h.gadget_combat.holding);assert!(s.svc.gadget_combat.drill.is_none());assert!(s.svc.sounds.is_empty());assert!(t.mobys[1].has_collision);
    }
}
#[test]
fn native_contact_selects_the_touched_enemy_not_another_target_in_the_aim_cone(){
    let mut h=hero(26);h.cap_moby=Some(2);h.cap_hit=1;
    let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[101.2,100.,10.]),actor(2,459,[100.8,100.,10.])],4);
    let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings.drill_enabled=true;
    fire(&mut h,&mut t,&mut s,1,&mut RecordingAnim::default());assert_eq!(s.svc.gadget_combat.drill.unwrap().id,2);
    assert!(t.mobys[1].has_collision);assert!(!t.mobys[2].has_collision);
}
#[test]
fn capture_requires_real_native_capsule_overlap_and_keeps_the_shifted_pivot_fixed(){
    use crate::collision_query::{MobyScene,TableMobys};
    let mut h=hero(26);h.cap_hit=0;h.cap_moby=None;
    let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[100.8,100.,10.])],4);
    t.mobys[1].rows[3]=[0.,0.,716.8,1536.];scheduler::rebuild_matrix(&mut t.mobys[1],None);
    let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings.drill_enabled=true;s.svc.bomb_evo.gadget_settings.drill_dps=0.;
    let mut raw=[0;8];raw[0]=1|4<<8|0x8003<<16;raw[6]=716.8f32.to_bits();raw[7]=512f32.to_bits();
    s.svc.coll_classes=std::sync::Arc::new(std::collections::HashMap::from([(459,rc_formats::moby_collision::MobyCollision{
        joint_counts:[0,0],prims:vec![rc_formats::moby_collision::MobyCollPrim{raw}],vertices:vec![],faces:vec![]})]));
    for m in &mut t.mobys{std::sync::Arc::make_mut(&mut s.svc.grid).register(m);}
    {
        let source=TableMobys{table:&t,classes:&s.classes,snapshots:&[]};let scene=MobyScene{mobys:&source,grid:&s.svc.grid,classes:&s.svc.coll_classes,cache:&s.svc.pose_cache};
        let floor=testkit::floor(10.,20,40,20,40);let pad=PadState::default();let(rows,yaw)=testkit::cam_x();
        let env=physics::Env{coll:&floor,pad:&pad,cam_yaw:yaw,cam_rows:rows,mirror:false,death_z:Pf::ZERO,mobys:Some(&scene),hero_moby:Some(0),water:None,world:None};
        h.move_collide(&env);
    }
    assert_eq!(h.cap_moby,Some(1));assert_ne!(h.cap_hit,0);
    h.body_point=from_f32x3(add(to_f32x3(h.pos),[0.,0.,0.7]));
    let mut anim=RecordingAnim::default();fire(&mut h,&mut t,&mut s,1,&mut anim);assert!(h.gadget_combat.holding);
    let (_,pivot,radius)=s.svc.gadget_combat.drill_pivot.unwrap();
    let start=h.items.slot.item.as_ref().unwrap().position;let expected=add(start,[1.+(radius*0.65).clamp(0.2,1.),0.,0.]);
    for frame in 2..60{fire(&mut h,&mut t,&mut s,frame,&mut anim);
        let m=&t.mobys[1];let actual:V=std::array::from_fn(|k|m.position[k]+(0..3).map(|r|m.rows[r][k]*pivot[r]).sum::<f32>());
        for k in 0..3{assert!((actual[k]-expected[k]).abs()<0.0002);}
        assert!(!m.has_collision);assert!(h.gadget_combat.holding);
    }
}
