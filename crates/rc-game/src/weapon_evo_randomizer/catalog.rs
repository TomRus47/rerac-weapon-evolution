//! Stable reward identities and conservative, glitch-free access rules.
//! Casual gadget alternatives reviewed against bordplate/rac1-randomizer.
//! See docs/mod/RANDOMIZER.md for provenance and runtime adaptations.
use super::Location;
/// Native regular Gadgetron sources. Starter gifts, physical pickups and NPC
/// purchases remain separate checks even when an old stock table names them.
/// Planet identity comes from the same catalog used by seed reachability.
pub fn is_gadgetron_source(source:usize,level:i32)->bool {
    matches!(source,11|13..=20|24|25)
        && locations().iter().any(|l|l.id==source&&l.level as i32==level)
}
pub fn locations()->Vec<Location> {let mut out=vec![
Location{id:2,level:3,name:"Heli-pack",requires:&[&[]]},
Location{id:3,level:11,name:"Thruster-pack",requires:&[&[]]},
Location{id:4,level:12,name:"Hydro-pack",requires:&[&[22]]},
Location{id:5,level:2,name:"Sonic Summoner",requires:&[&[48]]},
Location{id:6,level:11,name:"O2 Mask",requires:&[&[7]]},
Location{id:7,level:9,name:"Pilot's Helmet",requires:&[&[]]},
Location{id:9,level:4,name:"Suck Cannon",requires:&[&[2],&[3]]},
Location{id:11,level:8,name:"Devastator",requires:&[&[]]},
Location{id:12,level:3,name:"Swingshot",requires:&[&[]]},
Location{id:13,level:10,name:"Visibomb",requires:&[&[6]]},
Location{id:14,level:6,name:"Taunter",requires:&[&[]]},
Location{id:15,level:3,name:"Blaster",requires:&[&[]]},
Location{id:16,level:1,name:"Pyrocitor",requires:&[&[]]},
Location{id:17,level:5,name:"Mine Glove",requires:&[&[]]},
Location{id:18,level:9,name:"Walloper",requires:&[&[]]},
Location{id:19,level:14,name:"Tesla Claw",requires:&[&[]]},
Location{id:20,level:4,name:"Glove of Doom",requires:&[&[]]},
Location{id:21,level:14,name:"Morph-O-Ray",requires:&[&[12]]},
Location{id:22,level:6,name:"Hydrodisplacer",requires:&[&[26]]},
Location{id:23,level:5,name:"R.Y.N.O.",requires:&[&[27,2],&[27,3]]},
Location{id:24,level:12,name:"Drone Device",requires:&[&[]]},
Location{id:25,level:11,name:"Decoy Glove",requires:&[&[]]},
Location{id:26,level:2,name:"Trespasser",requires:&[&[12]]},
Location{id:27,level:8,name:"MetalDetector",requires:&[&[28]]},
Location{id:28,level:10,name:"Magneboots",requires:&[&[]]},
Location{id:29,level:6,name:"Grindboots",requires:&[&[12]]},
Location{id:30,level:2,name:"Hoverboard",requires:&[&[]]},
Location{id:31,level:16,name:"Hologuise",requires:&[&[30,2,12,29],&[30,3,12,29]]},
Location{id:32,level:14,name:"PDA",requires:&[&[28]]},
Location{id:33,level:16,name:"Map-O-Matic",requires:&[&[29]]},
Location{id:34,level:15,name:"Bolt Grabber",requires:&[&[4,6]]},
Location{id:35,level:11,name:"Persuader",requires:&[&[26,49,22]]},
Location{id:48,level:5,name:"Zoomerator",requires:&[&[30,2],&[30,3]]},
Location{id:49,level:12,name:"Raritanium",requires:&[&[12,2],&[12,3]]},
Location{id:50,level:17,name:"Codebot",requires:&[&[4,6]]},
Location{id:52,level:10,name:"Premium Nanotech",requires:&[&[6,2],&[6,3]]},
Location{id:53,level:10,name:"Ultra Nanotech",requires:&[&[6,3,52,27],&[6,2,52,27]]},
Location{id:65,level:0,name:"Planet coordinates 1",requires:&[&[]]},
Location{id:66,level:1,name:"Planet coordinates 2",requires:&[&[]]},
Location{id:67,level:1,name:"Planet coordinates 3",requires:&[&[]]},
Location{id:68,level:3,name:"Planet coordinates 4",requires:&[&[2],&[3]]},
Location{id:69,level:6,name:"Planet coordinates 5",requires:&[&[]]},
Location{id:70,level:4,name:"Planet coordinates 6",requires:&[&[26,12,2],&[26,12,3]]},
Location{id:71,level:5,name:"Planet coordinates 7",requires:&[&[12,22]]},
Location{id:72,level:7,name:"Planet coordinates 8",requires:&[&[12,22]]},
Location{id:73,level:8,name:"Planet coordinates 9",requires:&[&[29]]},
Location{id:74,level:8,name:"Planet coordinates 10",requires:&[&[]]},
Location{id:75,level:10,name:"Planet coordinates 11",requires:&[&[]]},
Location{id:76,level:10,name:"Planet coordinates 12",requires:&[&[6,12,28,3]]},
Location{id:77,level:12,name:"Planet coordinates 13",requires:&[&[11],&[13],&[15]]},
Location{id:78,level:13,name:"Planet coordinates 14",requires:&[&[12,28,26,11],&[12,28,26,13]]},
Location{id:79,level:14,name:"Planet coordinates 15",requires:&[&[29]]},
Location{id:80,level:15,name:"Planet coordinates 16",requires:&[&[12]]},
Location{id:81,level:15,name:"Planet coordinates 17",requires:&[&[3,12,31]]},
Location{id:82,level:17,name:"Planet coordinates 18",requires:&[&[28,31]]},
];
for (level,&count) in super::super::golden::COUNTS.iter().enumerate().skip(1) {
for index in 0..count as usize {out.push(Location{id:96+level*4+index,level:level as u32,
name:gold_name(level,index),requires:gold_requirements(level,index)});}}
for (i,level) in [13,3,10,11,12,1].into_iter().enumerate(){
out.push(Location{id:176+i,level,name:"Additional world location",requires:extra_requirements(i)});}
out.sort_by_key(|l|l.id);out}
// Extra-site requirements are independent of the native Gold Bolt index order.
fn extra_requirements(i:usize)->&'static [&'static [u16]] {match i {
0=>&[&[2,12,28,26,11],&[2,12,28,26,13],&[3,12,28,26,11],&[3,12,28,26,13]],
1=>&[&[2],&[3]],2=>&[&[6,2],&[6,3]],3=>&[&[]],
4=>&[&[2],&[3]],5=>&[&[4]],_=>panic!("unknown extra location")}}

fn gold_name(p:usize,i:usize)->&'static str {match (p,i) {
(6,0)=>"Blarg exterior Gold Bolt (Ratchet, airlock)",
(6,1)=>"Blarg interior Gold Bolt (upper cage)",
_=>"Native Gold Bolt (conservative route requirements)",
}}

/// Never infer the native pvar index from the order in a walkthrough. Blarg's
/// two identities are verified by the reported pickup/save pair. For remaining
/// unsurveyed indices use a shared, conservative route envelope on that planet.
/// This deliberately over-gates some easy checks instead of putting mandatory
/// progression behind a harder sibling with the wrong requirements.
fn gold_requirements(p:usize,i:usize)->&'static [&'static [u16]] {
assert!(i<super::super::golden::COUNTS[p] as usize,"unknown native Gold Bolt index");
match (p,i) {
(6,0)=>&[&[6,26]],(6,1)=>&[&[2],&[3]],
(1,_)=>&[&[2,4,6,10],&[2,4,6,11],&[2,4,6,13],&[3,4,6,10],&[3,4,6,11],&[3,4,6,13]],
(2,_)=>&[&[2,12,26,28,10],&[2,12,26,28,11],&[2,12,26,28,13],&[3,12,26,28,10],&[3,12,26,28,11],&[3,12,26,28,13]],
(3,_)=>&[&[2],&[3]],
(4,_)=>&[&[2,12,26]],
(5,_)=>&[&[2,4,6,12,22],&[3,4,6,12,22]],
(7,_)=>&[&[2,12,22],&[3,12,22]],
(8,_)=>&[&[2,12,26,28,29],&[3,12,26,28,29]],
(9,_)=>&[&[2,12],&[3,12]],
(10,_)=>&[&[6,12,13,28,3]],
// This native route has a Versa-target and high ledge; the added seabed does not.
// Neither location needs the O2 Mask or Hydro-Pack.
(11,_)=>&[&[2,12],&[3,12]],
(12,_)=>&[&[2,4,6,22],&[3,4,6,22]],
(13,_)=>&[&[2,12,28,26,11],&[2,12,28,26,13],&[3,12,28,26,11],&[3,12,28,26,13]],
(14,_)=>&[&[2,12,21,28,29,32],&[3,12,21,28,29,32]],
(15,_)=>&[&[2,12,50],&[3,12,50]],
(16,_)=>&[&[2,12,29,30],&[3,12,29,30]],
(17,_)=>&[&[3,4,6,12,28,31]],
(18,_)=>&[&[2,6,12,14,29,31],&[3,6,12,14,29,31]],
_=>panic!("unknown native Gold Bolt planet"),
}}

#[cfg(test)]
mod route_tests {
    use super::*;
    fn can(source:usize,items:&[u16])->bool {
        let l=locations().into_iter().find(|l|l.id==source).unwrap();
        let mut have=[false;super::super::SLOTS];have[64+l.level as usize]=true;
        for &item in items{have[item as usize]=true;}
        l.accessible(&have)
    }
    #[test]
    fn reported_blarg_pickups_bind_requirements_to_native_ids() {
        assert!(can(121,&[2]));assert!(can(121,&[3]));
        assert!(!can(120,&[2]));assert!(!can(120,&[3]));
        assert!(!can(120,&[6]));assert!(can(120,&[6,26]));
        assert!(locations().into_iter().find(|l|l.id==120).unwrap().name.contains("exterior"));
    }
    #[test]
    fn orxon_ratchet_checks_are_never_free_clank_checks() {
        for source in [13,52,53,76,136,137,178] {
            let all:Vec<_>=(2..96).filter(|&i|i!=6).collect();
            assert!(!can(source,&all),"source {source} needs a real O2 Mask");
        }
        assert!(can(28,&[]));assert!(can(75,&[]));
        assert!(can(178,&[6,2]));assert!(!can(178,&[2]));
    }
    #[test]
    fn extra_and_native_water_routes_remain_distinct() {
        assert!(can(181,&[4]));assert!(!can(181,&[]));
        assert!(can(179,&[]));
        assert!(!can(140,&[]));assert!(can(140,&[2,12]));
    }
    #[test]
    fn catalogue_is_complete_and_has_no_unknown_or_impossible_requirement() {
        let all=locations();let mut ids=std::collections::HashSet::new();
        assert_eq!(all.len(),101);
        for l in &all {
            assert!(ids.insert(l.id));assert!(!l.requires.is_empty());
            assert!(l.requires.iter().all(|r|r.iter().all(|&i|i==10||all.iter().any(|a|a.id==i as usize))));
            assert!(can(l.id,&(2..96).collect::<Vec<_>>()));
        }
        assert_eq!(all.iter().filter(|l|(96..176).contains(&l.id)).count(),40);
    }
}
