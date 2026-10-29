//! Opt-in local health test. A fresh Veldin II run never reads ordinary saves.
use super::{Evolution,location_test,save,world};
use crate::game_state::GameState;
pub const ENV: &str = "RC_NANOTECH_TEST";
pub const HP_ENV: &str = "RC_NANOTECH_TEST_HP";
pub const THRESHOLDS: [i32;15]=[30,31,60,61,90,91,120,121,150,151,180,181,209,210,8];
pub fn enabled()->bool {
    std::env::var(ENV).is_ok_and(|v|matches!(v.trim().to_ascii_lowercase().as_str(),"1"|"true"|"on"))
}
pub fn start_health()->i32 {
    std::env::var(HP_ENV).ok().and_then(|v|v.parse().ok()).unwrap_or(210).clamp(8,world::MAX_NANOTECH)
}
pub fn next_health(current:i32)->i32 {
    THRESHOLDS.iter().position(|&hp|hp==current).map_or(210,|i|THRESHOLDS[(i+1)%THRESHOLDS.len()])
}
pub fn configure(e:&mut Evolution) {
    if !enabled(){return;}
    e.enabled=true;e.golden_settings.enabled=false;e.randomizer_mode=false;
    e.ammo_unlock_mode=false;e.hide_ryno=false;
    e.world_settings.enemy_damage_by_health=true;
    e.world_settings.nanotech_max_health=world::MAX_XP_NANOTECH_BONUS;
}
pub fn prepare_new_game_at(gs:&mut GameState,hp:i32) {
    location_test::prepare_new_game(gs);
    gs.weapon_progress=save::Progress::default();
    gs.global.level=18;gs.global.bolts=5_000_000;
    gs.levels[18].missions.fill(0);gs.levels[18].killed.fill(0);
    for item in super::golden::ITEMS {gs.global.owned[item]=1;gs.global.acquired[item]=1;}
    for item in super::ammo_mode::AMMO_IDS {gs.global.ammo[item]=9999;}
    gs.global.quick_select=[8,10,15,16,19,23,24,13];
    let hp=hp.clamp(8,world::MAX_NANOTECH);
    gs.weapon_progress.nanotech.bonus=(hp-world::native_nanotech(&gs.global.flags)) as u32;
    gs.global.max_hp=hp;
}
pub fn prepare_new_game(gs:&mut GameState) {prepare_new_game_at(gs,start_health());}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_boot_has_live_encounters_all_equipment_and_exact_health() {
        for hp in [8,30,31,60,61,90,91,120,121,150,151,180,181,210] {
            let mut gs=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
            prepare_new_game_at(&mut gs,hp);
            assert_eq!((gs.global.level,gs.global.max_hp),(18,hp));
            assert_eq!(gs.levels[18].missions,[0;16]);
            assert_eq!(gs.weapon_progress.nanotech.bonus,hp as u32-8);
            assert!(gs.global.planet_unlocked[1..19].iter().all(|&b|b!=0));
            for item in super::super::golden::ITEMS {assert_eq!(gs.global.owned[item],1);}
            assert!(gs.weapon_progress.valid());
        }
    }
    #[test] fn boundary_cycle_covers_all_color_changes_and_wraps() {
        for pair in THRESHOLDS.windows(2) {assert_eq!(next_health(pair[0]),pair[1]);}
        assert_eq!(next_health(8),30);assert_eq!(next_health(52),210);
    }
}
