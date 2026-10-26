//! **Orxon's Clank section** (class 22, level10 0x298b68; census U323, one placed instance): the controller that puts
//! Clank in charge on Orxon. Without the O2 Mask (item 6, `0x13d4c6`) Ratchet cannot breathe there: the first visit hands
//! the hero to Clank (`SwitchCharacter(1, 0x43, clank)`, `crate::hero::bodies`) at one of its cuboids; with the mask
//! owned, a Clank in charge is given back to Ratchet (the level's leave copy 0x204198) at another cuboid, Clank's moby is
//! hidden and the controller deletes itself. It also flips two checkpoints by the mission state and keeps a help hint up
//! (record 0x141948). Read from the level10 decomp (the only copy).
//!
//! **Pvar block** (s32): +0x00 Clank's moby (class 0x57), +0x04 the cuboid Ratchet comes back at, +0x08 the cuboid Clank
//! starts at while the mission +0xb0 is not done, +0x14 the one after, +0x18 / +0x1c two checkpoints (class 805).
//!
//! | address | what | port |
//! |---|---|---|
//! | 0x298b68 | +0x30 = 0xff | [`update`] |
//! | | gp−0x6878 (0x160388) = 1 → 0, and the hint record 0x141948's count = 0 | NOT ported (G-UI-017: no ported class writes 0x160388; it stays 0) |
//! | | both checkpoints of class 805: the mission done → the first disabled (+8 \|= 8) and the second enabled, else the reverse | [`update`] |
//! | | this spawn id not collected nor dead before: → the death bits | [`update`] |
//! | | already visited: O2 Mask owned → Clank hidden (+0x34 \|= 3), no collision (+0x94 = 0); else game mode ≠ 0 or the hint shown ≥ ticks(300) times → in Clank: return; else the hint record bumped, `try_set_help_message(1, 0x53f1)`, in Clank: return | [`update`] |
//! | | the death bits 0x14c190 / 0x1ba950 for +0xb2 | [`update`] |
//! | | +0x08 or +0x00 = −1: return | [`update`] |
//! | | O2 Mask owned: not Clank → 0x14161b = 1, `DeleteMoby(self)`; Clank → leave (0x204198), `HeroTeleport(cuboid +0x04, 0, 1)`, `MusicRequestTrack(0, 5)`, Clank hidden and without collision, 0x14161b = 1, `DeleteMoby(self)` | [`update`] (`bodies::queue_leave`, `cinematic::hero_teleport`, `SoundSink::music_request`, `HeroFields::airless`) |
//! | | no mask: Clank's collision on (+0x94 = class +0x10); `HeroTeleport(cuboid +0x08 while the mission is not done and +0x14 ≠ −1, else +0x14; 0, 1)`; `SwitchCharacter(1, 0x43, clank)`; `MusicRequestTrack(2, 4)` | [`update`] (`bodies::queue_switch`) |
//! | | no sound, particle, light, HUD element | n/a |

use crate::moby_runtime::MobyId;
use crate::moby_update::services::{pvar as p, World};

/// The update in the level10 class table.
pub const UPDATE_FN: u32 = 0x29_8b68;
pub const REFERENCE_LEVEL: u32 = 10;
pub const CLASSES: [i16; 1] = [22];

/// The checkpoints' class.
const CHECKPOINT: i16 = 805;
/// The O2 Mask (item 6).
const O2_MASK: usize = 6;
/// "Clank's hint" (moves record 32 at 0x141948) and its prompt 0x53f1.
const HINT_RECORD: usize = 32;
const HINT_MSG: i32 = 0x53f1;

fn pv(w: &World, id: MobyId, o: usize) -> i32 { p::i32(&w.m(id).pvars, o) }

fn cuboid(w: &World, i: i32) -> Option<([f32; 3], [f32; 3])> {
    let s = w.svc.volumes.shape(rc_formats::volumes::ShapeKind::Cuboid, i)?;
    Some((s.centre(), s.euler))
}

fn teleport(w: &mut World, i: i32) {
    if let Some((pos, euler)) = cuboid(w, i) { crate::cinematic::hero_teleport(w, pos, euler, 0, true); }
}

fn music(w: &mut World, track: i16, stinger: i16) {
    if let Some(s) = w.sound.as_deref_mut() { s.music_request(track, stinger); }
}

/// Clank's moby hidden and frozen (+0x34 |= 1 | 2), no collision (+0x94 = 0).
fn hide_clank(w: &mut World, clank: MobyId) {
    let m = w.mm(clank);
    m.mode |= crate::moby_runtime::mode::HIDDEN | crate::moby_runtime::mode::NO_UPDATE;
    m.has_collision = false;
}

pub fn update(w: &mut World, id: MobyId) {
    w.mm(id).update_dist = 0xff;
    let level = w.svc.level;
    let done = crate::moby_update::classes::units::hints::mission_done(w, w.m(id).mission as i32);
    // The two checkpoints by the mission state.
    let (c6, c7) = (pv(w, id, 0x18), pv(w, id, 0x1c));
    if c6 != -1 && c7 != -1 {
        let (a, b) = (c6 as usize, c7 as usize);
        if w.table.mobys.get(a).is_some_and(|m| m.o_class == CHECKPOINT) && w.table.mobys.get(b).is_some_and(|m| m.o_class == CHECKPOINT) {
            let (fa, fb) = (p::u32(&w.m(a).pvars, 8), p::u32(&w.m(b).pvars, 8));
            let (fa, fb) = if done { (fa | 8, fb & !8) } else { (fa & !8, fb | 8) };
            p::set_u32(&mut w.mm(a).pvars, 8, fa);
            p::set_u32(&mut w.mm(b).pvars, 8, fb);
        }
    }
    let sid = w.m(id).spawn_id;
    // Travel and this native controller must agree. Reading ownership directly
    // here would delete Clank on arrival and strand his randomized checks when
    // the real Mask was found before Orxon. Ownership itself remains unchanged.
    let o2 = crate::weapon_evo::randomizer::travel_mask(
        w.svc.bomb_evo.progress().randomizer, level, w.hero.owned.has(O2_MASK));
    // Last tick's body word, not `World::body`: the hand-over this class queued in the load pass is made during the
    // landing's space scene, which leaves Clank in the scene state 100; the port relies on the first tick's second
    // `SwitchCharacter` to give him his idle state. A death reload's switch back into Clank counts too: the game makes
    // it in `0x29adc8` before any class runs, the port on the next tick (`Bodies::restore`); without it this class
    // would hand Clank the hero again and teleport him to its start, over the checkpoint's respawn.
    let clank = crate::hero::bodies::body::CLANK;
    let clank_now = w.hero.mode == clank || w.hero.bodies.restore.is_some_and(|r| r.0 == clank);
    let visited = w.svc.save.collected.get(&sid).is_some_and(|&b| b != 0) || w.svc.save.death.contains(&(level, sid));
    if visited {
        if o2 {
            let c = pv(w, id, 0);
            if c != -1 { hide_clank(w, c as usize); }
        } else if w.svc.game_mode != 0 || w.ticks(300) <= w.svc.help.records.moves[HINT_RECORD].count as i32 {
            if clank_now { return; }
        } else {
            let play = w.svc.help.play_time;
            crate::help::bump(&mut w.svc.help.records.moves[HINT_RECORD], level as i32, play);
            w.svc.interact.try_prompt(1, HINT_MSG);
            if clank_now { return; }
        }
    }
    // The death bits of this spawn id (persistent and this visit's).
    w.svc.save.death.insert((level, sid));
    w.svc.save.death_level.insert(sid);
    if pv(w, id, 8) == -1 || pv(w, id, 0) == -1 { return; }
    let clank = pv(w, id, 0) as usize;
    if o2 {
        if clank_now {
            crate::hero::bodies::queue_leave(w);
            teleport(w, pv(w, id, 4));
            music(w, 0, 5);
            hide_clank(w, clank);
        }
        w.hero_fields_mut().airless = Some(1);
        w.delete_moby(id);
        return;
    }
    let col = w.classes.info(w.m(clank).o_class).is_some_and(|i| i.has_collision);
    w.mm(clank).has_collision = col;
    let at = if !done && pv(w, id, 0x14) != -1 { pv(w, id, 8) } else { pv(w, id, 0x14) };
    teleport(w, at);
    crate::hero::bodies::queue_switch(w, crate::hero::bodies::body::CLANK, crate::hero::bodies::clank::IDLE, clank);
    music(w, 2, 4);
}


#[cfg(test)]
mod randomizer_tests {
    use super::*;
    use crate::hero::{bodies::body, Hero};
    use crate::moby_runtime::{mode, Moby, MobyTable};
    use crate::moby_update::{ClassTable, services::{HeroCall, Services}};
    use crate::rng::Rng;
    use crate::weapon_evo::{randomizer::State, save::Progress};

    fn fixture() -> (MobyTable, Hero, ClassTable, Services, Rng) {
        let mut controller=Moby::zeroed();controller.o_class=22;controller.spawn_id=73;
        controller.pvars=vec![0xff;0x20];
        for (offset,value) in [(0,1),(4,1),(8,0),(0x14,0)]{p::set_i32(&mut controller.pvars,offset,value);}
        let mut clank=Moby::zeroed();clank.o_class=87;
        let table=MobyTable::new(vec![controller,clank],0);
        let mut hero=Hero::new();hero.owned.0[O2_MASK]=1;
        let mut svc=Services::new();svc.level=10;svc.bomb_evo.randomizer_mode=true;
        let mut progress=Progress::default();progress.randomizer=State::generate(723);
        svc.bomb_evo.restore_progress(&progress);
        for pos in [[10.,20.,30.],[40.,50.,60.]] {
            let mut shape=rc_formats::volumes::Shape::default();
            shape.matrix[3]=[pos[0],pos[1],pos[2],1.];std::sync::Arc::make_mut(&mut svc.volumes).cuboids.push(shape);
        }
        (table,hero,ClassTable::default(),svc,Rng::new())
    }

    #[test]
    fn early_mask_keeps_native_clank_controller_and_death_checkpoint_alive() {
        let (mut table,mut hero,classes,mut svc,mut rng)=fixture();
        {
            let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);
            update(&mut w,0);
            assert!(w.m(0).state<128);
            assert_eq!(w.m(1).mode&(mode::HIDDEN|mode::NO_UPDATE),0);
            assert!(w.hero_fields().calls.iter().flatten().any(|c|matches!(c,HeroCall::SwitchCharacter{mode:body::CLANK,moby:1,..})));
        }
        // A checkpoint restore must keep Clank at that checkpoint, rather than
        // teleporting him back to the start or deleting him because of the Mask.
        hero.mode=body::CLANK;
        {
            let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,1);
            update(&mut w,0);
            assert!(w.m(0).state<128);assert!(w.hero_fields().calls.iter().all(Option::is_none));
            assert!(w.hero_fields().pose.is_none());assert!(hero.owned.has(O2_MASK));
        }
        // Finishing the active Clank checks permits the ordinary Ratchet handover.
        let mut progress=svc.bomb_evo.progress();
        for source in [28,75]{if progress.randomizer.reward(source).is_some(){progress.randomizer.collect(source);}}
        svc.bomb_evo.restore_progress(&progress);
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,2);
        update(&mut w,0);
        assert!(w.m(0).state>=128);
        assert_ne!(w.m(1).mode&mode::HIDDEN,0);
        assert!(w.hero_fields().calls.iter().flatten().any(|c|matches!(c,HeroCall::LeaveBody{..})));
        assert_eq!(w.hero_fields().airless,Some(1));
    }
}
