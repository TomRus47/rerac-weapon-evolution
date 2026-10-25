//! Public engine-facing controls API: physical edges, item lookup and saved modes.
use rc_game::{game_state::{GameState,SessionState},hero::{Hero,items::ItemData},pad::{button,PadInput,PadState},weapon_evo::{Evolution,controls,save::Progress}};
use rc_formats::save_game::ChunkTables;

#[test]
fn paused_device_snapshot_and_live_equipment_request_use_the_public_api() {
    let mut pad=PadState::default();pad.lock=2;
    pad.update(Some(&PadInput{buttons:button::UP as u16,..Default::default()}.bytes()),false);
    let raw=controls::physical_edges(&pad,false);
    assert_eq!(raw&controls::DIRECTION_MASK,button::UP);
    let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});gs.global.owned[15]=1;
    let mut sess=SessionState::default();let mut h=Hero::new();h.health=4;
    let mut data=ItemData::default();data.defs.resize(37,Default::default());
    let state=controls::State{dpad:[15,0,0,0],..Default::default()};
    let item_data=Some(data);let defs=item_data.as_ref();
    assert!(controls::equip(&state,raw,&gs,&mut sess,&h,|id|defs.and_then(|d|d.defs.get(id as usize)).map(|d|d.slot)));
    assert_eq!(sess.temp_hand,15);assert_eq!(gs.global.owned[23],0);
}

#[test]
fn an_engine_mode_change_is_exported_with_xp_and_loads_into_another_runtime() {
    let mut evo=Evolution::default();let mut persistent=Progress::default();
    assert_eq!(persistent.controls.cycle(23),Some(1));evo.set_controls(persistent.controls);
    evo.sync_progress(&mut persistent);
    let bytes=persistent.encode(b"native card slot").unwrap();
    let loaded=Progress::decode(&bytes,b"native card slot").unwrap();
    let mut next=Evolution::default();next.restore_progress(&loaded);
    assert_eq!(next.controls().mode(23),Some(1));assert_eq!(next.controls().mode(19),None);
}
