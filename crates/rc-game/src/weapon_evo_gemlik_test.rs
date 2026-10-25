//! Explicit local Gold-shop test. Normal boots and ordinary save slots are untouched.
use super::{Evolution, golden, location_test, save};
use crate::game_state::GameState;
use std::path::PathBuf;

pub const ENV: &str = "RC_GEMLIK_GOLD_TEST";
pub fn enabled() -> bool {
    std::env::var(ENV).is_ok_and(|v| matches!(v.trim().to_ascii_lowercase().as_str(),"1"|"true"|"on"))
}
pub fn configure(e: &mut Evolution) {
    if enabled() {
        e.golden_settings.enabled=false;
        e.ammo_unlock_mode=false;
        e.hide_ryno=false;
    }
}
pub fn card_root(root: PathBuf) -> PathBuf { root.join("gemlik_ngplus_test") }

/// A fresh direct boot, never an edit of a loaded normal save. Mission state
/// unlocks the upper tower and its return teleporter; purchases start unowned.
pub fn prepare_new_game(gs: &mut GameState) {
    location_test::prepare_new_game(gs);
    gs.global.level=13;gs.global.completes=1;gs.global.game_beaten=1;
    gs.global.bolts=5_000_000;gs.global.gold_weapons.fill(0);
    gs.weapon_progress=save::Progress::default();
    for track in &mut gs.weapon_progress.tracks {track.level=5;track.xp=0.;}
    for &item in &golden::ITEMS {
        gs.global.owned[item]=1;gs.global.acquired[item]=1;
    }
    for item in super::ammo_mode::AMMO_IDS {gs.global.ammo[item]=9999;}
    // The ordinary inventory synchronization clamps these to the active V5 cap.
    gs.global.ammo[9]=0;
    gs.global.quick_select=[8,10,15,16,19,23,24,13];
    for (level,&count) in gs.levels.iter_mut().zip(golden::COUNTS.iter()) {
        level.gold_bolts.fill(0);
        for bit in level.gold_bolts.iter_mut().take(count as usize) {*bit=1;}
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn fresh_shop_state_has_exact_funds_and_all_weapons_at_five_without_gold_entitlements() {
        let mut g=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
        prepare_new_game(&mut g);
        assert_eq!((g.global.level,g.global.completes,g.global.bolts),(13,1,5_000_000));
        assert_eq!(g.levels.iter().flat_map(|l|l.gold_bolts).filter(|&b|b!=0).count(),40);
        assert!(g.global.gold_weapons.iter().all(|&b|b==0));
        assert!(g.weapon_progress.tracks.iter().all(|t|t.level==5&&t.xp==0.));
        assert_eq!(g.weapon_progress.gold_spent,0);
        for item in golden::ITEMS {assert_eq!(g.global.owned[item],1);}
        assert_eq!(g.levels[13].missions,[0xff;16]);
        assert!(g.global.planet_unlocked[1..19].iter().all(|&b|b!=0));
    }
    #[test] fn test_card_is_a_different_directory_and_state_reset_is_repeatable() {
        assert_eq!(card_root(PathBuf::from("normal")),PathBuf::from("normal/gemlik_ngplus_test"));
        let mut g=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
        prepare_new_game(&mut g);let original=g.weapon_progress;
        g.global.bolts=0;g.global.gold_weapons[24]=1;g.weapon_progress.gold_spent=3;
        prepare_new_game(&mut g);assert_eq!(g.weapon_progress,original);
        assert_eq!(g.global.bolts,5_000_000);assert_eq!(g.global.gold_weapons[24],0);
    }
}
