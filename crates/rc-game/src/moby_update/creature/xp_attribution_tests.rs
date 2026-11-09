//! Exercise native delivery before attribution, not pre-labelled Attack fixtures.
use super::*;
use crate::moby_update::{real_hp, services::HitTemplate};

fn xp(evo: &crate::weapon_evo::Evolution, item: i32) -> f32 {
    match item {
        10 => evo.xp,
        16 => evo.pyro.xp,
        _ => evo.extras.iter().find(|track| track.id == item).unwrap().xp,
    }
}

/// Native area_push copies collision flags to w20. The old drill marker
/// overlapped 0x800000 in those flags, even with every gadget feature disabled.
#[test]
fn native_area_blasts_credit_their_weapon_at_every_grade_and_gold_setting() {
    for (item, class, flags, kind, subtype) in [
        (10, 121, 0x83_0000, 2, 1),
        (17, 74, 0x81_0000, 4, 1),
        (13, 172, 0x83_0000, 3, 2),
        (21, 270, 0x83_0000, 2, 1),
    ] {
        for gold in [0, 1] {
            for grade in 1..=8 {
                let mut sim = real_hp_sim(749, 10.);
                let evo = &mut sim.svc.bomb_evo;
                evo.xp_required = [10000.; 7];
                evo.world_settings.enemy_damage_by_health = true;
                evo.world_settings.nanotech_xp_per_upgrade = 10000.;
                if item == 10 { evo.level = grade; }
                else { evo.extras.iter_mut().find(|t| t.id == item).unwrap().level = grade; }
                evo.configure_ng_plus(grade >= 6);
                evo.damage_hp[item as usize][grade - 1] = Some(200.);
                sim.hero.weapons.gold[item as usize] = gold;
                sim.svc.counters.gold_weapons.resize(37, 0);
                sim.svc.counters.gold_weapons[item as usize] = gold;
                let mut w = World::new(&mut sim.table, &sim.hero, &mut sim.rng,
                    &sim.classes, &mut sim.svc, 1);
                let source = w.create_moby(class).unwrap();
                // Repeated expanding-blast contacts must still be one HP event.
                for _ in 0..5 {
                    attack::area_push(&mut w, &[1], [120., 120., 12., 1.], source,
                        3., 1., 1., None, flags, kind, subtype);
                }
                w.delete_moby(source);
                let hit = w.get_hit(1, u32::MAX, false).expect("native blast was not queued");
                assert_eq!(hit.evolution.unwrap().weapon, item, "weapon {item}, grade {grade}");
                assert!((hit.damage.to_f32() - 2.).abs() < 0.0001,
                    "weapon {item}, grade {grade}: blast must use 200 HP, not drill DPS");
                let result = damage::resolve(&mut w, 1, Some(hit), 0x20, 0, 4);
                let before = pf(&w, 1, 0x20);
                set_pf(&mut w, 1, 0x20, before - result.damage);
                assert!((pf(&w, 1, 0x20) - 8.).abs() < 0.0001);
                // V5/V8 correctly stop weapon XP; health XP still receives accepted loss.
                let expected = if grade == 5 || grade == 8 { 0. } else { 20. };
                assert!((xp(&w.svc.bomb_evo, item) - expected).abs() < 0.0001,
                    "weapon {item}, grade {grade}: wrong XP recipient/amount");
                assert!((w.svc.bomb_evo.nanotech.xp - 20.).abs() < 0.0001);
                assert!(w.get_hit(1, u32::MAX, false).is_none());
            }
        }
    }
}

#[test]
fn drill_tag_requires_a_real_trespasser_hit_owned_by_ratchet() {
    let hero = Some(0);
    let valid = HitTemplate { attacker: hero, h1a: 188, flags: 0x1_0000,
        b18: 0, b19: 1, w20: 1 | crate::gadget_combat::DRILL_HIT,
        damage: Pf::ONE, ..Default::default() };
    assert_eq!(real_hp::weapon(&valid, hero), Some(26));
    assert_eq!(real_hp::weapon(&valid, None), None);
    assert_eq!(real_hp::weapon(&HitTemplate { attacker: Some(2), ..valid }, hero), None);
    assert_eq!(real_hp::weapon(&HitTemplate { w20: 1, ..valid }, hero), None);
    assert_eq!(real_hp::weapon(&HitTemplate { h1a: 121, ..valid }, hero), Some(10));
    assert_eq!(real_hp::weapon(&HitTemplate { h1a: 74, ..valid }, hero), Some(17));
    assert_eq!(real_hp::weapon(&HitTemplate { h1a: 172, ..valid }, hero), Some(13));
}

#[test]
fn every_native_weapon_template_keeps_its_recipient_with_explosion_or_private_bits() {
    // item, native class, owner, type, subtype, native w20. These include the
    // actual area_push form (w20 = flags), direct bullets and continuous hits.
    for (item, class, owner, kind, subtype, native) in [
        (8, 0x47, 0, 0, 1, 1), (9, 0x351, 2, 1, 1, 1),
        (10, 121, 2, 2, 1, 0x83_0000), (11, 153, 2, 3, 1, 1),
        (13, 172, 2, 3, 2, 0x83_0000), (14, 175, 0, 5, 1, 1),
        (15, 305, 2, 1, 1, 1), (15, 168, 0, 1, 1, 1),
        (16, 176, 0, 5, 1, 1), (16, 0, 0, 5, 1, 1),
        (17, 74, 2, 4, 1, 0x81_0000), (17, 74, 2, 5, 1, 1),
        (18, 180, 0, 0, 1, 1), (19, 177, 0, 5, 3, 1),
        (20, 186, 2, 2, 3, 0x1_0000), (21, 270, 2, 2, 1, 0x83_0000),
        (23, 457, 2, 3, 3, 1), (24, 479, 2, 1, 2, 1),
        (25, 203, 2, 2, 1, 0), (25, 1900, 2, 2, 1, 0),
    ] {
        for extra in [0, 0x80_0000, crate::gadget_combat::DRILL_HIT] {
            let template = HitTemplate { attacker: Some(owner), h1a: class,
                b18: kind, b19: subtype, w20: native | extra, ..Default::default() };
            assert_eq!(real_hp::weapon(&template, Some(0)), Some(item),
                "weapon {item}, native class {class}, w20 {:#x}", template.w20);
        }
    }
    for class in [388, 1093, 1126] {
        let environment = HitTemplate { attacker: Some(2), h1a: class,
            flags: 0x83_0001, w20: 0x83_0001, ..Default::default() };
        assert_eq!(real_hp::weapon(&environment, Some(0)), None,
            "scenery or an enemy explosion must not become Ratchet's drill/Nanotech XP");
    }
}
