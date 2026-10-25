//! NG+ ownership, legacy Gold entitlements and composed encounter difficulty.
use super::*;

fn profile() -> Evolution {
    Evolution::parse(include_str!("weapon_evo_release.ini")).unwrap()
}
fn set_level(e: &mut Evolution, item: i32, level: usize) {
    match item {
        10 => e.level = level,
        16 => e.pyro.level = level,
        _ => e.extras.iter_mut().find(|t| t.id == item).unwrap().level = level,
    }
}

#[test]
fn challenge_reset_keeps_ownership_but_clears_history_and_displays_still_work() {
    use rc_formats::save_game::ChunkTables;
    let mut gs = crate::game_state::GameState::zeroed(ChunkTables {
        global: vec![],
        level: vec![],
    });
    let ids: Vec<usize> = WEAPON_IDS
        .iter()
        .filter(|&&id| id != 8)
        .map(|&i| i as usize)
        .collect();
    for &i in &ids {
        gs.global.owned[i] = 1;
        gs.global.acquired[i] = 1;
    }
    gs.challenge_reset(&ids, |g| {
        g.global.owned.fill(0);
        g.global.acquired.fill(0);
    });
    let mut e = profile();
    let mut shop = gold_shop::State::default();
    for &i in &ids {
        set_level(&mut e, i as i32, 5);
        shop.ready.insert(i);
    }
    for &i in &ids {
        assert_eq!((gs.global.owned[i], gs.global.acquired[i]), (1, 0));
        assert!(
            shop.item_visible(&e, gs.global.completes, &gs.global.owned, i),
            "item {i}"
        );
        assert!(!shop.item_visible(&e, 0, &gs.global.owned, i));
        gs.global.owned[i] = 0;
        gs.global.acquired[i] = 1;
        assert!(
            !shop.item_visible(&e, 1, &gs.global.owned, i),
            "history must not grant ownership"
        );
    }
}

#[test]
fn every_legacy_gold_weapon_becomes_six_once_without_xp_or_refills() {
    let mut e = profile();
    e.refill_ammo_on_upgrade = true;
    let mut gold = [0; 40];
    for id in WEAPON_IDS {
        gold[id as usize] = 1;
    }
    let before = e.nanotech;
    assert_eq!(e.migrate_legacy_gold(&gold), 15);
    for id in WEAPON_IDS {
        assert_eq!(e.weapon_level(id), Some(if id == 8 { 1 } else { 6 }));
    }
    assert_eq!(e.gold_spent, 40);
    assert_eq!(e.upgrade_refills, 0);
    assert_eq!(e.nanotech, before);
    assert_eq!(e.migrate_legacy_gold(&gold), 0);
    assert_eq!(e.gold_spent, 40);
    let saved = e.progress();
    assert!(saved.valid());
    let mut next = profile();
    next.restore_progress(&saved);
    assert_eq!(next.migrate_legacy_gold(&gold), 0);
    assert_eq!(next.progress(), saved);
}

#[test]
fn reconciliation_preserves_upper_grade_xp_and_randomization_never_spends_gold() {
    let mut e = profile();
    e.golden_settings.enabled = true;
    let t = e.extras.iter_mut().find(|t| t.id == 15).unwrap();
    t.level = 7;
    t.xp = 37.;
    let mut gold = [0; 40];
    gold[15] = 1;
    gold[19] = 1;
    assert_eq!(e.migrate_legacy_gold(&gold), 1);
    let t = e.extras.iter().find(|t| t.id == 15).unwrap();
    assert_eq!((t.level, t.xp), (7, 37.));
    assert_eq!(e.weapon_level(19), Some(6));
    assert_eq!(e.gold_spent, 0);
}

#[test]
fn reconciliation_survives_sync_and_respects_loading_another_save() {
    let mut e = profile();
    let mut state = save::Progress::default();
    let mut gold = [0; 40];
    gold[10] = 1;
    assert_eq!(e.sync_gold_progress(&mut state, &gold), 1);
    assert_eq!(e.weapon_level(10), Some(6));
    e.sync_progress(&mut state);
    assert_eq!(e.weapon_level(10), Some(6));
    assert_eq!(state.gold_spent, 2);
    assert_eq!(e.sync_gold_progress(&mut state, &gold), 0);
    state = save::Progress::default();
    gold.fill(0);
    assert_eq!(e.sync_gold_progress(&mut state, &gold), 0);
    assert_eq!(e.weapon_level(10), Some(1));
    assert_eq!(state.gold_spent, 0);
}

#[test]
fn ng_plus_scales_seven_coefficients_without_compounding_on_ticks_or_restores() {
    let mut e = profile();
    let s = &mut e.world_settings;
    s.ngplus_multiplier = 2.;
    s.enemy_health_multiplier = 1.5;
    s.enemy_damage_multiplier = 1.5;
    s.enemy_speed_multiplier = 1.5;
    s.enemy_perception_multiplier = 1.5;
    s.enemy_chase_range_multiplier = 1.5;
    s.enemy_projectile_speed_multiplier = 1.5;
    s.enemy_projectile_range_multiplier = 1.5;
    s.extra_enemies_per_level = 23;
    s.nanotech_xp_per_upgrade = 1234.;
    let original = *s;
    e.configure_ng_plus(false);
    assert_eq!(e.world_settings, original);
    for _ in 0..1000 {
        e.configure_ng_plus(true);
    }
    let s = e.world_settings;
    assert_eq!(
        [
            s.enemy_health_multiplier,
            s.enemy_damage_multiplier,
            s.enemy_speed_multiplier,
            s.enemy_perception_multiplier,
            s.enemy_chase_range_multiplier,
            s.enemy_projectile_speed_multiplier,
            s.enemy_projectile_range_multiplier
        ],
        [3.; 7]
    );
    assert_eq!(
        (s.extra_enemies_per_level, s.nanotech_xp_per_upgrade),
        (23, 1234.)
    );
    let save = e.progress();
    e.restore_progress(&save);
    assert_eq!(e.world_settings, s);
    e.configure_ng_plus(false);
    assert_eq!(e.world_settings, original);
    e.configure_ng_plus(true);
    assert_eq!(e.world_settings, s);
}

#[test]
fn ng_plus_option_defaults_neutral_and_rejects_invalid_values() {
    let mut s = world::Settings::default();
    assert_eq!(s.ngplus_multiplier, 1.);
    for raw in ["0", "-1", "NaN", "inf", "4.01"] {
        assert!(s.parse_field("ngplus_multiplier", raw).is_err());
    }
    for raw in ["0.1", "1", "1.5", "2", "4"] {
        assert!(s.parse_field("ngplus_multiplier", raw).unwrap());
    }
}

#[test]
fn composed_action_speeds_above_four_run_each_collision_substep() {
    use crate::{
        hero::Hero,
        moby_runtime::{Moby, MobyTable},
        moby_update::{services::World, ClassTable, Services},
        rng::Rng,
    };
    fn step(w: &mut World, id: usize) {
        w.mm(id).position[0] += 1.;
    }
    let mut actor = Moby::zeroed();
    actor.o_class = 577;
    actor.state = 1;
    let mut table = MobyTable::new(vec![actor], 0);
    let hero = Hero::new();
    let mut rng = Rng::new();
    let classes = ClassTable::default();
    let mut svc = Services::new();
    svc.bomb_evo.world_settings.enemy_speed_multiplier = 4.;
    svc.bomb_evo.world_settings.ngplus_multiplier = 2.;
    svc.bomb_evo.configure_ng_plus(true);
    let mut w = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 1);
    world::motion::run(&mut w, 0, step);
    assert_eq!(w.m(0).position[0], 8.);
    world::motion::run(&mut w, 0, step);
    assert_eq!(w.m(0).position[0], 16.);
}
