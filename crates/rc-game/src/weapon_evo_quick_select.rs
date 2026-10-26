//! Native eight-slot quick select in world-reward modes.
//! The wrench is always available through its attack button, but has no valid
//! native quick-select icon. Keep its old placeholder out of the saved ring.
use crate::game_state::GameState;

pub fn world_rewards(gs: &GameState) -> bool {
    gs.weapon_progress.golden.active() || gs.weapon_progress.randomizer.active()
}

pub fn slots(gs: &GameState) -> [i32; 8] {
    let mut result = gs.global.quick_select;
    if world_rewards(gs) {
        for item in &mut result {
            if *item == 8 { *item = 0; }
        }
    }
    result
}

/// Repair old saves in place without compacting or moving player bindings.
pub fn repair(gs: &mut GameState) { gs.global.quick_select = slots(gs); }

pub fn can_bind(gs: &GameState, item: i32) -> bool {
    item != 8 || !world_rewards(gs)
}

/// Native acquisition policy: fill an empty slot, never overwrite a full ring.
pub fn insert_found(gs: &mut GameState, item: usize) {
    repair(gs);
    if !super::golden::ITEMS.contains(&item) || gs.global.quick_select.contains(&(item as i32)) { return; }
    if let Some(slot) = gs.global.quick_select.iter_mut().find(|slot| **slot == 0) {
        *slot = item as i32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> GameState {
        GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]})
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
