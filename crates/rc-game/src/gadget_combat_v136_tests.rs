//! Renderer-path regressions, including a missing snapshot and Clank's cached
//! back attachment; a nonzero sequence header alone cannot establish a pose.
use super::*;
use super::v134_tests::{hero,actor,classes,fire,Sink};
use super::v135_tests::minimal_class;
use crate::hero::{anim::{RatchetAnim,RecordingAnim},physics,testkit};
use crate::moby_update::Services;
use crate::pad::{PadInput,PadState};
use rc_formats::moby_anim::{self,SNAPSHOT_SEQ};
pub(super) fn swing_class()->moby_anim::MobyAnimClass{
    let mut c=minimal_class();c.skeleton=vec![moby_anim::IDENTITY;c.joint_count];let seq=c.sequences[0].clone();c.sequences.resize(0x31,seq);c
}
#[test]
fn custom_surface_pull_never_selects_native_state_25s_full_body_pose(){
    let c=swing_class();let mut a=RatchetAnim::new(&c);let mut ctl=a.ctl(&c);let mut h=hero(12);
    for phase in [1,2,2,2]{h.gadget_combat.cable_pose=phase;apply_hero_pose(&mut h,&mut ctl,&mut Rng::new());
        assert_eq!(ctl.a.state.seq_b,0x2f);assert!(ctl.pose_frame().is_some());ctl.advance(Pf::ONE);}
}
#[test]
fn missing_snapshot_is_repaired_before_repeated_custom_blends_can_render_identity(){
    let c=swing_class();let mut a=RatchetAnim::new(&c);let mut ctl=a.ctl(&c);let mut h=hero(12);h.gadget_combat.cable_pose=2;
    for _ in 0..30{
        ctl.a.state.seq_a=SNAPSHOT_SEQ;ctl.a.state.seq_b=0;ctl.a.state.frame_a=0;ctl.a.state.t=0.2;ctl.a.snapshot=None;
        let broken=moby_anim::evaluate_posed(&c,&ctl.a.state,ctl.a.snapshot.as_ref(),&[],&[]);
        assert_eq!(broken[0],moby_anim::IDENTITY);
        apply_hero_pose(&mut h,&mut ctl,&mut Rng::new());
        let fixed=moby_anim::evaluate_posed(&c,&ctl.a.state,ctl.a.snapshot.as_ref(),&[],&[]);
        assert_ne!(fixed[0],moby_anim::IDENTITY);assert!(ctl.pose_frame().is_some());
    }
}
#[test]
fn advertised_but_undecoded_sequences_are_not_valid_gadget_fallbacks(){
    let mut c=swing_class();c.sequences[0x2f].as_mut().unwrap().frames.clear();
    let mut a=RatchetAnim::new(&c);let mut ctl=a.ctl(&c);let mut h=hero(12);h.gadget_combat.cable_pose=2;
    assert!(ctl.frame_count(0x2f)>0);assert!(!ctl.gadget_sequence_valid(0x2f));
    apply_hero_pose(&mut h,&mut ctl,&mut Rng::new());assert_ne!(ctl.view().seq_b,0x2f);assert_ne!(ctl.view().seq_b,0x30);
    assert!(ctl.pose_frame().is_some());
}
#[test]
fn hero_update_attaches_clank_using_the_same_final_pose_as_the_renderer(){
    let c=swing_class();let mut a=RatchetAnim::new(&c);let mut ctl=a.ctl(&c);let mut h=hero(12);
    h.gadget_combat.cable_pose=2;h.gadget_combat.grapple_step=Some([0.,0.,0.1]);
    let mut chains=vec![Vec::new();crate::hero::fx::BACK_LIST+1];chains[crate::hero::fx::BACK_LIST]=vec![0];h.fx.joints.hero=chains.into();
    let floor=testkit::floor(10.,20,40,20,40);let mut pad=PadState::default();let(rows,yaw)=testkit::cam_x();
    pad.update(Some(&PadInput::neutral().press(button::CIRCLE).bytes()),false);
    let mut m=actor(0,0,[100.,100.,10.]);let mut rng=Rng::new();
    for _ in 0..20{
        let env=physics::Env{coll:&floor,pad:&pad,cam_yaw:yaw,cam_rows:rows,mirror:false,death_z:Pf::ZERO,mobys:None,hero_moby:None,water:None,world:None};
        crate::hero::hero_update_with_combat_mode(&mut h,&mut m,&env,&mut ctl,&mut rng,&mut crate::hero::NoHeroSounds,false);
        assert_eq!(ctl.view().seq_b,0x2f);assert_eq!(h.fx.view.seq_b,ctl.view().seq_b);
        let p=ctl.eval_chains(&[&[0]])[0][3];
        let expected=crate::hero::fx::joint_world_point(&[m.rows[0],m.rows[1],m.rows[2]],m.position,m.scale,p);
        for axis in 0..3{assert!((h.fx.back.position[axis]-expected[axis]).abs()<0.0001);}
    }
}
#[test]
fn defeat_keeps_the_native_corpse_visible_and_never_spawns_a_render_projectile(){
    let mut h=hero(26);let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],8);
    let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings.drill_enabled=true;
    let mut anim=RecordingAnim::default();fire(&mut h,&mut t,&mut s,1,&mut anim);assert!(h.gadget_combat.holding);
    let length=t.mobys.len();crate::moby_update::services::pvar::set_ff(&mut t.mobys[1].pvars,0x20,0.);
    fire(&mut h,&mut t,&mut s,2,&mut anim);
    assert!(!h.gadget_combat.holding);assert!(s.svc.gadget_combat.drill.is_none());
    assert_eq!(t.mobys.len(),length);assert_eq!(t.mobys[1].mode&mode::HIDDEN,0);
    assert!(t.mobys[1].has_collision,"native death update owns collision removal");
}
