//! Locomotion ownership, actual hand chains, model-space capture pivots and
//! release-before-fire are exercised independently of disc assets and the GPU.
use super::*;
use super::v134_tests::{actor,classes,hero,Sink};
use super::v136_tests::swing_class;
use crate::{hero::{anim::{RatchetAnim,RecordingAnim},items::ItemData,physics,testkit},moby_update::Services,pad::PadState};
fn close(a:V,b:V){for k in 0..3{assert!((a[k]-b[k]).abs()<0.0002,"{a:?} != {b:?}");}}
#[test]
fn releasing_an_extended_cable_restores_the_unchanged_native_run_state(){
    let class=swing_class();let mut ratchet=RatchetAnim::new(&class);let mut anim=ratchet.ctl(&class);let mut h=hero(12);
    h.state=2;h.substate=1;h.group=1;let mut rng=Rng::new();
    h.set_anim(&mut anim,&mut rng,Pf::ONE,4,0);h.gadget_combat.cable_pose=1;
    apply_hero_pose(&mut h,&mut anim,&mut rng);assert_eq!(anim.view().seq_b,0x2f);
    let mut pad=PadState{lx:Pf::ONE,held:button::RIGHT,..Default::default()};
    prepare_hero_input(&mut h,&mut pad);assert_eq!(pad.lx,Pf::ONE);
    for _ in 0..25{apply_hero_pose(&mut h,&mut anim,&mut rng);anim.advance(Pf::ONE);assert_eq!(anim.view().seq_b,4);}
    assert_eq!(h.state,2);assert_eq!(h.substate,1);assert_eq!(h.anim_speed,Pf::ONE);
}
#[test]
fn gadget_release_preserves_a_native_jump_or_damage_pose(){
    let class=swing_class();let mut ratchet=RatchetAnim::new(&class);let mut anim=ratchet.ctl(&class);let mut h=hero(12);let mut rng=Rng::new();
    for (state,group,seq) in [(3,2,0xb),(7,7,0x15)]{
        h.state=state;h.group=group;h.gadget_combat.pose_sequence=Some(0x2f);h.gadget_combat.pose_item=12;h.anim_speed=Pf::ZERO;
        h.set_anim(&mut anim,&mut rng,Pf::ONE,seq,0);apply_hero_pose(&mut h,&mut anim,&mut rng);
        assert_eq!(anim.view().seq_b,seq);assert_eq!(h.anim_speed,Pf::ONE);
    }
}
#[test]
fn hook_latch_masks_translation_but_keeps_camera_fire_and_release(){
    let mut h=hero(12);h.gadget_combat.cable_latched=true;h.gadget_combat.cable_pose=2;
    let mut p=PadState{lx:Pf::ONE,ly:Pf::ONE,rx:Pf::f(0.3),ry:Pf::f(-0.7),held:button::CIRCLE|button::L1|button::DOWN|button::CROSS,
        pressed:button::CROSS,raw:button::DOWN,..Default::default()};
    prepare_hero_input(&mut h,&mut p);assert_eq!(p.lx,Pf::ZERO);assert_eq!(p.ly,Pf::ZERO);assert_eq!(p.raw,0);assert_eq!(p.pressed,0);
    assert_eq!(p.held,button::CIRCLE|button::L1);assert_eq!(p.rx,Pf::f(0.3));assert_eq!(p.ry,Pf::f(-0.7));
    p.held=button::DOWN;p.ly=Pf::ONE;prepare_hero_input(&mut h,&mut p);
    assert_eq!(p.ly,Pf::ONE);assert!(!h.gadget_combat.cable_latched);assert!(!cable_holds_motion(&h,&p));
}
#[test]
fn latched_surface_pull_still_moves_up_while_voluntary_movement_is_locked(){
    let mut h=hero(12);h.gadget_combat.cable_latched=true;h.gadget_combat.cable_pose=2;h.gadget_combat.grapple_step=Some([0.,0.,0.15]);
    let floor=testkit::floor(10.,20,40,20,40);let(rows,yaw)=testkit::cam_x();let mut anim=RecordingAnim::default();let mut rng=Rng::new();
    let pad=PadState{lx:Pf::ONE,held:button::CIRCLE|button::RIGHT,..Default::default()};let mut m=actor(0,0,[100.,100.,10.]);
    for _ in 0..10{
        let env=physics::Env{coll:&floor,pad:&pad,cam_yaw:yaw,cam_rows:rows,mirror:false,death_z:Pf::ZERO,mobys:None,hero_moby:None,water:None,world:None};
        crate::hero::hero_update_with_combat_mode(&mut h,&mut m,&env,&mut anim,&mut rng,&mut crate::hero::NoHeroSounds,false);
    }
    close(to_f32x3(h.pos),[100.,100.,11.5]);
}
#[test]
fn low_swingshot_nozzle_uses_the_actual_animated_hand_chain(){
    let mut h=hero(12);let mut data=ItemData::default();
    let mut class=swing_class();class.sequences.truncate(1);class.rest[0]=[512.,0.,256.];
    // The native chain's translation is in model units, not a chest offset.
    data.classes.push(crate::hero::items::ItemClass{o_class:208,scale:1.,anim:class,chains:vec![vec![0]]});
    let it=h.items.slot.item.as_mut().unwrap();it.position=[100.,100.,10.];it.rows=rc_formats::moby_light::rotation_rows([0.,0.,0.]);
    let pad=PadState::default();let env=ItemEnv{data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};
    let expected=crate::hero::guns::item_point(&h,&env,0);close(hand_origin(&h,&env),expected);
    assert!(expected[2]<10.6);assert_ne!(hand_origin(&h,&env),launch_origin(&h,&env,[1.,0.,0.]));
}
#[test]
fn an_offset_model_rotates_about_one_fixed_centre_even_when_mirrored_or_scaled(){
    let h=hero(26);let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],1);
    let mut svc=Services::new();let mut c=classes();let mut class=swing_class();
    class.sequences[0].as_mut().unwrap().header.sphere=[300.,-200.,800.,600.];c.classes.get_mut(&459).unwrap().1=Some(class);
    let mut rng=Rng::new();let mut w=World::new(&mut t,&h,&mut rng,&c,&mut svc,1);
    for mirror in [0,mode::MIRROR]{
        w.mm(1).mode=mirror;w.mm(1).scale=1.7;let(pivot,radius)=drill_bounds(&w,1);assert!(radius>0.9);
        for n in 0..73{let centre=[102.,100.,11.];let direction=unit([1.,0.2,0.5]);
            place_drill_target(&mut w,1,centre,direction,n as f32*0.13,pivot);
            let m=w.m(1);let actual:V=std::array::from_fn(|k|m.position[k]+(0..3).map(|r|m.rows[r][k]*pivot[r]).sum::<f32>());
            close(actual,centre);assert!(m.uid_hi&65535>0);
        }
    }
}
#[test]
fn captured_enemy_bounds_are_cached_once_and_original_authored_rows_restore(){
    let h=hero(26);let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],1);
    let mut svc=Services::new();let c=classes();let mut rng=Rng::new();let mut w=World::new(&mut t,&h,&mut rng,&c,&mut svc,1);
    w.mm(1).mode|=mode::KEEP_ROWS;w.mm(1).rows[..3].copy_from_slice(&rc_formats::moby_light::rotation_rows([0.,0.,0.]).map(|r|r.map(f32::from_bits)));
    let original=w.m(1).rows;let mut rt=Runtime::default();hold(&mut rt,&w,1,3,None,true);rt.drill=Some(Lease::of(&w,1));
    place_drill_target(&mut w,1,[102.,100.,11.],[1.,0.,0.],1.3,[0.,0.,0.7]);assert_eq!(w.m(1).mode&mode::KEEP_ROWS,0);
    release_drill(&mut rt,&mut w,false);assert_ne!(w.m(1).mode&mode::KEEP_ROWS,0);assert_eq!(w.m(1).rows[..3],original[..3]);
}
#[test]
fn capture_aim_has_one_stable_decoded_pose_and_releases_its_playback_lock(){
    let class=swing_class();let mut ratchet=RatchetAnim::new(&class);let mut anim=ratchet.ctl(&class);let mut h=hero(26);
    h.weapons.defs.resize(37,Default::default());h.weapons.defs[16].anims[0]=8;h.gadget_combat.holding=true;let mut rng=Rng::new();
    for _ in 0..40{anim.advance(Pf::ONE);apply_hero_pose(&mut h,&mut anim,&mut rng);
        assert_eq!(anim.view().seq_a,8);assert_eq!(anim.view().seq_b,8);assert_eq!(anim.view().frame_a,1);assert_eq!(anim.view().frame_b,1);
        assert_eq!(anim.view().t,0.);assert!(anim.pose_frame().is_some());assert_eq!(h.anim_speed,Pf::ZERO);
    }
    let mut p=PadState::default();prepare_hero_input(&mut h,&mut p);apply_hero_pose(&mut h,&mut anim,&mut rng);
    assert_eq!(h.anim_speed,Pf::ONE);assert_ne!(anim.view().seq_b,8);
}
fn ice_tick(h:&mut Hero,t:&mut MobyTable,s:&mut Sink,frame:i32,held:u32){
    let data=ItemData::default();let pad=PadState{held,pressed:held,..Default::default()};
    let env=ItemEnv{data:&data,pad:&pad,frame,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};
    item_tick(h,t,&mut RecordingAnim::default(),&env,s,&mut Rng::new());
}
#[test]
fn circle_r2_or_both_used_to_equip_cannot_spend_a_freezer_round(){
    for held in [button::CIRCLE,button::R2,button::CIRCLE|button::R2]{
        let mut h=hero(22);h.items.slot.state=3;let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.])],4);
        let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings.ice_enabled=true;
        ice_tick(&mut h,&mut t,&mut s,1,held);h.items.slot.state=2;
        for frame in 2..10{ice_tick(&mut h,&mut t,&mut s,frame,held);assert!(s.svc.gadget_combat.ice.is_empty());assert_eq!(s.svc.gadget_combat.ice_ammo,Some(10));}
        ice_tick(&mut h,&mut t,&mut s,10,0);ice_tick(&mut h,&mut t,&mut s,11,held);
        assert_eq!(s.svc.gadget_combat.ice.len(),1);assert_eq!(s.svc.gadget_combat.ice_ammo,Some(9));
    }
}
#[test]
fn freezer_swap_resets_fire_arming_but_keeps_its_ammo_and_repeat_interval(){
    let mut h=hero(22);let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.])],4);
    let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings=Settings{ice_enabled:true,ice_interval:2,..Default::default()};
    for(frame,held)in [(1,0),(2,button::R2),(3,button::R2),(4,button::R2)]{ice_tick(&mut h,&mut t,&mut s,frame,held);}
    assert_eq!(s.svc.gadget_combat.ice_ammo,Some(8));
    h.items.slot.id=12;ice_tick(&mut h,&mut t,&mut s,5,button::R2);h.items.slot.id=22;ice_tick(&mut h,&mut t,&mut s,6,button::R2);
    assert_eq!(s.svc.gadget_combat.ice_ammo,Some(8));ice_tick(&mut h,&mut t,&mut s,7,0);ice_tick(&mut h,&mut t,&mut s,8,button::R2);
    assert_eq!(s.svc.gadget_combat.ice_ammo,Some(7));
}

#[test]
fn a_menu_handoff_mask_is_not_a_physical_fire_release(){
    let mut h=hero(22);let mut t=MobyTable::new(vec![actor(0,0,[100.,100.,10.])],4);
    let mut s=Sink{svc:Services::new(),classes:classes()};s.svc.bomb_evo.gadget_settings.ice_enabled=true;
    let data=ItemData::default();let pad=PadState{physical_buttons:Some((button::CIRCLE,button::CIRCLE)),..Default::default()};
    let env=ItemEnv{data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,mobys:None,camera:None,camera_up:None,targets:&[]};
    item_tick(&mut h,&mut t,&mut RecordingAnim::default(),&env,&mut s,&mut Rng::new());
    assert!(!h.gadget_combat.freezer_armed);
    ice_tick(&mut h,&mut t,&mut s,2,button::CIRCLE);assert!(s.svc.gadget_combat.ice.is_empty());
    ice_tick(&mut h,&mut t,&mut s,3,0);ice_tick(&mut h,&mut t,&mut s,4,button::CIRCLE);
    assert_eq!(s.svc.gadget_combat.ice.len(),1);
}
