//! Cross-crate contract used by the engine reward renderer and grant handler.
use rc_game::weapon_evo::{Evolution,golden::{self,Settings,State,WorldState}};
#[test]
fn renderer_and_engine_grants_use_public_reward_state_and_native_preview_rows() {
    let mut evolution=Evolution::default();
    evolution.golden_settings=Settings{enabled:true,early_bias_percent:50,..Default::default()};
    let mut progress=rc_game::weapon_evo::save::Progress::default();
    progress.golden=State::generate(1,50);
    evolution.restore_progress(&progress);
    let mut world=WorldState::default();
    world.set_ready(23);
    assert!(world.visuals(&evolution.golden_progress(),true).is_empty());
    progress.golden.collect(23);
    evolution.commit_golden_reward(progress.golden);
    assert!(evolution.golden_progress().collected(23));
    let rotation=rc_formats::moby_light::rotation_rows([0.,std::f32::consts::FRAC_PI_2,0.]);
    let rows:[[f32;3];3]=rotation.map(|r|[r[0],r[1],r[2]].map(f32::from_bits));
    assert!(rows.iter().flatten().all(|f|f.is_finite()));
    assert_eq!(golden::notice(23),b"RYNO acquired!");
}

#[test]
fn ammo_crates_and_forced_packs_never_offer_an_unfound_randomized_weapon() {
    use rc_game::{hero::Hero,moby_runtime::MobyTable,moby_update::{ClassTable,Services,services::World,classes::pickup},rng::Rng};
    let mut hero=Hero::new();hero.owned.0[10]=1;hero.owned.0[23]=1;
    let mut records=vec![[0u8;0x18];37];
    for item in rc_game::weapon_evo::ammo_mode::AMMO_IDS {records[item][14..16].copy_from_slice(&50u16.to_le_bytes());}
    let inventory=pickup::ItemTables::new(&records,[0xff;12]).with_hero(&hero);
    let mut table=MobyTable::default();let mut rng=Rng::new();let classes=ClassTable::default();let mut svc=Services::new();
    let mut progress=rc_game::weapon_evo::save::Progress::default();progress.golden=State::generate(108,0);
    svc.bomb_evo.restore_progress(&progress);
    let mut world=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);world.inventory=&inventory;
    for unrestricted in [false,true] {assert_eq!(pickup::random_ammo_item(&mut world,unrestricted),None);}
    let mut ty=15;assert_eq!(pickup::ammo_pick(&mut world,None,&mut ty),0);assert_eq!(ty,0);
    progress.golden.collect(16);world.svc.bomb_evo.commit_golden_reward(progress.golden);
    for _ in 0..100 {
        let mut ty=15;assert!((1..=2).contains(&pickup::ammo_pick(&mut world,None,&mut ty)));assert_eq!(ty,16);
        assert_eq!(pickup::random_ammo_item(&mut world,true),Some(16));
    }
    progress.golden.collect(23);world.svc.bomb_evo.commit_golden_reward(progress.golden);
    let choices:std::collections::HashSet<_>=(0..3000).map(|_|pickup::random_ammo_item(&mut world,false).unwrap()).collect();
    assert_eq!(choices,std::collections::HashSet::from([16,23]));
    // Survival retains its pack count/rarity controls, but can never grant
    // a weapon which the randomized save has not physically discovered.
    world.svc.bomb_evo.ammo_unlock_mode=true;
    world.svc.bomb_evo.ammo_mode_settings.drop_chances.fill(0.0);
    let ryno=rc_game::weapon_evo::ammo_mode::AMMO_IDS.iter().position(|&i|i==23).unwrap();
    world.svc.bomb_evo.ammo_mode_settings.drop_chances[ryno]=1.;
    let mut sizes=std::collections::HashSet::new();let mut heavy_count=0;
    for _ in 0..4000 {
        let mut ty=16;let count=pickup::ammo_pick(&mut world,None,&mut ty);
        assert!([16,23].contains(&ty));heavy_count+=usize::from(ty==23);
        assert!((1..=4).contains(&count));sizes.insert(count);
    }
    assert_eq!(sizes,std::collections::HashSet::from([1,2,3,4]));
    assert!((800..1200).contains(&heavy_count),"shared 25% heavy-ammo branch remains active in randomized Survival");
}

#[test]
fn authored_reward_sites_are_registered_before_render_setup_and_survive_reload() {
    let progress=(1..1000).map(|s|State::generate(s,0)).find(|s|s.item_at(81).is_some()).unwrap();
    let item=progress.item_at(81).unwrap();
    let mut world=WorldState::from_gameplay(3,&progress,&[],None);
    assert!(world.rewards.iter().any(|r|r.item==item));
    world.set_ready(item);
    let before=world.visuals(&progress,true);
    assert!(before.iter().any(|(i,v)|*i==item&&v.position==[346.85,193.75,51.]));
    world.reset_for_reload();assert_eq!(world.visuals(&progress,true),before);
}

#[test]
fn weapons_only_layout_does_not_reserve_an_early_power_weapon() {
    let mut unrestricted_early=0;
    for seed in 1..=500 {
        let state=State::generate(seed,0);
        assert!(state.valid());
        assert_eq!(state.placements.iter().filter(|&&i|i!=0).count(),15);
        if !state.placements.iter().enumerate().any(|(slot,&item)|
            [11,13].contains(&item) && ((slot<80 && matches!(slot/4,3|4)) || matches!(slot,81|86))) {
            unrestricted_early+=1;
        }
    }
    // Native route reachability is verified by the generator's library tests;
    // no fixed early Devastator/Visibomb slot is part of the public contract.
    assert!(unrestricted_early>0);
}
