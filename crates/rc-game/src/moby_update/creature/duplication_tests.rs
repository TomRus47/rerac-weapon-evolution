use super::*;
use crate::collision_query::mobys::MobyGrid;
use rc_formats::moby_collision::{MobyCollision,MobyCollPrim};

/// Use a loader-registered template and real collision queries. Delivering a
/// hit directly to an actor would hide a missing spatial-grid registration.
fn copies()->(Sim,Vec<usize>) {
    let mut s=Sim::new(577,[120.,120.,10.],critter_pvars(),[150.,150.,10.]);
    let (info,anim)=s.classes.classes.get_mut(&577).unwrap();info.has_collision=true;info.ty=5;
    for seq in anim.as_mut().unwrap().sequences.iter_mut().flatten(){seq.header.sphere=[0.,0.,512.,768.];}
    s.table.mobys[1].has_collision=true;s.table.mobys[1].mode|=0x5000;
    scheduler::rebuild_matrix(&mut s.table.mobys[1],s.classes.classes[&577].1.as_ref());
    s.svc.grid=std::sync::Arc::new(MobyGrid::build(&mut s.table));
    let template=s.table.mobys[1].clone();s.table=MobyTable::new(s.table.mobys[..2].to_vec(),128);
    let mut raw=[0u32;8];raw[0]=1|4<<8|0x8003<<16;raw[6]=512f32.to_bits();raw[7]=768f32.to_bits();
    std::sync::Arc::make_mut(&mut s.svc.coll_classes).insert(577,MobyCollision{prims:vec![MobyCollPrim{raw}],..Default::default()});
    s.load();s.svc.bomb_evo.enabled=true;s.svc.bomb_evo.real_hp=true;s.svc.bomb_evo.pyro.enabled=true;
    s.svc.bomb_evo.pyro.xp_required=[100000.; 7];s.svc.bomb_evo.damage_hp[15][0]=Some(25.);
    for item in [16,19]{s.svc.bomb_evo.damage_hp[item][0]=Some(1500.);}
    enable_special_tracks(&mut s);s.svc.bomb_evo.xp_per_damage=0.1;
    for track in &mut s.svc.bomb_evo.extras{track.xp_required=[100000.; 7];}
    let mut ids=Vec::new();let mut w=World::new(&mut s.table,&s.hero,&mut s.rng,&s.classes,&mut s.svc,s.counter);
    for i in 0..20{ids.push(crate::weapon_evo::world::spawn_copy(&mut w,&template,
        [123.+(i%5)as f32*3.,120.+(i/5)as f32*3.,10.]).unwrap());}
    (s,ids)
}
#[test]
fn twenty_novalis_copies_have_complete_collision_before_moving() {
    let (mut s,ids)=copies();
    for &id in &ids {
        let m=&s.table.mobys[id];let p=m.position;let [x0,y0,x1,y1]=m.ba0;
        for y in y0..=y1{for x in x0..=x1{assert!(s.svc.grid.cell(x.into(),y.into()).contains(&(id as u16)),"copy {id} missing cell {x},{y}");}}
        let mut w=World::new(&mut s.table,&s.hero,&mut s.rng,&s.classes,&mut s.svc,s.counter);
        assert!(w.sphere_mobys_list(Pf::f(0.9),v4(p[0],p[1],p[2]+0.5),0,None,None).contains(&id));
    }
    let mut w=World::new(&mut s.table,&s.hero,&mut s.rng,&s.classes,&mut s.svc,s.counter);
    w.delete_moby(ids[0]);assert!(w.svc.grid.cells.iter().all(|cell|!cell.contains(&(ids[0]as u16))));
    for &id in &ids[1..]{assert!(w.svc.grid.cells.iter().any(|cell|cell.contains(&(id as u16))));}
}
#[test]
fn copies_take_repeated_bullets_fire_electricity_and_award_scaled_actual_hp() {
    let (mut s,ids)=copies();s.svc.bomb_evo.world_settings.enemy_health_multiplier=2.;
    let mut expected=1.;
    for item in [15,16,19,15,16,19,15,15] {
        {
            let mut w=World::new(&mut s.table,&s.hero,&mut s.rng,&s.classes,&mut s.svc,s.counter);
            for &id in &ids {
                let mut t=if item==15{let a=w.create_moby(305).unwrap();real_bullet(a)}else{real_bullet(0)};
                if item!=15{t.h1a=if item==16{176}else{177};t.b18=5;t.b19=if item==16{1}else{2};}
                let p=w.m(id).position;
                assert!(w.sphere_mobys_list(Pf::f(0.9),v4(p[0],p[1],p[2]+0.5),0,None,Some(&t)).contains(&id));
            }
        }
        s.tick();expected-=0.125;
        for &id in &ids{assert!((crate::targeting::record_health(&s.table.mobys[id]).unwrap()-expected).abs()<0.0001,"copy {id}, weapon {item}");}
        assert_eq!(crate::targeting::record_health(&s.table.mobys[1]),Some(1.));
    }
    assert!(ids.iter().all(|&id|s.table.mobys[id].state==99));
    let xp=s.svc.bomb_evo.pyro.xp+s.svc.bomb_evo.extras.iter().filter(|t|[15,19].contains(&t.id)).map(|t|t.xp).sum::<f32>();
    assert!((xp-400.).abs()<0.01,"20 copies, 200 effective HP each, 0.1 XP/HP: {xp}");
}
