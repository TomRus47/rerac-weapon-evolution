//! This external-crate test covers the public interface consumed by rc-engine.
use rc_game::{hero::{Hero, items::{ItemData, ItemEnv, NoHits}, pyro_lava::Settings},
    moby_runtime::MobyTable, moby_update::real_hp::StreamSnapshot, pad::PadState};

#[test]
fn engine_can_draw_and_clear_the_public_lava_stream_without_private_field_access() {
    let mut hero=Hero::new();
    let damage=StreamSnapshot{damage_hp_per_second:100.0,interval:1,burn_color:None};
    hero.weapons.pyro.lava.launch([0.0;3],[1.0,0.0,0.0],Settings::default(),damage);
    let data=ItemData::default();let pad=PadState::default();
    let env=ItemEnv{ mobys: None,data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,camera:None,camera_up:None,targets:&[]};
    hero.weapons.pyro.lava.step(&mut MobyTable::default(),&env,&mut NoHits,&mut hero.fx.parts);
    let geometry=hero.weapons.pyro.lava.quads([0.0,-5.0,2.0]);
    assert_eq!(geometry.len(),2);
    assert!(geometry.iter().all(|q| q.corners.iter().flatten().all(|v|v.is_finite())));
    hero.weapons.pyro.lava.clear();
    assert!(hero.weapons.pyro.lava.quads([0.0;3]).is_empty());
}

#[test]
fn engine_draws_all_three_lanes_through_the_unchanged_quad_interface() {
    let mut hero=Hero::new();
    let damage=StreamSnapshot{damage_hp_per_second:100.0,interval:1,burn_color:None};
    let settings=Settings{gravity:0.0,launch_angle_degrees:0.0,..Default::default()};
    let directions=[[1.0,0.0,0.0],[0.8,-0.6,0.0],[0.8,0.6,0.0]];
    let data=ItemData::default();let pad=PadState::default();
    let env=ItemEnv{ mobys: None,data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,camera:None,camera_up:None,targets:&[]};
    for _ in 0..5 {
        hero.weapons.pyro.lava.launch_lanes([0.0;3],&directions,settings,damage);
        hero.weapons.pyro.lava.step(&mut MobyTable::default(),&env,&mut NoHits,&mut hero.fx.parts);
    }
    let geometry=hero.weapons.pyro.lava.quads([0.0,-5.0,2.0]);
    assert_eq!(geometry.len(),30);
    assert!(geometry.iter().all(|q|q.corners.iter().flatten().all(|v|v.is_finite())));
    hero.weapons.pyro.lava.clear();
    assert!(hero.weapons.pyro.lava.is_empty());
    assert!(hero.weapons.pyro.lava.quads([0.0;3]).is_empty());
}
