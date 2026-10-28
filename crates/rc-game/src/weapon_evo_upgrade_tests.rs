use super::*;
use crate::hero::weapons::Weapons;

fn profile()->Evolution {
    let mut e=Evolution {enabled:true,refill_ammo_on_upgrade:true,xp_per_damage:1.,xp_required:[10.;7],..Default::default()};
    e.pyro.enabled=true;e.pyro.xp_required=[10.; 7];
    for (i,s) in e.stats.iter_mut().enumerate(){s.max_ammo=40+i as i32*10;}
    for (i,s) in e.pyro.stats.iter_mut().enumerate(){s.max_ammo=50+i as i32*10;}
    for t in &mut e.extras {
        t.enabled=true;t.xp_required=[10.; 7];
        for (i,p) in t.params.iter_mut().enumerate(){p.insert("max_ammo".into(),60.+i as f32*10.);}
    }
    e
}
#[test]
fn earned_upgrades_refill_only_the_upgraded_owned_weapon_and_only_once() {
    for item in [10,11,13,15,16,17,19,20,23,24,25] {
        let mut e=profile();let mut w=Weapons::default();w.ammo.fill(3);
        match item {
            10=>e.award_damage(10.,121),16=>e.award_pyro_damage(10.,5,176,true),
            _=>e.award_extra_damage(item,10.),
        }
        assert_eq!(e.weapon_level(item),Some(2));
        e.apply_upgrade_refills(&mut w,&[1;37]);
        assert_eq!(w.ammo[item as usize],if item==10{50}else if item==16{60}else{70});
        assert!(w.ammo.iter().enumerate().all(|(id,&n)|id==item as usize||n==3));
        w.ammo[item as usize]=1;e.apply_upgrade_refills(&mut w,&[1;37]);
        assert_eq!(w.ammo[item as usize],1);
    }
}
#[test]
fn multigrade_upgrade_uses_final_experimental_capacity_and_restore_never_refills() {
    let mut e=profile();e.stats[4].cluster.enabled=true;e.stats[4].experimental_max_ammo=Some(7);
    e.award_damage(100.,121);assert_eq!(e.level,5);
    let mut w=Weapons::default();e.apply_upgrade_refills(&mut w,&[1;37]);assert_eq!(w.ammo[10],7);
    w.ammo[10]=0;let progress=e.progress();e.restore_progress(&progress);
    e.apply_upgrade_refills(&mut w,&[1;37]);assert_eq!(w.ammo[10],0);
    let mut e=profile();e.award_damage(10.,121);let progress=e.progress();e.restore_progress(&progress);
    e.apply_upgrade_refills(&mut w,&[1;37]);assert_eq!(w.ammo[10],0,"pending reward must not survive a load");
}
#[test]
fn disabled_unowned_and_ammo_free_tracks_never_gain_ammo() {
    let mut e=profile();e.refill_ammo_on_upgrade=false;e.award_damage(10.,121);
    let mut w=Weapons::default();e.apply_upgrade_refills(&mut w,&[1;37]);assert_eq!(w.ammo[10],0);
    e.refill_ammo_on_upgrade=true;e.award_extra_damage(15,10.);
    e.apply_upgrade_refills(&mut w,&[0;37]);assert_eq!(w.ammo[15],0);
    for item in [8,9,14,18,21] {e.award_extra_damage(item,10.);}
    e.apply_upgrade_refills(&mut w,&[1;37]);assert!(w.ammo.iter().all(|&v|v==0));
    // Parsing rules must not depend on the author's approved release toggle.
    for (value, expected) in [("false",false),("on",true)] {
        let raw=format!("{}\n[general]\nrefill_ammo_on_upgrade={value}\n",include_str!("weapon_evo_release.ini"));
        assert_eq!(Evolution::parse(&raw).unwrap().refill_ammo_on_upgrade,expected);
    }
    let invalid=format!("{}\n[general]\nrefill_ammo_on_upgrade=invalid\n",include_str!("weapon_evo_release.ini"));
    assert!(Evolution::parse(&invalid).is_err());
}
