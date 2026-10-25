//! Eight-grade gate, shared NG+ rewards and save migration contracts.
use super::*;
fn profile() -> Evolution {
    let mut e = Evolution::parse(include_str!("weapon_evo_release.ini")).unwrap();
    e.xp_required = [10000.; 7];
    e.pyro.xp_required = [10000.; 7];
    for t in &mut e.extras {
        t.xp_required = [10000.; 7];
    }
    e
}
fn set_level(e: &mut Evolution, id: i32, level: usize) {
    match id {
        10 => e.level = level,
        16 => e.pyro.level = level,
        _ => e.extras.iter_mut().find(|t| t.id == id).unwrap().level = level,
    }
}
fn award(e: &mut Evolution, id: i32, damage: f32) {
    match id {
        10 => e.award_damage(damage, 121),
        16 => e.award_pyro_damage(damage, 5, 176, true),
        _ => e.award_extra_damage(id, damage),
    }
}
#[test]
fn all_fifteen_weapons_stop_at_five_until_a_single_ng_plus_purchase_and_then_reach_eight() {
    for id in WEAPON_IDS.into_iter().filter(|&id| id != 8) {
        let mut e = profile();
        set_level(&mut e, id, 5);
        e.configure_ng_plus(false);
        assert!(!e.purchase_gold_upgrade(id));
        e.configure_ng_plus(true);
        award(&mut e, id, 10000.);
        assert_eq!(e.weapon_level(id), Some(5));
        assert!(e.purchase_gold_upgrade(id));
        assert_eq!(e.weapon_level(id), Some(6));
        assert!(!e.purchase_gold_upgrade(id));
        award(&mut e, id, 1_000_000.);
        assert_eq!(e.weapon_level(id), Some(8));
        assert!(e.weapon_at_cap(id));
        let p = e.progress();
        assert!(p.valid());
        let bytes = p.encode(b"native").unwrap();
        assert_eq!(save::Progress::decode(&bytes, b"native").unwrap(), p);
    }
}
#[test]
fn wrench_uses_xp_only_after_ng_plus_and_keeps_partial_fifth_grade_xp_across_save_restore() {
    let mut e = profile();
    set_level(&mut e, 8, 5);
    e.configure_ng_plus(false);
    award(&mut e, 8, 1_000_000.);
    assert_eq!(e.weapon_level(8), Some(5));
    e.configure_ng_plus(true);
    assert!(!e.purchase_gold_upgrade(8));
    award(&mut e, 8, 1.);
    let xp = e.extras.iter().find(|t| t.id == 8).unwrap().xp;
    assert!(xp > 0.);
    let p = e.progress();
    assert!(p.valid());
    let mut restored = profile();
    restored.configure_ng_plus(true);
    restored.restore_progress(&p);
    assert_eq!(restored.extras.iter().find(|t| t.id == 8).unwrap().xp, xp);
    award(&mut restored, 8, 1_000_000.);
    assert_eq!(restored.weapon_level(8), Some(8));
}
#[test]
fn the_same_ng_plus_factor_scales_weapon_and_nanotech_xp_once_without_changing_damage() {
    for id in [10, 16, 15, 19, 17, 24] {
        let mut normal = profile();
        normal.world_settings.enemy_damage_by_health = true;
        normal.world_settings.nanotech_xp_per_upgrade = 100000.;
        let mut boosted = normal.clone();
        boosted.configure_ng_plus(true);
        boosted.bolt_chain.kills = 30;
        let before = boosted.absolute_values();
        award(&mut normal, id, 1.);
        award(&mut boosted, id, 1.);
        let xp = |e: &Evolution| {
            e.progress().tracks[save::IDS.iter().position(|&i| i == id).unwrap()].xp
        };
        assert!(
            (xp(&boosted) - 3. * xp(&normal)).abs() < 0.001,
            "weapon {id}"
        );
        assert!(
            (boosted.nanotech.xp - 3. * normal.nanotech.xp).abs() < 0.001,
            "Nanotech {id}"
        );
        assert_eq!(boosted.absolute_values(), before);
    }
}
#[test]
fn version_six_saves_keep_levels_xp_and_nanotech_and_initialize_new_fields() {
    let mut old = save::Progress::default();
    old.tracks[0] = save::Track { level: 4, xp: 37. };
    old.nanotech = world::NanotechProgress { bonus: 9, xp: 17. };
    let mut bytes = old.encode(b"native").unwrap();
    // Discard the v7 reward fields and v8 control choices, plus checksum.
    bytes.truncate(bytes.len() - 36);
    bytes[5] = 6;
    let checksum = rc_formats::sha1::sha1(&bytes);
    bytes.extend_from_slice(&checksum);
    assert_eq!(save::Progress::decode(&bytes, b"native").unwrap(), old);
}
#[test]
fn reward_chain_and_gold_spending_survive_progress_sync_and_new_world_restore() {
    let mut e = profile();
    e.configure_ng_plus(true);
    e.bolt_chain = bolt_chain::State {
        kills: 12,
        remainder: 8,
    };
    e.gold_spent = 9;
    let mut state = save::Progress::default();
    e.sync_progress(&mut state);
    assert_eq!((state.bolt_chain.kills, state.gold_spent), (12, 9));
    let encoded = state.encode(b"save").unwrap();
    let loaded = save::Progress::decode(&encoded, b"save").unwrap();
    let mut next = profile();
    next.configure_ng_plus(true);
    next.restore_progress(&loaded);
    assert_eq!((next.bolt_chain, next.gold_spent), (e.bolt_chain, 9));
    next.bolt_chain.hurt();
    next.sync_progress(&mut state);
    assert_eq!(state.bolt_chain, bolt_chain::State::default());
}
#[test]
fn all_upper_grades_have_independent_damage_ammo_and_experimental_fields() {
    let mut raw = include_str!("weapon_evo_release.ini").to_string();
    for grade in 6..=8 {
        raw += &format!(
            "\n[weapon.bomb_glove.level{grade}]\ndamage_hp={}\nmax_ammo={}\n[weapon.pyrocitor.level{grade}]\ndamage_per_second={}\nmax_ammo={}\n[weapon.blaster.level{grade}]\ndamage_hp={}\nmax_ammo={}\ngold_texture=true\n",
            grade * 100,
            grade * 10,
            grade * 10,
            grade * 10,
            grade * 200,
            grade * 20
        );
    }
    let e = Evolution::parse(&raw).unwrap();
    for grade in 6..=8 {
        assert_eq!(e.damage_hp[10][grade - 1], Some((grade * 100) as f32));
        assert_eq!(e.stats[grade - 1].max_ammo, (grade * 10) as i32);
        assert_eq!(e.damage_hp[15][grade - 1], Some((grade * 200) as f32));
    }
}
#[test]
fn hide_ryno_removes_the_seller_without_any_grant_or_flag_change() {
    use crate::moby_runtime::{Moby, MobyTable};
    use crate::moby_update::{
        classes::{bomb_water::tests::Bench, units::rilgar_story},
        services::World,
    };
    let mut b = Bench::new(
        crate::hero::testkit::floor(100., 0, 100, 0, 100),
        &[],
        [40., 40., 10.],
    );
    let mut actor = Moby::zeroed();
    actor.o_class = 925;
    b.table = MobyTable::new(vec![Moby::zeroed(), actor], 0);
    b.svc.bomb_evo.hide_ryno = true;
    b.svc.interact.game.acquired = vec![0; 37];
    let mut w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, 0);
    rilgar_story::salesman_update(&mut w, 1);
    assert!(w.m(1).is_deleted());
    assert_eq!(w.svc.interact.game.acquired[23], 0);
    assert!(!w.svc.interact.writes.iter().any(|e| matches!(
        e,
        crate::moby_update::interact::GameWrite::GiveItem { item: 23, .. }
    )));
}
#[test]
fn real_health_loss_events_ignore_zero_damage_and_healing() {
    let mut h = crate::hero::Hero::new();
    h.health = 10;
    crate::hero::damage::take_damage(&mut h, 0);
    assert_eq!(h.damage.health_loss_events, 0);
    crate::hero::damage::take_damage(&mut h, -1);
    assert_eq!(h.damage.health_loss_events, 0);
    crate::hero::damage::take_damage(&mut h, 1);
    assert_eq!(h.damage.health_loss_events, 1);
}

#[test]
fn gold_bolt_summary_uses_the_actual_prices_and_keeps_native_counts_without_the_mod_marker() {
    use rc_formats::save_game::ChunkTables;
    let mut g = crate::game_state::GameState::zeroed(ChunkTables {
        global: vec![],
        level: vec![],
    });
    for b in g
        .levels
        .iter_mut()
        .flat_map(|l| l.gold_bolts.iter_mut())
        .take(40)
    {
        *b = 1;
    }
    g.global.gold_weapons[10] = 1;
    assert_eq!(crate::menus::pause::pages::gold_counts(&g), (40, 4, 36));
    g.weapon_progress.gold_shop_active = true;
    g.weapon_progress.gold_spent = 2;
    assert_eq!(crate::menus::pause::pages::gold_counts(&g), (40, 2, 38));
}
