//! Accepted-hit visuals, independent of class stun/AI clocks. Never writes HP.
//! Ambient is restored before native updates, then tinted only for this draw.
use std::collections::HashMap;
use crate::moby_runtime::{MobyId,MobyTable};
use super::services::{World,HitRecord};
#[derive(Clone,Debug)]
struct Victim { class:i16, uid:u32, flash_until:u64, flash_ticks:u64,
    burn_until:u64, next_burn:u64, ambient:Option<[u8;4]> }
#[derive(Clone,Debug,Default)]
pub struct Tracker { victims:HashMap<MobyId,Victim> }
impl Tracker {
    pub fn forget(&mut self,id:MobyId){self.victims.remove(&id);}
    pub fn manages_fire(&self,id:MobyId)->bool{self.victims.get(&id).is_some_and(|v|v.burn_until!=0)}
    /// Refresh an existing accepted burn during real stream contact; never ignite
    /// a target merely because a hit was queued or a shield rejected damage.
    pub fn contact(&mut self,id:MobyId,frame:u64){if let Some(v)=self.victims.get_mut(&id){if v.burn_until!=0{v.burn_until=frame+2;}}}
    fn accept(&mut self,id:MobyId,class:i16,uid:u32,frame:u64,fade:u64,fire:bool){
        let v=self.victims.entry(id).or_insert(Victim{class,uid,flash_until:0,flash_ticks:fade,burn_until:0,next_burn:0,ambient:None});
        v.flash_until=frame+fade;v.flash_ticks=fade;
        if fire{v.burn_until=frame+2;}
    }
    fn restore(&mut self,table:&mut MobyTable){
        self.victims.retain(|&id,v|{
            let Some(m)=table.mobys.get_mut(id).filter(|m|m.o_class==v.class&&m.uid_hi==v.uid)else{return false};
            if let Some(base)=v.ambient.take(){m.ambient=base;}
            m.state<0x80
        });
    }
}
pub fn accepted(w:&mut World,hit:HitRecord){
    if !w.svc.bomb_evo.real_hp_enabled(){return;}
    let m=w.m(hit.target);let(class,uid)=(m.o_class,m.uid_hi);
    // Visual eligibility also covers native creature classes without an XP donor
    // adapter. Their accepted HP writes retain exactly their native semantics.
    if !super::real_hp::donor_class(class) && !w.classes.info(class).is_some_and(|c|c.ty==5){return;}
    let native_fire=hit.evolution.is_none() && (hit.h2a==176 || (hit.b28==5 && matches!(hit.h2a,0|74)));
    let wrench_fire=hit.h2a==0x47 && hit.w30 & crate::weapon_evo::wrench::BURN!=0;
    let fire=hit.evolution.is_some_and(|a|a.burn_color.is_some())||native_fire||wrench_fire;
    if hit.evolution.is_none() && fire {
        let (item,color)=if wrench_fire {(8,None)}else if hit.h2a==74 {
            (17,hit.attacker.and_then(|id|w.table.mobys.get(id)).and_then(super::classes::mine::experimental::burn_color))
        }else{(16,crate::weapon_evo::pyro_burn_color())};
        w.svc.hits.real_hp.burn_colors.insert(hit.target,(item,color));
    }
    let fade=w.ticks(15).max(1)as u64;
    w.svc.hits.real_hp.feedback.accept(hit.target,class,uid,w.counter,fade,fire);
}
pub fn before(w:&mut World){w.svc.hits.real_hp.feedback.restore(w.table);}
pub fn after(w:&mut World){
    let mut tracker=std::mem::take(&mut w.svc.hits.real_hp.feedback);
    let mut ids:Vec<_>=tracker.victims.keys().copied().collect();ids.sort_unstable();
    for id in ids {
        let v=tracker.victims.get_mut(&id).unwrap();
        let Some(m)=w.table.mobys.get_mut(id).filter(|m|m.state<0x80&&m.o_class==v.class&&m.uid_hi==v.uid)else{continue};
        let strength=(v.flash_until.saturating_sub(w.counter)as f32/v.flash_ticks.max(1)as f32).clamp(0.,1.);
        if strength>0.{
            let base=m.ambient;v.ambient=Some(base);
            m.ambient[0]=(base[0]as f32+(127.-base[0]as f32)*strength).round()as u8;
            m.ambient[1]=(base[1]as f32*(1.-0.85*strength)).round()as u8;
            m.ambient[2]=(base[2]as f32*(1.-0.85*strength)).round()as u8;
        }
        if v.burn_until>=w.counter && v.burn_until!=0 && w.counter>=v.next_burn {
            let life=super::creature::knock::burn_sparks_cycle(w,id,[0.;4],u32::MAX).max(1);
            v.next_burn=w.counter+life as u64;
        }
    }
    tracker.victims.retain(|_,v|v.ambient.is_some()||w.counter<=v.flash_until.max(v.next_burn).max(v.burn_until));
    w.svc.hits.real_hp.feedback=tracker;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hero::Hero,moby_runtime::Moby,moby_update::services::{Services,ClassTable},rng::Rng};
    #[test]
    fn contact_does_not_ignite_an_unaccepted_or_shielded_target(){
        let mut tracker=Tracker::default();tracker.contact(2,10);assert!(tracker.victims.is_empty());
    }
    #[test]
    fn flash_and_fire_repeat_without_reaction_restarts_or_particle_stacking(){
        let mut table=MobyTable::new(vec![Moby{o_class:577,state:1,ambient:[40,60,80,0],position:[20.,20.,2.,1.],..Default::default()}],0);
        let hero=Hero::new();let mut rng=Rng::new();let classes=ClassTable::default();let mut svc=Services::new();
        svc.hits.real_hp.burn_colors.insert(0,(16,Some(0xff0000)));
        let base=table.mobys[0].ambient;
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,1);
        w.svc.hits.real_hp.feedback.accept(0,577,0,1,15,true);
        after(&mut w);
        assert!(w.m(0).ambient[0]>base[0]);assert!(w.m(0).ambient[1]<base[1]);
        let count=*w.svc.fx.part_spawns.get(&4).unwrap();assert!(count>0);
        let end=w.svc.hits.real_hp.feedback.victims[&0].next_burn;
        for frame in 2..end {
            w.counter=frame;before(&mut w);assert_eq!(w.m(0).ambient,base);
            w.svc.hits.real_hp.feedback.contact(0,frame);after(&mut w);
            assert_eq!(w.svc.fx.part_spawns[&4],count,"one finite batch, never one per frame");
        }
        w.counter=end;before(&mut w);w.svc.hits.real_hp.feedback.contact(0,end);after(&mut w);
        assert!(w.svc.fx.part_spawns[&4]>count,"continued contact restarts only the expired batch");
        let total=w.svc.fx.part_spawns[&4];
        w.counter=end+100;before(&mut w);after(&mut w);
        assert_eq!(w.svc.fx.part_spawns[&4],total);assert_eq!(w.m(0).ambient,base);
        assert!(w.svc.hits.real_hp.feedback.victims.is_empty());
    }
    #[test]
    fn native_creature_visuals_do_not_require_an_xp_donor_adapter(){
        use crate::moby_runtime::ClassInfo;
        let mut table=MobyTable::new(vec![Moby{o_class:9999,state:1,ambient:[40;4],..Default::default()}],0);
        let hero=Hero::new();let mut rng=Rng::new();let mut classes=ClassTable::default();
        classes.classes.insert(9999,(ClassInfo{ty:5,..Default::default()},None));
        let mut svc=Services::new();svc.bomb_evo.enabled=true;svc.bomb_evo.real_hp=true;
        let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,1);
        accepted(&mut w,HitRecord{target:0,h2a:176,..Default::default()});
        assert!(w.svc.hits.real_hp.feedback.manages_fire(0));
        let before=w.m(0).pvars.clone();after(&mut w);
        assert_eq!(w.m(0).pvars,before,"cosmetics cannot modify native HP or AI records");
        assert_eq!(w.m(0).ambient[0],127);
    }
    #[test]
    fn recycled_actor_slot_never_inherits_a_tint_or_burn(){
        let mut tracker=Tracker::default();tracker.accept(0,577,1,0,15,true);
        tracker.victims.get_mut(&0).unwrap().ambient=Some([1;4]);
        let mut table=MobyTable::new(vec![Moby{o_class:577,uid_hi:2,state:1,ambient:[70;4],..Default::default()}],0);
        tracker.restore(&mut table);assert_eq!(table.mobys[0].ambient,[70;4]);assert!(tracker.victims.is_empty());
    }
}
