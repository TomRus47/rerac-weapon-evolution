//! Local opt-in location verification. No production setting or save format is
//! changed. The engine places this run on its own memory card directory.
use super::{golden, Evolution};
use crate::game_state::GameState;
use crate::moby_update::services::{pvar as p,World};

pub const ENV: &str = "RC_WEAPON_LOCATION_TEST";
pub fn enabled() -> bool {
    std::env::var(ENV).is_ok_and(|v|matches!(v.trim().to_ascii_lowercase().as_str(),"1"|"true"|"on"))
}
pub fn configure(e:&mut Evolution) {
    if enabled() {e.golden_settings.enabled=true;e.ammo_unlock_mode=false;}
}
/// New test games only; loading a save never regrants items or clears progress.
pub fn prepare_new_game(gs:&mut GameState) {
    gs.global.level=18;
    gs.global.game_beaten=0;gs.global.completes=0;
    gs.global.owned.fill(0);gs.global.acquired.fill(0);gs.global.ammo.fill(0);
    for item in 1..gs.global.owned.len() {
        if !golden::ITEMS.contains(&item) {gs.global.owned[item]=1;gs.global.acquired[item]=1;}
    }
    gs.global.equipped=[8,0,5,3,0,0,0];
    gs.global.last_hand_item=8;gs.global.wrench_held=1;gs.global.thruster_last=1;
    gs.global.quick_select=[8,0,0,0,0,0,0,0];
    gs.global.gold_weapons.fill(0);gs.global.max_hp=8;
    gs.global.flags[4]=1;gs.global.flags[5]=1;gs.global.flags[8]=1;
    gs.global.planet_unlocked.fill(0);gs.global.map_order.fill(0);
    // 0 terminates the ship's destination list; initial Veldin is not a travel
    // destination. Only playable return destinations 1..18 belong in the list.
    for planet in 1..19 {gs.unlock_planet(planet);}
    for level in &mut gs.levels {
        level.visited=2;level.missions.fill(0xff);level.gold_bolts.fill(0);
        level.killed.fill(0);
    }
}
/// Each extra site uses a free native Gold Bolt bit on that planet (all six
/// have fewer than four originals). Thus collection survives save/load without
/// a new companion format, duplicate reward indices or respawning on death.
pub fn extra_bolt_index(level:u32)->Option<usize> {
    golden::COUNTS.get(level as usize).copied().map(usize::from).filter(|&n|n<4)
}
/// Native sites retain their authored index; extra sites use one unused bit.
pub fn bolt_index(site:golden::locations::Site)->Option<usize> {
    if site.slot<80 {
        let index=site.slot%4;
        (site.slot/4==site.level as usize && index<golden::COUNTS.get(site.level as usize).copied()? as usize).then_some(index)
    } else {extra_bolt_index(site.level)}
}
/// Enough spare actors even when a native mission-spawn filter hid a marker.
pub fn actor_reserve(level:u32)->usize {
    golden::COUNTS.get(level as usize).copied().unwrap_or(0) as usize
        + usize::from(matches!(level,1|3|10|11|12|13))
}
pub fn spawn_bolt(w:&mut World,site:golden::locations::Site)->bool {
    use crate::moby_update::classes::gold_bolt::pv;
    let Some(index)=bolt_index(site) else{return true;};
    let Some(id)=w.create_moby(1134) else{return false;};
    let m=w.mm(id);m.position=[site.base[0],site.base[1],site.base[2],1.];
    m.state=0;m.visible=1;m.update_dist=0xff;
    m.pvars.resize(pv::LEN,0);
    p::set_i32(&mut m.pvars,pv::INDEX,index as i32);
    p::set_i32(&mut m.pvars,pv::NO_CUTAWAY,1);
    p::set_i32(&mut m.pvars,pv::CUBOID_A,-1);p::set_i32(&mut m.pvars,pv::CUBOID_B,-1);
    true
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_game_starts_on_final_veldin_with_every_mission_complete_and_no_rewards_owned() {
        let mut gs=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
        gs.weapon_progress.golden=golden::State::generate(73,0);
        let layout=gs.weapon_progress.golden;
        prepare_new_game(&mut gs);
        assert_eq!(gs.global.level,18);assert_eq!(gs.global.game_beaten,0);
        assert_eq!(crate::travel::ship_for(&gs.global.planet_unlocked,1),2);
        assert!(gs.global.planet_unlocked[1..19].iter().all(|&v|v!=0));
        assert_eq!(gs.weapon_progress.golden,layout);assert!(layout.valid());
        for item in golden::ITEMS {assert_eq!((gs.global.owned[item],gs.global.ammo[item]),(0,0));}
        for item in [1,2,3,4,5,6,7,8,12,22,26,27,28,29,30,31,32] {assert_eq!(gs.global.owned[item],1);}
        assert_eq!(gs.global.quick_select,[8,0,0,0,0,0,0,0]);
        for data in &gs.levels {assert_eq!(data.missions,[0xff;16]);assert_eq!(data.gold_bolts,[0;4]);}
    }
    #[test] fn native_ship_menu_lists_every_return_destination_and_selects_each_one() {
        use crate::menus::pause::{List,planet_select};
        let mut gs=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
        prepare_new_game(&mut gs);
        assert_eq!(&gs.global.map_order[..18],&(1..19).collect::<Vec<i32>>());
        assert_eq!(&gs.global.map_order[18..],&[0,0]);
        assert_eq!(gs.global.planet_unlocked[0],0);
        let names:Vec<_>=(0..20).map(|id|(id,id+100)).collect();
        for dest in 1..19 {
            let mut list=List{flags:0,items_addr:0,items:vec![],above:0,below:0,cursor:0,scroll:0};
            planet_select::build_list(&mut list,&gs,dest,&names);
            assert_eq!(list.items.len(),18);
            assert_eq!(list.cursor,dest-1);
            assert_eq!(gs.global.map_order[list.cursor as usize],dest);
            assert!(list.items.iter().enumerate().all(|(i,item)|item.action==1 && item.label==i as i16+1));
        }
    }
    #[test] fn all_extra_sites_have_a_unique_free_native_collection_bit() {
        for level in [1,3,10,11,12,13] {
            let index=extra_bolt_index(level).unwrap();
            assert!(index<4);assert_eq!(index,golden::COUNTS[level as usize] as usize);
            assert_eq!(golden::locations::all_for_level(level,Some([1.,2.,3.])).len(),1);
        }
        assert!(extra_bolt_index(2).is_none());
    }
    #[test] fn extra_gold_bolt_is_a_native_pickup_and_records_its_free_collection_bit() {
        use crate::{hero::Hero,moby_runtime::{ClassInfo,Moby,MobyTable},rng::Rng};
        use crate::moby_update::{classes::gold_bolt,services::{ClassTable,Services}};
        let mut classes=ClassTable::default();
        classes.classes.insert(1134,(ClassInfo{update_fn:Some(gold_bolt::UPDATE_FN),scale:1.,..Default::default()},None));
        let mut table=MobyTable::new(vec![Moby::init_instance(0,0,None)],16);
        let mut hero=Hero::new();hero.pos=[crate::ps2v::Pf::f(20.),crate::ps2v::Pf::f(20.),crate::ps2v::Pf::f(10.),crate::ps2v::Pf::ONE];
        let mut rng=Rng::new();let mut svc=Services::new();svc.level=1;
        svc.bomb_evo.golden_settings.enabled=true;svc.bomb_evo.state_progress.golden=golden::State::generate(73,0);svc.golden.test_mode=true;
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
        assert!(spawn_bolt(&mut w,golden::locations::Site{slot:85,level:1,name:"test",base:[20.,20.,10.],underwater:true}));
        let id=w.table.mobys.iter().position(|m|m.o_class==1134).unwrap();
        assert!(!golden::replace_bolt(&mut w,id));
        gold_bolt::update(&mut w,id);gold_bolt::update(&mut w,id);
        assert!(w.svc.interact.writes.iter().any(|write|matches!(write,crate::moby_update::interact::GameWrite::GoldBolt{level:1,index:3})));
        assert!(w.m(id).state>=0x80);
        // A revisit sees the same native bit and immediately deletes the copy.
        assert!(spawn_bolt(&mut w,golden::locations::Site{slot:85,level:1,name:"test",base:[20.,20.,10.],underwater:true}));
        let id=w.table.mobys.iter().position(|m|m.o_class==1134&&m.state<0x80).unwrap();
        gold_bolt::update(&mut w,id);assert!(w.m(id).state>=0x80);
    }
}
