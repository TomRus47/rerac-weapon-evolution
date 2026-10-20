//! Regression fixtures for callbacks surviving a compacted death-reload table.
use super::*;
use crate::moby_update::classes::{draw_callbacks::{self, Callback}, units::{self, oltanis_arcs}};

#[test]
fn respawn_does_not_dispatch_old_arc_callback_to_reused_slot() {
    let row = units::row(oltanis_arcs::REFERENCE_LEVEL, oltanis_arcs::DRAW_FN).unwrap();
    for list in 0..3 {
        let mut svc = Services::new();
        // The last pre-death tick queued the keeper's frame callback at slot 0.
        match list {
            0 => svc.draw_callbacks.register(Callback::UnitFrame(row), 0),
            1 => svc.draw_callbacks.register2(Callback::UnitFrame(row), 0),
            _ => svc.draw_callbacks.register_ties(Callback::UnitFrame(row), 0),
        }
        // A collected/killed placed instance is absent on reload. Compaction leaves
        // an unrelated live object, with no pvars, at that same numeric slot.
        let mut table = MobyTable::new(vec![Moby::zeroed()], 0);
        svc.reset_moby_transients();
        let hero = Hero::new();
        let classes = ClassTable::default();
        let mut rng = Rng::new();
        let mut world = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 1);
        draw_callbacks::run_frame(&mut world);
    }
}

#[test]
fn new_load_can_register_and_run_effect_callbacks_after_reset() {
    let mut svc = Services::new();
    svc.reset_moby_transients();
    let row = units::row(oltanis_arcs::REFERENCE_LEVEL, oltanis_arcs::DRAW_FN).unwrap();
    let mut keeper = Moby::zeroed();
    keeper.o_class = 1331;
    keeper.state = 1;
    keeper.pvars = vec![0; 8];
    let mut table = MobyTable::new(vec![keeper], 0);
    svc.draw_callbacks.register(Callback::UnitFrame(row), 0);
    svc.draw_callbacks.register2(Callback::Walloper, 0);
    let hero = Hero::new();
    let classes = ClassTable::default();
    let mut rng = Rng::new();
    let mut world = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 1);
    draw_callbacks::run_frame(&mut world);
    assert!(world.svc.draw_callbacks.walloper_dim.is_some());
}

#[test]
fn reload_forgets_companion_reactions_and_arc_owners_but_preserves_progress_and_assets() {
    let mut svc = Services::new();
    svc.mouse.summoned = Some(17);
    assert!(svc.evo_stuns.attempt(2, 28, 0, 180, true));
    svc.evo_stuns.launch(2, [1.0; 4], 60);
    svc.bomb_evo.xp = 67.5;
    svc.bomb_evo.level = 3;
    svc.bomb_evo.pyro.xp = 31.0;
    let thresholds = svc.bomb_evo.xp_required;
    svc.save.death.insert((14, 8));
    svc.save.checkpoint_light = Some((42, [1, 2, 3, 4]));
    svc.joint_lists.insert(1331, vec![vec![7, 8]]);
    svc.level_words.insert(123, 456);
    svc.draw_callbacks.matrices.insert(17, [[1.0; 4]; 4]);
    svc.draw_callbacks.disguise = Some(([[1.0; 3]; 3], 0));
    let hero = Hero::new();
    let classes = ClassTable::default();
    let mut rng = Rng::new();
    let mut table = MobyTable::new(Vec::new(), 0);
    {
        let mut world = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 0);
        assert!(oltanis_arcs::register(&mut world, 17, 0, 0));
        assert!(oltanis_arcs::register(&mut world, 18, 0, 0));
        assert!(oltanis_arcs::register(&mut world, 19, 0, 0));
        assert!(!oltanis_arcs::register(&mut world, 20, 0, 0));
    }
    svc.reset_moby_transients();
    assert_eq!(svc.mouse.summoned, None);
    assert!(!svc.evo_stuns.frozen(2, 28, 1));
    assert!(svc.evo_stuns.launch_step(2).is_none());
    assert!(svc.draw_callbacks.matrices.is_empty());
    assert!(svc.draw_callbacks.disguise.is_none());
    assert_eq!((svc.bomb_evo.xp, svc.bomb_evo.level, svc.bomb_evo.pyro.xp), (67.5, 3, 31.0));
    assert_eq!(svc.bomb_evo.xp_required, thresholds);
    assert!(svc.save.death.contains(&(14, 8)));
    assert_eq!(svc.save.checkpoint_light, Some((42, [1, 2, 3, 4])));
    assert_eq!(svc.joint_lists[&1331], vec![vec![7, 8]]);
    assert_eq!(svc.level_words[&123], 456);
    let mut world = World::new(&mut table, &hero, &mut rng, &classes, &mut svc, 1);
    assert!(oltanis_arcs::register(&mut world, 20, 0, 0));
    assert!(oltanis_arcs::register(&mut world, 21, 0, 0));
    assert!(oltanis_arcs::register(&mut world, 22, 0, 0));
}
