//! Cross-module contracts for distance-based pursuit and accelerated AI probes.
use rc_game::{hero::Hero,moby_runtime::{Moby,MobyTable,ClassInfo},rng::Rng,ps2v::Pf};
use rc_game::moby_update::{scheduler,services::{World,Services,ClassTable},creature::target};
use rc_game::weapon_evo::world;

#[test]
fn territory_queries_are_read_only_and_perception_and_chase_factors_compose() {
    let mut m=Moby::init_instance(0,1202,None);m.state=1;
    let mut hero_moby=Moby::init_instance(1,11,None);hero_moby.position=[15.,0.,10.,1.];
    let mut table=MobyTable::new(vec![m,hero_moby],4);
    let mut classes=ClassTable::default();
    for class in [1202,11] {classes.classes.insert(class,(ClassInfo{ty:5,..Default::default()},None));}
    let mut hero=Hero::new();hero.pos=[Pf::f(15.),Pf::ZERO,Pf::f(10.),Pf::ONE];
    let mut rng=Rng::new();let mut svc=Services::new();
    svc.splines.push([[-10.,-10.,10.,0.],[10.,-10.,10.,0.],[10.,10.,10.,0.],[-10.,10.,10.,0.]].map(|p|p.map(f32::to_bits)).to_vec());
    let original=svc.splines.clone();
    let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,0);w.hero_moby=Some(1);
    assert_eq!(target::acquire_in(&w,0,30.,Some(0)).kind,2);
    w.svc.bomb_evo.world_settings.enemy_perception_multiplier=2.;
    assert_eq!(target::acquire_in(&w,0,60.,Some(0)).kind,0);
    assert_eq!(world::chase_range(&w,0,30.),60.);
    w.svc.bomb_evo.world_settings.enemy_chase_range_multiplier=2.;
    assert_eq!(world::chase_range(&w,0,30.),120.);
    assert_eq!(world::chase_range(&w,1,30.),30.,"vendor is not an enemy");
    for _ in 0..100 {assert!(world::chase_in_area(&w,0,0,[30.,0.,10.,1.]));}
    assert_eq!(w.svc.splines,original);
    for value in ["0","-1","NaN","inf","11"] {assert!(world::Settings::default().parse_field("enemy_chase_range_multiplier",value).is_err());}
    assert!(world::Settings::default().parse_field("enemy_chase_range_multiplier","2").unwrap());
}

#[test]
fn fast_periodic_probes_sample_once_per_world_tick_and_reset_on_slot_reuse() {
    let mut table=MobyTable::new(vec![Moby::init_instance(0,1202,None)],4);
    let hero=Hero::new();let mut rng=Rng::new();let classes=ClassTable::default();let mut svc=Services::new();
    svc.bomb_evo.world_settings.enemy_speed_multiplier=2.;
    let mut w=World::new(&mut table,&hero,&mut rng,&classes,&mut svc,7);
    assert!(world::motion::periodic_sample(&mut w,0));
    for _ in 0..3 {assert!(!world::motion::periodic_sample(&mut w,0));}
    w.counter=14;assert!(world::motion::periodic_sample(&mut w,0));
    w.svc.motion.forget(0);assert!(world::motion::periodic_sample(&mut w,0));
    w.svc.bomb_evo.world_settings.enemy_speed_multiplier=1.;
    assert!(world::motion::periodic_sample(&mut w,0));assert!(world::motion::periodic_sample(&mut w,0));
}

#[test]
fn distant_enemy_activity_can_expand_without_overflow_or_moving_scenery() {
    let make=|id,class| {let mut m=Moby::init_instance(id,class,None);m.state=1;m.visible=0;m.mode=rc_game::moby_runtime::mode::TARGETABLE;m.update_dist=20;m.position=[40.,0.,0.,1.];m};
    let table=MobyTable::new(vec![make(0,1202),make(1,11)],2);
    let groups=Default::default();let camera=[Pf::ZERO;4];
    assert!(scheduler::build_active_list(&table,camera,&groups).0.is_empty());
    let (list,targets)=scheduler::build_active_list_scaled(&table,camera,&groups,|m|if m.o_class==1202 {3.}else{1.});
    assert_eq!(list,vec![0]);assert_eq!(targets,vec![0]);
    assert_eq!(table.mobys[0].update_dist,20);
}
