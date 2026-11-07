//! Two saved eight-slot rings; the first retains the native save layout.
//! The wrench is always available through its attack button, but has no valid
//! native quick-select icon. Keep its old placeholder out of the saved ring.
use crate::game_state::GameState;

/// The native save owns ring one. Only ring two and the active page are added
/// to the content-addressed companion, keeping old native saves compatible.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub second: [u8; 8],
    pub ring: u8,
}
impl State {
    pub const ENCODED_SIZE: usize = 9;
    pub fn valid(&self) -> bool { self.ring < 2 && self.second.iter().all(|&id| id <= 36) }
    pub fn cycle(&mut self) { self.ring ^= 1; }
}

pub fn ring_slots(gs: &GameState, ring: u8) -> [i32; 8] {
    let mut result = if ring == 1 { gs.weapon_progress.quick_select.second.map(i32::from) }
        else { gs.global.quick_select };
    if world_rewards(gs) {
        for item in &mut result { if *item == 8 { *item = 0; } }
    }
    result
}

/// Write a displayed editor page without swapping or overwriting the other ring.
pub fn set_slots(gs: &mut GameState, slots: [i32; 8]) {
    if gs.weapon_progress.quick_select.ring == 1 {
        gs.weapon_progress.quick_select.second = slots.map(|id| if (0..=36).contains(&id) { id as u8 } else { 0 });
    } else { gs.global.quick_select = slots; }
}

/// One assignment across both rings, matching the native no-duplicate policy.
pub fn remove_other_binding(gs: &mut GameState, item: i32) {
    let other = gs.weapon_progress.quick_select.ring ^ 1;
    if other == 0 { for slot in &mut gs.global.quick_select { if *slot == item { *slot = 0; } } }
    else { for slot in &mut gs.weapon_progress.quick_select.second { if i32::from(*slot) == item { *slot = 0; } } }
}

/// Ratchet's menu owns the optional freeze. Clank command pages and all native
/// movie/vehicle/frontend modes keep their existing clocks and control rules.
pub fn pause_world(enabled: bool, gameplay: bool, open: bool, command_page: bool, alive: bool) -> bool {
    enabled && gameplay && open && !command_page && alive
}

/// Only an R2 press while the Ratchet ring is held may change the saved page.
pub fn cycle_on_press(state: &mut State, open: bool, command_page: bool, held: u32, pressed: u32) -> bool {
    if !open || command_page || held & crate::pad::button::TRIANGLE == 0
        || pressed & crate::pad::button::R2 == 0 { return false; }
    state.cycle();
    true
}

pub fn world_rewards(gs: &GameState) -> bool {
    gs.weapon_progress.golden.active() || gs.weapon_progress.randomizer.active()
}

pub fn slots(gs: &GameState) -> [i32; 8] {
    let mut slots=ring_slots(gs, gs.weapon_progress.quick_select.ring);
    // A remembered binding must not bypass Survival ownership or a profile
    // change. Keep its storage intact so a later unlock can reveal it again.
    for slot in &mut slots {
        if *slot != 0 && !crate::inventory::owns(gs,*slot) { *slot=0; }
    }
    slots
}

/// Repair old saves in place without compacting or moving player bindings.
pub fn repair(gs: &mut GameState) {
    gs.global.quick_select = ring_slots(gs, 0);
    gs.weapon_progress.quick_select.second = ring_slots(gs, 1).map(|id| id as u8);
}

pub fn can_bind(gs: &GameState, item: i32) -> bool {
    item != 8 || !world_rewards(gs)
}

/// Native acquisition policy: fill an empty slot, never overwrite a full ring.
pub fn insert_found(gs: &mut GameState, item: usize) {
    repair(gs);
    if !super::golden::ITEMS.contains(&item) || gs.global.quick_select.contains(&(item as i32)) || gs.weapon_progress.quick_select.second.contains(&(item as u8)) { return; }
    if let Some(slot) = gs.global.quick_select.iter_mut().find(|slot| **slot == 0) {
        *slot = item as i32;
    } else if let Some(slot) = gs.weapon_progress.quick_select.second.iter_mut().find(|slot| **slot == 0) {
        *slot = item as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> GameState {
        GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]})
    }

    #[test]
    fn r2_edges_cycle_once_and_ignore_clank_closed_and_release_frames() {
        use crate::pad::{PadInput,PadState,button};
        let mut state=State::default(); let mut pad=PadState::default();
        for (buttons, expected) in [(button::TRIANGLE,0), (button::TRIANGLE|button::R2,1),
            (button::TRIANGLE|button::R2,1), (button::TRIANGLE,1),
            (button::TRIANGLE|button::R2,0), (0,0)] {
            pad.update(Some(&PadInput::neutral().press(buttons).bytes()),false);
            cycle_on_press(&mut state,true,false,pad.held_unmirrored,pad.pressed_unmirrored);
            assert_eq!(state.ring,expected);
        }
        assert!(!cycle_on_press(&mut state,false,false,button::TRIANGLE,button::R2));
        assert!(!cycle_on_press(&mut state,true,true,button::TRIANGLE,button::R2));
        assert!(!cycle_on_press(&mut state,true,false,0,button::R2));
    }

    #[test]
    fn rings_are_independent_and_new_rewards_fill_second_after_native_ring() {
        let mut gs=game(); gs.global.owned=[1;37]; gs.global.quick_select=[10,11,13,15,16,17,19,20];
        insert_found(&mut gs,23);
        assert_eq!(gs.weapon_progress.quick_select.second,[23,0,0,0,0,0,0,0]);
        let first=gs.global.quick_select;
        gs.weapon_progress.quick_select.cycle();
        assert_eq!(slots(&gs),[23,0,0,0,0,0,0,0]);
        set_slots(&mut gs,[24,25,14,0,0,0,0,0]);
        assert_eq!(gs.global.quick_select,first);
        remove_other_binding(&mut gs,15);
        assert_eq!(gs.global.quick_select[3],0);
        gs.weapon_progress.quick_select.cycle();
        assert_eq!(slots(&gs)[0],10);
        assert_eq!(gs.weapon_progress.quick_select.second,[24,25,14,0,0,0,0,0]);
    }

    #[test]
    fn saved_second_ring_never_equips_a_weapon_locked_by_survival() {
        let mut gs=game(); gs.weapon_progress.quick_select.ring=1;
        gs.weapon_progress.quick_select.second[0]=23;
        assert_eq!(slots(&gs),[0;8]);
        assert_eq!(gs.weapon_progress.quick_select.second[0],23);
        gs.global.owned[23]=1; assert_eq!(slots(&gs)[0],23);
        gs.global.owned[23]=0; assert_eq!(slots(&gs)[0],0);
    }

    #[test]
    fn freeze_is_optional_and_scoped_to_alive_ratchet_gameplay_radial() {
        assert!(pause_world(true,true,true,false,true));
        for args in [(false,true,true,false,true),(true,false,true,false,true),
            (true,true,false,false,true),(true,true,true,true,true),(true,true,true,false,false)] {
            assert!(!pause_world(args.0,args.1,args.2,args.3,args.4));
        }
    }

    #[test]
    fn ordinary_slots_are_not_rewritten() {
        let mut gs = game(); gs.global.quick_select = [8,15,0,11,0,0,0,0];
        repair(&mut gs);
        assert_eq!(gs.global.quick_select,[8,15,0,11,0,0,0,0]);
        assert!(can_bind(&gs,8));
    }

    #[test]
    fn legacy_placeholder_is_removed_without_moving_other_bindings() {
        let mut gs = game();
        gs.weapon_progress.golden = super::super::golden::State::generate(8801,0);
        gs.global.quick_select = [8,15,0,11,0,0,0,0];
        repair(&mut gs);
        assert_eq!(gs.global.quick_select,[0,15,0,11,0,0,0,0]);
        assert!(!can_bind(&gs,8));
        insert_found(&mut gs,16);
        assert_eq!(gs.global.quick_select,[16,15,0,11,0,0,0,0]);
    }

    #[test]
    fn mixed_mode_repairs_the_same_legacy_slot_and_full_rings_are_preserved() {
        let mut gs = game();
        gs.weapon_progress.randomizer = super::super::randomizer::State::generate(8802);
        gs.global.quick_select = [8,15,11,13,16,17,19,20];
        insert_found(&mut gs,23);
        assert_eq!(gs.global.quick_select,[23,15,11,13,16,17,19,20]);
        let full = gs.global.quick_select;
        insert_found(&mut gs,24);
        assert_eq!(gs.global.quick_select,full);
        insert_found(&mut gs,15);
        assert_eq!(gs.global.quick_select,full);
    }

    #[test]
    fn both_new_game_modes_start_with_eight_free_slots_and_keep_the_wrench_owned() {
        let mut gs=game();
        super::super::golden::initialize_new_game(&mut gs,
            super::super::golden::Settings{enabled:true,..Default::default()},8803);
        assert_eq!(gs.global.quick_select,[0;8]);assert_eq!(gs.global.owned[8],1);
        let mut gs=game();super::super::randomizer::initialize(&mut gs,true,8804);
        assert_eq!(gs.global.quick_select,[0;8]);assert_eq!(gs.global.owned[8],1);
    }
}
