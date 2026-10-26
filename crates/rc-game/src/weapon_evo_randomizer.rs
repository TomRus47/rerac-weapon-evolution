//! Mixed-pool randomizer. Source completion and actual inventory are distinct.
//! Layout IDs are stable, persisted, validated and never derived from live RNG.
use super::{Evolution,save::Progress};
use crate::game_state::{GameState,SessionState};
use crate::moby_update::{interact::GameWrite,services::World};
use rc_formats::save_game::ItemTables;
#[path="weapon_evo_randomizer/catalog.rs"]
pub mod catalog;
pub const SLOTS:usize=192;
pub const ENCODED_SIZE:usize=8+4+SLOTS*2+24+4;
pub const RENDER_BASE:usize=1000;
pub const KEY:&str="randomizer_mode";
#[derive(Clone,Copy,Debug)]
pub struct Location {
    pub id:usize,pub level:u32,pub name:&'static str,
    pub requires:&'static [&'static [u16]],
}
impl Location {
    fn accessible(self,have:&[bool;SLOTS])->bool {
        have[64+self.level as usize] && self.requires.iter().any(|all|all.iter().all(|&r|have[r as usize]))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct State {pub seed:u64,pub version:u32,pub placements:[u16;SLOTS],pub claimed:[u64;3],pub pending_equip:u32}
impl Default for State {fn default()->Self{Self{seed:0,version:0,placements:[0;SLOTS],claimed:[0;3],pending_equip:0}}}
fn next(s:&mut u64)->u64 {
    *s=s.wrapping_add(0x9e3779b97f4a7c15);let mut z=*s;
    z=(z^(z>>30)).wrapping_mul(0xbf58476d1ce4e5b9);z=(z^(z>>27)).wrapping_mul(0x94d049bb133111eb);z^(z>>31)
}
fn pool()->Vec<u16>{let mut ids:Vec<_>=catalog::locations().into_iter().map(|l|l.id as u16).filter(|&i|i<176).collect();ids.push(10);ids}
fn is_new_planet(reward:u16)->bool{(66..=82).contains(&reward)}

/// Keep coordinate concentration separate from the reward draw. Other reward
/// categories remain in one pool and retain the same access requirements.
fn coordinate_fits(state:&State,at:&Location,catalog:&[Location],initial:&[usize])->bool {
    let on_planet=catalog.iter().filter(|l|l.level==at.level&&is_new_planet(state.placements[l.id])).count();
    if on_planet>=2{return false;}
    !initial.contains(&at.id)||!initial.iter().any(|&id|is_new_planet(state.placements[id]))
}
impl State {
    pub fn active(self)->bool{self.seed!=0}
    pub fn collected(self,id:usize)->bool{id<SLOTS && self.claimed[id/64]&(1<<(id%64))!=0}
    pub fn collect(&mut self,id:usize){if id<SLOTS{self.claimed[id/64]|=1<<(id%64);}}
    pub fn reward(self,id:usize)->Option<u16>{self.placements.get(id).copied().filter(|&i|i!=0)}
    pub fn owns(self,reward:u16)->bool{self.placements.iter().enumerate().any(|(s,&r)|r==reward&&self.collected(s))}
    pub fn generate(seed:u64)->Self {
        let seed=seed.max(1);let catalog=catalog::locations();let mut rng=seed;
        // Start directly on Novalis; source 65 is its initial coordinate.
        let start=1usize;
        // Story, shop and coordinate sources stay active: their native scripts
        // also control mission state. World-only Gold Bolt checks and all six
        // surveyed additions share one candidate pool. Five surplus sites stay
        // empty, selected afresh on each deterministic generation attempt.
        let mandatory=catalog.iter().filter(|l|l.id<96).count();
        let optional_count=pool().len()-mandatory;
        let mut starting_inventory=[false;SLOTS];starting_inventory[64]=true;starting_inventory[65]=true;
        // Sample the COMPLETE pool, including Gold Bolts. Always maximizing the
        // number of checks a reward opens forces high-fanout planet coordinates
        // into early checks and makes different seeds look like the same route.
        // Only constrain a choice when spending the last reachable check would
        // strand the run. Retry a stranded partial layout, never its saved seed.
        // Reserve Veldin II until the other progression can be reached.
        // Pacing rejection is deterministic and never substitutes another seed.
        // The larger bounded retry budget covers the extra layout constraints.
        for attempt in 0..8192 {
            let mut optional:Vec<_>=catalog.iter().filter(|l|l.id>=96).map(|l|l.id).collect();
            for i in (1..optional.len()).rev(){let j=next(&mut rng) as usize%(i+1);optional.swap(i,j);}
            let mut active=[false;SLOTS];
            for l in &catalog{if l.id<96{active[l.id]=true;}}
            for &id in optional.iter().take(optional_count){active[id]=true;}
            let eligible=|id:usize|active[id];
            let initial:Vec<_>=catalog.iter().filter(|l|eligible(l.id)&&l.id!=65&&l.accessible(&starting_inventory)).map(|l|l.id).collect();
            let mut state=Self{seed,version:6,..Default::default()};
            state.placements[65]=(64+start) as u16;
            let mut have=[false;SLOTS];have[64]=true;have[64+start]=true;
            let mut remaining:Vec<_>=pool().into_iter().filter(|&r|r!=82&&r!=64+start as u16).collect();
            let mut ranks=[0u16;SLOTS];let mut step=1;
            for l in &catalog {if eligible(l.id)&&l.accessible(&have){ranks[l.id]=step;}}
            while remaining.iter().any(|&r|r<96) {
                let available:Vec<_>=catalog.iter().filter(|l|eligible(l.id)&&state.placements[l.id]==0&&l.accessible(&have)).collect();
                if available.is_empty(){break;}
                let coordinate_slots:Vec<_>=available.iter().copied().filter(|l|coordinate_fits(&state,l,&catalog,&initial)).collect();
                let mut options:Vec<_>=remaining.iter().copied().filter(|&r| {
                    if is_new_planet(r)&&coordinate_slots.is_empty(){return false;}
                    if available.len()>2||remaining.iter().filter(|&&i|i<96).count()<=available.len(){return true;}
                    // With two checks left, do not spend one on filler or an
                    // unrelated item. Either open a route now or contribute to
                    // a two-item gate on an already discovered planet. The last
                    // check keeps the original immediate-escape requirement.
                    if available.len()==2 {
                        have[r as usize]=true;
                        let opens=catalog.iter().any(|l|eligible(l.id)&&state.placements[l.id]==0
                            &&!available.iter().any(|a|a.id==l.id)&&l.accessible(&have));
                        have[r as usize]=false;
                        if opens{return true;}
                        return catalog.iter().any(|l|eligible(l.id)&&state.placements[l.id]==0
                            &&have[64+l.level as usize]&&l.requires.iter().any(|all| {
                                let missing:Vec<_>=all.iter().filter(|&&i|!have[i as usize]).copied().collect();
                                missing.len()==2&&missing.contains(&r)&&missing.iter().all(|i|remaining.contains(i))
                            }));
                    }
                    have[r as usize]=true;
                    let opens=catalog.iter().any(|l|eligible(l.id)&&l.id!=available[0].id
                        &&state.placements[l.id]==0&&l.accessible(&have));
                    have[r as usize]=false;opens
                }).collect();
                if options.is_empty(){break;}
                if attempt>=128&&available.len()<=3 {
                    let scored:Vec<_>=options.iter().map(|&r| {
                        have[r as usize]=true;
                        let opens=catalog.iter().filter(|l|eligible(l.id)&&state.placements[l.id]==0
                            &&!available.iter().any(|a|a.id==l.id)&&l.accessible(&have)).count();
                        have[r as usize]=false;
                        (r,opens)
                    }).collect();
                    let best=scored.iter().map(|x|x.1).max().unwrap_or(0);
                    if best>0{options.retain(|r|scored.iter().any(|&(v,n)|v==*r&&n==best));}
                }
                let reward=options[next(&mut rng) as usize%options.len()];
                let destinations=if is_new_planet(reward){&coordinate_slots}else{&available};
                let at=destinations[next(&mut rng) as usize%destinations.len()].id;
                state.placements[at]=reward;have[reward as usize]=true;ranks[at]=step;step+=1;
                // Record when a check first becomes reachable. Final coordinates
                // go behind the latest dependency, not merely a late planet ID.
                for l in &catalog {if ranks[l.id]==0&&l.accessible(&have){ranks[l.id]=step;}}
                remaining.retain(|&r|r!=reward);
            }
            if remaining.iter().any(|&r|r<96){continue;}
            // A previously placed Gold Bolt can also host the final coordinate:
            // do not prefer an early empty check over a late filler location.
            // Relocating only filler cannot break another item's dependencies.
            let candidates:Vec<_>=catalog.iter().filter(|l|eligible(l.id)
                &&(state.placements[l.id]==0||state.placements[l.id]>=96)&&l.accessible(&have)
                &&coordinate_fits(&state,l,&catalog,&initial)).collect();
            let Some(rank)=candidates.iter().map(|l|ranks[l.id]).max() else{continue;};
            let latest:Vec<_>=candidates.iter().filter(|l|ranks[l.id]==rank).collect();
            let end=latest[next(&mut rng) as usize%latest.len()].id;
            let displaced=state.placements[end];
            if displaced!=0{remaining.push(displaced);}
            state.placements[end]=82;
            let mut free:Vec<_>=catalog.iter().filter(|l|eligible(l.id)&&state.placements[l.id]==0).map(|l|l.id).collect();
            if free.len()!=remaining.len(){continue;}
            for i in (1..free.len()).rev(){let j=next(&mut rng) as usize%(i+1);free.swap(i,j);}
            for r in remaining{state.placements[free.pop().unwrap()]=r;}
            if state.solvable()&&state.planet_discovery_balanced(){return state;}
        }
        // This is a programming/data error: never silently write a softlocked
        // layout or substitute another seed into the player's save.
        panic!("randomizer: no valid layout for seed {seed}");
    }
    pub fn solvable(self)->bool {
        let mut have=[false;SLOTS];have[64]=true;
        loop {
            let old=have;for l in catalog::locations(){if l.accessible(&old){if let Some(r)=self.reward(l.id){have[r as usize]=true;}}}
            if have==old{break;}
        }
        pool().iter().all(|&r|have[r as usize])
    }
    /// Validate pacing against actual reachability spheres, not the arbitrary
    /// order used while filling the seed. The player may collect accessible
    /// checks in any order; this bounds what a new set of unlocked routes offers,
    /// rather than promising a particular personal pickup sequence.
    pub fn planet_discovery_balanced(self)->bool {
        let catalog=catalog::locations();
        let mut per_planet=[0usize;19];
        for l in &catalog {if is_new_planet(self.placements[l.id]){
            per_planet[l.level as usize]+=1;
            if per_planet[l.level as usize]>2{return false;}
        }}
        let mut have=[false;SLOTS];have[64]=true;
        let mut claimed=[false;SLOTS];let(mut coordinates,mut other_rewards)=(0usize,0usize);
        loop {
            let before=have;let(mut checks,mut new_coordinates)=(0usize,0usize);
            for l in &catalog {
                if claimed[l.id]||!l.accessible(&before){continue;}
                let Some(reward)=self.reward(l.id)else{continue;};
                claimed[l.id]=true;have[reward as usize]=true;
                // The guaranteed Novalis start is not a discovered planet.
                if l.id==65{continue;}
                checks+=1;
                if is_new_planet(reward){coordinates+=1;new_coordinates+=1;}else{other_rewards+=1;}
            }
            // One initial escape coordinate is allowed. Afterwards, new routes
            // grow alongside equipment/filler rather than outrunning them.
            if coordinates>1+other_rewards/3{return false;}
            if checks>0&&new_coordinates>checks.div_ceil(4){return false;}
            if have==before{break;}
        }
        true
    }
    pub fn valid(self)->bool {
        if !self.active(){return self==Self::default();}if !matches!(self.version,1..=6){return false;}
        if !matches!(self.placements[65],65..=81){return false;}
        if self.pending_equip!=0 && (!super::golden::ITEMS.contains(&(self.pending_equip as usize))||!self.owns(self.pending_equip as u16)){return false;}
        let catalog=catalog::locations();
        // Older saves populated all native Gold Bolt checks. Never regenerate
        // them on load; only new version-5 layouts allow empty native world sites.
        let required_below=if self.version>=5{96}else{176};
        if catalog.iter().any(|l|l.id<required_below&&self.placements[l.id]==0){return false;}
        let mut actual=Vec::new();
        for (id,&r) in self.placements.iter().enumerate(){
            if (r!=0||self.collected(id))&&!catalog.iter().any(|l|l.id==id){return false;}
            if r!=0{actual.push(r);}
            if self.collected(id)&&r==0{return false;}
        }
        let mut expected=pool();actual.sort();expected.sort();actual==expected
    }
}
pub fn enabled()->bool{super::inspect_profile().is_some_and(|e|e.randomizer_mode)}
pub fn initialize(gs:&mut GameState,on:bool,seed:u64){
    if !on{return;}gs.weapon_progress.randomizer=State::generate(seed);
    gs.weapon_progress.golden=Default::default();gs.weapon_progress.ammo_mode=Default::default();
    gs.global.owned.fill(0);gs.global.acquired.fill(0);gs.global.ammo.fill(0);
    gs.global.owned[8]=1;gs.global.acquired[8]=1;gs.global.equipped=[8,0,0,0,0,0,0];
    gs.global.quick_select=[0;8];gs.global.last_hand_item=8;
    for i in [0,1,2,4,5]{gs.global.flags[i]=0;}
    gs.global.level=1;gs.global.planet_unlocked.fill(0);gs.global.map_order.fill(0);
    gs.weapon_progress.randomizer.collect(65);gs.unlock_planet(1);
    gs.global.flags[8]=1;gs.levels[0].visited=2;
    // No mayor completion or ship flags are pre-granted.

}
pub fn new_game_destination(gs:&GameState)->i32{if gs.weapon_progress.randomizer.active(){1}else{0}}
pub fn card_root(root:std::path::PathBuf)->std::path::PathBuf{if enabled(){root.join("randomizer")}else{root}}
pub fn accepts(progress:&Progress,on:bool)->bool{progress.randomizer.active()==on}
pub fn item_source(item:usize)->Option<usize>{catalog::locations().iter().find(|l|l.id==item&&item<64).map(|l|l.id)}
pub fn special_for_flag(flag:usize)->Option<usize>{match flag{0=>Some(48),1=>Some(49),2=>Some(50),4=>Some(52),5=>Some(53),_=>None}}
pub fn source_owned(w:&World,item:usize,native:bool)->bool{
    let state=w.svc.bomb_evo.state_progress.randomizer;
    if state.active()&&item_source(item).is_some(){state.collected(item)}else{native}
}
pub fn source_planet(w:&World,p:usize,native:bool)->bool {
    let state=w.svc.bomb_evo.state_progress.randomizer;
    if state.active()&&(2..19).contains(&p){state.collected(64+p)}else{native}
}
pub fn queue(w:&mut World,source:usize)->bool {
    if !w.svc.bomb_evo.randomizer_mode{return false;}
    let state=w.svc.bomb_evo.state_progress.randomizer;
    if state.reward(source).is_none(){return false;}
    if !state.collected(source)&&!w.svc.randomizer.pending.contains(&source){
        w.svc.randomizer.pending.push(source);w.svc.interact.writes.push(GameWrite::RandomizerReward(source));
    }true
}
/// Low-level grant bypasses source randomization exactly once. Never feed a
/// randomized reward back into the source interceptor.
pub fn grant(gs:&mut GameState,session:&mut SessionState,items:&ItemTables,source:usize,equip:bool)->Option<u16>{
    let mut state=gs.weapon_progress.randomizer;if state.collected(source){return None;}
    let reward=state.reward(source)?;state.collect(source);gs.weapon_progress.randomizer=state;
    if reward<37 {
        gs.give_item_native(reward as usize,equip,items,session);
        let item=reward as usize;
        if equip && items.slot_type[item]==1{session.temp_feet=reward as i32;}
        if super::ammo_mode::AMMO_IDS.contains(&item){
            let native=u16::from_le_bytes(items.records[item].0[14..16].try_into().unwrap()) as i32;
            let max=super::effective_max_ammo(item,native).max(1);
            gs.global.ammo[item]=gs.global.ammo[item].max(1).min(max);
        }
        super::quick_select::insert_found(gs,item);
        if !equip&&super::golden::ITEMS.contains(&(reward as usize)){gs.weapon_progress.randomizer.pending_equip=u32::from(reward);}
    }
    else if reward<64 {
        if let Some(flag)=[0,1,2,4,5].into_iter().find(|&f|special_for_flag(f)==Some(reward as usize)){
            let before=super::world::native_nanotech(&gs.global.flags);
            gs.global.flags[flag]=1;
            if flag==4||flag==5{gs.global.max_hp+=super::world::native_nanotech(&gs.global.flags)-before;session.hp=gs.global.max_hp;}
        }
    }else if reward<96 {gs.unlock_planet(reward as usize-64);}
    else {let s=reward as usize-96;gs.levels[s/4].gold_bolts[s%4]=1;}
    Some(reward)
}
impl Evolution {
    pub fn commit_randomizer(&mut self,state:State){if state.valid()&&state.seed==self.state_progress.randomizer.seed{self.state_progress.randomizer=state;}}
}
pub fn source_flag(w:&World,item:usize,native:u8)->u8{
    if w.svc.bomb_evo.randomizer_mode{u8::from(w.svc.bomb_evo.state_progress.randomizer.collected(item))}else{native}
}
/// One presentation policy for world rewards and Gadgetron merchandise.
pub fn display_class(reward:u16,native:i32)->i32{
    match reward {24=>479,65..=82=>750,96.. =>1134,49=>287,50=>1428,48|52|53=>1134,
        _ if native>0=>native,_=>1134}
}
pub fn acquisition_movie(item:usize)->Option<i32>{match item{31=>Some(0),27=>Some(1),5=>Some(2),_=>None}}
pub fn talk_source(level:u32,item:usize)->Option<usize>{
    if catalog::locations().iter().any(|l|l.id==item&&l.level==level){return Some(item);}
    match (level,item){(1,37)=>Some(66),(5,38)=>Some(71),(8,39)=>Some(73),(14,42)=>Some(79),_=>None}
}
pub fn departure(state:State,from:u32,intended:i32,unlocked:&[u8])->i32{
    if !state.active()||intended<0{return intended;}
    let source=match (from,intended){(0,1)=>65,(7,8)=>72,_=>return intended};
    if let Some(reward)=state.reward(source).filter(|r|(65..=82).contains(r)){return reward as i32-64;}
    unlocked.iter().enumerate().skip(1).take(18).find(|(p,u)|**u!=0&&*p!=from as usize)
        .map(|(p,_)|p as i32).unwrap_or(state.placements[65] as i32-64)
}
pub fn ship_available(state:State,level:u32)->bool{level!=1||state.collected(67)}
/// Keep Orxon's native Clank landing and section controller available until
/// every active Clank-side check is complete, even with an early real O2 Mask.
/// This preserves the doors and gadgetbot puzzles instead of deleting actors.
pub fn travel_mask(state:State,level:u32,actual:bool)->bool{
    if state.active()&&level==10&&[28,75].iter().any(|&s|state.reward(s).is_some()&&!state.collected(s)){false}else{actual}
}
#[derive(Clone,Debug)]pub struct Anchor{pub source:usize,pub position:[f32;3],pub mission:Option<i32>,pub infobot:bool,pub shown:bool}
#[derive(Clone,Debug,Default)]pub struct WorldState{
    pub anchors:Vec<Anchor>,pub pending:Vec<usize>,pub ready:std::collections::HashSet<usize>,pub elapsed:u64,
}
impl WorldState {
    pub fn reset_for_reload(&mut self){self.pending.clear();self.elapsed=0;for a in &mut self.anchors{a.shown=!a.infobot;}}
    pub fn from_gameplay(level:u32,bytes:&[u8],gemlik:Option<[f32;3]>,state:State)->Self{
        if !state.active(){return Self::default();}let mut out=Self::default();
        if let (Ok(ms),Ok(vars))=(rc_formats::gameplay::parse_moby_instances(bytes),rc_formats::gameplay::parse_pvars(bytes)){
            for m in ms.into_iter().filter(|m|m.position.iter().all(|p|p.is_finite())) {
                let source=match m.o_class {
                    750=>m.pvar(&vars).and_then(|v|v.get(4..8)).map(|v|i32::from_le_bytes(v.try_into().unwrap())).filter(|p|(2..19).contains(p)).map(|p|64+p as usize),
                    1134=>m.pvar(&vars).and_then(|v|v.get(..4)).map(|v|i32::from_le_bytes(v.try_into().unwrap())).filter(|i|(0..4).contains(i)).map(|i|96+level as usize*4+i as usize),
                    1120 if level==4=>Some(9),1354 if level==14=>Some(21),
                    1005 if level==2=>Some(26),1016 if level==6=>Some(22),
                    18 if level==10=>Some(28),1428 if level==17=>Some(50),1290 if level==9=>Some(7),
                    _=>None,
                };
                if let Some(source)=source.filter(|&s|state.reward(s).is_some()){
                    if !out.anchors.iter().any(|a|a.source==source){out.anchors.push(Anchor{source,position:m.position,mission:Some(i32::from(m.unknown_4 as u8)),infobot:m.o_class==750,shown:m.o_class!=750});}
                }
            }
        }
        for site in super::golden::locations::all_for_level(level,gemlik){
            let source=176+site.slot-80;
            if state.reward(source).is_some(){out.anchors.push(Anchor{source,position:site.base,mission:None,infobot:false,shown:true});}
        }out
    }
    pub fn visuals(&self,state:State,alive:bool)->Vec<(usize,super::discovery::Visual)>{
        if !alive{return vec![];}self.anchors.iter().filter(|a|a.shown&&self.ready.contains(&a.source)&&!state.collected(a.source)&&!self.pending.contains(&a.source))
            .map(|a|{let mut p=a.position;p[2]+=1.+0.15*((self.elapsed%180) as f32/180.*std::f32::consts::TAU).sin();
                (RENDER_BASE+a.source,super::discovery::Visual{position:p,yaw:super::pickup_visuals::spin_angle(self.elapsed)})}).collect()
    }
}
pub fn tick(w:&mut World){
    if !w.svc.bomb_evo.randomizer_mode{return;}w.svc.randomizer.elapsed=w.svc.randomizer.elapsed.wrapping_add(1);
    // Preserve flight/switch/mission timing of native Infobots, replacing only
    // their visible reward and collection feedback. Hidden stand-ins stay hidden.
    for moby in &mut w.table.mobys {
        if moby.o_class!=750||moby.state>=128{continue;}
        let Some(planet)=moby.pvars.get(4..8).map(|v|i32::from_le_bytes(v.try_into().unwrap())) else{continue;};
        if let Some(anchor)=w.svc.randomizer.anchors.iter_mut().find(|a|a.infobot&&a.source==64+planet.max(0) as usize){
            anchor.shown=matches!(moby.state,2..=7);
            anchor.position=[moby.position[0],moby.position[1],moby.position[2]-1.];
            moby.mode|=1;
            if moby.pvars.len()>14{moby.pvars[12..15].fill(0xff);}
        }
    }
    if w.svc.game_mode==0 && w.svc.level!=0 {
        let available=ship_available(w.svc.bomb_evo.state_progress.randomizer,w.svc.level);
        if let Some(id)=w.svc.travel.moby {
            if let Some(m)=w.table.mobys.get_mut(id){
                if available{m.mode&=!3;m.has_collision=true;}else{m.mode|=3;m.has_collision=false;}
            }
        }
    }
    if w.hero.health<=0||w.svc.game_mode!=0||!matches!(w.hero.mode,0|1|3){return;}
    let state=w.svc.bomb_evo.state_progress.randomizer;let h=w.hero.pos.map(|f|f.to_f32());
    for (key,v) in w.svc.randomizer.visuals(state,true){
        if w.svc.randomizer.anchors.iter().any(|a|a.source==key-RENDER_BASE&&a.infobot){continue;}
        if (v.position[0]-h[0]).hypot(v.position[1]-h[1])>1.6||(v.position[2]-h[2]).abs()>2.{continue;}
        if w.line([h[0],h[1],h[2]+0.75,0.].map(crate::ps2v::Pf::f),[v.position[0],v.position[1],v.position[2],0.].map(crate::ps2v::Pf::f),0x12,w.hero_moby).is_some(){continue;}
        let source=key-RENDER_BASE;
        if let Some(mission)=w.svc.randomizer.anchors.iter().find(|a|a.source==source).and_then(|a|a.mission){
            if (0..16).contains(&mission){crate::cinematic::set_mission_done(w,mission as u8);}
        }
        queue(w,source);
    }
}

pub fn replace_physical(w:&mut World,id:crate::moby_runtime::MobyId)->bool {
    if !w.svc.bomb_evo.randomizer_mode{return false;}
    let source=match (w.svc.level,w.m(id).o_class){
        (4,1120)=>9,(14,1354)=>21,(2,1005)=>26,(6,1016)=>22,
        (10,18)=>28,(17,1428)=>50,(9,1290)=>7,_=>return false,
    };
    if !w.svc.randomizer.anchors.iter().any(|a|a.source==source){return false;}
    // Actor identity remains available to mission references. Rendering and
    // collection are owned by the replacement, independent of native ownership.
    w.mm(id).mode|=1;w.mm(id).has_collision=false;true
}

pub fn reward_name(reward:u16,lang:u32,names:&std::collections::HashMap<usize,Vec<u8>>)->Vec<u8>{
    if reward<37 {if let Some(name)=names.get(&(reward as usize)){return name.clone();}}
    let text=if (66..=82).contains(&reward){
        let planet=["Veldin","Novalis","Aridia","Kerwan","Eudora","Rilgar","Blarg Station","Umbris","Batalia","Gaspar","Orxon","Pokitaru","Hoven","Gemlik Base","Oltanis","Quartu","Kalebo III","Drek's Fleet","Veldin II"][reward as usize-64];
        let prefix=match lang{2=>"Coordonnees : ",3=>"Koordinaten: ",4=>"Coordenadas: ",5=>"Coordinate: ",_=>"Coordinates: "};
        format!("{prefix}{planet}")
    }else if reward>=96 {match lang{2=>"Boulon en or",3=>"Gold-Bolt",4=>"Guiton de oro",5=>"Bolt d'oro",_=>"Gold Bolt"}.into()}
    else{match reward {48=>"Zoomerator",49=>"Raritanium",50=>"Codebot",52=>"Premium Nanotech",53=>"Ultra Nanotech",_=>"Item acquired!"}.into()};
    text.into_bytes()
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]
    fn early_checks_sample_the_mixed_pool_instead_of_greedily_forcing_coordinates() {
        let mut hist=[[0usize;5];4];
        let mut unique:[std::collections::HashSet<u16>;4]=Default::default();
        for seed in 1..=256 {
            let state=State::generate(seed);
            assert_eq!(state.version,6);assert!(state.valid()&&state.solvable()&&state.planet_discovery_balanced());
            for (i,source) in [16,100,101,102].into_iter().enumerate() {
                let Some(reward)=state.reward(source)else{continue;};unique[i].insert(reward);
                let category=if super::super::golden::ITEMS.contains(&(reward as usize)){0}
                    else{match reward{2..=35=>1,65..=82=>2,96.. =>3,_=>4}};
                hist[i][category]+=1;
            }
        }
        // A deterministic sample catches the old 169/256 coordinate bias and
        // 12/256 weapons at Novalis's first vendor. This is not a promise of
        // mathematically uniform layouts: progression still constrains choices.
        assert!(hist[0][2]<128,"coordinate-heavy first vendor: {hist:?}");
        assert!(hist[0][0]>256/12,"weapons crowded out of first vendor: {hist:?}");
        for i in 0..4 {
            assert!(unique[i].len()>30,"source {} lacks variety",[16,100,101,102][i]);
            assert!(hist[i].iter().all(|&n|n>0),"missing reward category: {hist:?}");
        }
        println!("v95 Novalis distribution over 256 seeds (weapon/gadget/coordinate/gold/special): {hist:?}");
    }
    #[test]
    fn stored_v1_through_v4_layouts_load_verbatim_instead_of_regenerating() {
        // Real synthetic layout emitted by the v90 generator for seed 723.
        // No user's save or disc data is embedded in this fixture.
        let placements: [u16;SLOTS] = [0, 0, 12, 117, 145, 15, 100, 102, 0, 73, 0, 66, 81, 101, 26, 69, 53, 30, 116, 160, 28, 13, 18, 82, 124, 128, 33, 133, 67, 154, 25, 156, 157, 14, 112, 148, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 107, 6, 109, 0, 52, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 74, 120, 72, 136, 78, 79, 20, 153, 170, 3, 71, 21, 27, 129, 105, 23, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 49, 31, 0, 50, 29, 9, 169, 68, 140, 19, 0, 76, 0, 0, 0, 168, 11, 0, 0, 152, 155, 0, 0, 2, 17, 0, 0, 48, 132, 0, 0, 77, 7, 0, 0, 137, 70, 0, 0, 24, 0, 0, 0, 80, 10, 0, 0, 106, 0, 0, 0, 5, 22, 110, 32, 144, 125, 0, 0, 165, 164, 0, 0, 108, 35, 0, 0, 121, 104, 161, 0, 0, 0, 0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        for version in [1,2,3,4] {
            let mut old=State{seed:723,version,placements,..Default::default()};
            old.collect(65);old.collect(16);
            assert!(old.valid(),"legacy layouts remain readable even when corrected routes reject them");
            let mut progress=Progress::default();progress.randomizer=old;progress.nanotech.xp=17.;
            let bytes=progress.encode(b"v91 synthetic old card").unwrap();
            let loaded=Progress::decode(&bytes,b"v91 synthetic old card").unwrap();
            assert_eq!(loaded,progress);
            let mut evolution=Evolution::default();evolution.restore_progress(&loaded);
            assert_eq!(evolution.progress().randomizer,old);
            assert_ne!(old.placements,State::generate(723).placements);
        }
    }


    #[test]
    fn balanced_layouts_spread_planets_and_legacy_v3_layouts_are_not_rebalanced_on_load() {
        // Synthetic v91 layout, not a user save. It is valid and completable,
        // but deliberately remains readable even if it fails the new pacing.
        let old=State{seed:723,version:3,placements:[0, 0, 66, 76, 74, 12, 10, 75, 0, 124, 0, 120, 21, 6, 30, 104, 117, 71, 145, 25, 26, 108, 49, 13, 22, 160, 31, 82, 136, 23, 48, 155, 112, 72, 169, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 35, 20, 0, 16, 79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 73, 125, 34, 3, 29, 156, 161, 153, 101, 70, 69, 27, 152, 28, 32, 100, 144, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 52, 80, 67, 0, 106, 116, 19, 128, 2, 78, 9, 0, 81, 0, 0, 0, 165, 121, 0, 0, 68, 18, 0, 0, 140, 50, 0, 0, 17, 110, 0, 0, 137, 33, 0, 0, 170, 102, 0, 0, 107, 0, 0, 0, 168, 164, 0, 0, 4, 0, 0, 0, 148, 15, 14, 129, 7, 154, 0, 0, 53, 5, 0, 0, 24, 157, 0, 0, 109, 105, 133, 0, 0, 0, 0, 0, 132, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],..Default::default()};
        assert!(old.valid(),"legacy layouts remain readable even when corrected routes reject them");
        assert!(!old.planet_discovery_balanced());
        let mut progress=Progress::default();progress.randomizer=old;
        let encoded=progress.encode(b"v92 synthetic v3 layout").unwrap();
        assert_eq!(Progress::decode(&encoded,b"v92 synthetic v3 layout").unwrap(),progress);
        for seed in [0,1,723,1<<63,u64::MAX] {
            let new=State::generate(seed);assert!(new.valid()&&new.solvable()&&new.planet_discovery_balanced());
            let first=[16,66,67].iter().filter(|&&id|is_new_planet(new.placements[id])).count();
            assert!(first<=1);
            let catalog=catalog::locations();
            for level in 1..=18 {
                let count=catalog.iter().filter(|l|l.level==level&&is_new_planet(new.placements[l.id])).count();
                assert!(count<=2,"seed={seed}, level={level}, coordinates={count}");
            }
        }
    }
    #[test]fn mixed_seeds_are_deterministic_complete_and_have_unique_rewards(){
        let item_checks:Vec<_>=catalog::locations().into_iter().filter(|l|l.id<64).map(|l|l.id).collect();
        let expected:Vec<_>=(2..36).filter(|&i|i!=8&&i!=10).chain([48,49,50,52,53]).collect();
        assert_eq!(item_checks,expected,"all native inventory and special rewards must be present");
        let mut additions=[false;6];let mut empty_native=std::collections::HashSet::new();
        let mut most_additions=0;
        for seed in 1..=300 {
            let s=State::generate(seed);assert!(s.valid()&&s.solvable());
            assert_eq!(s,State::generate(seed));
            assert_eq!(s.placements.iter().filter(|&&r|r!=0).count(),pool().len());
            let locations=catalog::locations();
            assert!(locations.iter().filter(|l|l.id<96).all(|l|s.reward(l.id).is_some()));
            assert_eq!(locations.iter().filter(|l|s.reward(l.id).is_none()).count(),5);
            let extra=s.placements[176..182].iter().filter(|&&r|r!=0).count();
            assert!((1..=6).contains(&extra));most_additions=most_additions.max(extra);
            for i in 0..6{additions[i]|=s.reward(176+i).is_some();}
            for l in locations{if l.id>=96&&l.id<176&&s.reward(l.id).is_none(){empty_native.insert(l.id);}}
        }
        assert!(additions.into_iter().all(|active|active));
        assert_eq!(most_additions,6,"all surveyed sites can coexist in one layout");
        assert!(empty_native.len()>20,"surplus removal must not always target the same native sites");
        assert_ne!(State::generate(1),State::generate(2));
    }
    #[test]
    fn corrected_underwater_requirements_are_used_by_the_solver() {
        let catalog=catalog::locations();let mut have=[false;SLOTS];
        have[65]=true;have[75]=true;
        let novalis=catalog.iter().find(|l|l.id==181).unwrap();
        assert!(!novalis.accessible(&have));have[4]=true;
        assert!(novalis.accessible(&have));assert!(!have[6]);
        have[4]=false;
        let seabed=catalog.iter().find(|l|l.id==179).unwrap();
        assert!(seabed.accessible(&have),"added Pokitaru seabed needs no water gadget");
        have[74]=true;
        let orxon=catalog.iter().find(|l|l.id==178).unwrap();
        assert!(!orxon.accessible(&have));have[6]=true;assert!(!orxon.accessible(&have));
        have[2]=true;assert!(orxon.accessible(&have));
    }
    #[test]
    fn selected_and_empty_sites_survive_save_reload_and_challenge_reset() {
        for seed in [1,2,723,8401,u64::MAX] {
            let mut state=State::generate(seed);
            for source in [65,16,28,75,137]{if state.reward(source).is_some(){state.collect(source);}}
            assert!(state.valid());
            let mut p=Progress::default();p.randomizer=state;p.nanotech.xp=37.;
            let encoded=p.encode(b"v94 layout selection").unwrap();
            let loaded=Progress::decode(&encoded,b"v94 layout selection").unwrap();assert_eq!(loaded,p);
            let mut e=Evolution::default();e.restore_progress(&loaded);assert_eq!(e.progress().randomizer,state);
            let mut gs=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
            gs.weapon_progress=loaded;
            gs.challenge_reset(&[],|g|*g=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]}));
            assert_eq!(gs.weapon_progress.randomizer,state);
        }
    }
    #[test]
    fn omitted_orxon_gold_check_does_not_force_clank_forever() {
        let mut state=State::generate((1..=100).find(|&seed|State::generate(seed).reward(137).is_none()).unwrap());
        assert!(!travel_mask(state,10,true));state.collect(28);
        assert!(!travel_mask(state,10,true));state.collect(75);
        assert!(travel_mask(state,10,true));assert!(state.valid());
        assert!(!travel_mask(state,10,false),"a real Mask is still needed for Ratchet");
        assert!(travel_mask(State::default(),10,true));
    }
    #[test]
    fn v94_exterior_blarg_coordinate_layout_is_rejected_by_corrected_routes() {
        // Only a reward permutation is retained; no native save or personal data.
        let old=State{seed:14461498566715420534,version:5,placements:[0, 0, 69, 13, 144, 168, 120, 34, 0, 29, 0, 30, 25, 128, 154, 102, 4, 156, 109, 169, 110, 33, 104, 78, 121, 74, 68, 15, 28, 2, 100, 148, 9, 105, 124, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 17, 77, 3, 0, 117, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 145, 70, 132, 18, 48, 31, 157, 152, 6, 12, 165, 32, 66, 73, 72, 108, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 49, 27, 0, 11, 140, 170, 81, 71, 52, 16, 0, 160, 0, 0, 0, 24, 164, 0, 0, 79, 23, 0, 0, 112, 53, 0, 0, 26, 116, 0, 0, 0, 82, 0, 0, 35, 7, 0, 0, 22, 0, 0, 0, 136, 21, 0, 0, 0, 0, 0, 0, 76, 153, 20, 133, 125, 0, 0, 0, 19, 0, 0, 0, 106, 101, 0, 0, 107, 161, 137, 0, 0, 0, 0, 0, 14, 0, 67, 155, 5, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],..Default::default()};
        assert!(old.valid(),"old saves stay readable");
        assert!(!old.solvable(),"the old route relied on the exterior bolt without O2");
        let new=State::generate(old.seed);
        assert!(new.valid()&&new.solvable()&&new.planet_discovery_balanced());
        assert_ne!(new.placements,old.placements);
    }
    #[test]
    fn ratchet_only_orxon_gold_never_delays_his_return() {
        let mut state=State::generate(723);
        state.collect(28);state.collect(75);
        assert!(!state.collected(137));
        assert!(travel_mask(state,10,true));
        assert!(!travel_mask(state,10,false));
    }
    #[test]fn collected_location_is_not_the_same_as_owning_its_original_item(){
        let mut s=State::generate(72);let source=s.placements.iter().position(|&r|r==12).unwrap();
        assert!(!s.owns(12));s.collect(source);assert!(s.owns(12));assert!(s.collected(source));assert!(s.valid());
    }
    fn fixture()->(GameState,SessionState,ItemTables){
        use rc_formats::save_game::{ChunkTables,ItemRecord,ITEM_COUNT,VENDOR_TABLE_LEN};
        let mut gs=GameState::zeroed(ChunkTables{global:vec![],level:vec![]});
        initialize(&mut gs,true,723);
        let mut items=ItemTables{vendor_items:[0;VENDOR_TABLE_LEN],records:[ItemRecord([0;0x18]);ITEM_COUNT],slot_type:[0;ITEM_COUNT],item_defs_addr:0};
        for r in &mut items.records{r.0[14..16].copy_from_slice(&40u16.to_le_bytes());}
        (gs,SessionState::default(),items)
    }
    #[test]fn every_reward_kind_grants_once_without_unlocking_the_source_reward(){
        let (mut gs,mut sess,items)=fixture();
        for reward in [12,15,49,52,53,71,100]{
            let at=gs.weapon_progress.randomizer.placements.iter().position(|&r|r==reward).unwrap();
            assert_eq!(grant(&mut gs,&mut sess,&items,at,true),Some(reward));
            let before=gs.clone();assert_eq!(grant(&mut gs,&mut sess,&items,at,true),None);
            assert_eq!(gs.global,before.global);
            if reward<37{assert_eq!(gs.global.owned[reward as usize],1);}
        }
        assert!(gs.global.quick_select.contains(&15));assert_eq!(gs.global.ammo[15],1);
        assert_eq!((gs.global.flags[1],gs.global.flags[4],gs.global.flags[5]),(1,1,1));
        assert_eq!(gs.global.planet_unlocked[7],1);assert!(gs.global.map_order.contains(&7));
        assert_eq!(gs.levels[1].gold_bolts[0],1);
    }
    #[test]fn submerged_weapon_defers_equipment_and_preserves_health_progress(){
        let (mut gs,mut sess,items)=fixture();gs.global.max_hp=14;
        let at=gs.weapon_progress.randomizer.placements.iter().position(|&r|r==13).unwrap();
        grant(&mut gs,&mut sess,&items,at,false);assert_eq!(sess.temp_hand,0);
        assert_eq!(gs.weapon_progress.randomizer.pending_equip,13);
        let at=gs.weapon_progress.randomizer.placements.iter().position(|&r|r==52).unwrap();
        grant(&mut gs,&mut sess,&items,at,false);assert_eq!(gs.global.max_hp,15);
        let p=gs.weapon_progress;let bytes=p.encode(b"native").unwrap();
        assert_eq!(Progress::decode(&bytes,b"native").unwrap(),p);
        assert!(Progress::decode(&bytes,b"other").is_err());
    }
    #[test]fn challenge_reset_keeps_seed_claims_coordinates_and_actual_inventory(){
        let (mut gs,mut sess,items)=fixture();
        for reward in [15,71,48]{let at=gs.weapon_progress.randomizer.placements.iter().position(|&r|r==reward).unwrap();grant(&mut gs,&mut sess,&items,at,true);}
        let state=gs.weapon_progress.randomizer;
        gs.challenge_reset(&[],|g|*g=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]}));
        assert_eq!(gs.weapon_progress.randomizer,state);assert_eq!(gs.global.owned[15],1);
        assert_eq!(gs.global.planet_unlocked[7],1);assert!(gs.global.map_order.contains(&7));assert_eq!(gs.global.flags[0],1);
    }
    #[test]fn departures_follow_seed_and_novalis_ship_requires_its_mayor_check(){
        let mut starts=std::collections::HashSet::new();
        for seed in 1..50{
            let mut state=State::generate(seed);let start=departure(state,0,1,&[0;20]);starts.insert(start);
            assert!((1..18).contains(&start));assert!(!ship_available(state,1));assert!(ship_available(state,7));
            state.collect(67);assert!(ship_available(state,1));
            let at=state.placements.iter().position(|&r|r==72).unwrap();
            state.placements.swap(at,72);
            assert_eq!(departure(state,7,8,&[0;20]),8);
            assert_eq!(departure(state,0,-1,&[0;20]),-1);
        }
        assert_eq!(starts,std::collections::HashSet::from([1]));
    }
    #[test]fn previous_v8_companions_decode_without_randomizer_or_changed_controls(){
        let mut p=Progress::default();p.controls.dpad=[15,0,0,0];p.tracks[0].xp=12.;
        let current=p.encode(b"old").unwrap();
        let mut bytes=current[..current.len()-20-ENCODED_SIZE].to_vec();bytes[5]=8;
        bytes.extend_from_slice(&rc_formats::sha1::sha1(&bytes));
        assert_eq!(Progress::decode(&bytes,b"old").unwrap(),p);
    }
    #[test]fn mode_priority_and_save_type_guards_do_not_depend_on_item_ownership(){
        let ini=include_str!("weapon_evo_release.ini");
        let e=Evolution::parse(&format!("{ini}\n[general]\nrandomizer_mode=true\ngolden_bolts_to_weapons=true\nexperimental_ammo_unlock_mode=true\nhide_ryno=true\n")).unwrap();
        assert!(e.randomizer_mode);assert!(!e.golden_settings.enabled);assert!(!e.ammo_unlock_mode);assert!(!e.hide_ryno);
        let mut p=Progress::default();assert!(accepts(&p,false));assert!(!accepts(&p,true));
        p.randomizer=State::generate(8410);assert!(accepts(&p,true));assert!(!accepts(&p,false));
    }
    #[test]
    fn omitted_sites_create_no_extra_anchor_and_native_gold_cannot_leak_loot() {
        use crate::moby_update::classes::bomb_water::tests::{Bench,pool};
        let state=State::generate(723);
        for level in 1..=18 {
            let world=WorldState::from_gameplay(level,&[],Some([10.,20.,30.]),state);
            assert!(world.anchors.iter().all(|a|state.reward(a.source).is_some()));
            for site in super::super::golden::locations::all_for_level(level,Some([10.,20.,30.])) {
                let source=176+site.slot-80;
                assert_eq!(world.anchors.iter().any(|a|a.source==source),state.reward(source).is_some());
            }
        }
        // The native actor remains suppressed even when no replacement exists.
        let mut b=Bench::new(pool(10.,-10.),&[],[20.,20.,10.]);
        b.svc.bomb_evo.randomizer_mode=true;b.svc.bomb_evo.state_progress.randomizer=state;
        let mut actor=crate::moby_runtime::Moby::zeroed();actor.o_class=1134;actor.has_collision=true;
        b.table.mobys.push(actor);let id=b.table.mobys.len()-1;
        let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,0);
        crate::moby_update::classes::gold_bolt::update(&mut w,id);
        assert_ne!(w.m(id).mode&crate::moby_runtime::mode::HIDDEN,0);
        assert!(!w.m(id).has_collision);assert!(w.svc.interact.writes.is_empty());
    }
    #[test]fn collection_waits_for_a_visible_model_and_removes_mesh_glow_together(){
        use crate::moby_update::classes::bomb_water::tests::{Bench,pool};
        let mut b=Bench::new(pool(10.,-10.),&[],[20.,20.,10.]);b.hero.health=4;
        b.svc.bomb_evo.randomizer_mode=true;b.svc.bomb_evo.state_progress.randomizer=State::generate(8411);
        let source=b.svc.bomb_evo.state_progress.randomizer.placements.iter().enumerate().find(|(id,r)|**r!=0&&*id!=65).unwrap().0;
        b.svc.randomizer.anchors.push(Anchor{source,position:[20.,20.,10.],mission:None,infobot:false,shown:true});
        let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,0);
        tick(&mut w);assert!(w.svc.interact.writes.is_empty());
        w.svc.randomizer.ready.insert(source);tick(&mut w);
        assert_eq!(w.svc.interact.writes,vec![GameWrite::RandomizerReward(source)]);
        assert!(w.svc.randomizer.visuals(w.svc.bomb_evo.progress().randomizer,true).is_empty());
        w.svc.randomizer.reset_for_reload();assert_eq!(w.svc.randomizer.visuals(w.svc.bomb_evo.progress().randomizer,true).len(),1);
        assert!(w.svc.randomizer.visuals(w.svc.bomb_evo.progress().randomizer,false).is_empty());
        w.svc.bomb_evo.state_progress.randomizer.collect(source);
        assert!(w.svc.randomizer.visuals(w.svc.bomb_evo.progress().randomizer,true).is_empty());
    }
}
