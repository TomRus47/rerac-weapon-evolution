//! Public contracts consumed by the engine, independently of Bevy.
use rc_game::weapon_evo::{randomizer::{self,State},save::Progress,Evolution};
#[test]
fn mixed_rewards_have_a_visible_model_policy_and_safe_departures(){
    let state=State::generate(8401);
    assert!(state.valid()&&state.solvable());
    for reward in state.placements.into_iter().filter(|&r|r!=0){assert!(randomizer::display_class(reward,777)>0);}
    assert_eq!(randomizer::display_class(70,-1),750);
    assert_eq!(randomizer::display_class(10,121),121);
    assert_eq!(randomizer::display_class(24,433),479);
    assert_eq!(randomizer::display_class(100,-1),1134);
    assert_eq!(randomizer::display_class(50,-1),1428);
    assert!(!randomizer::travel_mask(state,10,true));
    assert!(randomizer::travel_mask(state,9,true));
    let mut complete=state;for source in [28,75,137]{if complete.reward(source).is_some(){complete.collect(source);}}
    assert!(randomizer::travel_mask(complete,10,true));assert!(complete.valid());
    let mut gs=rc_game::game_state::GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
    randomizer::initialize(&mut gs,true,8401);
    assert_eq!(gs.global.level,1);assert_eq!(randomizer::new_game_destination(&gs),1);
    assert!(gs.weapon_progress.randomizer.collected(65));assert_eq!(gs.global.planet_unlocked[1],1);
    assert!(!randomizer::ship_available(gs.weapon_progress.randomizer,1));
    assert_eq!(gs.global.flags[8],1);assert!(!gs.weapon_progress.randomizer.collected(67));
    assert_eq!(rc_game::travel::transition::plan(&gs,0,&|n|n).dest,1);
    let destination=randomizer::departure(gs.weapon_progress.randomizer,0,1,&gs.global.planet_unlocked);
    let plan=rc_game::travel::transition::plan(&gs,destination,&|n|n);
    assert_eq!(plan.dest,destination);
    if destination!=1{assert!(!plan.steps.iter().any(|s|matches!(s,rc_game::travel::transition::Step::Movie(3..=5))));}
    gs.global.level=7;gs.global.planet_unlocked[8]=1;
    let plan=rc_game::travel::transition::plan(&gs,2,&|n|n);
    assert_eq!(plan.dest,2);
    assert!(!plan.steps.contains(&rc_game::travel::transition::Step::Movie(7)));

}
#[test]
fn live_updates_preserve_save_state_and_use_an_acknowledgeable_parse_boundary(){
    let mut live=Evolution::default();let mut p=Progress::default();p.randomizer=State::generate(8402);
    live.restore_progress(&p);
    let before=live.progress();let mut profile=Evolution::default();profile.hud.name_opacity_percent=42;
    live.apply_live_configuration(&profile);
    assert_eq!(live.progress(),before);assert_eq!(live.hud.name_opacity_percent,42);
    assert!(Evolution::parse("[general]\nrandomizer_mode=maybe\n").is_err());
}

#[test]
fn pickup_visual_contracts_do_not_require_native_particle_textures_or_inventory_changes() {
    use rc_game::weapon_evo::pickup_visuals as visual;
    assert!(visual::fresh_background(true,false,false));
    assert!(!visual::fresh_background(false,false,false));
    assert_eq!(visual::pose_sequence(randomizer::display_class(70,-1),1),0);
    let before=State::generate(8602);
    for height in [0.5,2.,20.] {
        let halo=visual::halo([10.,20.,height],[10.,20.,height+1.]).unwrap();
        assert_eq!(halo.rim.len(),visual::HALO_SEGMENTS);
        assert!(halo.rim.iter().flatten().all(|v|v.is_finite()));
    }
    assert_eq!(before,State::generate(8602));
}

#[test]
fn gold_bolt_rewards_and_flag_only_proxies_use_the_real_registered_class() {
    let native=i32::from(rc_game::moby_update::classes::gold_bolt::CLASSES[0]);
    assert_eq!(native,1134);
    for reward in [96,100,132,169] {
        assert_eq!(randomizer::display_class(reward,-1),native);
    }
    for reward in [48,52,53]{assert_eq!(randomizer::display_class(reward,-1),511);}
    assert_eq!(randomizer::display_class(999,-1),1134);
    assert_eq!(randomizer::display_class(34,-1),511);
    assert_eq!(randomizer::display_class(70,-1),750);
}

#[test]
fn actual_randomizer_visuals_preserve_model_size_and_positions_over_long_sessions() {
    use rc_game::weapon_evo::pickup_visuals;
    let state=State::generate(9001);
    let mut world=randomizer::WorldState::default();
    // Exercise real coordinate, Gold Bolt/proxy and inventory reward placements.
    for reward in [70,48,15] {
        let source=state.placements.iter().position(|&r|r==reward).unwrap();
        world.anchors.push(randomizer::Anchor{source,position:[10.,20.,30.],
            mission:None,infobot:false,shown:true});
        world.ready.insert(source);
    }
    let original=world.visuals(state,true);
    for elapsed in [240,480,600,1800,36000,240*180*3600,u64::MAX] {
        world.elapsed=elapsed;
        let visuals=world.visuals(state,true);
        assert_eq!(visuals.len(),3);
        for (_,visual) in &visuals {
            assert!((0. ..std::f32::consts::TAU).contains(&visual.yaw));
            assert_eq!(&visual.position[..2],&[10.,20.]);
            assert!((30.85..=31.15).contains(&visual.position[2]));
            let rows=pickup_visuals::display_rows([0.,0.,visual.yaw]);
            let length=rows[0].iter().map(|v|v*v).sum::<f32>().sqrt();
            assert!((length-1.).abs()<0.0001);
        }
        if elapsed%(240*180)==0 { assert_eq!(original,visuals); }
    }
    assert_eq!(state,State::generate(9001));
}
