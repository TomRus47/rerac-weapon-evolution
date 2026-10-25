//! Optional world sites surveyed against the native map transform and collision.
//! Positions are world-space reward bases, not map pixels or screen coordinates.
//! See docs/mod/GOLDEN_WEAPON_RANDOMIZER.md for the stable location catalogue.
use super::State;

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Site {
    pub slot:usize,
    pub level:u32,
    pub name:&'static str,
    pub base:[f32;3],
    pub underwater:bool,
}
pub const FIXED:[Site;5]=[
    Site {slot:81,level:3,name:"Kerwan empty room",base:[346.85,193.75,50.0],underwater:false},
    Site {slot:82,level:10,name:"Orxon laboratory grate alcove",base:[201.0,152.0,53.2],underwater:false},
    Site {slot:83,level:11,name:"Pokitaru second boat route seabed",base:[575.0,520.0,209.2],underwater:true},
    Site {slot:84,level:12,name:"Hoven suspended landing platform",base:[306.5,292.5,39.9],underwater:false},
    Site {slot:85,level:1,name:"Novalis upstream dead end",base:[40.0,278.0,33.3],underwater:true},
];
/// Gemlik's room uses immutable authored offers before the randomizer removes
/// their live models. A missing anchor disables that site's collection/rendering
/// together rather than placing an invisible reward at an invented coordinate.
pub fn for_level(level:u32,progress:&State,gemlik:Option<[f32;3]>)->Vec<Site> {
    if !progress.active(){return Vec::new();}
    all_for_level(level,gemlik).into_iter().filter(|s|progress.item_at(s.slot).is_some()).collect()
}
/// All authored extra sites, used only by the local location verification run.
pub fn all_for_level(level:u32,gemlik:Option<[f32;3]>)->Vec<Site> {
    let mut sites:Vec<_>=FIXED.into_iter().filter(|s|s.level==level).collect();
    if level==13 {
        if let Some(mut base)=gemlik {
            base[2]-=1.; // The shared visual adds its one-unit hover.
            sites.push(Site {slot:80,level,name:"Gemlik tower gold room",base,underwater:false});
        }
    }
    sites
}

/// Native weapon rewards are stable optional locations too. Read the authored
/// actors before spawn filters or the randomizer remove them; never infer a
/// position from Ratchet, a checkpoint or another save's live actor table.
pub fn native_weapon_sites(level:u32,gameplay:&[u8])->Vec<Site> {
    let (class,slot,name)=match level {
        4=>(1120,86,"Eudora native Suck Cannon reward"),
        14=>(1354,87,"Oltanis native Morph-o-Ray reward"),
        _=>return Vec::new(),
    };
    rc_formats::gameplay::parse_moby_instances(gameplay).ok().into_iter()
        .flatten().filter(|m|m.o_class==class && m.position.iter().all(|p|p.is_finite()))
        .take(1).map(|m|Site{slot,level,name,base:m.position,underwater:false}).collect()
}

pub fn native_morph_mission(level:u32,gameplay:&[u8])->Option<i32> {
    if level!=14{return None;}
    rc_formats::gameplay::parse_moby_instances(gameplay).ok()?.into_iter()
        .find(|m|m.o_class==1354).map(|m|i32::from(m.unknown_4 as u8)).filter(|m|*m<16)
}

#[cfg(test)]
mod v80_tests {
    use super::*;
    fn native_data(class:i32,mission:i32)->Vec<u8> {
        use rc_formats::gameplay::{MOBY_INSTANCES_POINTER,MOBY_INSTANCE_SIZE};
        let mut data=vec![0u8;0x110+MOBY_INSTANCE_SIZE];
        data[MOBY_INSTANCES_POINTER..MOBY_INSTANCES_POINTER+4].copy_from_slice(&0x100u32.to_le_bytes());
        data[0x100..0x104].copy_from_slice(&1i32.to_le_bytes());
        let m=&mut data[0x110..];
        m[0..4].copy_from_slice(&(MOBY_INSTANCE_SIZE as i32).to_le_bytes());
        m[4..8].copy_from_slice(&mission.to_le_bytes());
        m[0x18..0x1c].copy_from_slice(&class.to_le_bytes());
        for (i,p) in [12.0f32,23.,34.].into_iter().enumerate(){m[0x30+i*4..0x34+i*4].copy_from_slice(&p.to_le_bytes());}
        data
    }
    #[test]
    fn native_weapon_sites_use_raw_placed_coordinates_and_mission_identity() {
        for (level,class,slot) in [(4,1120,86),(14,1354,87)] {
            let data=native_data(class,3);
            let site=native_weapon_sites(level,&data).pop().unwrap();
            assert_eq!((site.slot,site.level,site.base),(slot,level,[12.,23.,34.]));
            let progress=(1..1000).map(|seed|State::generate(seed,0)).find(|s|s.item_at(slot).is_some()).unwrap();
            let world=super::super::WorldState::from_gameplay(level,&progress,&data,None);
            assert!(world.rewards.iter().any(|r|r.index==slot as i32&&r.position==site.base));
            assert_eq!(world.native_morph_mission,if level==14{Some(3)}else{None});
        }
        assert!(native_weapon_sites(14,&[]).is_empty());
        assert_eq!(native_morph_mission(14,&native_data(1354,-1)),None);
    }
}
