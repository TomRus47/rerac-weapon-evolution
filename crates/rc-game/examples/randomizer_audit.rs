//! Inspect the exact persisted layout; never regenerate or modify a save.
//! cargo run -p rc-game --example randomizer_audit -- save.bin matching.wev
//! cargo run -p rc-game --example randomizer_audit -- --seed 123
use rc_game::weapon_evo::{randomizer::{catalog,State,SLOTS},save::Progress};
use std::{env,fs,error::Error};

fn label(reward:u16)->String {
    if reward==10{return "Bomb Glove".into();}
    if (65..=82).contains(&reward){
        let planets=["Veldin","Novalis","Aridia","Kerwan","Eudora","Rilgar","Blarg Station","Umbris","Batalia","Gaspar","Orxon","Pokitaru","Hoven","Gemlik Base","Oltanis","Quartu","Kalebo III","Drek's Fleet","Veldin II"];
        return format!("Coordinates: {}",planets[reward as usize-64]);
    }
    if reward>=96{return format!("Gold Bolt reward {reward}");}
    catalog::locations().into_iter().find(|l|l.id==reward as usize).map_or_else(||format!("Reward {reward}"),|l|l.name.into())
}
fn main()->Result<(),Box<dyn Error>> {
    let args:Vec<_>=env::args().skip(1).collect();
    if args==["--catalog"] {
        println!("source\tplanet\tname\trequirement_alternatives");
        for l in catalog::locations(){println!("{}\t{}\t{}\t{:?}",l.id,l.level,l.name,l.requires);}
        return Ok(());
    }
    let state=match args.as_slice(){
        [flag,seed] if flag=="--seed"=>State::generate(seed.parse()?),
        [native,companion]=>Progress::decode(&fs::read(companion)?,&fs::read(native)?)?.randomizer,
        _=>return Err("Usage: randomizer_audit save.bin matching.wev | --seed NUMBER".into()),
    };
    println!("seed={} layout={} valid={} complete_from_start={}",state.seed,state.version,state.valid(),state.solvable());
    let mut have=[false;SLOTS];have[64]=true;
    for (id,r) in state.placements.iter().enumerate(){if state.collected(id){have[*r as usize]=true;}}
    let locations=catalog::locations();
    println!("\nUNCOLLECTED CHECKS ACCESSIBLE WITH SAVED REWARDS:");
    for l in &locations {
        if state.reward(l.id).is_none()||state.collected(l.id){continue;}
        if have[64+l.level as usize]&&l.requires.iter().any(|all|all.iter().all(|r|have[*r as usize])) {
            println!("source={} planet={} {} -> {}",l.id,l.level,l.name,label(state.placements[l.id]));
        }
    }
    let mut reached=have;
    loop {
        let before=reached;
        for l in &locations {
            if before[64+l.level as usize]&&l.requires.iter().any(|all|all.iter().all(|r|before[*r as usize])) {
                if let Some(r)=state.reward(l.id){reached[r as usize]=true;}
            }
        }
        if reached==before{break;}
    }
    let missing:Vec<_>=state.placements.iter().copied().filter(|&r|r!=0&&!reached[r as usize]).collect();
    println!("\ncomplete_from_saved_rewards={} missing_rewards={:?}",missing.is_empty(),missing);
    if !missing.is_empty(){println!("BLOCKED_BY_ROUTE_MODEL: keeping this save readable does not certify its old layout.");}
    println!("\nALL PLACEMENTS (claimed, source, planet, source name, reward):");
    for l in &locations {if let Some(r)=state.reward(l.id){println!("{}\t{}\t{}\t{}\t{}",state.collected(l.id),l.id,l.level,l.name,label(r));}}
    println!("\nThis checks the configured route model, not physical collision or affordability. No save is written.");
    Ok(())
}
