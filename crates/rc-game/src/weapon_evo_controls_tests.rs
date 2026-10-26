//! Regression contracts for physical input, slot ownership and v8 companions.
use super::*;
use crate::weapon_evo::{Evolution, save::Progress};
use rc_formats::save_game::ChunkTables;

fn pad(held:u32,prev:u32)->PadState {
    PadState {held,raw:held,prev_held:prev,prev_raw:prev,held_store:held,
        held_unmirrored:held,pressed:held&!prev,released:prev&!held,
        raw_pressed:held&!prev,raw_released:prev&!held,..Default::default()}
}
#[test]
fn fire_alias_is_a_union_while_l1_aims_and_only_r1_crouches() {
    for locked in [false,true] {
        for item in [8,10,16,23] {
            let fire=if item==8{button::SQUARE}else{button::CIRCLE};
            let p=gameplay_pad(&pad(button::R2,0),item,true,false,locked);
            assert_eq!(p.held&button::CROUCH,0);assert_ne!(p.pressed&fire,0);
            let p=gameplay_pad(&pad(fire,fire|button::R2),item,true,false,locked);
            assert_eq!(p.released&fire,0);assert_ne!(p.held&fire,0);
            let p=gameplay_pad(&pad(button::R2,fire|button::R2),item,true,false,locked);
            assert_eq!(p.pressed&fire,0);assert_eq!(p.released&fire,0);
            let p=gameplay_pad(&pad(button::L1|button::R1,0),item,true,false,locked);
            assert_eq!(p.held&button::CROUCH,button::R1);
            let p=gameplay_pad(&pad(button::R1,0),item,true,false,locked);
            assert_eq!(p.held&button::CROUCH,button::R1);
            let p=gameplay_pad(&pad(button::L1,0),item,true,false,locked);
            assert_eq!(p.held&button::CROUCH,0);
            assert_ne!(p.held&button::L1,0);
        }
    }
    let physical=pad(button::R2|button::L1,0);
    assert_eq!(gameplay_pad(&physical,10,false,true,false).held,physical.held);
}
#[test]
fn trigger_pressure_and_physical_direction_edges_do_not_leak() {
    let mut p=pad(button::UP|button::R2|button::L1,button::UP);
    p.raw=button::UP|button::R2|button::L1;p.prev_raw=0;
    p.raw_pressed&=!button::UP; // Native analog merge can hide this press.
    p.ly=Pf::f(-1.);p.pressure[11]=Pf::f(0.6);p.pressure[8]=Pf::f(0.8);
    let mapped=gameplay_pad(&p,10,true,true,false);
    assert_eq!(physical_edges(&p,false)&DIRECTION_MASK,button::UP);
    assert_eq!(mapped.raw&DIRECTION_MASK,0);assert_eq!(mapped.held&DIRECTION_MASK,button::UP);
    assert_eq!(mapped.pressure[5],Pf::f(0.6));assert_eq!(mapped.pressure[8],Pf::f(0.8));
    assert_eq!(mapped.pressure[9],Pf::ZERO);
    p.raw=0;assert_eq!(physical_edges(&p,false)&DIRECTION_MASK,0);
    assert_eq!(physical_edges(&pad(button::LEFT,0),true),button::RIGHT);
    assert_eq!(direction(button::UP|button::LEFT),None);
}
#[test]
fn assignments_are_owned_equippable_unique_and_independent_from_native_radial() {
    let mut state=State::default();
    assert!(!state.assign(0,15,false,0));assert!(!state.assign(0,35,true,4));
    assert!(state.assign(0,15,true,0));assert!(state.assign(1,15,true,0));
    assert_eq!(state.dpad,[0,15,0,0]);assert!(state.valid());
    assert!(state.assign(2,1,true,3));assert!(state.assign(3,4,true,2));
    let mut invalid=state;invalid.dpad[0]=15;assert!(!invalid.valid());
    assert_eq!(state.mode(9),None); // Suck Cannon is launcher-only.
    for id in [10,24,25] {assert_eq!(state.cycle(id),None);}
    assert_eq!([state.cycle(23),state.cycle(23),state.cycle(23)],[Some(1),Some(2),Some(0)]);
}
#[test]
fn equip_obeys_ownership_water_magnetic_surface_turret_and_grind_rules() {
    let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});
    gs.global.owned= [0;37];gs.global.owned[15]=1;gs.global.owned[4]=1;
    gs.global.acquired= [0;37]; // NG+ acquisition flags can be empty.
    let mut h=Hero::new();h.health=4;h.state=0;
    let state=State{dpad:[15,4,0,0],..Default::default()};
    let mut sess=SessionState::default();
    assert!(equip(&state,button::UP,&gs,&mut sess,&h,|_|Some(0)));assert_eq!(sess.temp_hand,15);
    for forbidden in [0x32,0x72,0x78] {h.state=forbidden;assert!(!equip(&state,button::UP,&gs,&mut sess,&h,|_|Some(0)));}
    h.state=0;h.gravity_mode=1;assert!(!can_equip(&h,1,0));h.gravity_mode=0;
    h.state=0x28;assert!(can_equip(&h,1,0));assert!(!can_equip(&h,1,1));
    h.state=0x3f;assert!(!can_equip(&h,1,0));
    h.state=0;h.mode=1;assert!(!input_layout_allowed(&h));h.mode=0;
    h.items.f13fc=1;assert!(input_layout_allowed(&h));assert!(!direct_control(&h));
    h.items.f13fc=0;h.gadgets.game_mode=3;assert!(!input_layout_allowed(&h));
    h.gadgets.game_mode=0;h.health=0;assert!(!input_layout_allowed(&h));
    gs.global.owned[15]=0;h.health=4;assert!(!equip(&state,button::UP,&gs,&mut sess,&h,|_|Some(0)));
}
#[test]
fn manual_modes_override_only_four_switches_and_leave_tuning_unchanged() {
    let mut registry=HashMap::new();
    for (id,key) in [(10,"cluster_enabled"),(16,"triple_flame_enabled"),(9,"crate_suction_enabled"),(19,"dual_beam_enabled"),(24,"sand_mouse_enabled"),(23,"shotgun_damage_hp")] {
        registry.insert((id,key.into()),73.);
    }
    let original=registry.clone();
    let state=State{modes:[1,1,1,2],..Default::default()};
    apply_registry(Settings{r1:false,dpad:false},&state,&mut registry,supported_mode_max);assert_eq!(registry,original);
    apply_registry(Settings{r1:true,dpad:false},&state,&mut registry,supported_mode_max);
    assert_eq!(registry[&(23,"shotgun_enabled".into())],1.);
    assert_eq!(registry[&(23,"continuous_missiles_enabled".into())],0.);
    for (k,v) in original{assert_eq!(registry[&k],v);}
    apply_registry(Settings{r1:true,dpad:false},&State::default(),&mut registry,supported_mode_max);
    assert_eq!(registry[&(17,"flame_mine_enabled".into())],0.);
}
#[test]
fn saved_modes_choose_grade_ammo_without_mutating_the_ini() {
    let raw="[general]\nenabled=true\nr1_experimental=on\ndpad_quick_select=on\n[weapon.ryno]\nid=23\n[weapon.ryno.level1]\nmax_ammo=50\nexperimental_max_ammo=7\nshotgun_enabled=true\n[weapon.pyrocitor]\nid=16\n[weapon.pyrocitor.level1]\nlava_stream_enabled=true\n";
    let raw=format!("{}\n{}",include_str!("weapon_evo_release.ini"),raw);
    let mut e=Evolution::parse(&raw).unwrap();assert!(e.control_settings.r1 && e.control_settings.dpad);
    assert!(!e.effective_pyro().lava.enabled);
    let track=e.extras.iter().find(|t|t.id==23).unwrap();assert_eq!(e.track_ammo_capacity(track),Some(50));
    let mut state=State::default();state.modes=[1,0,0,2];e.set_controls(state);
    assert!(e.effective_pyro().lava.enabled);
    let track=e.extras.iter().find(|t|t.id==23).unwrap();assert_eq!(e.track_ammo_capacity(track),Some(7));
    assert_eq!(track.params[0]["shotgun_enabled"],1.);
    e.control_settings.r1=false;assert!(e.effective_pyro().lava.enabled);
    assert_eq!(e.controls(),state);
}
#[test]
fn companion_v8_keeps_each_slots_choices_and_v7_loads_with_native_modes() {
    let mut a=Progress::default();a.controls=State{dpad:[15,4,1,8],modes:[1,1,0,2]};
    a.tracks[1].xp=17.;a.nanotech.xp=9.;a.gold_spent=4;
    let b=Progress::default();
    let bytes=a.encode(b"native A").unwrap();
    assert_eq!(Progress::decode(&bytes,b"native A").unwrap(),a);
    assert!(Progress::decode(&bytes,b"native B").is_err());
    assert_eq!(Progress::decode(&b.encode(b"native B").unwrap(),b"native B").unwrap(),b);
    let mut old=bytes[..bytes.len()-28-crate::weapon_evo::randomizer::ENCODED_SIZE].to_vec();old[5]=7;
    let sum=rc_formats::sha1::sha1(&old);old.extend_from_slice(&sum);
    let loaded=Progress::decode(&old,b"native A").unwrap();
    assert_eq!(loaded.controls,State::default());assert_eq!(loaded.tracks,a.tracks);
    assert_eq!(loaded.nanotech,a.nanotech);assert_eq!(loaded.gold_spent,4);
    let mut e=Evolution::default();e.restore_progress(&a);assert_eq!(e.controls(),a.controls);
    e.restore_progress(&b);assert_eq!(e.controls(),b.controls);
    let mut invalid=a;invalid.controls.modes[3]=3;assert!(invalid.encode(b"native A").is_err());
}

#[test]
fn a_radial_gameplay_lock_does_not_hide_r1_or_repeat_a_held_press() {
    use crate::pad::PadInput;
    let mut p=PadState::default();
    p.update(Some(&PadInput::default().bytes()),false);
    p.lock=2;
    p.update(Some(&PadInput{buttons:(button::R1|button::TRIANGLE) as u16,..Default::default()}.bytes()),false);
    assert_eq!(p.raw&button::R1,0);
    assert_ne!(physical_edges(&p,false)&button::R1,0);
    p.lock=2;
    p.update(Some(&PadInput{buttons:(button::R1|button::TRIANGLE) as u16,..Default::default()}.bytes()),false);
    assert_eq!(physical_edges(&p,false)&button::R1,0);
}

#[test]
fn gadget_shortcuts_request_native_hand_back_head_and_feet_slots() {
    let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});gs.global.level=1;
    let mut hero=Hero::new();hero.health=4;
    // Swingshot, Heli-Pack, Grindboots, Sonic Summoner.
    let state=State{dpad:[12,2,29,5],..Default::default()};
    for id in state.dpad {gs.global.owned[id as usize]=1;}
    let mut sess=SessionState::default();
    let slot=|id|match id {12=>Some(0),2=>Some(3),29=>Some(1),5=>Some(2),_=>None};
    for direction in DIRECTIONS {assert!(equip(&state,direction,&gs,&mut sess,&hero,slot));}
    assert_eq!((sess.temp_hand,sess.temp_back,sess.temp_feet,sess.temp_head),(12,2,29,5));
    gs.global.owned[5]=0;sess.temp_head=0;
    assert!(!equip(&state,button::LEFT,&gs,&mut sess,&hero,slot));assert_eq!(sess.temp_head,0);
}

#[test]
fn boots_and_helmets_toggle_using_pending_requests_as_well_as_saved_equipment() {
    let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});gs.global.level=1;
    let mut hero=Hero::new();hero.health=4;
    for (item,slot) in [(29,Slot::Feet),(5,Slot::Head)] {
        gs.global.owned[item as usize]=1;
        let state=State{dpad:[item as u8,0,0,0],..Default::default()};
        let mut sess=SessionState::default();
        for expected in [item,inventory::NOTHING,item,inventory::NOTHING] {
            assert!(equip(&state,button::UP,&gs,&mut sess,&hero,|_|Some(slot as i32)));
            assert_eq!(inventory::request(&sess,slot),expected);
        }
        // The native update consumed the request and committed the equipment.
        inventory::set_request(&mut sess,slot,0);gs.global.equipped[slot.index()]=item;
        assert!(equip(&state,button::UP,&gs,&mut sess,&hero,|_|Some(slot as i32)));
        assert_eq!(inventory::request(&sess,slot),inventory::NOTHING);
        gs.global.equipped[slot.index()]=0;
    }
}
#[test]
fn repeated_backpack_binding_selects_then_cycles_owned_packs_and_wraps() {
    let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});gs.global.level=1;
    let mut hero=Hero::new();hero.health=4;
    for id in CLANK_PACKS {gs.global.owned[id as usize]=1;}
    // Start with a different pack worn: the first press must honor the binding.
    gs.global.equipped[Slot::Back.index()]=4;
    let state=State{dpad:[2,0,0,0],..Default::default()};
    let mut sess=SessionState::default();
    for expected in [2,3,4,2,3,4,2] {
        assert!(equip(&state,button::UP,&gs,&mut sess,&hero,|_|Some(3)));
        assert_eq!(sess.temp_back,expected);
        gs.global.equipped[3]=expected;sess.temp_back=0;
    }
    gs.global.owned[3]=0;
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,|_|Some(3)));assert_eq!(sess.temp_back,4);
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,|_|Some(3)));assert_eq!(sess.temp_back,2);
    gs.global.owned[4]=0;
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,|_|Some(3)));assert_eq!(sess.temp_back,2);
}
#[test]
fn backpack_cycles_restart_after_external_changes_and_never_mutate_bindings() {
    let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});gs.global.level=1;
    let mut hero=Hero::new();hero.health=4;
    for id in [2,3,4,15] {gs.global.owned[id]=1;}
    let state=State{dpad:[2,15,0,0],..Default::default()};let original=state;
    let slots=|id|Some(if CLANK_PACKS.contains(&id){3}else{0});
    let mut sess=SessionState::default();
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,slots));assert_eq!(sess.temp_back,2);
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,slots));assert_eq!(sess.temp_back,3);
    sess.temp_back=4; // An ordinary inventory/menu request supersedes the cycle.
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,slots));assert_eq!(sess.temp_back,2);
    assert!(equip(&state,button::RIGHT,&gs,&mut sess,&hero,slots));
    assert!(equip(&state,button::UP,&gs,&mut sess,&hero,slots));assert_eq!(sess.temp_back,2);
    hero.state=0x32;let before=sess.temp_back;
    assert!(!equip(&state,button::UP,&gs,&mut sess,&hero,slots));assert_eq!(sess.temp_back,before);
    assert_eq!(state,original);
}

#[test]
fn aim_and_crouch_shoulders_keep_independent_edges_pressure_and_history() {
    let shoulders=button::L1|button::R1;
    for locked in [false,true] {
      for held in [0,button::L1,button::R1,shoulders] {
       for previous in [0,button::L1,button::R1,shoulders] {
        let mut physical=pad(held,previous);
        physical.pressure[8]=Pf::f(0.3);physical.pressure[9]=Pf::f(0.8);
        physical.analog_copy[12]=Pf::f(0.3);physical.analog_copy[13]=Pf::f(0.8);
        physical.hist_count=1;physical.hist_index=1;physical.hist_pressed[0]=physical.pressed;
        let p=gameplay_pad(&physical,15,true,false,locked);
        assert_eq!(p.held&shoulders,held);assert_eq!(p.prev_held&shoulders,previous);
        assert_eq!(p.pressed&shoulders,held&!previous);
        assert_eq!(p.released&shoulders,previous&!held);
        assert_eq!(p.raw_pressed&shoulders,held&!previous);
        assert_eq!(p.hist_pressed[0]&shoulders,physical.hist_pressed[0]&shoulders);
        assert_eq!(&p.pressure[8..10],&physical.pressure[8..10]);
        assert_eq!(&p.analog_copy[12..14],&physical.analog_copy[12..14]);
       }
      }
    }
    let mut physical=pad(button::L1|button::R1|button::R2,0);
    physical.pressure[8]=Pf::f(0.3);physical.pressure[9]=Pf::f(0.8);
    let p=gameplay_pad(&physical,15,true,false,true);
    assert_eq!(p.pressure[9],Pf::f(0.8));
    assert_ne!(p.held&button::CIRCLE,0);assert_eq!(p.held&button::R2,0);
}

#[test]
fn grade_ceilings_gate_modes_capacity_and_never_modify_saved_choices() {
    let mut e=Evolution::default();e.control_settings.r1=true;
    let mut saved=State::default();saved.modes=[1,1,1,2];e.set_controls(saved);
    for stats in &mut e.pyro.stats{stats.lava.enabled=true;stats.max_ammo=50;stats.experimental_max_ammo=Some(7);}
    e.r1_experimental_max[16]=[0,1,0,1,0,1,0,1];
    for grade in 1..=8 {
        e.pyro.level=grade;
        assert_eq!(e.effective_pyro().lava.enabled,grade%2==0);
        assert_eq!(e.effective_pyro().ammo_capacity(),if grade%2==0{7}else{50});
    }
    let track=e.extras.iter_mut().find(|t|t.id==23).unwrap();track.enabled=true;
    for params in &mut track.params{params.insert("max_ammo".into(),50.);params.insert("experimental_max_ammo".into(),7.);}
    e.r1_experimental_max[23]=[0,1,2,0,1,2,0,2];
    for grade in 1..=8 {
        e.extras.iter_mut().find(|t|t.id==23).unwrap().level=grade;
        let track=e.extras.iter().find(|t|t.id==23).unwrap();
        assert_eq!(e.track_ammo_capacity(track),Some(if e.r1_mode_max(23)==2{7}else{50}));
    }
    assert_eq!(e.controls(),saved);
    e.control_settings.r1=false;e.pyro.level=1;
    assert!(e.effective_pyro().lava.enabled);assert_eq!(e.effective_pyro().ammo_capacity(),7);
    assert_eq!(e.r1_mode_max(16),0);
}

#[test]
fn a_saved_mode_above_the_grade_ceiling_is_native_until_the_next_allowed_press() {
    let mut state=State{modes:[1,1,1,2],..Default::default()};let original=state;
    for item in MODE_ITEMS {assert_eq!(state.mode_limited(item,0),None);assert_eq!(state.cycle_limited(item,0),None);}
    assert_eq!(state,original);assert_eq!(state.mode_limited(23,1),Some(0));
    assert_eq!(state.cycle_limited(23,1),Some(1));assert_eq!(state.cycle_limited(23,1),Some(0));
    assert_eq!(state.cycle_limited(23,2),Some(1));assert_eq!(state.cycle_limited(23,2),Some(2));assert_eq!(state.cycle_limited(23,2),Some(0));
    let mut registry=HashMap::from([((17,"flame_mine_enabled".into()),1.),((23,"shotgun_enabled".into()),1.),((16,"triple_flame_enabled".into()),1.)]);
    apply_registry(Settings{r1:true,dpad:false},&original,&mut registry,|_|0);
    assert_eq!(registry[&(17,"flame_mine_enabled".into())],0.);assert_eq!(registry[&(23,"shotgun_enabled".into())],0.);
    assert_eq!(registry[&(16,"triple_flame_enabled".into())],1.);
    let before=registry.clone();apply_registry(Settings{r1:false,dpad:false},&original,&mut registry,supported_mode_max);assert_eq!(registry,before);
}

#[test]
fn every_weapon_grade_accepts_integer_limits_and_invalid_modes_are_rejected() {
    let base=include_str!("weapon_evo_release.ini");
    for (name,item) in [("wrench",8),("bomb_glove",10),("pyrocitor",16),("blaster",15),("glove_of_doom",20),("suck_cannon",9),("taunter",14),("mine_glove",17),("devastator",11),("visibomb",13),("walloper",18),("decoy_glove",25),("drone_device",24),("tesla_claw",19),("morph_ray",21),("ryno",23)] {
        let max=supported_mode_max(item);
        for grade in 1..=8 {
            let e=Evolution::parse(&format!("{base}\n[weapon.{name}.level{grade}]\nr1_experimental_max = {max}\n")).unwrap();
            assert_eq!(e.r1_experimental_max[item as usize][grade-1],max);
        }
        assert!(Evolution::parse(&format!("{base}\n[weapon.{name}.level1]\nr1_experimental_max = {}\n",max+1)).is_err());
    }
    for value in ["-1","1.5","true","NaN","256"] {
        assert!(Evolution::parse(&format!("{base}\n[weapon.ryno.level1]\nr1_experimental_max = {value}\n")).is_err());
    }
}
