//! Early O2 ownership must not skip Clank's checks. Open only the vertical
//! shortcut whose upper spline endpoint is closest to the original Magneboots
//! pedestal. Match the relocated update function, not Novalis's class number.
//! Orxon uses class 1141 (docs/plan/level_generalisation.md, C1).
use crate::moby_runtime::{MobyId,MobyTable};
use crate::moby_update::services::World;
use crate::moby_update::services::pvar as p;

fn candidate(table:&MobyTable,splines:&[Vec<[u32;4]>],pedestal:Option<[f32;3]>)->Option<(MobyId,bool)> {
    // The randomized source anchor survives collection and actor-slot reuse.
    let boots=pedestal.or_else(||table.mobys.iter().find(|m|m.o_class==18)
        .map(|m|[m.position[0],m.position[1],m.position[2]]))?;
    let mut found=Vec::new();
    for (id,m) in table.mobys.iter().enumerate() {
        if m.update_fn!=Some(crate::moby_update::classes::path_platform::UPDATE_FN)
            || m.pvars.len()<0xd0 || m.state>=0x80 {continue;}
        let Some(path)=usize::try_from(p::i32(&m.pvars,0xb4)).ok().and_then(|s|splines.get(s)) else{continue;};
        let (Some(first),Some(last))=(path.first(),path.last()) else{continue;};
        let (first,last)=(first.map(f32::from_bits),last.map(f32::from_bits));
        if !first[..3].iter().chain(&last[..3]).chain(&boots[..3]).all(|v|v.is_finite())
            || (first[2]-last[2]).abs()<3. {continue;}
        let lower_first=first[2]<last[2];let upper=if lower_first{last}else{first};
        let distance=(0..3).map(|i|(upper[i]-boots[i]).powi(2)).sum::<f32>();
        if distance<=64.*64. {found.push((distance,id,lower_first));}
    }
    found.sort_by(|a,b|a.0.total_cmp(&b.0));
    let &(distance,id,first)=found.first()?;
    // Ambiguous or malformed data must never unlock another platform.
    if found.get(1).is_some_and(|other|(other.0-distance).abs()<1.){return None;}
    Some((id,first))
}

fn eligible(w:&World)->bool {
    let state=w.svc.bomb_evo.progress().randomizer;
    let mask=w.hero.owned.has(6);
    w.svc.level==10 && w.svc.bomb_evo.randomizer_mode && state.active() && mask
        && !super::randomizer::travel_mask(state,10,mask)
}

/// Called before building the scheduler run list, including the load pass.
/// The shortcut may be hidden, frozen or outside the camera's update distance.
pub fn prepare(w:&mut World) {
    if !eligible(w) || w.svc.randomizer.orxon_shortcut_lowered {return;}
    let pedestal=w.svc.randomizer.anchors.iter().find(|a|a.source==28).map(|a|a.position);
    let Some((id,lower_first))=candidate(w.table,&w.svc.splines,pedestal) else{return;};
    let aboard=w.svc.game_mode==0 && w.body()==crate::hero::bodies::body::CLANK
        && w.hero_on_moby(id);
    let m=w.mm(id);
    p::set_i32(&mut m.pvars,0xc0,-1);
    p::set_i32(&mut m.pvars,0xcc,-1);
    // Only this selected carrier becomes active. Do not change global puzzle,
    // door, mission or save latches. An existing rider must never be snapped.
    if !aboard {
        p::set_u8(&mut m.pvars,0xa0,u8::from(lower_first));
        m.state=0;
    }
    m.mode&=!0x43;
    m.has_collision=true;
    m.update_dist=0xff;
    w.svc.randomizer.orxon_shortcut_lowered=true;
    w.svc.randomizer.orxon_shortcut_waiting=(!aboard).then_some(id);
}

/// Prevent the native distance-based call from raising the newly opened lift
/// during the landing scene or while Clank is still elsewhere in the level.
/// Once boarded, all movement, carrying, collision and sound are native again.
pub fn hold_initial_lower(w:&mut World,id:MobyId)->bool {
    if w.svc.randomizer.orxon_shortcut_waiting!=Some(id) {return false;}
    if !eligible(w) {
        w.svc.randomizer.orxon_shortcut_waiting=None;
        return false;
    }
    if w.svc.game_mode==0 && w.body()==crate::hero::bodies::body::CLANK
        && w.hero_on_moby(id) {
        w.svc.randomizer.orxon_shortcut_waiting=None;
        // Native departure requires its rider latch to have been rearmed;
        // Clank may board without previously moving more than two units away.
        p::set_i16(&mut w.mm(id).pvars,0xb8,1);
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_runtime::{Moby,ClassInfo};
    #[test]
    fn recognizes_only_the_lift_ending_at_boots_and_rejects_ties() {
        let mut boots=Moby::init_instance(0,18,None);boots.position=[10.,10.,30.,0.];
        let info=ClassInfo{update_fn:Some(crate::moby_update::classes::path_platform::UPDATE_FN),..Default::default()};
        let mut lift=Moby::init_instance(1,1141,Some(&info));lift.pvars=vec![0;0xd0];p::set_i32(&mut lift.pvars,0xb4,0);
        let mut other=lift.clone();p::set_i32(&mut other.pvars,0xb4,1);
        let paths=vec![vec![[10.,10.,0.,0.].map(f32::to_bits),[10.,10.,30.,0.].map(f32::to_bits)],
            vec![[100.,100.,0.,0.].map(f32::to_bits),[100.,100.,30.,0.].map(f32::to_bits)]];
        let mut table=MobyTable::new(vec![boots,lift.clone(),other],0);
        assert_eq!(candidate(&table,&paths,None),Some((1,true)));
        // A nearby actor with the Novalis class ID but a different function
        // must not steal the Orxon match.
        let mut fake=lift.clone();fake.o_class=726;fake.update_fn=None;
        table.mobys.push(fake);assert_eq!(candidate(&table,&paths,None),Some((1,true)));
        // The immutable source still locates the lift after the story actor
        // has been collected and its slot reused for another class.
        table.mobys[0].o_class=0;
        assert_eq!(candidate(&table,&paths,None),None);
        assert_eq!(candidate(&table,&paths,Some([10.,10.,30.])),Some((1,true)));
        table.mobys.push(lift);assert_eq!(candidate(&table,&paths,Some([10.,10.,30.])),None);
    }
    struct Bench {
        table:MobyTable, svc:crate::moby_update::services::Services,
        classes:crate::moby_update::ClassTable, hero:crate::hero::Hero, rng:crate::rng::Rng,
    }
    impl Bench {
        fn new()->Self {
            let info=ClassInfo{update_fn:Some(crate::moby_update::classes::path_platform::UPDATE_FN),
                has_collision:true,scale:1.,..Default::default()};
            let mut classes=crate::moby_update::ClassTable::default();classes.classes.insert(1141,(info,None));
            let mut lift=Moby::init_instance(1,1141,Some(&info));lift.mode|=0x63;lift.update_dist=0;
            lift.position=[10.,10.,30.,0.];lift.pvars=vec![0;0xd0];
            for (off,value) in [(0,0x20),(8,0x60),(0xb4,0),(0xc0,7),(0xcc,8)] {
                p::set_i32(&mut lift.pvars,off,value);
            }
            p::set_u8(&mut lift.pvars,0xa0,1);p::set_ff(&mut lift.pvars,0xb0,1.);
            let mut boots=Moby::init_instance(2,18,None);boots.position=[10.,10.,30.,0.];
            let table=MobyTable::new(vec![Moby::init_instance(0,0,None),lift,boots],0);
            let mut svc=crate::moby_update::services::Services::new();svc.level=10;
            svc.set_splines(&[vec![[10.,10.,30.,0.],[10.,10.,10.,0.]]]);
            svc.bomb_evo.randomizer_mode=true;
            let mut progress=svc.bomb_evo.progress();progress.randomizer=super::super::randomizer::State::generate(723);
            if progress.randomizer.reward(28).is_none() {
                let source=progress.randomizer.placements.iter().enumerate().find(|(i,v)|*i!=65&&**v!=0).unwrap().0;
                progress.randomizer.placements.swap(28,source);
            }
            svc.bomb_evo.restore_progress(&progress);
            let mut hero=crate::hero::Hero::new();hero.mode=crate::hero::bodies::body::CLANK;
            hero.state=0x43;hero.health=4;hero.owned.0[6]=1;
            // Standing near the upper endpoint would normally call the lift up.
            hero.pos=crate::hero::physics::v4(10.,10.,30.);
            Self{table,svc,classes,hero,rng:crate::rng::Rng::new()}
        }
        fn step(&mut self,counter:u64,load:bool) {
            let mut w=World::new(&mut self.table,&self.hero,&mut self.rng,&self.classes,&mut self.svc,counter);
            let mut scheduler=crate::moby_update::scheduler::Scheduler::new();
            if load {scheduler.load_pass(&mut w);} else {scheduler.tick(&mut w);}
        }
    }
    #[test]
    fn hidden_orxon_alias_initializes_below_and_waits_for_clank_to_board() {
        let mut b=Bench::new();let before=b.svc.bomb_evo.progress();
        b.step(0,true);
        assert_eq!(b.table.mobys[1].position,[10.,10.,10.,0.]);
        assert_eq!(b.table.mobys[1].mode&3,0);assert_eq!(b.table.mobys[1].update_dist,0xff);
        assert_eq!(b.svc.randomizer.orxon_shortcut_waiting,Some(1));
        for tick in 1..=10 {b.step(tick,false);}
        assert_eq!(b.table.mobys[1].position,[10.,10.,10.,0.]);assert_eq!(b.table.mobys[1].cmd,0);
        // Boarding works even though the XY distance has never rearmed it.
        b.hero.pos=crate::hero::physics::v4(10.,10.,10.);b.hero.ground_moby=Some(1);b.hero.air_ticks=0;
        b.step(11,false);
        assert_eq!(b.svc.randomizer.orxon_shortcut_waiting,None);assert_eq!(b.table.mobys[1].cmd,2);
        b.step(12,false);assert!(b.table.mobys[1].position[2]>10.);
        assert_eq!(b.svc.bomb_evo.progress(),before);assert!(b.svc.save.death.is_empty());
        assert_eq!(b.hero.mode,crate::hero::bodies::body::CLANK);
    }
    #[test]
    fn scope_requires_all_randomizer_real_mask_pending_clank_checks_and_orxon() {
        for case in 0..6 {
            let mut b=Bench::new();
            match case {
                0=>b.hero.owned.0[6]=0,
                1=>b.svc.bomb_evo.randomizer_mode=false,
                2=>{b.svc.bomb_evo.randomizer_mode=false;b.svc.bomb_evo.golden_settings.enabled=true;},
                3=>b.svc.level=1,
                4=>{let mut p=b.svc.bomb_evo.progress();for id in [28,75] {p.randomizer.collect(id);}b.svc.bomb_evo.restore_progress(&p);},
                _=>b.svc.bomb_evo.restore_progress(&super::super::save::Progress::default()),
            }
            let original=b.table.mobys[1].clone();b.step(0,true);
            assert_eq!(b.table.mobys[1].position,original.position,"case {case}");
            assert_eq!(b.table.mobys[1].pvars,original.pvars,"case {case}");
            assert_eq!(b.table.mobys[1].mode,original.mode,"case {case}");
            assert!(!b.svc.randomizer.orxon_shortcut_lowered);assert_eq!(b.svc.randomizer.orxon_shortcut_waiting,None);
        }
    }
    #[test]
    fn existing_rider_is_not_snapped_and_death_reload_resets_only_transient_wait() {
        let mut b=Bench::new();b.table.mobys[1].mode&=!0x43;b.table.mobys[1].state=1;b.table.mobys[1].cmd=1;
        p::set_ff(&mut b.table.mobys[1].pvars,0xa4,0.);b.hero.ground_moby=Some(1);b.hero.air_ticks=0;
        let pos=b.table.mobys[1].position;b.step(0,false);
        assert_eq!(b.table.mobys[1].position,pos);assert_eq!(b.svc.randomizer.orxon_shortcut_waiting,None);
        let before=b.svc.bomb_evo.progress();b.svc.randomizer.orxon_shortcut_waiting=Some(1);
        b.svc.randomizer.reset_for_reload();assert!(!b.svc.randomizer.orxon_shortcut_lowered);
        assert_eq!(b.svc.randomizer.orxon_shortcut_waiting,None);assert_eq!(b.svc.bomb_evo.progress(),before);
    }

}
