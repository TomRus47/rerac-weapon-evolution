use super::*;
use crate::weapon_evo::save::Progress;
use rc_formats::save_game::{ChunkTables,ItemRecord,ITEM_COUNT,VENDOR_TABLE_LEN};

fn inventory()->ItemTables {
    let mut items=ItemTables {vendor_items:[0;VENDOR_TABLE_LEN],records:[ItemRecord([0;0x18]);ITEM_COUNT],slot_type:[0;ITEM_COUNT],item_defs_addr:0};
    for id in super::super::ammo_mode::AMMO_IDS {
        items.records[id].0[8..10].copy_from_slice(&1u16.to_le_bytes());
        items.records[id].0[0xe..0x10].copy_from_slice(&50u16.to_le_bytes());
        items.records[id].0[0x12..0x14].copy_from_slice(&3u16.to_le_bytes());
    }
    items
}
fn game()->GameState {GameState::zeroed(ChunkTables {global:vec![],level:vec![]})}
fn settings()->Settings {Settings {enabled:true,early_bias_percent:0,..Default::default()}}

#[test]
fn bomb_glove_moves_and_all_six_optional_sites_are_possible_not_mandatory() {
    let mut visited=[false;6];let mut omitted=[false;6];let mut bomb_levels=std::collections::HashSet::new();
    for seed in 1..=1000 {
        let state=State::generate(seed,0);
        assert!(state.valid());
        for i in 0..6 {
            let selected=state.item_at(80+i).is_some();
            visited[i]|=selected;omitted[i]|=!selected;
        }
        assert!(!(state.item(14,2).is_some() && state.item(14,3).is_some()));
        let slot=state.placements.iter().position(|&i|i==10).unwrap();
        bomb_levels.insert(layout::level(slot).unwrap());
    }
    assert!(visited.into_iter().all(|x|x));assert!(omitted.into_iter().all(|x|x));
    assert!(bomb_levels.len()>5,"Bomb Glove is no longer a forced early unlock");
}

#[test]
fn new_game_is_wrench_only_native_grants_cannot_bypass_rewards_and_gadgets_still_work() {
    let mut gs=game();let mut session=SessionState::default();let items=inventory();
    gs.global.owned[10]=1;gs.global.ammo[10]=40;gs.global.owned[12]=1;
    gs.global.quick_select=[10,12,0,0,0,0,0,0];
    initialize_new_game(&mut gs,settings(),91);
    // The wrench remains owned, but has no icon in the native quick-select ring.
    assert_eq!(gs.global.quick_select,[0;8]);
    assert_eq!(gs.global.equipped[0],8);assert_eq!(gs.global.owned[12],1);
    for item in ITEMS {
        gs.give_item(item,true,&items,&mut session);
        assert_eq!(gs.global.owned[item],0);assert_eq!(gs.global.acquired[item],0);assert_eq!(gs.global.ammo[item],0);
    }
    gs.apply_level_start(1,&items,&mut session);
    assert_eq!(gs.global.ammo[10],0);assert_eq!(gs.global.owned[10],0);assert_eq!(gs.global.equipped[0],8);
    gs.give_item(34,false,&items,&mut session);
    assert_eq!(gs.global.owned[34],1);
}

#[test]
fn survival_cannot_unlock_unfound_weapons_or_lock_empty_found_weapons() {
    let mut gs=game();let mut session=SessionState::default();
    initialize_new_game(&mut gs,settings(),92);
    gs.weapon_progress.golden.collect(16);
    gs.global.ammo[10]=100;gs.global.owned[10]=1;gs.global.owned[12]=1;
    gs.global.quick_select=[8,10,12,0,0,0,0,0];
    for on in [true,false] {
        super::super::ammo_mode::reconcile_configured(&mut gs,&mut session,on,1_000_000,false,&Default::default());
        assert_eq!(gs.global.owned[16],1);assert_eq!(gs.global.owned[10],0);assert_eq!(gs.global.owned[14],0);
        assert_eq!(gs.global.owned[12],1);assert!(!gs.global.quick_select.contains(&10));
    }
}

#[test]
fn swimming_pickup_unlocks_without_equipping_and_deferred_equip_survives_save_load() {
    let mut gs=game();let mut session=SessionState::default();let items=inventory();
    initialize_new_game(&mut gs,settings(),93);
    let mut hero=crate::hero::Hero::new();hero.in_water=1;hero.state=0x33;hero.group=0x11;
    assert!(grant_with_equip(&mut gs,&mut session,&items,13,&Default::default(),false));
    assert_eq!(gs.global.owned[13],1);assert!(gs.global.ammo[13]>0);assert!(gs.global.quick_select.contains(&13));
    assert_eq!(session.temp_hand,0);assert_eq!(gs.global.equipped[0],8);
    let encoded=gs.weapon_progress.encode(b"swimming save").unwrap();
    gs.weapon_progress=Progress::decode(&encoded,b"swimming save").unwrap();
    assert_eq!(gs.weapon_progress.golden.pending_equip,13);
    assert!(!finish_deferred_equip(&mut gs,&mut session,&hero,0));
    hero.in_water=0;hero.state=0;hero.group=0;hero.health=0;
    assert!(!finish_deferred_equip(&mut gs,&mut session,&hero,0));
    hero.health=4;
    assert!(!finish_deferred_equip(&mut gs,&mut session,&hero,2));
    assert!(finish_deferred_equip(&mut gs,&mut session,&hero,0));
    assert_eq!(session.temp_hand,13);assert_eq!(gs.weapon_progress.golden.pending_equip,0);
    assert!(!finish_deferred_equip(&mut gs,&mut session,&hero,0));
    let ammo=gs.global.ammo;
    assert!(!grant_with_equip(&mut gs,&mut session,&items,13,&Default::default(),true));
    assert_eq!(gs.global.ammo,ammo,"one discovery cannot be recollected as an ammo refill");
}

#[test]
fn native_gold_bolts_unselected_and_gold_shop_models_disappear_without_native_rewards() {
    use crate::moby_update::classes::{bomb_water::tests::{Bench,pool},item_offer};
    let mut classes=item_offer::CLASSES.to_vec();classes.extend([1134,1120,185,925]);
    let mut bench=Bench::new(pool(10.,-10.),&classes,[20.,20.,10.]);
    let progress=State::generate(100,0);
    let slot=layout::locations(true).into_iter().find(|&s|progress.item_at(s).is_none()).unwrap();
    bench.svc.level=(slot/4) as u32;bench.svc.bomb_evo.golden_settings.enabled=true;
    bench.svc.bomb_evo.state_progress.golden=progress;
    let bolt=bench.create(1134);
    crate::moby_update::services::pvar::set_i32(&mut bench.table.mobys[bolt].pvars,0,(slot%4) as i32);
    let offers:Vec<_>=item_offer::CLASSES.into_iter().map(|c|bench.create(c)).collect();
    let suck=bench.create(1120);let morph=bench.create(185);let dealer=bench.create(925);
    let mut world=World::new(&mut bench.table,&bench.hero,&mut bench.rng,&bench.ct,&mut bench.svc,0);
    assert!(replace_bolt(&mut world,bolt));assert!(world.m(bolt).state>=128);
    for id in offers {item_offer::update(&mut world,id);assert!(world.m(id).state>=128);}
    crate::moby_update::classes::units::eudora_story::suck_cannon_update(&mut world,suck);
    crate::moby_update::classes::units::oltanis_story::morph_update(&mut world,morph);
    crate::moby_update::classes::units::rilgar_story::salesman_update(&mut world,dealer);
    for id in [suck,morph,dealer] {assert!(world.m(id).state>=128);}
    assert!(world.svc.interact.writes.is_empty());assert!(world.svc.cinematic.requests.is_empty());
}

#[test]
fn submerged_collection_requests_no_scene_and_no_hand_swap() {
    use crate::moby_update::classes::bomb_water::tests::{Bench,pool};
    let mut bench=Bench::new(pool(10.,-10.),&[1134],[20.,20.,10.]);
    let progress=State::generate(101,0);
    let slot=layout::locations(true).into_iter().find(|&s|progress.item_at(s).is_some()).unwrap();
    let item=progress.item_at(slot).unwrap();
    bench.svc.level=(slot/4) as u32;bench.svc.bomb_evo.golden_settings.enabled=true;
    bench.svc.bomb_evo.state_progress.golden=progress;
    bench.hero.state=0x33;bench.hero.in_water=1;bench.hero.group=0x11;
    let bolt=bench.create(1134);bench.table.mobys[bolt].position=[20.,20.,10.,1.];
    crate::moby_update::services::pvar::set_i32(&mut bench.table.mobys[bolt].pvars,0,(slot%4) as i32);
    let mut world=World::new(&mut bench.table,&bench.hero,&mut bench.rng,&bench.ct,&mut bench.svc,0);
    assert!(replace_bolt(&mut world,bolt));world.svc.golden.set_ready(item);tick(&mut world);
    assert!(world.svc.interact.writes.iter().any(|write|matches!(write,
        interact::GameWrite::GiveItem {item:i,equip:false} if *i==item)));
    assert!(world.svc.cinematic.requests.is_empty());
}

#[test]
fn custom_models_keep_their_anchor_and_readiness_across_checkpoint_reload() {
    let progress=(1..100).map(|s|State::generate(s,0)).find(|s|s.item_at(80).is_some()).unwrap();
    let item=progress.item_at(80).unwrap();
    let mut world=WorldState::from_gameplay(13,&progress,&[],Some([430.,592.,403.]));
    assert_eq!(world.rewards.len(),1);assert!(world.visuals(&progress,true).is_empty());
    world.set_ready(item);
    assert_eq!(world.visuals(&progress,true)[0].1.position,[430.,592.,403.]);
    world.reset_for_reload();assert_eq!(world.visuals(&progress,true).len(),1);
    let mut claimed=progress;claimed.collect(item);
    assert!(world.visuals(&claimed,true).is_empty());assert!(world.visuals(&progress,false).is_empty());
}

#[test]
fn notices_keep_native_font_bytes_and_never_use_unrelated_message_ids() {
    let name=b"Native \x8a weapon";
    for lang in 0..8 {
        let text=localized_notice(23,lang,name);
        assert!(text.ends_with(name));assert!(!text.windows(7).any(|s|s==b"Paradox"));
    }
    assert_eq!(localized_notice(23,5,b"MIPS"),b"Arma ottenuta: MIPS");
    assert_eq!(localized_notice(23,0,b""),notice(23));
}


#[test]
fn one_reward_per_planet_preserves_all_fifteen_and_access_guarantees() {
    for bias in [0,25,50,75,100] {for seed in 1..=1000 {
        let state=State::generate_with_planet_limit(seed,bias,true);
        assert!(state.valid(),"seed {seed}, bias {bias}");
        let mut planets=std::collections::HashSet::new();
        for (slot,&item) in state.placements.iter().enumerate().filter(|(_,i)|**i!=0) {
            assert!(planets.insert(super::layout::level(slot).unwrap()));
            assert!(super::layout::placement_allowed(slot,item as usize));
        }
        assert_eq!(planets.len(),15);
        let initial=state;let mut gs=game();
        gs.weapon_progress.golden=state;
        // A changed INI does not regenerate a save's owned layout.
        assert_eq!(gs.weapon_progress.golden,initial);
    }}
}
