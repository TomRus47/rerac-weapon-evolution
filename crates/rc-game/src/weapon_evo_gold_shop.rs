//! Deterministic NG+ gold displays. Layout anchors come from the authored offers;
//! model loading and rendering are engine-owned, purchase state is gameplay-owned.
use crate::moby_runtime::{mode, MobyId};
use crate::moby_update::{
    interact::{owner, GameWrite},
    services::{pvar as p, World},
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Price {
    pub item: usize,
    pub bolts: i32,
    pub gold: u32,
}
pub const GEMLIK: [Price; 5] = [
    Price {
        item: 24,
        bolts: 20_000,
        gold: 3,
    },
    Price {
        item: 13,
        bolts: 60_000,
        gold: 4,
    },
    Price {
        item: 14,
        bolts: 10_000,
        gold: 1,
    },
    Price {
        item: 18,
        bolts: 20_000,
        gold: 2,
    },
    Price {
        item: 23,
        bolts: 300_000,
        gold: 5,
    },
];
pub fn novalis_gold(item: usize) -> u32 {
    if item == 19 {
        3
    } else if matches!(item, 11 | 15) {
        4
    } else {
        2
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Display {
    pub key: i32,
    pub item: usize,
    pub position: [f32; 3],
    pub yaw: f32,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub displays: Vec<Display>,
    pub names: HashMap<usize, Vec<u8>>,
    pub ready: HashSet<usize>,
    initialized: bool,
}
impl State {
    pub fn prepare(&mut self, w: &World) {
        if self.initialized {
            return;
        }
        self.initialized = true;
        if !matches!(w.svc.level, 1 | 13) {
            return;
        }
        let mut offers: Vec<_> = w
            .table
            .mobys
            .iter()
            .filter(|m| m.o_class == 304 && m.pvars.len() >= 0x44)
            .map(|m| (p::i32(&m.pvars, 0x40), m.position, m.rotation[2]))
            .filter(|(k, _, _)| (0..10).contains(k))
            .collect();
        offers.sort_by_key(|(k, _, _)| *k);
        offers.dedup_by_key(|(k, _, _)| *k);
        // Gemlik authors all ten NG+ slots, but the original first-playthrough
        // shop occupies the five odd keys across the entire room. Taking the
        // first five sorted offers would pack our replacement into half of it.
        // Keep authored positions, rotations and interaction anchors together.
        if w.svc.level == 13 && offers.len() == 10 {
            offers.retain(|(key, _, _)| key & 1 != 0);
        }
        for (index, (key, pos, yaw)) in offers.into_iter().enumerate() {
            let item = if w.svc.level == 13 {
                let Some(price) = GEMLIK.get(index) else {
                    continue;
                };
                price.item
            } else {
                crate::moby_update::classes::item_offer::GOLD_ITEMS[key as usize] as usize
            };
            // Authored gold-model props supply the display height when available.
            let prop = w.table.mobys.iter().find(|m| {
                (1456..=1465).contains(&m.o_class)
                    && m.pvars.len() >= 0x44
                    && p::i32(&m.pvars, 0x40) == key
            });
            let at = prop.map_or([pos[0], pos[1], pos[2] + 1.0], |m| {
                [m.position[0], m.position[1], m.position[2]]
            });
            self.displays.push(Display {
                key,
                item,
                position: at,
                yaw,
            });
        }
    }
    /// Shared by gameplay and both renderer visibility passes. Challenge mode
    /// retains owned items but resets the separate acquired-history array.
    pub fn item_visible(
        &self,
        evo: &crate::weapon_evo::Evolution,
        completed: i32,
        owned: &[u8],
        item: usize,
    ) -> bool {
        completed > 0
            && evo.weapon_level(item as i32) == Some(5)
            && owned.get(item).is_some_and(|&n| n != 0)
            && self.ready.contains(&item)
    }
    pub fn visible(&self, w: &World, item: usize) -> bool {
        self.item_visible(
            &w.svc.bomb_evo,
            w.svc.counters.times_completed,
            &w.hero.owned.0,
            item,
        )
    }
}
fn price(w: &World, item: usize) -> Price {
    if w.svc.level == 13 {
        *GEMLIK.iter().find(|p| p.item == item).unwrap()
    } else {
        Price {
            item,
            bolts: w.svc.interact.tables.shop.gold_price(item) as i32,
            gold: novalis_gold(item),
        }
    }
}
fn phrase(lang: u32, part: usize) -> &'static [u8] {
    // Accented fragments use the game's single-byte font codes only through native
    // weapon names; surrounding UI fragments keep the portable base glyph set.
    use rc_formats::strings::lang;
    match (lang, part) {
        (lang::ITALIAN, 0) => b"Compra ",
        (lang::ITALIAN, 1) => b" bolt d'oro",
        (lang::FRENCH, 0) => b"Acheter ",
        (lang::FRENCH, 1) => b" boulons en or",
        (lang::GERMAN, 0) => b"Kaufen: ",
        (lang::GERMAN, 1) => b" Gold-Bolts",
        (lang::SPANISH, 0) => b"Comprar ",
        (lang::SPANISH, 1) => b" guitones de oro",
        (_, 0) => b"Buy ",
        _ => b" Gold Bolts",
    }
}
pub fn prompt(lang: u32, name: &[u8], price: Price, randomized: bool, affordable: bool) -> Vec<u8> {
    let mut out = Vec::new();
    if affordable {
        out.push(0x12);
    } // replaced with native glyph below
    out.extend_from_slice(phrase(lang, 0));
    out.extend_from_slice(name);
    out.extend_from_slice(b" V6\x01");
    out.extend_from_slice(price.bolts.to_string().as_bytes());
    out.extend_from_slice(match lang {
        rc_formats::strings::lang::FRENCH => b" boulons".as_slice(),
        rc_formats::strings::lang::SPANISH => b" guitones".as_slice(),
        _ => b" bolts".as_slice(),
    });
    if !randomized {
        out.extend_from_slice(b" + ");
        out.extend_from_slice(price.gold.to_string().as_bytes());
        out.extend_from_slice(phrase(lang, 1));
    }
    out
}
pub fn update(w: &mut World, id: MobyId) {
    let mut state = std::mem::take(&mut w.svc.gold_shop);
    state.prepare(w);
    w.svc.gold_shop = state;
    let m = w.m(id);
    let key = if m.pvars.len() >= 0x44 {
        p::i32(&m.pvars, 0x40)
    } else {
        -1
    };
    if m.o_class != 304 {
        w.mm(id).mode |= mode::HIDDEN;
        w.mm(id).visible = 0;
        return;
    }
    if w.svc.counters.times_completed <= 0 {
        w.delete_moby(id);
        return;
    }
    let Some(display) = w
        .svc
        .gold_shop
        .displays
        .iter()
        .find(|d| d.key == key)
        .copied()
    else {
        w.delete_moby(id);
        return;
    };
    if !w.svc.gold_shop.visible(w, display.item) {
        return;
    }
    let pos = w.m(id).position;
    let hero = w.hero.pos.map(|p| p.to_f32());
    if (pos[0] - hero[0]).hypot(pos[1] - hero[1]) > 2.6
        || (pos[2] - hero[2]).abs() > 3.0
        || w.svc.game_mode != 0
    {
        return;
    }
    let price = price(w, display.item);
    let random = w.svc.bomb_evo.golden_settings.enabled;
    let available =
        (w.svc.interact.game.gold_bolts.max(0) as u32).saturating_sub(w.svc.bomb_evo.gold_spent);
    let affordable = w.svc.counters.bolts >= price.bolts && (random || available >= price.gold);
    let Some(name) = w.svc.gold_shop.names.get(&display.item) else {
        return;
    };
    let text = prompt(w.svc.interact.lang, name, price, random, affordable);
    // Hold the literal text with the usual context-prompt lease, no NPC talk node
    // can grant an item or charge a stale native price behind this path.
    let held = w.svc.interact.prompt.try_set(owner::VENDOR, 0x7fef, &text);
    if !affordable || held == 0 || !w.svc.interact.triangle() {
        return;
    }
    if !w.svc.bomb_evo.purchase_gold_upgrade(display.item as i32) {
        return;
    }
    w.svc.counters.bolts -= price.bolts;
    if !random {
        w.svc.bomb_evo.gold_spent = w.svc.bomb_evo.gold_spent.saturating_add(price.gold);
    }
    if let Some(v) = w.svc.counters.gold_weapons.get_mut(display.item) {
        *v = 1;
    }
    if let Some(v) = w.svc.interact.game.gold_weapons.get_mut(display.item) {
        *v = 1;
    }
    w.svc
        .interact
        .writes
        .push(GameWrite::GoldWeapon(display.item));
    w.svc.interact.prompt.release(owner::VENDOR, true);
    crate::cinematic::save(w);
    // Reuse the native optional purchase demo. Ownership/price are committed
    // once above; an absent scene must never cancel a successful V6 upgrade.
    w.svc.interact.handoffs.push(crate::moby_update::interact::Handoff::GoldUpgrade {
        offer: id, item: display.item as i32,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_runtime::{Moby, MobyTable};
    use crate::moby_update::classes::bomb_water::tests::Bench;
    use crate::moby_update::services::World;
    fn bench() -> Bench {
        let mut b = Bench::new(
            crate::hero::testkit::floor(100., 0, 100, 0, 100),
            &[],
            [40., 40., 10.],
        );
        let mut offer = Moby::zeroed();
        offer.o_class = 304;
        offer.position = [40., 40., 10., 1.];
        offer.pvars = vec![0; 0x44];
        b.table = MobyTable::new(vec![Moby::zeroed(), offer], 4);
        b.svc.level = 1;
        b.svc.counters.times_completed = 1;
        b.svc.bomb_evo =
            crate::weapon_evo::Evolution::parse(include_str!("weapon_evo_release.ini")).unwrap();
        b.svc.bomb_evo.configure_ng_plus(true);
        b.svc
            .bomb_evo
            .extras
            .iter_mut()
            .find(|t| t.id == 19)
            .unwrap()
            .level = 5;
        b.hero.owned.0 = [1; 37];
        b.svc.interact.game.acquired = vec![0; 37]; // NG+ acquisition history resets.
        b.svc.interact.game.gold_weapons = vec![0; 37];
        b.svc.interact.game.gold_bolts = 3;
        b.svc.interact.pressed = crate::pad::button::TRIANGLE;
        std::sync::Arc::make_mut(&mut b.svc.interact.tables)
            .shop
            .records = vec![[0; 0x18]; 43];
        std::sync::Arc::make_mut(&mut b.svc.interact.tables)
            .shop
            .records[19][0x14..0x16]
            .copy_from_slice(&60_000u16.to_le_bytes());
        b.svc.gold_shop.ready.insert(19);
        b.svc.gold_shop.names.insert(19, b"Tesla Claw".to_vec());
        b
    }
    #[test]
    fn prices_total_forty_and_gemlik_order_is_exact() {
        let native = crate::moby_update::classes::item_offer::GOLD_ITEMS;
        assert_eq!(
            native
                .iter()
                .map(|&i| novalis_gold(i as usize))
                .sum::<u32>()
                + GEMLIK.iter().map(|p| p.gold).sum::<u32>(),
            40
        );
        assert_eq!(
            GEMLIK.map(|p| (p.item, p.bolts, p.gold)),
            [
                (24, 20_000, 3),
                (13, 60_000, 4),
                (14, 10_000, 1),
                (18, 20_000, 2),
                (23, 300_000, 5)
            ]
        );
    }
    #[test]
    fn a_purchase_is_atomic_requires_level_five_and_cannot_repeat() {
        let mut b = bench();
        b.svc.counters.bolts = 59_999;
        let mut w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, 0);
        update(&mut w, 1);
        assert_eq!(w.svc.bomb_evo.weapon_level(19), Some(5));
        assert_eq!(w.svc.bomb_evo.gold_spent, 0);
        w.svc.counters.bolts = 60_000;
        w.svc.interact.game.gold_bolts = 2;
        update(&mut w, 1);
        assert_eq!(w.svc.counters.bolts, 60_000);
        w.svc.interact.game.gold_bolts = 3;
        update(&mut w, 1);
        assert_eq!((w.svc.counters.bolts, w.svc.bomb_evo.gold_spent), (0, 3));
        assert_eq!(w.svc.bomb_evo.weapon_level(19), Some(6));
        assert_eq!(w.svc.interact.handoffs.iter().filter(|h|matches!(h,
            crate::moby_update::interact::Handoff::GoldUpgrade{offer:1,item:19})).count(),1);
        update(&mut w, 1);
        assert_eq!(w.svc.interact.handoffs.len(),1,"holding Triangle cannot replay the demo or charge twice");
        assert_eq!((w.svc.counters.bolts, w.svc.bomb_evo.gold_spent), (0, 3));
        assert_eq!(
            w.svc
                .interact
                .writes
                .iter()
                .filter(|e| matches!(e, GameWrite::GoldWeapon(19)))
                .count(),
            1
        );
    }
    #[test]
    fn randomized_mode_has_no_gold_requirement_in_logic_or_any_language_prompt() {
        let mut b = bench();
        b.svc.bomb_evo.golden_settings.enabled = true;
        b.svc.interact.game.gold_bolts = 0;
        b.svc.counters.bolts = 60_000;
        let mut w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, 0);
        update(&mut w, 1);
        assert_eq!(w.svc.bomb_evo.weapon_level(19), Some(6));
        assert_eq!(w.svc.bomb_evo.gold_spent, 0);
        assert_eq!(w.svc.counters.bolts, 0);
        for language in [0, 2, 3, 4, 5] {
            let p = Price {
                item: 19,
                bolts: 60_000,
                gold: 3,
            };
            let normal = prompt(language, b"Tesla Claw", p, false, true);
            let random = prompt(language, b"Tesla Claw", p, true, true);
            assert_eq!(normal[0], 0x12);
            assert!(normal.len() < crate::moby_update::interact::PROMPT_TEXT_MAX);
            assert!(!random.contains(&b'+'));
            assert!(normal.contains(&b'+'));
        }
    }
    #[test]
    fn level_four_and_first_playthrough_never_charge_or_unlock() {
        let mut b = bench();
        b.svc.counters.bolts = 100_000;
        b.svc
            .bomb_evo
            .extras
            .iter_mut()
            .find(|t| t.id == 19)
            .unwrap()
            .level = 4;
        let mut w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, 0);
        update(&mut w, 1);
        assert_eq!(w.svc.counters.bolts, 100_000);
        assert_eq!(w.svc.bomb_evo.weapon_level(19), Some(4));
        w.svc.counters.times_completed = 0;
        update(&mut w, 1);
        assert!(w.m(1).is_deleted());
        assert_eq!(w.svc.counters.bolts, 100_000);
    }
    #[test]
    fn gemlik_ten_authored_offers_yield_exactly_five_new_displays() {
        let mut b = bench();
        b.svc.level = 13;
        let mut actors = vec![Moby::zeroed()];
        for key in 0i32..10 {
            let mut m = Moby::zeroed();
            m.o_class = 304;
            m.pvars = vec![0; 0x44];
            m.pvars[0x40..0x44].copy_from_slice(&key.to_le_bytes());
            m.position = [key as f32, 0., 50., 1.];
            actors.push(m);
        }
        b.table = MobyTable::new(actors, 0);
        let w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, 0);
        let mut s = State::default();
        s.prepare(&w);
        assert_eq!(
            s.displays.iter().map(|d| d.item).collect::<Vec<_>>(),
            vec![24, 13, 14, 18, 23]
        );
        assert_eq!(
            s.displays.iter().map(|d| d.key).collect::<Vec<_>>(),
            vec![1, 3, 5, 7, 9]
        );
        assert_eq!(
            s.displays.iter().map(|d| d.position[0]).collect::<Vec<_>>(),
            vec![1., 3., 5., 7., 9.]
        );
        // The trigger and display retain the same authored slot. The omitted
        // offers cannot sell a second weapon at the previous compressed spot.
        assert!(!s.displays.iter().any(|d| d.key == 0 || d.key == 2));
    }
    #[test]
    fn gemlik_five_authored_slots_are_never_filtered_a_second_time() {
        let mut b = bench();
        b.svc.level = 13;
        let mut actors = vec![Moby::zeroed()];
        for key in [1i32, 3, 5, 7, 9] {
            let mut m = Moby::zeroed();
            m.o_class = 304;
            m.pvars = vec![0; 0x44];
            m.pvars[0x40..0x44].copy_from_slice(&key.to_le_bytes());
            m.position = [key as f32, 0., 50., 1.];
            actors.push(m);
        }
        b.table = MobyTable::new(actors, 0);
        let w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, 0);
        let mut state = State::default();
        state.prepare(&w);
        assert_eq!(state.displays.len(), 5);
        assert_eq!(
            state.displays.iter().map(|d| d.key).collect::<Vec<_>>(),
            vec![1, 3, 5, 7, 9]
        );
        state.prepare(&w);
        assert_eq!(state.displays.len(), 5);
    }
}
