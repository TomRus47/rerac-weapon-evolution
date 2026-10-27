//! Save-owned reward layout. Stable slot IDs are independent of actor order.
use super::ITEMS;

pub const COUNTS: [u8; 19] = [0,3,4,3,1,2,2,2,2,2,2,1,2,1,4,2,2,2,3];
pub const SLOTS: usize = 96;
pub const CUSTOM_START: usize = 80;
pub const CUSTOM_LEVELS: [u32; 8] = [13,3,10,11,12,1,4,14];
pub const LAYOUT_LEGACY: u32 = 1;
pub const LAYOUT_PREVIOUS: u32 = 2;
pub const LAYOUT_CURRENT: u32 = 4;
// Most restrictive native access on a planet is deliberately conservative.
// Additional sites have their own access ranks; none is a required selection.
const ACCESS: [u8; 19] = [0,12,10,3,4,11,11,7,8,9,11,11,12,13,14,17,16,17,18];
// The Novalis upstream swim is conservatively ranked after the Hydro-Pack.
const CUSTOM_ACCESS: [u8;8] = [13,3,11,12,12,12,4,14];
// RYNO is excluded from this guarantee: its shotgun experiment is not an
// explosive wall breaker. These three remain explosive in every current mode.
const EXPLOSIVES: [usize;3] = [10,11,13];
// Gemlik progression specifically needs one of these; Bomb Glove/RYNO do not
// satisfy that gate. Older v3 saves reserved an early site for one.
const GEMLIK_TOOLS: [usize;2] = [11,13];
const OLTANIS_PAIR: [usize;2] = [14*4+2,14*4+3];

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct State {
    pub seed: u64,
    pub early_bias_percent: u32,
    pub claimed: u32,
    /// Native [planet*4+local bolt index], then optional sites at 80..88.
    /// Zero means no reward: the native Gold Bolt is removed in this mode.
    pub placements: [u8; SLOTS],
    pub layout_version: u32,
    /// A submerged discovery unlocks now, but equips only after leaving water.
    pub pending_equip: u32,
}
impl Default for State {
    fn default()->Self {
        Self {seed:0,early_bias_percent:0,claimed:0,placements:[0;SLOTS],layout_version:0,pending_equip:0}
    }
}
pub(super) fn locations(legacy:bool)->Vec<usize> {
    let mut out:Vec<_>=(1..COUNTS.len()).flat_map(|level|
        (0..COUNTS[level] as usize).map(move |i|level*4+i)).collect();
    if !legacy {out.extend(CUSTOM_START..CUSTOM_START+CUSTOM_LEVELS.len());}
    out
}
pub fn level(slot:usize)->Option<u32> {
    if slot<CUSTOM_START {
        let planet=slot/4;
        return COUNTS.get(planet).filter(|&&n|slot%4<usize::from(n)).map(|_|planet as u32);
    }
    CUSTOM_LEVELS.get(slot-CUSTOM_START).copied()
}
pub(super) fn access(slot:usize)->u8 {
    if slot<CUSTOM_START {ACCESS[slot/4]} else {CUSTOM_ACCESS[slot-CUSTOM_START]}
}
fn safe_early(slot:usize)->bool {matches!(level(slot),Some(3|4))}
fn next(seed:&mut u64)->u64 {
    *seed=seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut z=*seed;
    z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);
    z^(z>>31)
}
pub(super) fn placement_allowed(slot:usize,item:usize)->bool {
    // Orxon's native index 0 is the Visibomb membrane route. Its other bolt
    // and the new grate site do not require Visibombs. The old Gemlik bolt is
    // conservatively excluded; the tower room is a separate, independent site.
    !(item==13 && matches!(slot,40|52)) && !(item==14 && slot/4==18 && slot<CUSTOM_START)
}
fn legacy_allowed(slot:usize,item:usize)->bool {
    !(item==13 && matches!(slot/4,10|13)) && !(item==14 && slot/4==18)
}
impl State {
    pub fn active(&self)->bool {self.seed!=0}
    pub fn item_at(&self,slot:usize)->Option<usize> {
        if !self.active(){return None;}
        self.placements.get(slot).map(|&i|i as usize).filter(|i|ITEMS.contains(i))
    }
    pub fn item(&self,level:u32,index:i32)->Option<usize> {
        if level>=19 || !(0..4).contains(&index){return None;}
        self.item_at(level as usize*4+index as usize)
    }
    pub fn collected(&self,item:usize)->bool {
        ITEMS.iter().position(|&i|i==item).is_some_and(|i|self.claimed&(1<<i)!=0)
    }
    pub fn collect(&mut self,item:usize) {
        if let Some(i)=ITEMS.iter().position(|&i|i==item){self.claimed|=1<<i;}
    }
    pub fn valid(&self)->bool {
        if !self.active(){return *self==Self::default();}
        let legacy=self.layout_version==LAYOUT_LEGACY;
        if !legacy && ![LAYOUT_PREVIOUS,3,LAYOUT_CURRENT].contains(&self.layout_version){return false;}
        if self.early_bias_percent>100 || self.claimed&!0x7fff!=0{return false;}
        if self.pending_equip!=0 && !self.collected(self.pending_equip as usize){return false;}
        let slots=locations(legacy);
        let mut seen=0u32;
        for (slot,&item) in self.placements.iter().enumerate().filter(|(_,i)|**i!=0) {
            if !slots.contains(&slot) || (self.layout_version==LAYOUT_PREVIOUS && slot>=86){return false;}
            let Some(i)=ITEMS.iter().position(|&i|i==usize::from(item)) else{return false;};
            let allowed=if legacy {legacy_allowed(slot,item as usize)}else{placement_allowed(slot,item as usize)};
            if seen&(1<<i)!=0 || !allowed{return false;}
            seen|=1<<i;
        }
        if seen!=0x7fff{return false;}
        if self.layout_version==LAYOUT_CURRENT {
            return !(self.placements[OLTANIS_PAIR[0]]!=0 && self.placements[OLTANIS_PAIR[1]]!=0)
                && !self.placements.iter().enumerate().any(|(slot,&item)|
                    matches!(slot,6|8|40) && matches!(item,10|11|13|23));
        }
        if legacy {
            self.placements.iter().enumerate().any(|(s,&i)|i==10&&safe_early(s))
        } else {
            !(self.placements[OLTANIS_PAIR[0]]!=0 && self.placements[OLTANIS_PAIR[1]]!=0)
                && self.placements.iter().enumerate().any(|(s,&i)|safe_early(s)&&EXPLOSIVES.contains(&(i as usize)))
                && (self.layout_version!=3 || self.placements.iter().enumerate()
                    .any(|(s,&i)|safe_early(s)&&GEMLIK_TOOLS.contains(&(i as usize))))
        }
    }
    /// Weapons-only progression follows native gadget/coordinate rewards, while
    /// weapon ownership comes only from the saved randomized world placements.
    /// This checks access without reserving an early Devastator/Visibomb slot.
    pub(super) fn progression_safe(&self)->bool {
        let catalog=super::super::randomizer::catalog::locations();
        let accessible=|source:usize,have:&[bool;192]| {
            catalog.iter().find(|l|l.id==source).is_some_and(|l|
                have[64+l.level as usize] && l.requires.iter().any(|all|all.iter().all(|r|have[*r as usize])))
        };
        let mut have=[false;192];have[64]=true;have[65]=true;
        loop {
            let before=have;
            for l in &catalog {
                if l.id>=96 || ITEMS.contains(&l.id) {continue;}
                if accessible(l.id,&before) {have[l.id]=true;}
            }
            for (slot,&weapon) in self.placements.iter().enumerate().filter(|(_,w)|**w!=0) {
                let source=match slot {0..=75=>96+slot,80..=85=>176+slot-80,86=>9,87=>21,_=>return false};
                if accessible(source,&before){have[weapon as usize]=true;}
            }
            if !before[77] && have[77] && !have[11] && !have[13] {return false;}
            if before==have {return ITEMS.iter().all(|i|have[*i]);}
        }
    }

    pub fn generate(seed:u64,bias:u8)->Self {
        Self::generate_with_planet_limit(seed,bias,false)
    }
    /// Only new games use this policy. The serialized placements remain the
    /// authority on reload, including older saves with several rewards per planet.
    pub fn generate_with_planet_limit(seed:u64,bias:u8,one_per_planet:bool)->Self {
        let seed=seed.max(1);
        let mut rng=seed;
        let b=f64::from(bias.min(100))/100.;
        let mut ranked:Vec<_>=locations(false).into_iter().map(|slot| {
            let random=(next(&mut rng)>>11) as f64/((1u64<<53) as f64);
            (slot,(1.-b)*random+b*f64::from(access(slot))/19.,next(&mut rng))
        }).collect();
        ranked.sort_by(|a,b|a.1.total_cmp(&b.1).then(a.2.cmp(&b.2)));
        let mut slots=Vec::new();
        for &(slot,_,_) in &ranked {
            if one_per_planet && slots.iter().any(|&other|level(other)==level(slot)){continue;}
            if OLTANIS_PAIR.contains(&slot) && slots.iter().any(|s|OLTANIS_PAIR.contains(s)){continue;}
            slots.push(slot);
            if slots.len()==ITEMS.len(){break;}
        }
        // Keep at least one independently accessible site, without assigning
        // any particular weapon category to it.
        if !slots.iter().any(|&slot|safe_early(slot)) {
            let safe=ranked.iter().find(|entry|safe_early(entry.0)).unwrap().0;
            let at=slots.iter().position(|&slot|one_per_planet && level(slot)==level(safe)).unwrap_or(slots.len()-1);
            slots[at]=safe;
        }
        // Keep the selected locations and draw the entire weapon permutation.
        // Reject only access violations; no special initial weapon weight.
        for _ in 0..8192 {
            let mut weapons=ITEMS;
            for i in (1..weapons.len()).rev(){let j=(next(&mut rng)%(i+1) as u64) as usize;weapons.swap(i,j);}
            let mut out=Self{seed,early_bias_percent:u32::from(bias.min(100)),layout_version:LAYOUT_CURRENT,..Default::default()};
            for (&slot,&item) in slots.iter().zip(weapons.iter()){out.placements[slot]=item as u8;}
            if out.valid() && out.progression_safe(){return out;}
        }
        panic!("weapons randomizer: no reachable layout for seed {seed}")
    }
}
pub fn fresh_seed()->u64 {
    use std::sync::atomic::{AtomicU64,Ordering};
    static SEQUENCE:AtomicU64=AtomicU64::new(1);
    let time=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1,|d|d.as_nanos() as u64);
    let mut seed=time^(std::process::id() as u64).rotate_left(17)^SEQUENCE.fetch_add(1,Ordering::Relaxed).rotate_left(37);
    next(&mut seed).max(1)
}


#[cfg(test)]
mod v80_tests {
    use super::*;
    #[test]
    fn all_new_layouts_have_a_gemlik_tool_before_gemlik_including_one_per_planet() {
        let mut sites=[false;2];
        let mut strong_not_reserved=false;
        for limited in [false,true] {for bias in [0,50,100] {for seed in 1..=1000 {
            let s=State::generate_with_planet_limit(seed,bias,limited);
            assert!(s.valid());assert_eq!(s.layout_version,4);
            assert!(s.progression_safe());
            strong_not_reserved |= !s.placements.iter().enumerate().any(|(slot,&item)|safe_early(slot)&&GEMLIK_TOOLS.contains(&(item as usize)));
            for (i,slot) in [86,87].into_iter().enumerate(){sites[i]|=s.item_at(slot).is_some();}
            if limited {
                let levels:Vec<_>=s.placements.iter().enumerate().filter(|(_,i)|**i!=0)
                    .map(|(slot,_)|level(slot).unwrap()).collect();
                assert_eq!(levels.iter().copied().collect::<std::collections::HashSet<_>>().len(),15);
            }
        }}}
        assert!(sites.into_iter().all(|seen|seen));
        assert!(strong_not_reserved,"strong weapons are not forced into initial sites");
    }
    #[test]
    fn previous_layouts_load_unchanged_without_new_guarantees_or_rerolls() {
        let mut old=(1..1000).map(|seed|State::generate(seed,0))
            .find(|s|s.placements[86]==0 && s.placements[87]==0 && s.placements.iter().enumerate().any(|(slot,item)|safe_early(slot)&&*item==11)).unwrap();
        old.layout_version=LAYOUT_PREVIOUS;
        let early=old.placements.iter().enumerate().find(|(slot,item)|
            safe_early(*slot)&&**item==11).unwrap().0;
        let bomb=old.placements.iter().position(|&item|item==10).unwrap();
        old.placements.swap(early,bomb);
        assert!(old.valid(),"v2 keeps its original Bomb-Glove early guarantee");
        let saved=crate::weapon_evo::save::Progress{golden:old,..Default::default()};
        let loaded=crate::weapon_evo::save::Progress::decode(&saved.encode(b"native fixture").unwrap(),b"native fixture").unwrap();
        assert_eq!(loaded.golden,old);
    }
}
