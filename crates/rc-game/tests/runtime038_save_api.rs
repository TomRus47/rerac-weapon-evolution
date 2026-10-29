//! Settings/portable-card precedence must compose with mode-owned save roots.
//! Each environment case gets its own process; parallel tests never mutate a
//! shared RC_SAVE_DIR or configuration selector.
use std::{path::PathBuf,process::Command};

#[test]
fn configured_cards_keep_mode_isolation_and_environment_precedence() {
    let root=std::env::temp_dir().join(format!("wevo038-card-{}",std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for case in ["default","configured","environment","disabled","randomizer","weapons","randomizer_environment","none"] {
        let profile=root.join(format!("{case}.ini"));
        let mode=match case {"randomizer"|"randomizer_environment"=>"randomizer_mode=true", "weapons"=>"golden_bolts_to_weapons=true",_=>""};
        let mut text=format!("[general]\n{mode}\n");
        for grade in 1..=5 {text.push_str(&format!("[weapon.bomb_glove.level{grade}]\n"));}
        rc_game::weapon_evo::Evolution::parse(&text).expect("valid isolated card-test profile");
        std::fs::write(&profile,text).unwrap();
        let executable=std::env::current_exe().unwrap_or_else(|_|PathBuf::from(std::env::args_os().next().unwrap()));
        let output=Command::new(executable).args(["--exact","card_root_child","--nocapture"])
            .env("WEVO038_CARD_CASE",case).env("WEVO038_CARD_BASE",&root)
            .env("RC_WEAPON_EVOLUTION_INI",&profile).env_remove("RC_SAVE_DIR")
            .env_remove("RC_WEAPON_LOCATION_TEST").env_remove("RC_GEMLIK_GOLD_TEST")
            .output().unwrap();
        assert!(output.status.success(),"case {case}: {}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn card_root_child() {
    let Ok(case)=std::env::var("WEVO038_CARD_CASE") else {return;};
    let root=PathBuf::from(std::env::var_os("WEVO038_CARD_BASE").unwrap());
    let configured=root.join("portable");let base=root.join("config");let explicit=root.join("environment");
    if matches!(case.as_str(),"environment"|"randomizer_environment") {std::env::set_var("RC_SAVE_DIR",&explicit);}
    if case=="disabled" {std::env::set_var("RC_SAVE_DIR","0");}
    let selected=rc_game::memcard::save_root(
        (!matches!(case.as_str(),"default"|"none")).then_some(configured.clone()),
        (case!="none").then_some(base.as_path()));
    let actual=selected.map(|path|rc_game::weapon_evo::golden::card_root(path,rc_game::weapon_evo::golden::enabled()));
    let expected=match case.as_str() {
        "default"=>Some(base.join("rerac/memcard")),
        "configured"=>Some(configured),"environment"=>Some(explicit),
        "randomizer"=>Some(configured.join("randomizer")),
        "weapons"=>Some(configured.join("golden_bolts_to_weapons")),
        "randomizer_environment"=>Some(explicit.join("randomizer")),
        "disabled"|"none"=>None,_=>panic!("unknown case"),
    };
    assert_eq!(actual,expected);
}
