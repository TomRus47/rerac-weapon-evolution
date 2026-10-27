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
1=>&[&[2],&[3]],2=>&[&[6,12,28,3]],3=>&[&[]],
4=>&[&[2],&[3]],5=>&[&[4]],_=>panic!("unknown extra location")}}

/// Matched to native pvar IDs through the author's original-data survey and
/// annotated maps. Names describe positions, not a walkthrough's entry order.
fn gold_name(p:usize,i:usize)->&'static str {match (p,i) {
(1,0)=>"Novalis wall-jump room",
(1,1)=>"Novalis upstream cave",
(1,2)=>"Novalis explosive cave",
(2,0)=>"Aridia explosive wall",
(2,1)=>"Aridia mud-truck ledge",
(2,2)=>"Aridia Magneboots rail",
(2,3)=>"Aridia tower underside (conservative unsurveyed route)",
(3,0)=>"Kerwan station luggage ledge",
(3,1)=>"Kerwan fitness-course tower",
(3,2)=>"Kerwan train station rear",
(4,0)=>"Eudora high ledge",
(5,0)=>"Rilgar maze upper wall",
(5,1)=>"Rilgar flooded sewer",
(6,0)=>"Blarg exterior Gold Bolt (Ratchet, airlock)",
(6,1)=>"Blarg interior Gold Bolt (upper cage)",
(7,0)=>"Umbris pressure-switch room",
(7,1)=>"Umbris sharkigator pool ledge",
(8,0)=>"Batalia bridge-side plateau",
(8,1)=>"Batalia roof near landing area",
(9,0)=>"Gaspar helmet-route ledge",
(9,1)=>"Gaspar island fuel structure",
(10,0)=>"Orxon Visibomb wall tunnel",
(10,1)=>"Orxon laboratory wall-jump room",
(11,0)=>"Pokitaru hidden river route",
(12,0)=>"Hoven wall-jump area near landing",
(12,1)=>"Hoven water puzzle ledge",
(13,0)=>"Gemlik Visibomb generator route",
(14,0)=>"Oltanis statue Swingshot route",
(14,1)=>"Oltanis grind-route Swingshot alcove",
(14,2)=>"Oltanis Magneboots route lower bolt",
(14,3)=>"Oltanis Magneboots route upper bolt",
(15,0)=>"Quartu ceiling route",
(15,1)=>"Quartu Codebot vault",
(16,0)=>"Kalebo III break-room teleporter",
(16,1)=>"Kalebo III grind-rail pickup",
(17,0)=>"Drek fleet water rings",
(17,1)=>"Drek fleet ceiling backtrack",
(18,0)=>"Veldin II Taunter barrier",
(18,1)=>"Veldin II high platforms",
(18,2)=>"Veldin II hidden grind rail",
_=>panic!("unknown native Gold Bolt identity"),
}}

/// Alternatives contain BOTH traversal and local pickup requirements. A local
/// "no requirements" annotation never overrides a preceding route gadget gate.
/// Aridia index 3 remains conservative: it was not marked in the supplied map.
fn gold_requirements(p:usize,i:usize)->&'static [&'static [u16]] {
assert!(i<super::super::golden::COUNTS[p] as usize,"unknown native Gold Bolt index");
match (p,i) {
(1,0)=>&[&[]],
(1,1)=>&[&[4]],
(1,2)=>&[&[10],&[11],&[13]],
(2,0)=>&[&[10],&[11],&[13]],
(2,1)=>&[&[2],&[3]],
(2,2)=>&[&[12,26,28]],
(2,3)=>&[&[2,12,26,28,10],&[2,12,26,28,11],&[2,12,26,28,13],&[3,12,26,28,10],&[3,12,26,28,11],&[3,12,26,28,13]],
(3,0)=>&[&[2],&[3]],
(3,1)=>&[&[2],&[3]],
(3,2)=>&[&[2],&[3]],
(4,0)=>&[&[2,12,26]],
(5,0)=>&[&[2],&[3]],
(5,1)=>&[&[2,6,12,22],&[3,6,12,22]],
(6,0)=>&[&[6,26]],
(6,1)=>&[&[2],&[3]],
(7,0)=>&[&[12,22]],
(7,1)=>&[&[2,12,22],&[3,12,22]],
(8,0)=>&[&[2],&[3]],
(8,1)=>&[&[]],
(9,0)=>&[&[2],&[3]],
(9,1)=>&[&[2,12],&[3,12]],
(10,0)=>&[&[6,12,13,28,3]],
(10,1)=>&[&[6,12,28,3]],
(11,0)=>&[&[2,3,12]],
(12,0)=>&[&[]],
(12,1)=>&[&[2,22],&[3,22]],
(13,0)=>&[&[2,12,28,26,13],&[3,12,28,26,13]],
(14,0)=>&[&[2,12,28,29],&[3,12,28,29]],
(14,1)=>&[&[12,29]],
(14,2)=>&[&[28]],
(14,3)=>&[&[28]],
(15,0)=>&[&[3,12,31]],
(15,1)=>&[&[12,50]],
(16,0)=>&[&[29]],
(16,1)=>&[&[29]],
(17,0)=>&[&[4,6]],
(17,1)=>&[&[3,12,28,31]],
(18,0)=>&[&[2,6,12,14,31],&[3,6,12,14,31]],
(18,1)=>&[&[2,6,12,31],&[3,6,12,31]],
(18,2)=>&[&[2,6,12,29,31],&[3,6,12,29,31]],
_=>panic!("unknown native Gold Bolt identity"),
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
        assert!(can(178,&[6,12,28,3]));assert!(!can(178,&[6,2]));
    }
    #[test]
    fn extra_and_native_water_routes_remain_distinct() {
        assert!(can(181,&[4]));assert!(!can(181,&[]));
        assert!(can(179,&[]));
        assert!(!can(140,&[2,12]));assert!(!can(140,&[3,12]));assert!(can(140,&[2,3,12]));
    }
    #[test]
    fn surveyed_novalis_ids_do_not_share_sibling_requirements() {
        assert!(can(100,&[]));
        assert!(!can(101,&[]));assert!(can(101,&[4]));
        assert!(!can(102,&[4]));
        for explosive in [10,11,13]{assert!(can(102,&[explosive]));}
    }
    #[test]
    fn gemlik_tower_and_native_bolt_have_distinct_explosive_gates() {
        let path=[2,12,28,26,11];
        assert!(can(176,&path));assert!(!can(148,&path));
        assert!(!can(176,&[11]));assert!(!can(148,&[13]));
        assert!(can(176,&[2,12,28,26,13]));assert!(can(148,&[2,12,28,26,13]));
    }
    #[test]
    fn local_free_pickups_keep_traversal_and_character_gates() {
        assert!(can(129,&[]));assert!(can(144,&[]));
        assert!(!can(124,&[]));assert!(can(124,&[12,22]));
        assert!(!can(137,&[6]));assert!(can(137,&[6,12,28,3]));
        assert!(!can(137,&[12,28,3]));
        assert!(!can(156,&[12]));assert!(can(156,&[3,12,31]));
        assert!(!can(157,&[50]));assert!(can(157,&[12,50]));
    }
    #[test]
    fn oltanis_statue_gate_uses_route_equipment_not_randomized_rewards() {
        assert!(!can(152,&[12]));
        assert!(can(152,&[2,12,28,29]));
        assert!(!can(152,&[2,12,28]));
        assert!(can(154,&[28]));assert!(can(155,&[28]));
        assert!(!can(153,&[28]));assert!(can(153,&[12,29]));
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
