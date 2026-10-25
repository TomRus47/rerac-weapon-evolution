//! Regression coverage for the clock boundary between enemies and player items.
use rc_game::{hero::Hero, moby_runtime::{Moby, MobyId, MobyTable}, rng::Rng};
use rc_game::moby_update::{creature::react, services::{pvar as p, ClassTable, Services, World}};
use rc_game::weapon_evo::world::motion;

fn carried(w: &mut World, id: MobyId) { react::carried(w, id, 0); }
fn count(w: &mut World, id: MobyId) {
    let n = p::i32(&w.m(id).pvars, 0);
    p::set_i32(&mut w.mm(id).pvars, 0, n + 1);
}

#[test]
fn suction_completes_at_every_enemy_speed_and_still_releases_when_stopped() {
    for speed in [0.1, 0.5, 1., 1.25, 2., 4.] {
        let mut critter = Moby::init_instance(0, 577, None);
        critter.has_class = true;
        critter.state = react::CRITTER.held;
        critter.position = [0., 5., 10., 1.];
        critter.pvars = vec![0; 0x60 + react::rec::SIZE];
        p::set_i16(&mut critter.pvars, 0x60 + react::rec::STATE, 3);
        p::set_i16(&mut critter.pvars, 0x60 + react::rec::SLOT, 0);
        p::set_ff(&mut critter.pvars, 0x60 + react::rec::DIST0, 5.);
        let mut table = MobyTable::new(vec![critter], 8);
        let hero = Hero::new(); let mut rng = Rng::new();
        let classes = ClassTable::default(); let mut svc = Services::new();
        svc.bomb_evo.world_settings.enemy_speed_multiplier = speed;
        svc.creatures.react.cannon = Some(react::Cannon {
            pos: [0., 0., 10.], mouth: [0., 0., 10.],
            rows: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        });
        svc.creatures.react.slots[0] = 1; svc.creatures.react.coming = 1;
        let mut w = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 0);
        for _ in 0..120 {
            react::take(&mut w, 0, [0., 0., 10., 1.]);
            motion::run(&mut w, 0, carried);
            assert_ne!(react::rec_state(&w, 0), Some(7), "released at speed {speed}");
            if react::rec_state(&w, 0) == Some(4) { break; }
        }
        assert_eq!(react::rec_state(&w, 0), Some(4), "not swallowed at {speed}");
        assert_eq!((w.svc.creatures.react.coming, w.svc.creatures.react.held), (0, 1));
        // A new approaching creature with no refresh must still be released.
        w.mm(0).state = react::CRITTER.held;
        p::set_i16(&mut w.mm(0).pvars, 0x60 + react::rec::STATE, 2);
        p::set_ff(&mut w.mm(0).pvars, 0x60 + react::rec::HOLD, 1.);
        motion::run(&mut w, 0, carried);
        assert_eq!(react::rec_state(&w, 0), Some(2));
        motion::run(&mut w, 0, carried);
        assert_eq!(react::rec_state(&w, 0), Some(7));
    }
}

#[test]
fn all_native_capture_wrappers_use_one_clock_and_stale_records_do_not() {
    use react::Table;
    let families = [(577, Table::Critter), (866, Table::Amoeboid), (270, Table::Chicken),
        (749, Table::Veldin749), (568, Table::RollingMine568), (193, Table::Held7),
        (252, Table::Hover252), (1246, Table::Pokitaru1246), (580, Table::Aridia580),
        (827, Table::Blarg827), (238, Table::Hoven238), (340, Table::Eudora340),
        (1382, Table::Fleet1382), (1906, Table::Veldin1906)];
    for (class, reaction) in families {
        let mut m = Moby::init_instance(0, class, None);
        m.has_class = true; m.pvars = vec![0; 0x400];
        let mut table = MobyTable::new(vec![m], 2);
        let hero = Hero::new(); let mut rng = Rng::new();
        let classes = ClassTable::default(); let mut svc = Services::new();
        svc.creatures.react.tables.insert(class, reaction);
        svc.bomb_evo.world_settings.enemy_speed_multiplier = 4.;
        svc.bomb_evo.world_settings.enemy_projectile_speed_multiplier = 4.;
        let mut w = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 0);
        let wrapper = react::wrappers(&w, 0).unwrap();
        w.mm(0).state = wrapper.held;
        for state in 1..=7 {
            p::set_i16(&mut w.mm(0).pvars, wrapper.record + react::rec::STATE, state);
            assert!(react::cannon_controls(&w, 0), "class {class}, state {state}");
            let before = p::i32(&w.m(0).pvars, 0);
            motion::run(&mut w, 0, count);
            assert_eq!(p::i32(&w.m(0).pvars, 0), before + 1);
        }
        w.mm(0).state = wrapper.held.wrapping_add(1);
        assert!(!react::cannon_controls(&w, 0), "stale record {class}");
    }
}
