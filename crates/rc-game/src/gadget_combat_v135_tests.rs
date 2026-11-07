//! Exercise absent real animation data, grounded cancellation, timed ammunition,
//! and visible smoke geometry without assuming every planet has the same assets.
use super::*;
use super::v134_tests::{hero,actor,classes,fire,Sink};
use crate::{hero::anim::{RatchetAnim,RecordingAnim},moby_update::Services};
use rc_formats::moby_anim::{MobyAnimClass,MobyFrame,MobyFrameHeader,MobySequence,MobySequenceHeader};
pub(super) fn minimal_class()->MobyAnimClass {
    let frame=MobyFrame{header:MobyFrameHeader{rate:0.5,time:0,qwc:1,quat_bytes:8,scale_count:0,trans_offset:8,trans_count:0},
        quats:vec![[0,0,0,0x7fff]],scales:vec![],trans:vec![],payload:vec![0,0,0,0,0,0,255,127,0,0,0,0,0,0,0,0]};
    let sequence=MobySequence{header:MobySequenceHeader{frame_count:2,loop_sound:255,..Default::default()},frames:vec![frame.clone(),frame],triggers:vec![]};
    MobyAnimClass{joint_count:1,skeleton:vec![],rest:vec![[0.,1.,2.]],parent_word:vec![0],sequences:vec![Some(sequence)]}
}
#[test]
fn missing_swingshot_poses_never_replace_a_real_pose_with_an_absent_sequence(){
    let class=minimal_class();let mut ratchet=RatchetAnim::new(&class);let mut ctl=ratchet.ctl(&class);
    let mut h=hero(12);let mut rng=Rng::new();
    for phase in [1,2,1,2,0] {
        cable_animation(&mut h,&mut ctl,&mut rng,phase);h.gadget_combat.cable_pose=phase;
        for _ in 0..8{ctl.advance(Pf::ONE);assert_eq!(ctl.view().seq_b,0);assert!(ctl.pose_frame().is_some());}
    }
    assert_eq!(valid_pose(&ctl,&[-1,0x2f,0x30,0]),Some(0));
}
#[test]
fn a_one_frame_swingshot_sequence_never_requests_frame_two(){
    let mut class=minimal_class();let idle=class.sequences[0].clone();class.sequences.resize(0x31,None);
    class.sequences[0x2f]=idle.clone();class.sequences[0x30]=idle;
    for seq in [0x2f,0x30]{let s=class.sequences[seq].as_mut().unwrap();s.header.frame_count=1;s.frames.truncate(1);}
    let mut ratchet=RatchetAnim::new(&class);let mut ctl=ratchet.ctl(&class);let mut h=hero(12);
    for phase in [1,2]{cable_animation(&mut h,&mut ctl,&mut Rng::new(),phase);assert_eq!(ctl.view().frame_b,0);assert!(ctl.pose_frame().is_some());}
}
#[test]
fn airborne_drill_does_not_start_sound_capture_damage_or_a_standing_pose(){
    for (group,air_ticks) in [(2,0),(2,1),(4,2),(0,1)] {
        let mut h=hero(26);h.group=group;h.air_ticks=air_ticks;
        let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],8);
        let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.drill_enabled=true;
        let mut anim=RecordingAnim::default();fire(&mut h,&mut table,&mut sink,1,&mut anim);
        assert!(sink.svc.sounds.is_empty());assert!(!h.gadget_combat.holding);
        assert!(sink.svc.gadget_combat.drill.is_none());assert!(anim.calls.is_empty());
    }
}
#[test]
fn jumping_during_drill_capture_releases_target_and_loop_without_resetting_jump_animation(){
    let mut h=hero(26);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],8);
    let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.drill_enabled=true;
    let mut anim=RecordingAnim::default();fire(&mut h,&mut table,&mut sink,1,&mut anim);assert!(h.gadget_combat.holding);
    sink.svc.gadget_combat.sound=Some((0,7));sink.svc.sounds.clear();anim.calls.clear();h.group=2;h.air_ticks=1;
    fire(&mut h,&mut table,&mut sink,2,&mut anim);
    assert!(sink.svc.gadget_combat.sound.is_none());assert!(sink.svc.sounds.is_empty());
    assert!(sink.svc.gadget_combat.drill.is_none());assert!(!h.gadget_combat.holding);assert!(anim.calls.is_empty());
}
fn settings()->Settings {Settings{ice_enabled:true,ice_ammo:3,recharge_seconds:1.,..Default::default()}}
#[test]
fn recharge_grants_one_round_per_interval_and_never_banks_at_capacity(){
    let s=settings();let mut c=freezer::Recharge::default();let mut ammo=Some(0);
    for t in 1..60{c.tick(&mut ammo,s,t,true);c.tick(&mut ammo,s,t,true);assert_eq!(ammo,Some(0));}
    assert!((c.hud(0,s).fraction-59./60.).abs()<0.0001);
    c.tick(&mut ammo,s,60,true);assert_eq!(ammo,Some(1));assert_eq!(c.hud(1,s).fraction,0.);
    for t in 61..1000{c.tick(&mut ammo,s,t,true);}
    assert_eq!(ammo,Some(3));assert_eq!(c.elapsed(),0);assert!(c.hud(3,s).full());assert_eq!(c.hud(3,s).fraction,1.);
    ammo=Some(2);c.tick(&mut ammo,s,1000,true);assert_eq!(ammo,Some(2));assert_eq!(c.elapsed(),1);
}
#[test]
fn recharge_pauses_with_death_scenes_or_disabled_combat_and_handles_capacity_changes(){
    let s=settings();let mut c=freezer::Recharge::default();let mut ammo=Some(1);
    for t in 1..=30{c.tick(&mut ammo,s,t,true);}
    for t in 31..=100{c.tick(&mut ammo,s,t,false);}
    for t in 101..=150{c.tick(&mut ammo,Settings{ice_enabled:false,..s},t,true);}
    assert_eq!(c.elapsed(),30);assert_eq!(ammo,Some(1));
    for t in 151..=180{c.tick(&mut ammo,s,t,true);}assert_eq!(ammo,Some(2));
    c.tick(&mut ammo,Settings{ice_ammo:1,..s},181,true);assert_eq!(ammo,Some(1));assert_eq!(c.elapsed(),0);
    c.tick(&mut ammo,Settings{ice_ammo:0,..s},182,true);assert_eq!(ammo,Some(0));assert!(c.hud(0,Settings{ice_ammo:0,..s}).full());
}
#[test]
fn recharge_progress_survives_checkpoint_init_and_planet_change_starts_a_new_visit(){
    let mut session=crate::game_state::SessionState::default();session.hydro_combat_ammo=Some((1,1));session.hydro_recharge_ticks=40;
    session.hero_init(4);assert_eq!(session.hydro_recharge_ticks,40);
    let mut rt=Runtime::default();let s=settings();rt.sync_ammo(1,Some(1),s);rt.recharge.restore(session.hydro_recharge_ticks);
    for t in 1..=20{rt.recharge.tick(&mut rt.ice_ammo,s,t,true);}assert_eq!(rt.ice_ammo,Some(2));
    rt.sync_ammo(2,None,s);assert_eq!(rt.ice_ammo,Some(3));assert_eq!(rt.recharge.elapsed(),0);
}
#[test]
fn frozen_enemies_have_white_soft_mist_without_particle_definitions_and_it_ends_on_thaw(){
    let h=hero(22);let mut table=MobyTable::new(vec![actor(0,0,[100.,100.,10.]),actor(1,459,[103.,100.,10.])],4);
    let mut svc=Services::new();let c=classes();let mut rng=Rng::new();let mut w=World::new(&mut table,&h,&mut rng,&c,&mut svc,1);
    let mut rt=Runtime::default();hold(&mut rt,&w,1,20,Some(0x408CFF),false);w.svc.gadget_combat=rt;
    draw_status(&mut w);assert_eq!(w.svc.gadget_combat.frost_mist.len(),1);
    let before=condensation_quads(&w.svc.gadget_combat,1,[90.,90.,20.]);assert_eq!(before.len(),4);
    assert!(before.iter().flat_map(|q|q.corners).flatten().all(f32::is_finite));
    assert!(before.iter().any(|q|q.rgba[0]>>24>10));
    w.counter=2;world_tick(&mut w);w.mm(1).position[0]+=10.;draw_status(&mut w);
    assert!(w.svc.gadget_combat.frost_mist[0].0[0]>110.);
    w.counter=20;world_tick(&mut w);draw_status(&mut w);assert!(w.svc.gadget_combat.frost_mist.is_empty());
    let pixels=freezer::mist_texture();assert_eq!(pixels.len(),32*32*4);assert_eq!(pixels[3],0);
    assert!(pixels[(16*32+16)*4+3]>120);assert!(pixels.chunks_exact(4).all(|p|p[..3]==[128,128,128]));
}
#[test]
fn freezer_has_a_native_fading_ammo_panel_without_an_icon_and_uses_the_weapon_bar_style(){
    use crate::hud::{HudState,HudAssets,Inputs,Draw};
    let mut hud=HudState::new(HudAssets{icons:vec![],frame_sizes:vec![(32,32)],glyphs:[[rc_formats::font::Glyph::default();rc_formats::font::GLYPHS];3],messages:vec![]});
    let mut input=Inputs{hp:4,max_hp:4,held_item:22,weapon:Some((22,2,3)),freezer:Some(freezer::Hud{ammo:2,maximum:3,fraction:0.5}),..Default::default()};input.ammo[22]=2;
    let mut out=Vec::new();for _ in 0..16{out=hud.tick(input);}
    assert_eq!(hud.weapon_evo_display_item(true),Some(22));assert_eq!(hud.weapon_evo_fade(22,false),1.);
    assert_eq!(out.iter().filter(|d|matches!(d,Draw::Sprite{..})).count(),3,"only the three ammo vignette pieces, no icon");
    assert!(out.iter().any(|d|matches!(d,Draw::Text{text,..} if text==b"2/3")));
    let (left,y,width)=hud.weapon_evo_xp_anchor(22,false,20).unwrap();
    let full=crate::hud::xp_bar::draws(left,y,width,5,1.,128,true);
    assert!(matches!(full.last(),Some(Draw::Rect16{rgba,..}) if rgba&0xffffff==0x20c6f1));
    input.weapon=None;input.held_item=0;
    let mut faded=false;for _ in 0..80{hud.tick(input);if hud.weapon_evo_fade(22,false)>0.&&hud.weapon_evo_fade(22,false)<1.{faded=true;}}
    assert!(faded);assert_eq!(hud.weapon_evo_fade(22,false),0.);
    for lang in [0,2,3,4,5]{assert!(!freezer::name(lang).is_empty());assert!(freezer::name(lang).is_ascii());}
}
