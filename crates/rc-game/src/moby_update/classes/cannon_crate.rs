//! Optional crate ammunition for the Suck Cannon. The original crate instance
//! retains its class, contents, save/drop metadata and nanotech-cluster owner.
//! Only its motion is replaced while carried/fired; destruction uses native
//! crate rewards and one Suck Cannon damage instance, never contact + blast twice.
use crate::moby_runtime::{mode, MobyId, Moby};
use crate::moby_update::creature::react::{self, rec};
use crate::moby_update::services::{pvar as p, HitTemplate, World};
use crate::ps2v::Pf;

pub const RECORD: usize = 0x100;
const TAG: usize = 0x1b0;
const MAGIC: u32 = 0x43524154;
const LAST_PULL: usize = 0x1b4;
const ORIGINAL_MODE: usize = 0x1b8;
const ORIGINAL_SCALE: usize = 0x1bc;
const NATIVE_DAMAGE: usize = 0x1c0;
const SAVED_COLLISION: usize = 0x1c4;
pub const CARRIED: u8 = 0x40;

pub fn marked(m: &Moby) -> bool {
    super::crate_::CLASSES.contains(&m.o_class) && m.pvars.len() >= 0x200 && p::u32(&m.pvars, TAG) == MAGIC
}
pub fn swallowed(m: &Moby) -> bool { marked(m) && p::i16(&m.pvars, RECORD + rec::STATE) == 4 }
pub fn wrappers(m: &Moby) -> Option<react::Wrappers> {
    marked(m).then_some(react::Wrappers { held: CARRIED, record: RECORD, seqs: [0; 9],
        bounce_sound: None, release_seq: None, ..react::CRITTER })
}
pub fn allowed(class: i16) -> bool {
    allowed_with(class, |key| crate::weapon_evo::extra_flag(9, key, key != "crate_suction_enabled"))
}
fn allowed_with(class: i16, flag: impl Fn(&str) -> bool) -> bool {
    let key = match class {
        500 => "suck_wooden_crates", 505 => "suck_explosive_crates", 502 => "suck_steel_crates",
        511 => "suck_ammo_crates", 501 => "suck_nanotech_crates", _ => return false,
    };
    flag("crate_suction_enabled") && flag(key)
}

/// Called from the cannon's existing cone/visibility/obstacle walk. No crate is
/// initialized, detached or removed merely because the experiment is enabled.
pub fn pull(w: &mut World, id: MobyId, mouth: [f32; 4]) -> bool {
    if !allowed(w.m(id).o_class) { return false; }
    start_pull(w, id, mouth)
}

fn start_pull(w: &mut World, id: MobyId, mouth: [f32; 4]) -> bool {
    let m = w.m(id);
    if !m.has_class || !matches!(m.state, 1 | CARRIED) { return false; }
    if marked(m) && p::i16(&m.pvars, RECORD + rec::STATE) >= 4 { return false; }
    let limit = w.svc.creatures.react.limit();
    if !marked(m) || m.state != CARRIED {
        let Some(slot) = (0..limit).find(|&i| w.svc.creatures.react.slots[i] == 0) else { return false; };
        // Unlink the stack without breaking the crate or releasing its contents.
        let link = |raw: i32| raw.checked_sub(1).and_then(|v| usize::try_from(v).ok());
        let above = link(p::i32(&m.pvars, 0xa0));
        let below = link(p::i32(&m.pvars, 0xa4));
        for (other, offset, value) in [(above, 0xa4, below), (below, 0xa0, above)] {
            if let Some(other) = other.filter(|&i| w.table.mobys.get(i).is_some_and(|m| m.pvars.len() >= 0x100)) {
                let m = w.mm(other);
                p::set_i32(&mut m.pvars, offset, value.map_or(0, |i| i as i32 + 1));
                let flags = p::u32(&m.pvars, 0xac) | 4;
                p::set_u32(&mut m.pvars, 0xac, flags);
            }
        }
        let m = w.mm(id);
        m.pvars.resize(0x200, 0);
        p::set_u32(&mut m.pvars, TAG, MAGIC);
        p::set_u32(&mut m.pvars, ORIGINAL_MODE, m.mode as u32);
        p::set_ff(&mut m.pvars, ORIGINAL_SCALE, m.scale);
        p::set_i32(&mut m.pvars, SAVED_COLLISION, m.has_collision as i32);
        p::set_i32(&mut m.pvars, 0xa0, 0);p::set_i32(&mut m.pvars, 0xa4, 0);p::set_i32(&mut m.pvars, 0xf0, 0);
        p::set_i16(&mut m.pvars, RECORD + rec::SLOT, slot as i16);
        p::set_i16(&mut m.pvars, RECORD + rec::STATE, 3);
        p::set_i16(&mut m.pvars, RECORD + rec::SOUND_FIRE, -1);
        p::set_i16(&mut m.pvars, RECORD + rec::SOUND_BOUNCE, -1);
        m.state = CARRIED;
        m.has_collision = false;
        m.mode &= !(mode::NO_UPDATE | mode::TARGETABLE);
        w.svc.creatures.react.slots[slot] = id as u32 + 1;
        w.svc.creatures.react.coming += 1;
    }
    let tick = w.counter as u32;
    let m = w.mm(id);
    p::set_u32(&mut m.pvars, LAST_PULL, tick);
    p::set_v4f(&mut m.pvars, RECORD + rec::POINT, mouth);
    true
}

/// The normal slot_fire path supplies its native velocity and homing target.
/// Snapshot this grade's HP before destruction can level the weapon up.
pub fn fired(w: &mut World, id: MobyId) {
    if !marked(w.m(id)) { return; }
    let factor = if w.m(id).o_class == 502 { 2.0 } else { 1.0 };
    let native = 2.0 * crate::weapon_evo::extra(9, "shot_damage_multiplier", 1.0) * factor;
    let hp = w.svc.hits.real_hp.values.get(&9).copied().unwrap_or(native * 100.0 / factor) * factor;
    w.svc.hits.real_hp.set_source_damage_hp(id, 9, hp);
    let m = w.mm(id);
    p::set_ff(&mut m.pvars, NATIVE_DAMAGE, native);
    m.scale = p::ff(&m.pvars, ORIGINAL_SCALE);
    m.visible = 1;
    m.mode &= !(mode::HIDDEN | mode::NO_UPDATE | mode::KEEP_MATRIX | mode::KEEP_ROWS | 0x80);
    m.has_collision = false;
}

fn template(w: &World, id: MobyId) -> HitTemplate {
    let explosive = w.m(id).o_class == 505;
    HitTemplate { attacker: Some(id), flags: if explosive { 0x83_0000 } else { 0x43_0000 },
        b18: if explosive { 2 } else { 1 }, b19: 1, h1a: 0x351,
        dir: [Pf::ZERO, Pf::ZERO, Pf::ONE, Pf::b(0x45af_df66)],
        damage: Pf::f(p::ff(&w.m(id).pvars, NATIVE_DAMAGE)), w20: 1 }
}
fn finish(w: &mut World, id: MobyId, targets: &[MobyId]) {
    let tnt = w.m(id).o_class == 505;
    let tmpl = template(w, id);
    if tnt {
        let pos = w.m(id).position.map(Pf::f);
        // The blast, rather than a separate contact event, owns TNT damage/XP.
        w.sphere_mobys(Pf::f(3.0), pos, 0x10, Some(id), Some(&tmpl));
    } else {
        for &target in targets { if target != id && Some(target) != w.hero_moby { w.deliver_hit(target, &tmpl); } }
    }
    super::crate_::break_fx(w, id, tnt, None);
    super::crate_::drop_bolts(w, id);
    // Prevent the native TNT growing sphere from adding unattributed/repeated damage.
    w.delete_moby(id);
}

/// True owns the update; false leaves ordinary crates on their native path.
pub fn update(w: &mut World, id: MobyId) -> bool {
    if !marked(w.m(id)) || w.m(id).state != CARRIED { return false; }
    let state = p::i16(&w.m(id).pvars, RECORD + rec::STATE);
    if state == 3 {
        let point = p::v4f(&w.m(id).pvars, RECORD + rec::POINT);
        let old = w.m(id).position;
        let delta = [point[0]-old[0], point[1]-old[1], point[2]-old[2]];
        let distance = delta.iter().map(|v| v*v).sum::<f32>().sqrt();
        if w.counter.saturating_sub(p::u32(&w.m(id).pvars, LAST_PULL) as u64) > 3 {
            release(w, id);return true;
        }
        if distance <= 0.55 {
            let m=w.mm(id);m.position=point;m.scale=0.0;m.visible=0;m.mode|=mode::HIDDEN;
            p::set_i16(&mut m.pvars, RECORD + rec::STATE, 4);
            w.svc.creatures.react.coming=(w.svc.creatures.react.coming-1).max(0);
            w.svc.creatures.react.held+=1;
            w.svc.creatures.react.cannon_sound5+=1;
        } else {
            let step=0.55f32.min(distance);
            let m=w.mm(id);
            for k in 0..3 {m.position[k]+=delta[k]*step/distance;}
            m.scale=p::ff(&m.pvars, ORIGINAL_SCALE)*(distance/4.0).clamp(0.15,1.0);
        }
        w.build_matrix(id);
    } else if state == 4 {
        let m=w.mm(id);m.scale=0.0;m.visible=0;m.mode|=mode::HIDDEN;
    } else if matches!(state,5|6) {
        flight(w,id);
    } else { release(w,id); }
    true
}

fn release(w: &mut World,id:MobyId) {
    let slot=p::i16(&w.m(id).pvars,RECORD+rec::SLOT);
    if let Some(v)=w.svc.creatures.react.slots.get_mut(slot.max(0) as usize) { if *v==id as u32+1 {*v=0;} }
    let m=w.mm(id);
    m.state=1;m.visible=1;m.scale=p::ff(&m.pvars,ORIGINAL_SCALE);
    m.mode=p::u32(&m.pvars,ORIGINAL_MODE) as u16;
    m.has_collision=p::i32(&m.pvars,SAVED_COLLISION)!=0;
    p::set_i16(&mut m.pvars,RECORD+rec::STATE,0);
    p::set_u32(&mut m.pvars,TAG,0);
    let flags=p::u32(&m.pvars,0xac)|4;p::set_u32(&mut m.pvars,0xac,flags);
    w.build_matrix(id);
}

fn flight(w:&mut World,id:MobyId) {
    let old=w.m(id).position;
    let mut velocity=p::v4f(&w.m(id).pvars,RECORD+rec::VEL);
    let target=p::i32(&w.m(id).pvars,RECORD+rec::TARGET).checked_sub(1).and_then(|v|usize::try_from(v).ok());
    if let Some(target)=target.filter(|&i|w.table.mobys.get(i).is_some_and(|m|!m.is_deleted())) {
        let point=crate::targeting::aim_point(w.m(target));
        let delta=[point[0]-old[0],point[1]-old[1],point[2]-old[2]];
        let length=delta.iter().map(|v|v*v).sum::<f32>().sqrt();
        let speed=velocity[..3].iter().map(|v|v*v).sum::<f32>().sqrt();
        if length>0.001 {
            for k in 0..3 {velocity[k]+= (delta[k]*speed/length-velocity[k])*0.07;}
            let length=velocity[..3].iter().map(|v|v*v).sum::<f32>().sqrt();
            if length>0.001 {for v in &mut velocity[..3] {*v*=speed/length;}}
        }
    }
    let next=[old[0]+velocity[0],old[1]+velocity[1],old[2]+velocity[2],old[3]];
    // The crate has no collision body while airborne; skip Ratchet at the muzzle.
    let hit=w.coll_line(old.map(Pf::f),next.map(Pf::f),0x10,w.hero_moby);
    let point=hit.as_ref().map_or(next,|h|[h.point[0],h.point[1],h.point[2],next[3]]);
    let mut targets=w.sphere_mobys_list(Pf::f(0.6),point.map(Pf::f),0x10,w.hero_moby,None);
    if let Some(id)=hit.as_ref().and_then(|h|h.moby) {if !targets.contains(&id) {targets.push(id);}}
    let timer=p::i32(&w.m(id).pvars,RECORD+rec::TIMER)-1;
    let m=w.mm(id);m.position=point;m.rotation[0]+=0.08;m.rotation[1]+=0.11;
    p::set_v4f(&mut m.pvars,RECORD+rec::VEL,velocity);
    p::set_i32(&mut m.pvars,RECORD+rec::TIMER,timer);
    w.build_matrix(id);
    let h=w.hero.pos.map(|p|p.to_f32());
    let far=(0..3).map(|k|(point[k]-h[k]).powi(2)).sum::<f32>()>60.0*60.0;
    if hit.is_some() || !targets.is_empty() || timer<=0 || far {finish(w,id,&targets);}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::Hero;
    use crate::moby_runtime::{ClassInfo,MobyTable};
    use crate::moby_update::services::{ClassTable,Services};
    use crate::rng::Rng;

    fn setup(class:i16)->(MobyTable,Hero,Rng,ClassTable,Services) {
        let mut classes=ClassTable::default();
        for (slot,oc) in [0,500,501,502,505,511,577,13,357,362,363,348,806].into_iter().enumerate() {
            classes.classes.insert(oc,(ClassInfo {slot:slot as u8,scale:1.0,has_collision:true,mode_bits:0x5000,..Default::default()},None));
        }
        let mut source=Moby::init_instance(1,class,Some(&classes.classes[&class].0));
        source.pvars.resize(0x100,0);source.state=1;source.position=[100.0,100.0,50.0,1.0];
        source.b4=10;source.b6=10;source.spawn_flag=0xff;source.spawn_id=12;source.mission=0xff;
        p::set_i32(&mut source.pvars,0xc0,-1);
        if class==511 {p::set_i16(&mut source.pvars,0xc6,100);source.pvars[0xcb]=10;}
        let mut enemy=Moby::init_instance(2,577,Some(&classes.classes[&577].0));
        enemy.position=[100.5,100.0,50.0,1.0];enemy.pvars.resize(0x200,0);enemy.state=1;
        let mut table=MobyTable::new(vec![Moby::init_instance(0,0,Some(&classes.classes[&0].0)),source,enemy],128);
        for i in [1,2] {table.mobys[i].bsphere=[100.0*1024.0,100.0*1024.0,50.0*1024.0,512.0];}
        table.mobys[2].rows=[[1.0,0.0,0.0,0.0],[0.0,1.0,0.0,0.0],[0.0,0.0,1.0,0.0],[0.0;4]];
        let hero=Hero::spawn([90.0,100.0,50.0],0.0);
        let mut svc=Services::new();svc.hits.real_hp.enabled=true;svc.hits.real_hp.hero=Some(0);
        svc.hits.real_hp.values.insert(9,150.0);
        svc.bomb_evo.enabled=true;svc.bomb_evo.real_hp=true;svc.bomb_evo.damage_hp[9][0]=Some(150.0);
        let mut raw=[0u32;8];raw[0]=1|4<<8|0x8003<<16;raw[7]=512.0f32.to_bits();
        svc.set_moby_collision(vec![(577,rc_formats::moby_collision::MobyCollision {prims:vec![rc_formats::moby_collision::MobyCollPrim{raw}],..Default::default()})]);
        svc.build_grid(&mut table);
        (table,hero,Rng::new(),classes,svc)
    }
    #[test]
    fn each_crate_type_has_its_own_switch_and_the_master_defaults_off() {
        for (class,key) in [(500,"suck_wooden_crates"),(505,"suck_explosive_crates"),(502,"suck_steel_crates"),(511,"suck_ammo_crates"),(501,"suck_nanotech_crates")] {
            assert!(allowed_with(class, |_|true));
            assert!(!allowed_with(class, |k|k!="crate_suction_enabled"));
            assert!(!allowed_with(class, |k|k!=key));
        }
        assert!(!allowed_with(577, |_|true));assert!(!allowed(500));
    }
    #[test]
    fn carrying_keeps_contents_and_save_metadata_until_fired_destruction() {
        for class in super::super::crate_::CLASSES {
            let (mut table,hero,mut rng,classes,mut svc)=setup(class);
            let before=table.mobys[1].clone();
            let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,1);
            assert!(start_pull(&mut w,1,[100.0,100.0,50.0,1.0]));
            assert!(update(&mut w,1));assert!(swallowed(w.m(1)));
            assert_eq!(w.svc.creatures.react.slots[0],2);
            assert_eq!(w.svc.creatures.react.held,1);assert_eq!(w.m(1).scale,0.0);
            assert_eq!((w.m(1).b4,w.m(1).b6,w.m(1).spawn_id),(before.b4,before.b6,before.spawn_id));
            assert_eq!(&w.m(1).pvars[0xc4..0xcc],&before.pvars[0xc4..0xcc]);
            assert!(w.svc.save.death.is_empty());assert!(w.svc.bomb_evo.xp==0.0);
            assert_ne!(react::slot_fire(&mut w,1,0.5,[0.45,0.0,0.0,0.0],Some(2)),0);
            assert_eq!(w.m(1).o_class,class);assert_eq!(w.m(1).scale,1.0);
            assert_eq!(w.m(1).mode & 0x81,0);assert_eq!(w.m(1).visible,1);
            let tmpl=template(&w,1);
            w.deliver_hit(2,&tmpl);
            let hit=w.svc.hits.real_hp.frozen_next(2).unwrap();
            assert_eq!(hit.evolution.unwrap().weapon,9);
            assert_eq!(hit.evolution.unwrap().damage,if class==502 {3.0}else{1.5});
            // A later grade change does not change this already fired crate.
            w.svc.hits.real_hp.values.insert(9,999.0);
            finish(&mut w,1,&[2]);
            assert!(w.m(1).is_deleted());assert!(w.svc.save.death_level.contains(&12));
            let again=w.svc.hits.real_hp.frozen_next(2).unwrap();
            assert_eq!(hit.evolution,again.evolution);
        }
    }
    #[test]
    fn interrupted_suction_restores_a_crate_and_does_not_drop_its_contents() {
        let (mut table,hero,mut rng,classes,mut svc)=setup(500);
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,1);
        assert!(start_pull(&mut w,1,[110.0,100.0,50.0,1.0]));
        w.counter=5;update(&mut w,1);
        assert!(!marked(w.m(1)));assert_eq!(w.m(1).state,1);
        assert!(w.m(1).has_collision);assert_eq!(w.m(1).scale,1.0);
        assert_eq!(w.svc.creatures.react.slots[0],0);assert!(w.svc.save.death.is_empty());
    }
    #[test]
    fn explosive_crate_blast_owns_suck_cannon_damage_without_a_contact_double_hit() {
        let (mut table,hero,mut rng,classes,mut svc)=setup(505);
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,1);
        assert!(start_pull(&mut w,1,[100.0,100.0,50.0,1.0]));update(&mut w,1);
        react::slot_fire(&mut w,1,0.5,[0.45,0.0,0.0,0.0],None);
        finish(&mut w,1,&[]);
        let hit=w.svc.hits.real_hp.take(2,0x83_0000).unwrap();
        assert_eq!(hit.evolution.unwrap().weapon,9);assert_eq!(hit.evolution.unwrap().damage,1.5);
        assert!(w.svc.hits.real_hp.take(2,0x83_0000).is_none());
        assert!(w.m(1).is_deleted());
    }
}
