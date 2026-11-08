//! Cover complete tick order, including deferred native item writes and the
//! cached attachments published after custom release recovery.
use super::*;
use super::v134_tests::{actor,hero,classes,Sink};
use super::v136_tests::swing_class;
use crate::{hero::{anim::RatchetAnim,items::{ItemData,ItemClass,ItemDef},testkit,physics},pad::{PadState,PadInput},tick::{Game,GameOptions,TickHooks}};
use rc_formats::moby_anim::{self,SNAPSHOT_SEQ};
fn data()->ItemData{
    let mut d=ItemData::default();d.defs.resize(37,ItemDef::default());d.defs[12].o_class=208;
    d.hero_chains=vec![vec![0]];
    for class in [208,209]{d.classes.push(ItemClass{o_class:class,scale:1.,anim:swing_class(),chains:vec![vec![0]]});}
    d
}
fn airborne(state:i32,group:i32)->Hero{
    let mut h=hero(12);h.state=state;h.group=group;h.air_ticks=10;h.height=Pf::f(5.);
    h.pos=from_f32x3([100.,100.,15.]);h.body_point=from_f32x3([100.,100.,15.7]);
    h.group_gravity=physics::DT2*Pf::f(24.);h.vel=from_f32x3([0.05,0.,0.1]);h.eff=h.vel;
    h.gadget_combat.cable_pose=2;h.gadget_combat.cable_latched=true;
    let mut chains=vec![Vec::new();crate::hero::fx::BACK_LIST+1];chains[crate::hero::fx::BACK_LIST]=vec![0];h.fx.joints.hero=chains.into();h
}
fn assert_palette(class:&moby_anim::MobyAnimClass,a:&RatchetAnim){
    assert_eq!(a.state.seq_a,0xb);assert_eq!(a.state.seq_b,0xb);assert!(a.snapshot.is_none());
    assert_ne!(moby_anim::evaluate_posed(class,&a.state,a.snapshot.as_ref(),&[],&[])[0],moby_anim::IDENTITY);
}
#[test]
fn jump_running_jump_double_jump_and_hover_release_all_handoff_to_fall(){
    let class=swing_class();let floor=testkit::floor(10.,20,40,20,40);let d=data();let pad=PadState::default();
    for(state,group)in [(6,2),(7,4),(9,4),(0xe,4),(8,5)]{
        let mut h=airborne(state,group);let mut a=RatchetAnim::new(&class);let mut ctl=a.ctl(&class);let mut rng=Rng::new();
        apply_hero_pose(&mut h,&mut ctl,&mut rng);prepare_hero_input(&mut h,&mut PadState::default());
        let motion=(h.vel,h.momentum,h.eff);let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,15.])],4);
        let env=ItemEnv{data:&d,pad:&pad,frame:1,hero_moby:0,coll:Some(&floor),mobys:None,camera:None,camera_up:None,targets:&[]};
        assert!(finish_hero_items(&mut h,&mut t,&mut ctl,&env,&mut rng));
        assert_eq!((h.state,h.group),(6,2));assert_eq!((h.vel,h.momentum,h.eff),motion);assert_eq!(h.gadget_combat.cable_exit,2);
        assert_palette(&class,ctl.a);
        let hand=h.items.slot.item.as_ref().unwrap().anim;
        assert!(!finish_hero_items(&mut h,&mut t,&mut ctl,&env,&mut rng),"a valid fall must keep advancing");
        assert_eq!(h.items.slot.item.as_ref().unwrap().anim,hand);
    }
}
#[test]
fn a_late_native_item_write_is_repaired_after_the_early_handoff_has_finished(){
    let class=swing_class();let mut a=RatchetAnim::new(&class);let mut ctl=a.ctl(&class);let mut h=airborne(6,2);let mut rng=Rng::new();
    apply_hero_pose(&mut h,&mut ctl,&mut rng);prepare_hero_input(&mut h,&mut PadState::default());apply_hero_pose(&mut h,&mut ctl,&mut rng);
    assert_eq!(ctl.view().seq_b,0xb);
    let floor=testkit::floor(10.,20,40,20,40);let pad=PadState::default();let(rows,yaw)=testkit::cam_x();let d=data();
    let physics=physics::Env{coll:&floor,pad:&pad,cam_yaw:yaw,cam_rows:rows,mirror:false,death_z:Pf::ZERO,mobys:None,hero_moby:Some(0),water:None,world:None};
    // The native deferred pass really runs after HeroUpdate. A malformed source
    // can otherwise survive a subsequent SetAnim, whose snapshot attempt fails.
    ctl.a.state.seq_a=SNAPSHOT_SEQ;ctl.a.snapshot=None;
    h.gadgets.calls.push(crate::hero::gadgets::ItemCall::SetAnim{blend:14,seq:0x30,frame:255});
    crate::hero::gadgets::after_items(&mut h,&mut crate::hero::states::Ctx{env:&physics,anim:&mut ctl,rng:&mut rng,voice:None},&d);
    assert!(ctl.pose_frame().is_none());
    let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,15.])],4);
    let env=ItemEnv{data:&d,pad:&pad,frame:1,hero_moby:0,coll:Some(&floor),mobys:None,camera:None,camera_up:None,targets:&[]};
    assert!(finish_hero_items(&mut h,&mut t,&mut ctl,&env,&mut rng));assert_palette(&class,ctl.a);
    assert_eq!(t.mobys[0].anim.seq_b,0xb);assert_eq!(h.fx.view.seq_b,0xb);
}
#[test]
fn full_game_ticks_publish_a_fall_after_deferred_items_and_keep_clank_attached(){
    let class=swing_class();let mut a=RatchetAnim::new(&class);let mut ctl=a.ctl(&class);
    let floor=testkit::floor(10.,20,40,20,40);let mut game=Game::new(&floor,MobyTable::new(vec![actor(0,0,[100.,100.,15.])],4),0,GameOptions::default(),0.);
    game.hero=airborne(9,4);game.hero.vel[2]=Pf::f(-0.03);game.hero.eff=game.hero.vel;game.hero.owned.set(12,true);game.item_data=Some(data());
    apply_hero_pose(&mut game.hero,&mut ctl,&mut game.rng);
    let mut sink=Sink{svc:crate::moby_update::Services::new(),classes:classes()};
    let mut updates=0;
    for frame in 0..20{
        let mut mobys=|_:&mut MobyTable,_:&Hero,_:&mut Rng,_:&crate::follow_camera::CameraView,_:&rc_formats::collision::Collision,_:u64|{};
        let mut parts=|_:&Hero,_:&crate::follow_camera::CameraView,_:&mut Rng,_:u64|{};
        let mut hooks=TickHooks{mobys:&mut mobys,particles:&mut parts,world:None};
        if frame==1{
            // Native Swingshot release/retraction can request idle late, after
            // the earlier fall handoff has relinquished its pose ownership.
            game.hero.gadgets.calls.push(crate::hero::gadgets::ItemCall::SetState{id:0,play:false});
            game.hero.gadgets.calls.push(crate::hero::gadgets::ItemCall::SetAnim{blend:14,seq:0,frame:0});
        }
        game.tick_with_hits(Some(&PadInput::neutral().bytes()),&floor,&mut ctl,&mut hooks,&mut sink);
        assert_eq!(game.hero.state,6);assert_eq!(ctl.view().seq_b,0xb);assert!(ctl.pose_frame().is_some());
        assert_ne!(moby_anim::evaluate_posed(&class,&ctl.a.state,ctl.a.snapshot.as_ref(),&[],&[])[0],moby_anim::IDENTITY);
        let m=&game.mobys.mobys[0];let p=ctl.eval_chains(&[&[0]])[0][3];
        let expected=crate::hero::fx::joint_world_point(&[m.rows[0],m.rows[1],m.rows[2]],m.position,m.scale,p);
        for k in 0..3{assert!((game.hero.fx.back.position[k]-expected[k]).abs()<0.0001);}
        updates+=1;
    }
    assert_eq!(updates,20);assert!(game.hero.pos[2].to_f32()<15.);
}
#[test]
fn final_attachment_refresh_does_not_advance_the_hand_animation_twice(){
    let class=swing_class();let mut a=RatchetAnim::new(&class);let mut ctl=a.ctl(&class);let mut h=airborne(6,2);h.gadget_combat.cable_exit=1;
    let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,15.])],4);let floor=testkit::floor(10.,20,40,20,40);let d=data();let pad=PadState::default();
    let env=ItemEnv{data:&d,pad:&pad,frame:1,hero_moby:0,coll:Some(&floor),mobys:None,camera:None,camera_up:None,targets:&[]};
    let before=h.items.slot.item.as_ref().unwrap().anim;
    assert!(finish_hero_items(&mut h,&mut t,&mut ctl,&env,&mut Rng::new()));
    assert_eq!(h.items.slot.item.as_ref().unwrap().anim,before);
    let expected=crate::hero::guns::item_point(&h,&env,0);assert!(expected.into_iter().all(f32::is_finite));
}
#[test]
fn landing_damage_native_swing_and_new_jump_cancel_release_recovery(){
    let class=swing_class();let floor=testkit::floor(10.,20,40,20,40);let d=data();
    for kind in 0..4{
        let mut h=airborne(6,2);h.gadget_combat.cable_exit=1;
        let mut a=RatchetAnim::new(&class);let mut ctl=a.ctl(&class);let mut pad=PadState::default();
        match kind{0=>h.air_ticks=0,1=>{h.group=7;h.state=0x16;},2=>{h.group=0xe;h.state=0x2c;},_=>pad.pressed=button::CROSS}
        let env=ItemEnv{data:&d,pad:&pad,frame:1,hero_moby:0,coll:Some(&floor),mobys:None,camera:None,camera_up:None,targets:&[]};
        let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,15.])],4);let before=ctl.a.state;
        assert!(!finish_hero_items(&mut h,&mut t,&mut ctl,&env,&mut Rng::new()));assert_eq!(ctl.a.state,before);assert_eq!(h.gadget_combat.cable_exit,0);
    }
}
