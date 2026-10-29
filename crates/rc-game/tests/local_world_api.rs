//! Isolated-process test of the editable test profile and the full marker census.
use rc_game::{game_state::{GameState,SessionState},hero::Hero,weapon_evo::{self,golden}};
use rc_formats::save_game::{ChunkTables,ChunkDesc,ItemTables,ItemRecord};

fn tables()->ChunkTables {
    let fields=|v:&[(i32,u32)]|v.iter().map(|&(id,size)|ChunkDesc{id,size,addr:0}).collect();
    ChunkTables{global:fields(&[(0,4),(1,4),(5,128),(9,148),(10,37),(11,37),(13,32),(14,20),(19,4),(20,80),(21,4),(22,4),(23,4),(30,40),(31,4),(32,28)]),
        level:fields(&[(3001,1),(3003,4),(3004,16),(3005,256)])}
}

fn native_bolts(level:u32,count:usize)->Vec<u8> {
    use rc_formats::gameplay as g;
    // Include the two authored weapon actors now used as optional sites. A
    // bolt-only fixture silently lost any weapon randomized into either one.
    let native=match level {4=>Some(1120),14=>Some(1354),_=>None};
    let actors=count+usize::from(native.is_some());
    let mut data=vec![0u8;0x110+actors*g::MOBY_INSTANCE_SIZE+actors*12];
    let put=|data:&mut [u8],at:usize,v:u32|data[at..at+4].copy_from_slice(&v.to_le_bytes());
    put(&mut data,g::MOBY_INSTANCES_POINTER,0x100);put(&mut data,0x100,actors as u32);
    let ptable=0x110+actors*g::MOBY_INSTANCE_SIZE;let pbytes=ptable+actors*8;
    put(&mut data,g::PVAR_TABLE_POINTER,ptable as u32);put(&mut data,g::PVAR_DATA_POINTER,pbytes as u32);
    for i in 0..actors {
        let p=0x110+i*g::MOBY_INSTANCE_SIZE;
        put(&mut data,p,g::MOBY_INSTANCE_SIZE as u32);put(&mut data,p+0x18,if i<count {1134}else{native.unwrap()});
        put(&mut data,p+0x30,(20.+i as f32*30.).to_bits());put(&mut data,p+0x34,20f32.to_bits());put(&mut data,p+0x38,10f32.to_bits());
        put(&mut data,p+0x58,i as u32);put(&mut data,ptable+i*8,(i*4) as u32);put(&mut data,ptable+i*8+4,4);
        put(&mut data,pbytes+i*4,i as u32);
    }
    data
}

#[test]
fn editable_test_boot_retains_persistence_menus_and_every_reward_location() {
    // Environment switches are confined to a child, never raced with other tests.
    if std::env::var_os("RC_LOCATION_API_CHILD").is_none() {
        let path=std::env::temp_dir().join(format!("weapon-location-v74-{}.ini",std::process::id()));
        std::fs::write(&path,include_str!("../src/weapon_evo_release.ini")).unwrap();
        let result=std::process::Command::new(std::env::current_exe().unwrap_or_else(|_|std::path::PathBuf::from(std::env::args_os().next().unwrap())))
            .args(["--exact","editable_test_boot_retains_persistence_menus_and_every_reward_location","--nocapture"])
            .env("RC_LOCATION_API_CHILD","1").env("RC_WEAPON_LOCATION_TEST","1")
            .env("RC_WEAPON_EVOLUTION_INI",&path).output().unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(result.status.success(),"{}\n{}",String::from_utf8_lossy(&result.stdout),String::from_utf8_lossy(&result.stderr));
        return;
    }
    let template=GameState::zeroed(tables()).to_save_file().to_bytes();
    let mut gs=GameState::new_game(tables(),&template).unwrap();
    assert_eq!(gs.global.level,18);assert!(gs.weapon_progress.golden.valid());assert!(gs.weapon_progress.golden.active());
    let items=ItemTables{vendor_items:[0;20],records:[ItemRecord([0;0x18]);37],slot_type:[0;37],item_defs_addr:0};
    let mut session=SessionState::default();
    // The same native initialization sequence used by direct boot.
    gs.apply_level_start(0,&items,&mut session);gs.on_veldin_clank_init(&mut session);
    gs.apply_transition(18);gs.apply_level_start(18,&items,&mut session);
    assert_eq!(session.hp,8);assert_eq!(session.clank_hidden,0);
    for item in golden::ITEMS {assert_eq!(gs.global.owned[item],0);assert_eq!(gs.global.ammo[item],0);}
    for data in &gs.levels {assert_eq!(data.missions,[0xff;16]);assert_eq!(data.gold_bolts,[0;4]);}
    assert!(gs.global.planet_unlocked[1..19].iter().all(|&v|v==1));
    assert_eq!(rc_game::travel::ship_for(&gs.global.planet_unlocked,18),2);
    use rc_game::menus::mode::{TriggerIn,Trigger,in_level_trigger};
    assert_eq!(in_level_trigger(&TriggerIn{mode:0,frames_in_mode:30,level:18,pressed:rc_game::pad::button::START,connected:true,hp:session.hp,..Default::default()}),Some(Trigger::Menu(0)));
    let restored=GameState::from_save_file(tables(),&gs.to_save_file());assert_eq!(restored.1.total(),0);
    assert_eq!(restored.0.global.owned,gs.global.owned);
    let mut list=rc_game::menus::pause::List{flags:0,items_addr:0,items:vec![],above:0,below:0,cursor:0,scroll:0};
    let names:Vec<_>=(0..20).map(|id|(id,id+100)).collect();
    rc_game::menus::pause::planet_select::build_list(&mut list,&restored.0,18,&names);
    assert_eq!(list.items.len(),18);assert_eq!(list.cursor,17);
    assert_eq!(&restored.0.global.map_order[18..],&[0,0]);
    assert_eq!(golden::card_root("saves".into(),true),std::path::PathBuf::from("saves/weapon_location_test"));

    use rc_game::{moby_runtime::{Moby,MobyTable,ClassInfo},moby_update::{Services,ClassTable,services::World,classes::gold_bolt},rng::Rng};
    // A reproducible census, not an assertion about a fresh random layout.
    // Both native weapon actors are exercised as weapons; Oltanis already uses
    // all four native Gold Bolt bits, so its extra site's empty-marker policy
    // is outside this fixture's native-card marker census.
    gs.weapon_progress.golden=(1..10000).map(|seed|golden::State::generate(seed,0))
        .find(|s|s.item_at(86).is_some()&&s.item_at(87).is_some()).unwrap();
    let mut weapons=0;let mut bolts=0;
    for level in 1..19u32 {
        let mut svc=Services::new();svc.level=level;svc.bomb_evo.restore_progress(&gs.weapon_progress);
        let gameplay=native_bolts(level,golden::COUNTS[level as usize] as usize);
        svc.golden=golden::WorldState::from_gameplay(level,&gs.weapon_progress.golden,&gameplay,Some([100.,100.,10.]));
        let mut classes=ClassTable::default();classes.classes.insert(1134,(ClassInfo{update_fn:Some(gold_bolt::UPDATE_FN),scale:1.,..Default::default()},None));
        let mut table=MobyTable::new(vec![Moby::init_instance(0,0,None)],weapon_evo::location_test::actor_reserve(level)+2);
        let hero=Hero::new();let mut rng=Rng::new();
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
        golden::tick(&mut w);
        let ids:Vec<_>=w.table.mobys.iter().enumerate().filter(|(_,m)|m.o_class==1134&&m.state<0x80).map(|(id,_)|id).collect();
        let count=ids.len();
        for &id in &ids {gold_bolt::update(&mut w,id);gold_bolt::update(&mut w,id);assert!(w.m(id).state<0x80);}
        w.svc.golden.reset_for_reload();golden::tick(&mut w);
        assert_eq!(w.table.mobys.iter().filter(|m|m.o_class==1134&&m.state<0x80).count(),count,"no duplicate markers after reload");
        weapons+=w.svc.golden.rewards.len();bolts+=count;
        assert_eq!(count+w.svc.golden.rewards.len(),golden::COUNTS[level as usize] as usize
            +golden::locations::all_for_level(level,Some([100.,100.,10.])).len()
            +golden::locations::native_weapon_sites(level,&gameplay).len());
        if let Some(&id)=ids.first() {
            let index=i32::from_le_bytes(w.m(id).pvars[..4].try_into().unwrap()) as usize;
            w.svc.interact.game.gold_bolt_bits.resize(19,[0;4]);w.svc.interact.game.gold_bolt_bits[level as usize][index]=1;
            w.delete_moby(id);w.svc.golden.reset_for_reload();golden::tick(&mut w);
            assert_eq!(w.table.mobys.iter().filter(|m|m.o_class==1134&&m.state<0x80).count(),count-1);
        }
    }
    assert_eq!((weapons,bolts),(15,33));
}
