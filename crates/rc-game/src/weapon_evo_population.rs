//! Visit-local additions to native encounters. Configuration booleans are
//! eligibility filters, never fallbacks. Only successful copies spend the total
//! budget; actors killed later do not replenish it. No save or mission state.
use super::{radius, placement, spawn_copy_prepared, v, Settings};
use crate::moby_runtime::{Moby, MobyId};
use crate::moby_update::services::{pvar as pv, World};
use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Population {
    templates: HashMap<MobyId, Moby>,
    initialized: bool,
    capture_suspended: bool,
    requested: usize,
    pub spawned: usize,
    counts: [usize; 3],
    class_counts: HashMap<i16, usize>,
    source_counts: HashMap<MobyId, usize>,
    cursor: usize,
    next_tick: u64,
    reported_pending: bool,
}
/// Native HP is in units of 100 real HP. Never use current or scaled health.
pub fn category(hp: f32) -> Option<usize> {
    (hp.is_finite() && hp > 0.).then_some(if hp <= 1. { 0 } else if hp <= 2. { 1 } else { 2 })
}
fn allowed(s: &Settings, hp: f32) -> bool {
    category(hp).is_some_and(|c| [s.extra_enemy_prefer_small,
        s.extra_enemy_prefer_medium, s.extra_enemy_prefer_large][c])
}
/// Round-robin the selected categories, then the originals in each category.
/// Table ordering cannot let a long run of small enemies monopolize additions.
fn candidates(w: &World, state: &Population) -> Vec<MobyId> {
    let mut groups = [Vec::new(), Vec::new(), Vec::new()];
    let mut present = [false;3];
    for (&id, template) in &state.templates {
        let Some(m) = w.table.mobys.get(id) else { continue; };
        let Some(enemy) = w.svc.enemies.records.get(&id) else { continue; };
        if crate::targeting::record_health(m).is_some_and(|hp|!hp.is_finite()||hp<=0.) {continue;}
        // Hidden Amoeboid split children are reserved lives, not encounters.
        if matches!(m.o_class,865|866) && matches!(m.state,0xc|0xd) {continue;}
        if m.o_class == template.o_class && m.state > 0 && m.state < 128
            && allowed(&w.svc.bomb_evo.world_settings, enemy.base_hp) {
            let c=category(enemy.base_hp).unwrap();
            present[c]=true;
            if m.mode&1==0 {groups[c].push(id);}
        }
    }
    // Include dormant arrivals when reserving category shares, so visible small
    // enemies cannot spend the whole budget before medium/large actors wake.
    let minimum=(0..3).filter(|&c|present[c]).map(|c|state.counts[c]).min();
    for (c,group) in groups.iter_mut().enumerate() {
        if Some(state.counts[c])!=minimum {group.clear();}
        group.sort_by_key(|&id| (
            state.class_counts.get(&w.table.mobys[id].o_class).copied().unwrap_or(0),
            state.source_counts.get(&id).copied().unwrap_or(0),id));
    }
    let mut ids = Vec::new();
    for offset in 0..groups.iter().map(Vec::len).max().unwrap_or(0) {
        for c in 0..3 {
            let group = &groups[(state.spawned + c) % 3];
            if offset < group.len() { ids.push(group[offset]); }
        }
    }
    ids
}
fn private_path(w: &mut World, template: &mut Moby, offset: usize, shift: [f32; 3]) -> bool {
    let path = pv::i32(&template.pvars, offset);
    let Some(points) = usize::try_from(path).ok().and_then(|i| w.svc.splines.get(i)) else { return false; };
    if points.is_empty() { return false; }
    let points = points.iter().map(|point| {
        let mut p = *point;
        for axis in 0..3 { p[axis] = (f32::from_bits(p[axis]) + shift[axis]).to_bits(); }
        p
    }).collect();
    pv::set_i32(&mut template.pvars, offset, w.svc.splines.len() as i32);
    w.svc.splines.push(points);
    true
}
fn anchored_path(w:&mut World, template:&mut Moby, offset:usize, p:[f32;3])->bool {
    let anchor=usize::try_from(pv::i32(&template.pvars,offset)).ok()
        .and_then(|i|w.svc.splines.get(i)).and_then(|points|points.first()).copied();
    let Some(anchor)=anchor else {return false;};
    private_path(w,template,offset,std::array::from_fn(|i|p[i]-f32::from_bits(anchor[i])))
}

/// The native split searches a moby group for waiting children. Each added
/// encounter owns that entire tree: it never steals children from a placed
/// Amoeboid or alerts/completes its mission group. Children are reserved slots,
/// not additional entries in the requested encounter count.
fn amoeboid_family(w:&mut World,template:&Moby,p:[f32;3])->Option<MobyId> {
    let family:&[i16]=match template.o_class {
        572=>&[572,865,865,866,866,866,866],
        865=>&[865,866,866],866=>&[866],_=>return None,
    };
    if w.svc.groups.lists.len()>=crate::moby_update::scheduler::GROUPS
        || w.table.available_dynamic_slots(w.counter)<32+family.len()
        || family.iter().any(|&c|w.classes.info(c).is_none()) {return None;}
    let group=w.svc.groups.lists.len();
    w.svc.groups.lists.push(Some(Vec::new()));
    let mut ids=Vec::new();
    for (index,&class) in family.iter().enumerate() {
        let mut child=template.clone(); child.o_class=class;
        // All sizes use the same native layout and update. Native init assigns
        // their own stage HP/size; no active damage record is copied.
        pv::set_i32(&mut child.pvars,0x230,i32::from(index==0));
        pv::set_i32(&mut child.pvars,0x25c,-1);
        pv::set_i32(&mut child.pvars,0x260,-1);
        pv::set_i32(&mut child.pvars,0x248,-1);
        pv::set_i32(&mut child.pvars,0x258,0);
        let id=spawn_copy_prepared(w,&child,p,|w,id|w.mm(id).group=group as i8);
        let Some(id)=id else {
            for id in ids {w.delete_moby(id);}
            w.svc.groups.lists.pop();return None;
        };
        ids.push(id);
    }
    w.svc.groups.lists[group]=Some(ids.iter().map(|&id|id as u16).collect());
    ids.first().copied()
}
/// Class-specific adapters are deliberately small. Native initialization and AI
/// remain authoritative; never teleport the original or edit its shared path.
fn copy(w: &mut World, original: MobyId, template: &Moby, p: [f32; 3]) -> Option<MobyId> {
    if matches!(template.o_class,572|865|866) {return amoeboid_family(w,template,p);}
    let mut template = template.clone();
    let paths_before = w.svc.splines.len();
    let origin = w.table.mobys[original].position;
    let shift = [p[0]-origin[0], p[1]-origin[1], p[2]-origin[2]];
    match template.o_class {
        459 => {
            // Never attach a copy to a native dropship, camera or arrival gate.
            if !anchored_path(w,&mut template,0x164,p) {return None;}
            for field in [0x134,0x1c0,0x1c4,0x1e4] {pv::set_i32(&mut template.pvars,field,-1);}
            pv::set_i32(&mut template.pvars,0x130,0);
            pv::set_i32(&mut template.pvars,0x1b4,1);
            pv::set_i32(&mut template.pvars,0x1e8,1);
        }
        578 => {if !anchored_path(w,&mut template,0x134,p) {return None;}},
        749 => {
            for field in [0x1e4,0x1e8,0x1ec,0x1f0] {pv::set_i32(&mut template.pvars,field,-1);}
        }
        1440 => {pv::set_i32(&mut template.pvars,0x1b8,-1);},
        1231 => {
            pv::set_i32(&mut template.pvars,0x130,-1);
            pv::set_i32(&mut template.pvars,0x160,0);
        }
        217 => {
            if pv::i32(&template.pvars,0x434)>=0 && !private_path(w,&mut template,0x434,shift) {return None;}
            pv::set_i32(&mut template.pvars,0x424,0);
            pv::set_i32(&mut template.pvars,0x430,-1);
        }
        573 => {
            pv::set_i32(&mut template.pvars,0x248,-1);
            pv::set_i32(&mut template.pvars,0x270,-1);
            pv::set_i32(&mut template.pvars,0x27c,-1);
        }
        574 => {
            use crate::moby_update::classes::units::kerwan_trooper::pv as k;
            if template.pvars.len() < k::SIZE { return None; }
            let index = usize::try_from(pv::i32(&template.pvars, k::PATH_ID)).ok()?;
            let first = *w.svc.splines.get(index)?.first()?;
            let shift = std::array::from_fn(|i| p[i]-f32::from_bits(first[i]));
            if !private_path(w, &mut template, k::PATH_ID, shift) { return None; }
            for field in [k::LINK, k::RIDE, k::POD, k::JPATH, k::CUBOID] {
                pv::set_i32(&mut template.pvars, field, -1);
            }
            pv::set_u8(&mut template.pvars, k::HIDDEN, 0);
        }
        427 => {
            if template.pvars.len() < 0x210 { return None; }
            // Both retreat regions must follow the independent duplicate.
            for field in [0xe0, 0xe4] {
                if !private_path(w, &mut template, field, shift) {
                    w.svc.splines.truncate(paths_before);
                    return None;
                }
            }
            pv::set_i32(&mut template.pvars, 0xf8, 0); // Allocate its own gun.
        }
        _ => {},
    }
    let result = spawn_copy_prepared(w, &template, p, |_, _| {});
    if let Some(id) = result {
        let actual = w.table.mobys[id].position;
        if (actual[0]-p[0]).hypot(actual[1]-p[1]) <= radius(&w.table.mobys[id]) {
            return Some(id);
        }
        w.delete_moby(id);
    }
    // Failed native initialization cannot accumulate unused private paths.
    w.svc.splines.truncate(paths_before);
    None
}
/// Capture authored pvars immediately before a native initialization, including
/// actors created after the load pass. Each new table/visit starts a fresh pool.
/// Never snapshot a running damage/animation record as an initializer.
pub fn capture(w:&mut World,id:MobyId) {
    if w.svc.population.capture_suspended
        || w.svc.bomb_evo.world_settings.extra_enemies_per_level==0
        || (w.svc.population.initialized && w.svc.population.spawned>=w.svc.population.requested) {return;}
    let m=&w.table.mobys[id];
    if m.state==0 && super::population_class(m.o_class) {
        w.svc.population.templates.entry(id).or_insert_with(||m.clone());
        w.svc.population.next_tick=w.counter;
    }
}
/// Slots are reusable; class equality alone cannot identify an original life.
pub fn forget(state:&mut Population,id:MobyId) {
    if state.capture_suspended {return;}
    state.templates.remove(&id);
    state.source_counts.remove(&id);
}
pub fn begin(w: &mut World, templates: HashMap<MobyId, Moby>) {
    if w.svc.population.initialized { return; }
    let requested = w.svc.bomb_evo.world_settings.extra_enemies_per_level as usize;
    let mut captured=std::mem::take(&mut w.svc.population.templates);
    captured.extend(templates);
    w.svc.population = Population { templates:captured, requested, initialized: true, ..Default::default() };
    if requested > 0 {
        eprintln!("[Weapon Evolution] Extra enemies: target {requested}; selected HP categories only.");
        tick(w);
    }
}
pub fn tick(w: &mut World) {
    if w.svc.population.spawned >= w.svc.population.requested || w.counter < w.svc.population.next_tick { return; }
    let mut state = std::mem::take(&mut w.svc.population);
    // Native init runs recursively while copies are built. Never register copies
    // (including reserved Amoeboid children) as new original encounters.
    w.svc.population.capture_suspended=true;
    // Limit work per frame. Retry unmet additions instead of silently dropping
    // them at load time; keep the native combat/projectile pool available.
    let mut probes = 0;
    let mut added = 0;
    while state.spawned < state.requested && probes < 256 && added < 4 && w.table.free_slots >= 32 {
        let ids = candidates(w, &state);
        if ids.is_empty() { break; }
        let before = state.spawned;
        'sources: for id in ids {
            let r = radius(&w.table.mobys[id]);
            let mut origin = w.table.mobys[id].position;
            if w.table.mobys[id].o_class==577 && pv::i16(&w.table.mobys[id].pvars,0x20a)==0 {
                // Native initialization adds six units of hover height. Probe
                // around its home floor rather than lifting an already flying
                // source again or rejecting every floor below the short ray.
                origin[2]=pv::f(&w.table.mobys[id].pvars,0x1d8).to_f32();
            }
            // Probe multiple nearby rings, not just a single blocked circle.
            for probe in 0..96 {
                probes += 1;
                let angle = std::f32::consts::TAU * (probe as f32 / 24. + state.cursor as f32 * 0.381966);
                let distance = (2.*r + 0.75) * (1. + (probe/24) as f32 * 0.5);
                let p = [origin[0]+angle.cos()*distance, origin[1]+angle.sin()*distance, origin[2]];
                if w.line(v([origin[0],origin[1],origin[2]+r]), v([p[0],p[1],p[2]+r]), 2, Some(id)).is_none() {
                    if let Some(p) = placement(w, p, r, None) {
                        if copy(w, id, &state.templates[&id], p).is_some() {
                            state.counts[category(w.svc.enemies.records[&id].base_hp).unwrap()] += 1;
                            *state.class_counts.entry(state.templates[&id].o_class).or_default()+=1;
                            *state.source_counts.entry(id).or_default()+=1;
                            state.spawned += 1;
                            added += 1;
                            break 'sources;
                        }
                    }
                }
                if probes >= 256 { break 'sources; }
            }
        }
        state.cursor += 1;
        if state.spawned == before { break; }
    }
    if state.spawned == state.requested {
        eprintln!("[Weapon Evolution] Extra enemies: {}/{} created (small {}, medium {}, large {}).", state.spawned, state.requested, state.counts[0], state.counts[1], state.counts[2]);
        eprintln!("[Weapon Evolution] Extra enemies by native class: {:?}",state.class_counts);
        state.templates.clear();
    } else if added == 0 && !state.reported_pending {
        eprintln!("[Weapon Evolution] Extra enemies: {}/{} created; remaining additions await eligible enemies, actor slots or safe nearby space.", state.spawned, state.requested);
        state.reported_pending = true;
    }
    state.next_tick = w.counter + if added > 0 { 1 } else { 60 };
    w.svc.population = state;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_update::classes::bomb_water::tests::{Bench, pool};
    use crate::moby_update::scheduler;
    use crate::weapon_evo::world::Enemy;

    fn with_world(b: &mut Bench, f: impl FnOnce(&mut World)) {
        let mut w = World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc, b.counter);
        w.coll = Some(&b.coll);
        w.camera = b.camera;
        f(&mut w);
    }
    fn fixture(class: i16) -> (Bench, MobyId, HashMap<MobyId, Moby>) {
        let mut b = Bench::new(pool(10., -10.), &[class, 499], [45.,45.,10.]);
        let port = crate::moby_update::classes::ClassUpdate::every()
            .find(|u|u.classes().contains(&class) && u.reference_level()==if class==574 {3} else if class==459 {1} else {4}).unwrap();
        b.ct.classes.get_mut(&class).unwrap().0.update_fn=Some(port.key());
        b.ct.classes.get_mut(&class).unwrap().0.ty=5;
        b.svc.level=port.reference_level();
        let id = b.create(class);
        let m = &mut b.table.mobys[id];
        m.position = [20.,20.,10.,1.];
        m.mode = 0x20;
        m.has_collision = true;
        m.bsphere[3] = 700.;
        m.draw_dist = 255;
        m.update_dist = 255;
        m.pvars = vec![0; 0x2a0];
        pv::set_i32(&mut m.pvars, 0, 0x20);
        b.svc.splines = vec![vec![[20f32.to_bits(),20f32.to_bits(),10f32.to_bits(),0],
            [23f32.to_bits(),20f32.to_bits(),10f32.to_bits(),0],
            [23f32.to_bits(),23f32.to_bits(),10f32.to_bits(),0]]];
        if class == 574 {
            for field in [0x264,0x280,0x284,0x290,0x29c] { pv::set_i32(&mut m.pvars,field,-1); }
            pv::set_i32(&mut m.pvars,0x134,0);
        }
        if class==459 {
            pv::set_i32(&mut m.pvars,0x164,0);
            for field in [0x134,0x1c0,0x1c4,0x1e4] {pv::set_i32(&mut m.pvars,field,-1);}
            pv::set_i32(&mut m.pvars,0x1b4,1);
            pv::set_i32(&mut m.pvars,0x1e8,1);
        }
        let templates = HashMap::from([(id,m.clone())]);
        with_world(&mut b, |w| scheduler::run_moby(w,id));
        assert!(b.svc.enemies.records.contains_key(&id), "fixture must execute native initialization");
        (b,id,templates)
    }
    #[test]
    fn native_kerwan_copies_have_independent_paths_and_exact_total() {
        let (mut b, id, templates) = fixture(574);
        let source = b.table.mobys[id].clone();
        let original_path = b.svc.splines[0].clone();
        b.svc.bomb_evo.world_settings.extra_enemies_per_level = 5;
        b.svc.bomb_evo.world_settings.extra_enemy_prefer_small = false;
        b.svc.bomb_evo.world_settings.extra_enemy_prefer_medium = false;
        with_world(&mut b, |w| begin(w,templates));
        for _ in 0..120 { b.counter += 1; b.table.free_slot_pass(b.counter); with_world(&mut b,tick); }
        assert_eq!(b.svc.population.spawned,5);
        assert_eq!(b.table.mobys.iter().filter(|m| m.o_class==574 && m.state<128).count(),6);
        assert_eq!(b.table.mobys[id].position,source.position);
        assert_eq!(b.svc.splines[0],original_path);
        let mut paths = std::collections::HashSet::new();
        let actors:Vec<_> = b.table.mobys.iter().enumerate().filter(|(_,m)|m.o_class==574&&m.state<128).collect();
        for (i,m) in &actors {
            assert!(paths.insert(pv::i32(&m.pvars,0x134)));
            assert_eq!(b.svc.enemies.records[i].base_hp,3.);
            for (j,n) in &actors { if i!=j { assert!((m.position[0]-n.position[0]).hypot(m.position[1]-n.position[1]) > radius(m)+radius(n)); } }
        }
    }
    #[test]
    fn ng_plus_keeps_the_exact_additional_enemy_budget_and_native_categories() {
        let (mut b,_,templates)=fixture(427);
        let s=&mut b.svc.bomb_evo.world_settings;
        s.extra_enemies_per_level=5;s.extra_enemy_prefer_small=false;s.extra_enemy_prefer_large=false;
        s.ngplus_multiplier=2.;
        b.svc.bomb_evo.configure_ng_plus(true);
        assert_eq!(b.svc.bomb_evo.world_settings.extra_enemies_per_level,5);
        assert_eq!(b.svc.bomb_evo.world_settings.enemy_health_multiplier,2.);
        with_world(&mut b,|w|begin(w,templates));
        for _ in 0..120 {b.counter+=1;b.table.free_slot_pass(b.counter);with_world(&mut b,tick);}
        assert_eq!(b.svc.population.spawned,5);
        assert_eq!(b.table.mobys.iter().filter(|m|m.o_class==427&&m.state<128).count(),6);
        assert!(b.svc.enemies.records.values().all(|e|e.base_hp==2.));
    }
    #[test]
    fn medium_only_duplicates_native_two_hp_gunners() {
        let (mut b,_,templates) = fixture(427);
        let s = &mut b.svc.bomb_evo.world_settings;
        s.extra_enemies_per_level = 5;
        s.extra_enemy_prefer_small = false;
        s.extra_enemy_prefer_large = false;
        s.enemy_health_multiplier = 100.;
        with_world(&mut b,|w|begin(w,templates));
        for _ in 0..120 { b.counter += 1; b.table.free_slot_pass(b.counter); with_world(&mut b,tick); }
        assert_eq!(b.svc.population.spawned,5);
        assert_eq!(b.table.mobys.iter().filter(|m|m.o_class==427&&m.state<128).count(),6);
        assert!(b.svc.enemies.records.values().all(|e|e.base_hp==2.));
    }
    #[test]
    fn novalis_troopers_are_included_with_private_patrols_and_no_arrival_links() {
        let (mut b,id,templates)=fixture(459);
        let source=b.table.mobys[id].clone();let path=b.svc.splines[0].clone();
        b.svc.bomb_evo.world_settings.extra_enemies_per_level=5;
        with_world(&mut b,|w|begin(w,templates));
        for _ in 0..120 {b.counter+=1;b.table.free_slot_pass(b.counter);with_world(&mut b,tick);}
        assert_eq!(b.svc.population.spawned,5);
        assert_eq!(b.svc.splines[0],path);assert_eq!(b.table.mobys[id].position,source.position);
        let copies:Vec<_>=b.table.mobys.iter().enumerate().filter(|(i,m)|*i!=id&&m.o_class==459&&m.state<128).collect();
        assert_eq!(copies.len(),5);
        let paths:std::collections::HashSet<_>=copies.iter().map(|(_,m)|pv::i32(&m.pvars,0x164)).collect();assert_eq!(paths.len(),5);
        for (i,m) in copies {
            assert_eq!(b.svc.enemies.records[&i].base_hp,2.);
            for field in [0x134,0x1c0,0x1c4,0x1e4] {assert_eq!(pv::i32(&m.pvars,field),-1);}
        }
    }
    #[test]
    fn dormant_medium_arrivals_reserve_their_share_instead_of_twenty_small_copies() {
        let (mut b,id,templates)=fixture(459);
        let mut state=Population{templates,requested:20,..Default::default()};
        let small=b.create(577);b.table.mobys[small].state=1;b.table.mobys[small].mode=0x20;
        state.templates.insert(small,b.table.mobys[small].clone());
        b.svc.enemies.records.insert(small,Enemy{class:577,base_hp:1.});
        b.table.mobys[id].mode|=1;
        with_world(&mut b,|w|assert_eq!(candidates(w,&state),vec![small]));
        state.spawned=1;state.counts[0]=1;
        with_world(&mut b,|w|assert!(candidates(w,&state).is_empty(),"the next share waits for medium arrival"));
        b.table.mobys[id].mode&=!1;
        with_world(&mut b,|w|assert_eq!(candidates(w,&state),vec![id]));
    }
    #[test]
    fn amoeboid_copies_have_private_children_and_preserve_native_groups() {
        let mut b=Bench::new(pool(10.,-10.),&[572,865,866],[45.,45.,10.]);
        for class in [572,865,866] {
            let info=&mut b.ct.classes.get_mut(&class).unwrap().0;
            info.update_fn=Some(crate::moby_update::classes::amoeboid::UPDATE_FN);info.ty=5;
        }
        let mut template=Moby::init_instance(0,572,Some(&b.ct.classes[&572].0));
        template.mode|=0x5020;template.pvars=vec![0;0x2b0];
        for (o,ptr) in [(0,0x20),(0x10,0x60),(0x14,0xc0),(0x18,0x170)] {pv::set_i32(&mut template.pvars,o,ptr);}
        for o in [0x240,0x244,0x248,0x24c,0x25c,0x260] {pv::set_i32(&mut template.pvars,o,-1);}
        b.svc.bomb_evo.enabled=true;b.svc.bomb_evo.real_hp=true;
        b.svc.groups.lists=vec![Some(vec![0])];
        with_world(&mut b,|w| {
            let a=amoeboid_family(w,&template,[20.,20.,10.]).unwrap();
            let c=amoeboid_family(w,&template,[30.,20.,10.]).unwrap();
            assert_ne!(w.m(a).group,w.m(c).group);
            assert_eq!(w.svc.groups.lists[0],Some(vec![0]));
            for parent in [a,c] {
                let group=w.svc.groups.lists[w.m(parent).group as usize].as_ref().unwrap();
                assert_eq!(group.len(),7);
                assert_eq!(crate::targeting::record_health(w.m(parent)),Some(3.));
                assert_eq!(group.iter().filter(|&&id|w.m(id as usize).o_class==865).count(),2);
                assert_eq!(group.iter().filter(|&&id|w.m(id as usize).o_class==866).count(),4);
                assert!(group.iter().all(|&id|id as usize==parent||w.m(id as usize).state==0xc));
            }
        });
    }
    #[test]
    fn medium_filter_never_falls_back_to_kerwan_three_hp_enemies() {
        let (mut b,_,templates) = fixture(574);
        let s = &mut b.svc.bomb_evo.world_settings;
        s.extra_enemies_per_level=5;
        s.extra_enemy_prefer_small=false;
        s.extra_enemy_prefer_large=false;
        with_world(&mut b,|w|begin(w,templates));
        assert_eq!(b.svc.population.spawned,0);
        assert_eq!(b.svc.population.requested,5);
    }
    #[test]
    fn all_disabled_and_unknown_hp_do_not_fallback_and_all_selected_are_balanced() {
        let (mut b,id,templates) = fixture(574);
        let mut state=Population{templates,requested:5,..Default::default()};
        b.svc.bomb_evo.world_settings.extra_enemy_prefer_small=false;
        b.svc.bomb_evo.world_settings.extra_enemy_prefer_medium=false;
        b.svc.bomb_evo.world_settings.extra_enemy_prefer_large=false;
        with_world(&mut b,|w|assert!(candidates(w,&state).is_empty()));
        b.svc.bomb_evo.world_settings=Settings::default();
        for (class,hp) in [(577,1.),(427,2.)] {
            let new=b.create(class);
            b.table.mobys[new].o_class=class;
            b.table.mobys[new].state=1;
            b.table.mobys[new].mode=0x20;
            state.templates.insert(new,b.table.mobys[new].clone());
            b.svc.enemies.records.insert(new,Enemy{class,base_hp:hp});
        }
        for c in 0..3 { state.spawned=c; with_world(&mut b,|w| {
            let ids=candidates(w,&state);
            assert_eq!(category(w.svc.enemies.records[&ids[0]].base_hp),Some(c));
        }); }
        b.svc.enemies.records.remove(&id);
        with_world(&mut b,|w|assert!(!candidates(w,&state).contains(&id)));
    }
    #[test]
    fn blocked_slots_keep_budget_for_retry_and_reload_clears_it() {
        let (mut b,_,templates)=fixture(574);
        b.svc.bomb_evo.world_settings.extra_enemies_per_level=5;
        b.table.free_slots=0;
        with_world(&mut b,|w|begin(w,templates));
        assert_eq!(b.svc.population.spawned,0);
        b.counter=60;b.table.free_slot_pass(b.counter);
        with_world(&mut b,tick);
        b.counter+=1;b.table.free_slot_pass(b.counter);
        with_world(&mut b,tick);
        assert_eq!(b.svc.population.spawned,5);
        b.svc.reset_moby_transients();
        assert_eq!(b.svc.population.requested,0);
    }
    #[test]
    fn one_point_novalis_patrol_is_valid_and_copies_do_not_become_sources() {
        let (mut b,id,mut templates)=fixture(459);
        b.svc.splines[0].truncate(1);
        b.svc.bomb_evo.world_settings.extra_enemies_per_level=5;
        // Real native init accepts one-point patrols. The old adapter rejected
        // all 96 placements before trying to create a single actor.
        templates.get_mut(&id).unwrap().pvars=b.table.mobys[id].pvars.clone();
        with_world(&mut b,|w|begin(w,templates));
        for _ in 0..120 {b.counter+=1;b.table.free_slot_pass(b.counter);with_world(&mut b,tick);}
        assert_eq!(b.svc.population.spawned,5);
        assert_eq!(b.table.mobys.iter().filter(|m|m.o_class==459&&m.state<128).count(),6);
        assert_eq!(b.svc.splines.len(),6);
    }
    #[test]
    fn late_native_init_is_captured_and_slot_reuse_forgets_the_old_template() {
        let (mut b,id,templates)=fixture(459);
        let template=templates[&id].clone();
        b.svc.bomb_evo.world_settings.extra_enemies_per_level=2;
        // No eligible troop at load; it arrives later at a different position.
        with_world(&mut b,|w|begin(w,HashMap::new()));
        assert_eq!(b.svc.population.spawned,0);
        b.table.mobys[id]=template.clone();
        b.svc.splines[0][0][0]=30f32.to_bits();
        b.table.mobys[id].position[0]=30.;
        with_world(&mut b,|w|scheduler::run_moby(w,id));
        assert!(b.svc.population.templates.contains_key(&id));
        for _ in 0..120 {b.counter+=1;b.table.free_slot_pass(b.counter);with_world(&mut b,tick);}
        assert_eq!(b.svc.population.spawned,2);
        with_world(&mut b,|w|w.delete_moby(id));
        assert!(!b.svc.population.templates.contains_key(&id));
        // A death reload/revisit starts a new budget and captures the new table.
        b.svc.reset_moby_transients();
        b.table.mobys[id]=template;
        b.svc.splines[0][0][1]=30f32.to_bits();
        with_world(&mut b,|w|scheduler::run_moby(w,id));
        assert!(b.svc.population.templates.contains_key(&id));
        assert_eq!(b.svc.population.spawned,0);
        with_world(&mut b,|w|begin(w,HashMap::new()));
        assert!(b.svc.population.spawned>0);
    }

    #[test]
    fn recycled_source_slot_cannot_keep_its_previous_patrol_or_become_a_clone_source() {
        let (mut b,id,templates)=fixture(459);
        let mut template=templates[&id].clone();
        b.svc.bomb_evo.world_settings.extra_enemies_per_level=100;
        with_world(&mut b,|w|begin(w,templates));
        assert!(b.svc.population.spawned<100);
        assert_eq!(b.svc.population.templates.len(),1,"copies must not be captured recursively");
        with_world(&mut b,|w|w.delete_moby(id));
        assert!(!b.svc.population.templates.contains_key(&id));
        let next=b.svc.splines.len();
        b.svc.splines.push(vec![[35f32.to_bits(),35f32.to_bits(),10f32.to_bits(),0]]);
        pv::set_i32(&mut template.pvars,0x164,next as i32);
        b.table.mobys[id]=template;
        with_world(&mut b,|w|scheduler::run_moby(w,id));
        assert_eq!(pv::i32(&b.svc.population.templates[&id].pvars,0x164),next as i32);
    }

}
