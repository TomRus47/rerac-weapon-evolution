//! **The Visibomb** (item 13, gun class 163; level01 item update `0x2c8cc0`, read from the decompiler output; the
//! item table 0x179f40 gives item 13 the class 163). No weapon-check case: its update fires. Its missile is class 172
//! (`crate::moby_update::classes::visibomb`), the missile view the type-6 camera (`crate::follow_camera::type6`),
//! Ratchet's state while it flies 0x1d (`super::scripted`). docs/plan/hero_gameplay.md §17.
//!
//! **Pvars** ([`Visibomb`]): +0x00 the fire timer, +0x04 the missile in flight (cleared by the flight's end
//! `0x2cb788`: here, the missile gone from the table).
//!
//! **The update**: the fire timer steps (`FastDecTimer`). **Firing**: ○ pressed (the slot's fire mask), the timer out,
//! no missile in flight, not 0x1413fc, movement group 0, 1, 8 or 0xc (not 2..7, 9..0xb, ≥ 0xd), not in states 0x72 /
//! 0x65, the camera not the script camera (type 5: every script camera on the ported levels holds Ratchet in 0x72 or
//! 100, so the state tests cover it [L]), on the ground (0x13f650 ≠ 0), not on the classes 0x13e / 0x46f (the ground
//! moby 0x13f64c: never set in the port). One ammo (`0x249450(−1, 1)`); none → the gun's class sound 0 (the click).
//! With it: the aim — **first person** (state 1 with Ratchet hidden by the first-person camera, 0x1413f5): along the
//! view, yaw `atan(fwd.x, fwd.y)`, pitch `−atan(|fwd.xy|, fwd.z)`, from the eye + 0.3·fwd − 0.35·up; else Ratchet's
//! yaw (0x13f9d8 = 0x13f3e8 outside the grind group), pitch −0, from `polar(0.6, yaw, −0)` + the gun (0x141618 = 1,
//! the arm raised at a wall: the pitch of the gun's row 1, the point 0.6 along it); the gun's sequence 3; the help box
//! suspended (`0x225a28`); the launch `0x2cb540(yaw, pitch, gun, point)` (+ `SetState(0x1d, 1)`, made right after the
//! slot loop: `super::weapons::after_items`); the shot statistics 0x1416e8 (not ported: G-SAV-009); the fire timer
//! `ticks(60)`. **Tail**: the gun's sequence 1 once its sequence wrapped (+0x70 & 2); +0x34 |= 4 (keep the matrix:
//! the hand item's matrix is the attach's in the port, n/a).
//!
//! **Native.** Standard `f32`; the item is not a table moby. Gold (0x13e52d) is not mirrored.

use super::guns::{self, add3, with_len};
use super::items::{HitSink, ItemEnv};
use super::physics::{ticks, to_f32x3};
use super::Hero;
use crate::moby_runtime::{MobyId, MobyTable};
use crate::moby_update::classes::visibomb as missile;
use crate::moby_update::creature::atan;
use crate::rng::Rng;
use rc_formats::moby_anim;

pub const VISIBOMB: i32 = 13;
pub const CLASS: i16 = 163;
/// The gun's class sounds: the empty click.
pub const EMPTY_SOUND: i32 = 0;
/// The gun's sequences: the launch, and the one it returns to.
pub const FIRE_SEQ: u8 = 3;
pub const REST_SEQ: u8 = 1;

/// The gun's pvars (item moby +0x78).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Visibomb {
    pub fire_timer: i32,
    pub missile: Option<MobyId>,
}

/// Movement groups the Visibomb fires in (not 2..7, 9..0xb, ≥ 0xd).
pub fn group_fires(g: i32) -> bool { matches!(g, 0 | 1 | 8 | 0xc) }

/// The alternate launcher has no guided-missile camera or grounding requirement.
/// Only ordinary jumps/falls are added; hurt, death, rail and scripted states
/// must retain their native item restrictions.
fn grenade_can_fire(hero: &Hero) -> bool {
    hero.health > 0 && hero.items.f13fc == 0
        && (group_fires(hero.group)
            || (hero.group == 4 && matches!(hero.state, 7 | 9 | 0xa | 0xb | 0xd | 0xe | 0xf | 0x10 | 0x11 | 0x12))
            || (hero.group == 2 && hero.state == 6))
        && !matches!(hero.state, 0x72 | 0x65)
}

/// `0x2c8cc0`, the Visibomb's update (from the slot loop with the slot ready).
pub fn update(hero: &mut Hero, table: &mut MobyTable, _anim: &dyn super::anim::AnimCtl, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng) {
    let Some(class) = hero.items.slot.item.as_ref().and_then(|m| env.data.class(m.o_class)).cloned() else { return };
    let v = &mut hero.weapons.visibomb;
    guns::dec(&mut v.fire_timer);
    if v.missile.is_some_and(|m| table.mobys.get(m).is_none_or(|mo| mo.o_class != missile::CLASS || mo.state >= 0x80)) { v.missile = None; }
    if crate::weapon_evo::extra_flag(VISIBOMB,"grenade_launcher_enabled",false) && hero.weapons.visibomb.missile.is_none() {
        grenade_update(hero,table,env,hits,rng,&class);
        if let Some(it)=hero.items.slot.item.as_mut() {
            if it.anim.flags&2!=0 && it.anim.seq_b!=REST_SEQ {moby_anim::set_sequence(&mut it.anim,&class.anim,REST_SEQ,0,0,&mut it.snapshot);}
        }
        return;
    }
    let pressed = env.pad.pressed & hero.items.slot.fire_mask != 0;
    let fires = pressed
        && hero.weapons.visibomb.fire_timer == 0
        && hero.weapons.visibomb.missile.is_none()
        && hero.items.f13fc == 0
        && group_fires(hero.group)
        && !matches!(hero.state, 0x72 | 0x65)
        && hero.grounded_ticks != 0;
    if fires {
        let id = hero.items.slot.id;
        if !hero.weapons.use_ammo(id, 1) {
            hero.fx.item_sounds.push(EMPTY_SOUND);
        } else {
            fire(hero, table, env, hits, rng, &class);
        }
    }
    // The tail.
    if let Some(it) = hero.items.slot.item.as_mut() {
        if it.anim.flags & 2 != 0 && it.anim.seq_b != REST_SEQ { moby_anim::set_sequence(&mut it.anim, &class.anim, REST_SEQ, 0, 0, &mut it.snapshot); }
    }
}

/// The aim `(yaw, pitch, point)` of a launch (module docs).
pub fn aim(hero: &Hero, env: &ItemEnv) -> (f32, f32, [f32; 3]) {
    let look = hero.state == 1 && hero.f13f5 != 0;
    if let (true, Some((eye, f)), Some(u)) = (look, env.camera, env.camera_up) {
        let yaw = atan(f[0], f[1]);
        let pitch = -atan((f[0] * f[0] + f[1] * f[1]).sqrt(), f[2]);
        return (yaw, pitch, add3(add3(with_len(f, 0.3), with_len(u, -0.35)), eye));
    }
    let yaw = hero.rot[2].to_f32();
    let gun = hero.items.slot.item.as_ref().map_or(to_f32x3(hero.pos), |it| it.position);
    // The arm raised at a wall (0x141618 = 1): the pitch of the gun's row 1 (its +0xd0: the barrel is −row 1),
    // `atan(|row1.xy|, row1.z)`, the point 0.6 along it.
    if let (true, Some(r)) = (super::guns::arm_raised(hero), super::guns::item_rows(hero)) {
        let p = atan((r[1][0] * r[1][0] + r[1][1] * r[1][1]).sqrt(), r[1][2]);
        return (yaw, p, add3(crate::targeting::polar(0.6, yaw, -p), gun));
    }
    (yaw, -0.0, add3(crate::targeting::polar(0.6, yaw, -0.0), gun))
}

/// The firing part of `0x2c8cc0` (with the ammo used).
fn fire(hero: &mut Hero, table: &mut MobyTable, env: &ItemEnv, hits: &mut dyn HitSink, rng: &mut Rng, class: &super::items::ItemClass) {
    let (yaw, pitch, point) = aim(hero, env);
    if let Some(it) = hero.items.slot.item.as_mut() {
        if it.anim.seq_b != FIRE_SEQ { moby_anim::set_sequence(&mut it.anim, &class.anim, FIRE_SEQ, 0, 0, &mut it.snapshot); }
    }
    let gun = hero.items.slot.item.as_ref().map_or(to_f32x3(hero.pos), |it| it.position);
    let mut launched = None;
    hits.world(table, hero, rng, env.frame as u64, &mut |w| {
        w.svc.help.suspend();
        launched = missile::launch(w, yaw, pitch, gun, point);
    });
    hero.weapons.visibomb.missile = launched;
    if launched.is_some() { hero.weapons.deferred = Some(super::scripted::MISSILE); }
    hero.weapons.visibomb.fire_timer = ticks(0x3c);
}

/// Same launch solution supplies the landing marker and the actual grenade.
fn grenade_shot(hero:&Hero,env:&ItemEnv,settings:crate::moby_update::classes::bomb::visibomb_grenade::Settings)
    ->crate::moby_update::classes::bomb::visibomb_grenade::Shot {
    use super::physics::{from_f32x3,line_world};
    let from=aim(hero,env).2;
    let facing=[hero.rot[2].to_f32().cos(),hero.rot[2].to_f32().sin(),0.];
    let camera_aim=env.pad.held&crate::pad::button::STRAFE!=0 || hero.f13f5!=0;
    let direction=if camera_aim {env.camera.map_or(facing,|c|c.1)} else {facing};
    let horizontal=direction[0].hypot(direction[1]);
    let direction=if horizontal>0.0001 {direction.map(|v|v/horizontal)} else {facing};
    let end=[from[0]+direction[0]*settings.max_range,from[1]+direction[1]*settings.max_range,
        from[2]+direction[2].clamp(-8.,8.)*settings.max_range];
    let eye=if camera_aim {env.camera.map_or(from,|c|c.0)} else {from};
    let hit=env.coll.and_then(|coll|line_world(coll,from_f32x3(eye),from_f32x3(end),2));
    let target=if let Some(hit)=hit {hit.point} else {
        let top=[end[0],end[1],end[2]+1.];let bottom=[end[0],end[1],end[2]-settings.max_range.max(100.)*2.];
        env.coll.and_then(|coll|line_world(coll,from_f32x3(top),from_f32x3(bottom),2)).map_or([end[0],end[1],from[2]],|h|h.point)
    };
    crate::moby_update::classes::bomb::visibomb_grenade::solve(from,target,settings.arc_height,settings.max_range)
}

/// No hero state/velocity, deferred missile state or camera ownership changes.
/// Charge ammo and queue native Bomb Glove feedback only after successful creation.
fn grenade_update(hero:&mut Hero,table:&mut MobyTable,env:&ItemEnv,hits:&mut dyn HitSink,rng:&mut Rng,class:&super::items::ItemClass) {
    use crate::moby_update::classes::bomb::visibomb_grenade as grenade;
    let pressed=env.pad.pressed&hero.items.slot.fire_mask!=0;
    let allowed=grenade_can_fire(hero);
    if !allowed {return;}
    if hero.weapons.has_ammo(VISIBOMB)<=0 {if pressed {hero.fx.item_sounds.push(EMPTY_SOUND);}return;}
    let settings=grenade::Settings::current();let shot=grenade_shot(hero,env,settings);
    let fire=pressed && hero.weapons.visibomb.fire_timer==0;
    let mut launched=false;
    hits.world(table,hero,rng,env.frame as u64,&mut |w| {
        grenade::preview(w,shot);
        if fire {launched=grenade::launch(w,shot,settings).is_some();}
    });
    if launched {
        hero.weapons.use_ammo(VISIBOMB,1);
        hero.fx.item_voices.push(super::packs::SoundCmd::MobySound {moby:env.hero_moby,o_class:super::devastator::CLASS,pos:shot.from,index:super::devastator::FIRE_SOUND,flags:0});
        hero.fx.item_voices.push(super::packs::SoundCmd::Voice {index:super::weapons::THROW_VOICE,flags:0});
        hero.weapons.visibomb.fire_timer=ticks(settings.fire_interval);
        if let Some(it)=hero.items.slot.item.as_mut() {moby_anim::set_sequence(&mut it.anim,&class.anim,FIRE_SEQ,0,0,&mut it.snapshot);}
    }
}

#[cfg(test)]
mod grenade_tests {
    use super::*;
    use crate::hero::V4;
    use crate::ps2v::Pf;
    use crate::moby_update::classes::bomb_water::tests::classes_with;
    use crate::moby_update::services::{ClassTable,HitTemplate,Services,World};
    struct Sink {classes:ClassTable,services:Services}
    impl HitSink for Sink {
        fn sphere(&mut self,_:&mut MobyTable,_:Pf,_:V4,_:u32,_:Option<usize>,_:&HitTemplate)->Option<usize>{None}
        fn line(&mut self,_:&mut MobyTable,_:V4,_:V4,_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>>{None}
        fn world(&mut self,table:&mut MobyTable,hero:&Hero,rng:&mut Rng,counter:u64,f:&mut dyn FnMut(&mut World))->bool {
            f(&mut World::new(table,hero,rng,&self.classes,&mut self.services,counter));true
        }
    }
    fn fire_once(with_model:bool)->(Hero,MobyTable,Sink) { fire_in_state(with_model, 1, 1) }
    fn fire_in_state(with_model:bool,state:i32,group:i32)->(Hero,MobyTable,Sink) {
        let mut hero=Hero::new();hero.state=state;hero.group=group;hero.grounded_ticks=0;
        hero.vel=super::super::physics::v4(0.2,-0.3,0.1);hero.disp=super::super::physics::v4(0.01,-0.02,0.03);
        hero.items.slot.id=VISIBOMB;hero.items.slot.fire_mask=crate::pad::button::CIRCLE;
        hero.weapons.ammo[13]=3;hero.weapons.uses_ammo[13]=true;
        let mut table=MobyTable::new(vec![],96);
        let mut sink=Sink{classes:classes_with(&[]),services:Services::new()};
        if !with_model {sink.classes.classes.remove(&crate::moby_update::classes::bomb::BOMB_CLASS);}
        let class=super::super::items::ItemClass{o_class:CLASS,scale:1.,chains:vec![],
            anim:rc_formats::moby_anim::MobyAnimClass{joint_count:1,skeleton:vec![],rest:vec![[0.;3]],parent_word:vec![0],sequences:vec![]}};
        let data=super::super::items::ItemData::default();let mut pad=crate::pad::PadState::default();pad.pressed=crate::pad::button::CIRCLE;
        let env=ItemEnv{ mobys: None,data:&data,pad:&pad,frame:1,hero_moby:0,coll:None,camera:None,camera_up:None,targets:&[]};
        grenade_update(&mut hero,&mut table,&env,&mut sink,&mut Rng::new(),&class);(hero,table,sink)
    }
    #[test]
    fn replacement_preserves_movement_camera_and_native_throw_feedback() {
        let (hero,table,sink)=fire_once(true);
        assert_eq!(hero.state,1);assert_eq!(hero.vel,super::super::physics::v4(0.2,-0.3,0.1));
        assert_eq!(hero.disp,super::super::physics::v4(0.01,-0.02,0.03));
        assert_eq!(hero.weapons.deferred,None);assert_eq!(hero.weapons.visibomb.missile,None);
        assert_eq!(hero.weapons.ammo[13],2);
        assert_eq!(table.mobys.iter().filter(|m|crate::moby_update::classes::bomb::visibomb_grenade::is_grenade(m)).count(),1);
        assert!(!table.mobys.iter().any(|m|m.o_class==missile::CLASS));assert!(sink.services.visibomb.missile.is_none());
        assert!(hero.fx.item_voices.contains(&super::super::packs::SoundCmd::Voice{index:super::super::weapons::THROW_VOICE,flags:0}));
        assert_eq!(hero.fx.item_voices.iter().filter(|s|matches!(s,super::super::packs::SoundCmd::MobySound{o_class:super::super::devastator::CLASS,index:super::super::devastator::FIRE_SOUND,..})).count(),1);
    }
    #[test]
    fn real_jump_double_jump_and_fall_fire_without_taking_camera_or_momentum() {
        for (state,group) in [(7,4),(9,4),(0xa,4),(0xb,4),(0xd,4),(0xe,4),(0xf,4),(0x10,4),(0x11,4),(0x12,4),(6,2)] {
            let (hero,table,_)=fire_in_state(true,state,group);
            assert_eq!((hero.state,hero.group),(state,group));
            assert_eq!(hero.weapons.ammo[13],2,"state {state}");
            assert_eq!(hero.vel,super::super::physics::v4(0.2,-0.3,0.1));
            assert_eq!(hero.weapons.deferred,None);
            assert_eq!(hero.weapons.visibomb.missile,None);
            assert_eq!(table.mobys.iter().filter(|m|crate::moby_update::classes::bomb::visibomb_grenade::is_grenade(m)).count(),1);
        }
        for (state,group) in [(0x77,2),(0x72,0x14),(0x1d,0x14),(0x29,4)] {
            let (hero,_,_)=fire_in_state(true,state,group);assert_eq!(hero.weapons.ammo[13],3);
        }
        assert!(!group_fires(4));assert!(!group_fires(2));
    }
    #[test]
    fn missing_model_does_not_charge_ammo_or_throw_feedback() {
        let (hero,table,_)=fire_once(false);assert_eq!(hero.weapons.ammo[13],3);
        assert_eq!(hero.weapons.visibomb.fire_timer,0);assert!(hero.fx.item_voices.is_empty());
        assert!(!table.mobys.iter().any(crate::moby_update::classes::bomb::visibomb_grenade::is_grenade));
        assert_eq!(hero.weapons.deferred,None);
    }
}
