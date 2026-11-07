//! Opt-in gadget combat, isolated from story gadget state and weapon grades.
//!
//! Runtime leases use (slot, class, allocation generation), never the changing
//! matrix revision in uid_hi or a naked reusable moby index.
//! Freeze owns neither hostile projectiles nor their clocks. Damage still uses
//! accepted native HP writes; drill contact cannot mint XP from a held pose.
use std::{collections::HashMap, sync::{OnceLock,RwLock}};
use crate::{hero::{Hero,items::{HitSink,ItemEnv},anim::AnimCtl,physics::{from_f32x3,to_f32x3}},
    moby_runtime::{Moby,MobyId,MobyTable,mode},moby_update::{services::{World,HitTemplate},creature},pad::button,ps2v::Pf,rng::Rng};
#[path="gadget_combat_freezer.rs"]
pub mod freezer;
type V=[f32;3];
const DT:f32=1./60.;
pub const DRILL_HIT:u32=1<<23;
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Settings {
    pub swing_enabled:bool,pub pull_range:f32,pub pull_speed:f32,pub stun_seconds:f32,
    pub grapple_enabled:bool,pub grapple_range:f32,pub grapple_speed:f32,
    pub drill_enabled:bool,pub drill_range:f32,pub drill_spin:f32,pub drill_dps:f32,
    pub ice_enabled:bool,pub ice_ammo:u16,pub recharge_seconds:f32,pub ice_interval:u16,pub ice_radius:f32,pub freeze_seconds:f32,
    pub ice_speed:f32,pub ice_gravity:f32,pub ice_range:f32,pub ice_size:f32,
    pub ice_color:u32,pub mist_color:u32,pub explosion_color:u32,pub enemy_color:u32,
    pub holo_enabled:bool,pub holo_movement:f32,pub holo_animation:f32,
}
impl Default for Settings {fn default()->Self{Self{
    swing_enabled:false,pull_range:20.,pull_speed:16.,stun_seconds:2.,grapple_enabled:false,grapple_range:25.,grapple_speed:18.,
    drill_enabled:false,drill_range:5.,drill_spin:720.,drill_dps:50.,ice_enabled:false,ice_ammo:10,recharge_seconds:10.,ice_interval:60,
    ice_radius:6.,freeze_seconds:4.,ice_speed:18.,ice_gravity:18.,ice_range:50.,ice_size:100.,
    ice_color:0xC0EEFF,mist_color:0x80CCFF,explosion_color:0x80DCFF,enemy_color:0x408CFF,
    holo_enabled:false,holo_movement:1.,holo_animation:1.,
}}}
impl Settings {
    pub fn parse(&mut self,section:&str,key:&str,value:&str)->Result<(),String>{
        let b=|| match value.to_ascii_lowercase().as_str(){"on"|"true"|"1"=>Ok(true),"off"|"false"|"0"=>Ok(false),_=>Err(format!("[{section}] {key}: expected Off/On"))};
        let n=|lo:f32,hi:f32|->Result<f32,String>{value.parse::<f32>().ok().filter(|v|v.is_finite()&&(*v>=lo&&*v<=hi))
            .ok_or_else(||format!("[{section}] {key}: expected {lo}..{hi}"))};
        let integer=|lo:u16,hi:u16|->Result<u16,String>{value.parse::<u16>().ok().filter(|v|*v>=lo&&*v<=hi).ok_or_else(||format!("[{section}] {key}: expected integer {lo}..{hi}"))};
        let color=|fallback| if value.eq_ignore_ascii_case("default"){Ok(fallback)}else{u32::from_str_radix(value.trim_start_matches('#'),16).ok().filter(|v|*v<=0xFFFFFF)
            .ok_or_else(||format!("[{section}] {key}: expected RRGGBB"))};
        match (section,key){
            ("gadget.swingshot","enabled")=>self.swing_enabled=b()?,
            ("gadget.swingshot","pull_range")=>self.pull_range=n(1.,100.)?,
            ("gadget.swingshot","pull_speed")=>self.pull_speed=n(1.,60.)?,
            ("gadget.swingshot","stun_seconds")=>self.stun_seconds=n(0.,30.)?,
            ("gadget.swingshot","surface_grapple_enabled")=>self.grapple_enabled=b()?,
            ("gadget.swingshot","grapple_range")=>self.grapple_range=n(1.,100.)?,
            ("gadget.swingshot","grapple_speed")=>self.grapple_speed=n(1.,60.)?,
            ("gadget.trespasser","enabled")=>self.drill_enabled=b()?,
            ("gadget.trespasser","drill_range")=>self.drill_range=n(0.5,20.)?,
            ("gadget.trespasser","rotation_degrees_per_second")=>self.drill_spin=n(0.,3600.)?,
            ("gadget.trespasser","damage_per_second")=>self.drill_dps=n(0.,1_000_000.)?,
            ("gadget.hydrodisplacer","enabled")=>self.ice_enabled=b()?,
            ("gadget.hydrodisplacer","max_ammo")=>self.ice_ammo=integer(0,999)?,
            ("gadget.hydrodisplacer","recharge_seconds")=>self.recharge_seconds=n(0.1,3600.)?,
            ("gadget.hydrodisplacer","fire_interval_ticks")=>self.ice_interval=integer(1,600)?,
            ("gadget.hydrodisplacer","freeze_radius")=>self.ice_radius=n(0.1,50.)?,
            ("gadget.hydrodisplacer","freeze_seconds")=>self.freeze_seconds=n(0.,60.)?,
            ("gadget.hydrodisplacer","projectile_speed")=>self.ice_speed=n(1.,100.)?,
            ("gadget.hydrodisplacer","projectile_gravity")=>self.ice_gravity=n(0.,100.)?,
            ("gadget.hydrodisplacer","projectile_range")=>self.ice_range=n(1.,100.)?,
            ("gadget.hydrodisplacer","ice_size_percent")=>self.ice_size=n(10.,500.)?,
            ("gadget.hydrodisplacer","ice_color")=>self.ice_color=color(0xC0EEFF)?,
            ("gadget.hydrodisplacer","mist_color")=>self.mist_color=color(0x80CCFF)?,
            ("gadget.hydrodisplacer","explosion_color")=>self.explosion_color=color(0x80DCFF)?,
            ("gadget.hydrodisplacer","frozen_enemy_color")=>self.enemy_color=color(0x408CFF)?,
            ("gadget.hologuise","enabled")=>self.holo_enabled=b()?,
            ("gadget.hologuise","movement_multiplier")=>self.holo_movement=n(0.1,10.)?,
            ("gadget.hologuise","animation_multiplier")=>self.holo_animation=n(0.1,10.)?,
            _=>return Err(format!("Unknown gadget setting [{section}] {key}")),
        };Ok(())
    }
}
fn config()->&'static RwLock<Settings>{static CONFIG:OnceLock<RwLock<Settings>>=OnceLock::new();CONFIG.get_or_init(||RwLock::new(Settings::default()))}
pub fn publish(s:Settings){*config().write().unwrap_or_else(|p|p.into_inner())=s;}
pub fn published()->Settings{*config().read().unwrap_or_else(|p|p.into_inner())}
#[derive(Clone,Debug,Default,PartialEq)]
pub struct HeroOutput {
    pub spin:f32,pub holding:bool,pub previous_holding:bool,
    pub rope:Option<V>,pub cable_pose:u8,pub previous_cable_pose:u8,
    pub grapple_step:Option<V>,pub cable_latched:bool,
    /// Only this sequence may be replaced when returning to locomotion.
    pub pose_sequence:Option<u8>,pub pose_item:i32,
    /// Transient equip input latch; independent of ammunition and saved state.
    pub freezer_ready:bool,pub freezer_armed:bool,
}
/// Allocation lifetime is independent of uid_hi: its low half is the native
/// matrix revision, and its high half is merely the array index. Neither is a
/// reliable allocation-generation counter.
#[derive(Clone,Debug,Default)]
pub struct Lifetimes { generations:HashMap<MobyId,u64> }
impl Lifetimes {
    pub fn spawned(&mut self,id:MobyId){
        let generation=self.generations.entry(id).or_default();
        *generation=generation.checked_add(1).expect("gadget allocation generation overflow");
    }
    fn generation(&self,id:MobyId)->u64{self.generations.get(&id).copied().unwrap_or(0)}
}
#[derive(Clone,Copy,Debug,PartialEq)]
struct Lease {id:MobyId,class:i16,generation:u64}
impl Lease {
    fn of(w:&World,id:MobyId)->Self{Self{id,class:w.m(id).o_class,generation:w.svc.gadget_lifetimes.generation(id)}}
    fn same(self,w:&World)->bool{w.table.mobys.get(self.id).is_some_and(|m|m.o_class==self.class)
        &&w.svc.gadget_lifetimes.generation(self.id)==self.generation}
    fn valid(self,w:&World)->bool{self.same(w)&&w.m(self.id).state<0x80}
}
#[derive(Clone,Debug)]
struct Frozen {lease:Lease,until:u64,ice_until:u64,rotation:[f32;4],ambient:Option<[u8;4]>,color:Option<u32>,drill:bool,keep_rows:bool,saved_rows:[[f32;4];3]}
#[derive(Clone,Debug)]
struct Cable {target:Option<Lease>,point:V,tip:V,phase:u8,age:u16}
/// Only the physical collision presence is borrowed by capture. Primitive
/// masks, targetability and native damage intake remain owned by the enemy.
#[derive(Clone,Debug)]
struct DrillCollision {lease:Lease,enabled:bool}
#[derive(Clone,Debug)]
struct Upright {lease:Lease,base:[f32;4],start:f32,left:u8,yaw:f32}
#[derive(Clone,Debug)]
struct Ice {pos:V,vel:V,travel:f32,age:u16,settings:Settings}
#[derive(Clone,Debug,Default)]
pub struct Runtime {
    upright:Vec<Upright>,frozen:HashMap<MobyId,Frozen>,cable:Option<Cable>,drill:Option<Lease>,drill_pivot:Option<(Lease,V,f32)>,drill_collision:Option<DrillCollision>,
    ice:Vec<Ice>,
    pub ice_ammo:Option<u16>,pub ice_visit:Option<u32>,next_ice:u64,spin:f32,sound:Option<(MobyId,i32)>,
    pub drawn_ice:Vec<(V,f32,u32)>,
    pub frost_bursts:Vec<crate::weapon_evo::wrench::SlamVisual>,
    pub recharge:freezer::Recharge,
    /// Current frozen emitters; regenerated each simulation frame, never actors.
    pub frost_mist:Vec<(V,f32)>,
}
impl Runtime {
    pub fn freeze_count(&self)->usize{self.frozen.len()}
    pub fn freezer_hud(&self,settings:Settings)->Option<freezer::Hud>{
        settings.ice_enabled.then(||self.recharge.hud(self.ice_ammo.unwrap_or(settings.ice_ammo),settings))
    }
    pub fn sync_ammo(&mut self,level:u32,remaining:Option<u16>,s:Settings){
        if self.ice_visit!=Some(level){self.ice_visit=Some(level);self.ice_ammo=Some(remaining.unwrap_or(s.ice_ammo).min(s.ice_ammo));self.recharge=Default::default();}
    }
}
fn add(a:V,b:V)->V{std::array::from_fn(|i|a[i]+b[i])}
fn sub(a:V,b:V)->V{std::array::from_fn(|i|a[i]-b[i])}
fn scale(a:V,k:f32)->V{a.map(|v|v*k)}
fn length(a:V)->f32{a.into_iter().map(|v|v*v).sum::<f32>().sqrt()}
fn unit(a:V)->V{scale(a,1./length(a).max(0.00001))}
fn dot(a:V,b:V)->f32{(0..3).map(|i|a[i]*b[i]).sum()}
fn pos(m:&Moby)->V{[m.position[0],m.position[1],m.position[2]]}
fn target_point(m:&Moby)->V{let mut p=pos(m);p[2]+=0.7;p}
fn enemy(w:&World,id:MobyId)->bool{let m=w.m(id);m.state<0x80&&m.mode&(mode::HIDDEN|mode::NO_UPDATE)==0
    && crate::weapon_evo::world::motion::enemy(m) && w.classes.info(m.o_class).is_some_and(|c|c.ty==5)
    && !w.svc.bomb_evo.target_is_terminal(id)
    && creature::header(w,id).damage.is_none_or(|o|creature::pf(w,id,o)>0.)}
fn movable(w:&World,id:MobyId)->bool{enemy(w,id)&&!crate::weapon_evo::world::boss(w.m(id).o_class)
    && !matches!(w.m(id).o_class,688|660|435|326|336|409|1274|1278|1445)}
fn ground_class(h:&Hero,t:&MobyTable,class:i16)->bool{
    // Only a real contacted native pad wins, not an unrelated nearby decoration.
    h.air_ticks==0&&h.ground_moby.and_then(|id|t.mobys.get(id)).is_some_and(|m|m.o_class==class&&m.state<0x80)
}
fn allowed(h:&Hero)->bool{h.health>0&&h.mode==0&&h.frozen==0&&h.f13ff==0&&h.items.slot.state==2
    &&matches!(h.group,0|1|2|4|5|0xc)&&!h.in_water_groups()}
fn restore_frozen_transform(w:&mut World,f:&Frozen){
    let m=w.mm(f.lease.id);m.rotation=f.rotation;
    if f.keep_rows{m.mode|=mode::KEEP_ROWS;for n in 0..3{m.rows[n]=f.saved_rows[n];}}
}
fn clear_tint(w:&mut World,rt:&mut Runtime){for f in rt.frozen.values_mut(){if f.lease.valid(w){if let Some(a)=f.ambient.take(){w.mm(f.lease.id).ambient=a;}}}}
fn hold(rt:&mut Runtime,w:&World,id:MobyId,until:u64,color:Option<u32>,drill:bool){
    let m=w.m(id);let lease=Lease::of(w,id);
    let f=rt.frozen.entry(id).or_insert(Frozen{lease,until,ice_until:if color.is_some(){until}else{0},rotation:m.rotation,ambient:None,color,drill,keep_rows:m.mode&mode::KEEP_ROWS!=0,saved_rows:[m.rows[0],m.rows[1],m.rows[2]]});
    if f.lease!=lease{*f=Frozen{lease,until,ice_until:if color.is_some(){until}else{0},rotation:m.rotation,ambient:None,color,drill,keep_rows:m.mode&mode::KEEP_ROWS!=0,saved_rows:[m.rows[0],m.rows[1],m.rows[2]]};}
    f.until=f.until.max(until);if color.is_some(){f.color=color;f.ice_until=f.ice_until.max(until);}f.drill=drill;
}
fn sound_stop(rt:&mut Runtime,w:&mut World){if let Some((id,slot))=rt.sound.take(){w.release_sound(slot,id);}}
fn release_drill(rt:&mut Runtime,w:&mut World,throw:bool){
    sound_stop(rt,w);
    if let Some(saved)=rt.drill_collision.take(){
        // A dead or recycled actor must never regain collision from an old grip.
        if saved.lease.valid(w)&&enemy(w,saved.lease.id){
            w.mm(saved.lease.id).has_collision=saved.enabled;
        }
    }
    rt.drill_pivot=None;
    if let Some(lease)=rt.drill.take(){
        if lease.valid(w){
            if let Some(f)=rt.frozen.remove(&lease.id){
                if let Some(ambient)=f.ambient{w.mm(lease.id).ambient=ambient;}
                if f.keep_rows{let m=w.mm(lease.id);m.mode|=mode::KEEP_ROWS;for n in 0..3{m.rows[n]=f.saved_rows[n];}}

                if throw{
                    // Six frames to return upright; only the victim moves.
                    let start=w.m(lease.id).rotation[0];
                    rt.upright.push(Upright{lease,base:f.rotation,start,left:6,yaw:w.hero.rot[2].to_f32()});
                    hold(rt,w,lease.id,w.counter+7,None,false);
                    if let Some(frozen)=rt.frozen.get_mut(&lease.id){frozen.rotation=f.rotation;frozen.ice_until=f.ice_until;frozen.until=frozen.until.max(f.ice_until);frozen.color=f.color;}
                }else{w.mm(lease.id).rotation=f.rotation;
                    if f.ice_until>w.counter{let mut retained=f;retained.until=retained.ice_until;retained.drill=false;retained.ambient=None;rt.frozen.insert(lease.id,retained);}
                }
            }
        }
    }
}

fn seek(w:&World,origin:V,direction:V,range:f32,pull:bool)->Option<MobyId>{
    w.table.mobys.iter().enumerate().filter(|(id,_)|if pull{movable(w,*id)}else{enemy(w,*id)})
        .filter_map(|(id,m)|{let delta=sub(target_point(m),origin);let d=length(delta);let facing=dot(unit(delta),direction);
            if d>range||facing<0.85{return None;}
            if w.line(from_f32x3(origin),from_f32x3(target_point(m)),2,w.hero_moby)
                .is_some_and(|hit|hit.moby!=Some(id)&&length(sub(to_f32x3(hit.point),origin))+0.3<d){return None;}
            Some((id,d*(2.-facing)))})
        .min_by(|a,b|a.1.total_cmp(&b.1)).map(|v|v.0)
}
/// Acquisition requires a collision reported by this tick's native hero
/// capsule/wall resolver. Range remains a defensive upper bound, not a vacuum.
/// Once held, collision is deliberately absent and acquisition is not repeated.
fn drill_contact(w:&World,direction:V,range:f32)->Option<MobyId>{
    let capsule=(w.hero.cap_hit!=0).then_some(w.hero.cap_moby).flatten();
    [capsule,w.hero.wall_moby].into_iter().flatten().find(|&id|{
        if id>=w.table.mobys.len()||!enemy(w,id)||crate::weapon_evo::world::boss(w.m(id).o_class){return false;}
        let delta=sub(target_point(w.m(id)),to_f32x3(w.hero.body_point));
        length(delta)<=range&&dot(unit(delta),direction)>0.2
    })
}
/// True freeze skips the scheduler's animation and AI, but never its HP inbox.
pub fn frozen(w:&World,id:MobyId)->bool{w.svc.gadget_combat.frozen.get(&id)
    .is_some_and(|f|f.lease.valid(w)&&w.counter<f.until&&enemy(w,id))}
/// Before native updates: restore tint, expire leases and advance launched ice.
pub fn world_tick(w:&mut World){
    let mut rt=std::mem::take(&mut w.svc.gadget_combat);clear_tint(w,&mut rt);
    rt.recharge.tick(&mut rt.ice_ammo,w.svc.bomb_evo.gadget_settings,w.counter,w.hero.health>0&&w.svc.game_mode==0);
    if rt.drill.is_some_and(|lease|!lease.valid(w)||!enemy(w,lease.id)){
        release_drill(&mut rt,w,false);
    }
    if w.hero.health<=0{
        for f in rt.frozen.values(){if f.lease.valid(w){restore_frozen_transform(w,f);}}
        sound_stop(&mut rt,w);release_drill(&mut rt,w,false);rt.frozen.clear();rt.upright.clear();rt.cable=None;rt.ice.clear();rt.drawn_ice.clear();rt.frost_bursts.clear();rt.frost_mist.clear();}
    if w.hero.mode!=0{sound_stop(&mut rt,w);release_drill(&mut rt,w,false);rt.cable=None;}
    rt.frozen.retain(|_,f|{
        let keep=f.lease.valid(w)&&w.counter<f.until&&enemy(w,f.lease.id);
        if !keep&&f.lease.valid(w){restore_frozen_transform(w,f);}
        keep
    });
    rt.upright.retain_mut(|u|{
        if !u.lease.valid(w){return false;}
        u.left=u.left.saturating_sub(1);let t=1.-u.left as f32/6.;
        let m=w.mm(u.lease.id);m.rotation=u.base;
        m.rotation[0]=u.base[0]+(u.start-u.base[0])*(1.-t);
        let mut delta=[u.yaw.cos()*0.08,u.yaw.sin()*0.08,0.,0.];
        creature::walker::move_collide(w,u.lease.id,0.5,0.3,0.02,&mut delta,0);
        u.left>0
    });
    rt.frost_bursts.retain(|burst|burst.appearance(w.counter).is_some());
    let mut ice=std::mem::take(&mut rt.ice);rt.drawn_ice.clear();
    ice.retain_mut(|i|{
        let from=i.pos;i.vel[2]-=i.settings.ice_gravity*DT;let next=add(from,scale(i.vel,DT));i.age+=1;i.travel+=length(sub(next,from));
        let hit=w.line(from_f32x3(from),from_f32x3(next),2,w.hero_moby);
        i.pos=hit.as_ref().map_or(next,|h|to_f32x3(h.point));
        if hit.is_some()||i.travel>=i.settings.ice_range||i.age>=600{
            for id in 0..w.table.mobys.len(){if enemy(w,id)&&length(sub(target_point(w.m(id)),i.pos))<=i.settings.ice_radius{
                hold(&mut rt,w,id,w.counter+(i.settings.freeze_seconds*60.).round()as u64,Some(i.settings.enemy_color),false);
            }}
            // A bounded renderer-owned blast shares the exact freeze centre/radius.
            if rt.frost_bursts.len()>=8{rt.frost_bursts.remove(0);}
            let color=i.settings.explosion_color;
            rt.frost_bursts.push(crate::weapon_evo::wrench::SlamVisual{centre:i.pos,radius:i.settings.ice_radius,
                color:((color&255)<<16)|(color&0xff00)|((color>>16)&255),born:w.counter});
            frost_impact_sound(w,i.pos);
            // One coherent radial frost puff, not an active bomb/damage actor.
            for n in 0..12{let a=n as f32*std::f32::consts::TAU/12.;mist(w,add(i.pos,[a.cos()*i.settings.ice_radius*0.5,a.sin()*i.settings.ice_radius*0.5,0.3]),i.settings.explosion_color,0.8);}
            false
        }else{rt.drawn_ice.push((i.pos,0.35*i.settings.ice_size/100.,i.settings.ice_color));if i.age%3==0{mist(w,i.pos,i.settings.mist_color,0.3);}true}
    });rt.ice=ice;
    w.svc.gadget_combat=rt;
}

/// Native Hoven sinking-death ice: class 0x52a, sound 0 (hero::fx::Ice).
/// A fixed-position one-shot needs neither a visible ice actor nor a loop slot.
fn frost_impact_sound(w:&mut World,point:V){
    let Some(owner)=w.hero_moby else{return};
    let event=crate::moby_update::services::SoundEvent{index:0,flags:0,moby:owner,
        o_class:w.m(owner).o_class,sound_class:crate::hero::damage::ICE_CLASS,pos:point,tick:w.counter};
    w.svc.sounds.push(event);
    if let Some(sound)=w.sound.as_deref_mut(){sound.play_class_sound(&event,w.rng);}
}
fn drill_hit(w:&World,dps:f32)->HitTemplate{HitTemplate{attacker:w.hero_moby,flags:0x10000,b18:0,b19:1,h1a:188,
    damage:Pf::f(dps/100.),dir:[Pf::ZERO,Pf::ZERO,Pf::ONE,Pf::b(0x45af_df66)],w20:1|DRILL_HIT}}
fn mist(w:&mut World,point:V,color:u32,size:f32){
    w.part60(size,[point[0],point[1],point[2],1.],[0.,0.,0.015,0.],rgba(color,0x38),18,0,-1);
    // Enter the native TNT smoke phase directly. Generation 2 prevents spark
    // splitting; a cosmetic RNG keeps mist from changing combat/seed draws.
    if let Some(parts)=w.particles.as_deref_mut(){
        let mut rng=Rng::new();rng.srand((w.counter as u32)^point[0].to_bits()^point[1].to_bits());
        let position=[point[0],point[1],point[2],1.].map(f32::to_bits);
        if let Some(id)=crate::particles::type11::spawn(parts,&mut rng,(size*120000.).to_bits(),0.005f32.to_bits(),position,
            [0.,0.,0.01,0.].map(f32::to_bits),rgba(color,0x38),rgba(0xFFFFFF,0x08),18,0,2,1){
            let record=&mut parts.pool.recs[id];
            crate::particles::rec::set_f(record,0xc,(size*120000.).to_bits());record[3]=0x44;
            *w.svc.fx.part_spawns.entry(11).or_default()+=1;
        }
    }
}
fn rgba(c:u32,a:u8)->u32{let r=(c>>16)&255;let g=(c>>8)&255;let b=c&255;((a as u32)<<24)|((b/2)<<16)|((g/2)<<8)|(r/2)}
/// Called after native gadgets: real targets/pads have already claimed their action.
pub fn item_tick(hero:&mut Hero,table:&mut MobyTable,_anim:&mut dyn AnimCtl,env:&ItemEnv,hits:&mut dyn HitSink,rng:&mut Rng){
    let id=hero.items.slot.id;let fire=env.pad.held&(button::CIRCLE|button::R2)!=0;
    // Menu handoff can suppress gameplay buttons for one frame while the
    // physical equip button is still held. That is not a real fire release.
    let equip_fire=fire||env.pad.physical_buttons.is_some_and(|(held,_)|held&(button::CIRCLE|button::R2)!=0);
    let direction=aim_direction(hero,env);
    let start=if matches!(id,12|26){hand_origin(hero,env)}else{launch_origin(hero,env,direction)};
    let mut out=HeroOutput{previous_holding:hero.gadget_combat.holding,previous_cable_pose:hero.gadget_combat.cable_pose,pose_sequence:hero.gadget_combat.pose_sequence,pose_item:hero.gadget_combat.pose_item,..Default::default()};
    hits.world(table,hero,rng,env.frame as u64,&mut |w|{
        let s=w.svc.bomb_evo.gadget_settings;let mut rt=std::mem::take(&mut w.svc.gadget_combat);
        rt.sync_ammo(w.svc.level,None,s);
        let native_swing=hero.swing.pull_ok||hero.swing.swing_ok||matches!(hero.group,0xd|0xe);
        let native_drill=ground_class(hero,w.table,615);
        let native_hydro=ground_class(hero,w.table,341)||matches!(hero.state,0x38..=0x3a);
        let can=allowed(hero)&&w.svc.game_mode==0;
        out.freezer_ready=can&&id==22&&s.ice_enabled&&!native_hydro;
        out.freezer_armed=out.freezer_ready&&(!equip_fire||(hero.gadget_combat.freezer_ready&&hero.gadget_combat.freezer_armed));
        let drill_grounded=can&&grounded_drill(hero);
        if !drill_grounded||!fire||id!=26||!s.drill_enabled||native_drill{
            sound_stop(&mut rt,w);release_drill(&mut rt,w,can&&id==26&&!native_drill);
        }
        if drill_grounded&&fire&&id==26&&s.drill_enabled&&!native_drill{
            rt.spin=(rt.spin+s.drill_spin.to_radians()*DT)%std::f32::consts::TAU;
            if rt.drill.is_none(){if let Some(target)=drill_contact(w,direction,s.drill_range){
                let lease=Lease::of(w,target);let (pivot,radius)=drill_bounds(w,target);
                rt.drill=Some(lease);rt.drill_pivot=Some((lease,pivot,radius));
                rt.drill_collision=Some(DrillCollision{lease,enabled:w.m(target).has_collision});
                // Capture one real idle pose when available, without touching AI state.
                let m=w.mm(target);m.anim.seq_a=0;m.anim.seq_b=0;m.anim.frame_a=0;m.anim.frame_b=0;m.anim.t=0.;
            }}
            if let Some(lease)=rt.drill{
                let same=lease.same(w);
                let hp=if same{creature::header(w,lease.id).damage.map(|o|creature::pf(w,lease.id,o))}else{None};
                let defeated=same&&(hp.is_some_and(|hp|hp<=0.)||w.svc.bomb_evo.target_is_terminal(lease.id));
                if !same||(!lease.valid(w)&&!defeated){
                    release_drill(&mut rt,w,false);
                }else if defeated{
                    // Native HP intake owns death, drops and XP. No replacement actor.
                    release_drill(&mut rt,w,false);
                }else{
                    // Remove body collision before the next hero-scene snapshot;
                    // direct drill hits still use native accepted-damage intake.
                    w.mm(lease.id).has_collision=false;
                    hold(&mut rt,w,lease.id,w.counter+2,None,true);
                    let (pivot,radius)=rt.drill_pivot.filter(|(p,_,_)|*p==lease).map(|(_,p,r)|(p,r)).unwrap_or(([0.,0.,0.7],0.5));
                    let anchor=add(start,scale(direction,1.+(radius*0.65).clamp(0.2,1.)));
                    place_drill_target(w,lease.id,anchor,direction,rt.spin,pivot);
                    let t=drill_hit(w,s.drill_dps);w.deliver_hit(lease.id,&t);out.holding=true;
                }
            }
        }
        if out.holding{
            if rt.sound.is_none(){if let Some(owner)=w.hero_moby{
                let slot=w.play_sound_as(2,4,owner,crate::hero::suck_cannon::CLASS);
                if slot>=0{rt.sound=Some((owner,slot));}
            }}
        }else{sound_stop(&mut rt,w);}
        if !can||id!=12||!s.swing_enabled||native_swing{rt.cable=None;}
        if can&&id==12&&s.swing_enabled&&!native_swing{
            if rt.cable.is_none()&&env.pad.pressed&(button::CIRCLE|button::R2)!=0{
                let target=seek(w,start,direction,s.pull_range,true);
                let surface=if target.is_none()&&s.grapple_enabled{
                    w.line(from_f32x3(start),from_f32x3(add(start,scale(direction,s.grapple_range))),2,w.hero_moby)
                        .filter(|hit|surface_allowed(to_f32x3(hit.normal),hit.moby.map(|id|w.m(id).o_class)))
                        .map(|hit|to_f32x3(hit.point))
                }else{None};
                if target.is_some()||surface.is_some(){
                    let point=target.map_or_else(||surface.unwrap(),|id|target_point(w.m(id)));
                    rt.cable=Some(Cable{target:target.map(|id|Lease::of(w,id)),point,tip:start,phase:0,age:0});
                    if let Some(owner)=w.hero_moby{w.play_sound_as(0,0,owner,208);}
                }
            }
            if let Some(mut c)=rt.cable.take(){
                c.age+=1;
                if let Some(lease)=c.target{if lease.valid(w){c.point=target_point(w.m(lease.id));}else{c.phase=2;}}
                if !fire||c.age>240{c.phase=2;}
                if c.phase==0{let d=sub(c.point,c.tip);if length(d)<0.65{c.phase=1;if let Some(owner)=w.hero_moby{w.play_sound_as(1,0,owner,208);w.play_sound_as(2,0,owner,208);}}else{c.tip=add(c.tip,scale(unit(d),(48.*DT).min(length(d))));}}
                else if c.phase==1{
                    c.tip=c.point;
                    if let Some(lease)=c.target{
                        let destination=add(start,scale(direction,1.8));let d=sub(destination,c.point);
                        if length(d)<0.7{hold(&mut rt,w,lease.id,w.counter+(s.stun_seconds*60.)as u64,None,false);c.phase=2;}
                        else{hold(&mut rt,w,lease.id,w.counter+2,None,false);let step=scale(unit(d),(s.pull_speed*DT).min(length(d)));
                            let mut movement=[step[0],step[1],step[2],0.];creature::walker::move_collide(w,lease.id,0.5,0.3,0.02,&mut movement,0);}
                    }else{
                        let d=sub(c.point,to_f32x3(hero.body_point));let remaining=(length(d)-1.).max(0.);
                        if remaining<0.15{c.phase=2;}else{let step=scale(unit(d),(s.grapple_speed*DT).min(remaining));
                            // Stop one body radius short; the normal world resolver owns walls.
                            let from=to_f32x3(hero.body_point);let to=add(from,step);
                            if w.line(from_f32x3(from),from_f32x3(add(to,scale(unit(d),0.6))),2,w.hero_moby).is_none(){out.grapple_step=Some(step);}else{c.phase=2;}
                        }
                    }
                }else{let d=sub(start,c.tip);c.tip=add(c.tip,scale(unit(d),(80.*DT).min(length(d))));}
                out.rope=Some(c.tip);out.cable_latched=c.phase==1;out.cable_pose=if c.phase==2{0}else if c.target.is_none()&&c.phase==1{2}else{1};
                if c.phase!=2||length(sub(start,c.tip))>0.1{rt.cable=Some(c);}
            }
        }
        if can&&fire&&out.freezer_armed&&id==22&&s.ice_enabled&&!native_hydro&&w.counter>=rt.next_ice&&rt.ice_ammo.unwrap_or(0)>0&&rt.ice.len()<32{
            rt.ice_ammo=rt.ice_ammo.map(|a|a-1);rt.next_ice=w.counter+s.ice_interval as u64;
            rt.ice.push(Ice{pos:start,vel:scale(direction,s.ice_speed),travel:0.,age:0,settings:s});
            if let Some(owner)=w.hero_moby{w.play_sound_as(1,0,owner,208);}
        }
        w.svc.gadget_combat=rt;
    });
    if out.holding{
        hero.vel=[Pf::ZERO;4];hero.momentum=[Pf::ZERO;4];hero.speed=Pf::ZERO;hero.fwd_speed=Pf::ZERO;
    }
    if let Some(to)=out.rope{
        if let Some(hook)=hero.swing.item.hook.as_mut(){hook.pos=to;}
        hero.swing.item.rope=Some(crate::hero::swingshot::Rope{from:start,to,phase:env.frame as f32*0.2,wavelength:1.,amplitude:0.04});}
    hero.gadget_combat=out;
}
pub fn surface_allowed(normal:V,class:Option<i16>)->bool{
    matches!(class,Some(500|501|502|505|511))||(length(normal)>0.0001&&unit(normal)[2]<0.35)
}
/// Ice tint is a draw-time overlay; restore it before any native class update.
pub fn draw_status(w:&mut World){
    let mut rt=std::mem::take(&mut w.svc.gadget_combat);rt.frost_mist.clear();
    for f in rt.frozen.values_mut(){if f.lease.valid(w)&&w.counter<f.until{
        if let Some(color)=f.color{let m=w.mm(f.lease.id);f.ambient=Some(m.ambient);m.ambient=[((color>>16)&255)as u8/2,((color>>8)&255)as u8/2,(color&255)as u8/2,m.ambient[3]];
            if w.counter<f.ice_until&&rt.frost_mist.len()<64 {
                let m=w.m(f.lease.id);let size=m.scale.abs().clamp(0.7,2.5);
                rt.frost_mist.push((target_point(m),size));
            }
        }
    }}w.svc.gadget_combat=rt;
}
/// A portable faceted ice mesh: eight triangles, no level-local class/model IDs.
pub fn ice_quads(rt:&Runtime)->Vec<crate::hero::tesla::BeamQuad>{
    let mut out=Vec::new();for &(p,r,c) in &rt.drawn_ice{
        let ring=[[r,0.,0.],[0.,r,0.],[-r,0.,0.],[0.,-r,0.]];
        for sign in [-1.,1.]{for n in 0..4{let tip=add(p,[0.,0.,r*sign]);let a=add(p,ring[n]);let b=add(p,ring[(n+1)%4]);
            out.push(crate::hero::tesla::BeamQuad{fx:crate::hero::tesla::FX_CORE,corners:[tip,a,b,b],st:[[0.5,0.5];4],rgba:[rgba(c,0x70);4]});
        }}
    }out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hero::{anim::RecordingAnim,items::{ItemData,HandItem}},moby_runtime::ClassInfo,
        moby_update::{Services,ClassTable},pad::PadState};
    fn actor(id:usize,class:i16,p:V)->Moby{
        let mut m=Moby{index:id as u32,o_class:class,uid_hi:(id as u32+1)*65536,has_class:true,
            state:1,position:[p[0],p[1],p[2],1.],mode:0x4020,pvars:vec![0;0x238],has_collision:true,scale:1.,..Default::default()};
        crate::moby_update::services::pvar::set_i32(&mut m.pvars,0,0x20);
        crate::moby_update::services::pvar::set_ff(&mut m.pvars,0x20,3.);m
    }
    fn classes()->ClassTable{let mut c=ClassTable::default();for class in [577,1106,660]{c.classes.insert(class,(ClassInfo{ty:5,scale:1.,..Default::default()},None));}c}
    struct Sink {svc:Services,classes:ClassTable}
    impl HitSink for Sink {
        fn sphere(&mut self,_:&mut MobyTable,_:Pf,_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<usize>{None}
        fn line(&mut self,_:&mut MobyTable,_:[Pf;4],_:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>>{None}
        fn world(&mut self,t:&mut MobyTable,h:&Hero,r:&mut Rng,c:u64,f:&mut dyn FnMut(&mut World))->bool{
            let mut w=World::new(t,h,r,&self.classes,&mut self.svc,c);f(&mut w);true
        }
    }
    fn hero(item:i32)->Hero{
        let mut h=Hero::new();h.health=4;h.items.slot.id=item;h.items.slot.state=2;
        if item==26{h.cap_moby=Some(1);h.cap_hit=1;} // Simulated native body contact. 
        h.body_point=from_f32x3([0.,0.,1.]);h.ground_point=from_f32x3([0.,0.,-50.]);
        h.items.slot.item=Some(HandItem{o_class:match item{12=>208,26=>188,_=>1251},mstate:2,anim:crate::moby_runtime::Moby::default().anim,snapshot:None,scale:1.,position:[0.,0.,1.],rows:[[0;4];3],hit_timer:0,flight:Default::default()});h
    }
    fn fire(h:&mut Hero,t:&mut MobyTable,s:&mut Sink,frame:i32,held:bool){
        if h.items.slot.id==22&&frame==1&&held{fire(h,t,s,0,false);}
        let data=ItemData::default();let pad=PadState{held:if held{button::CIRCLE}else{0},pressed:if frame==1&&held{button::CIRCLE}else{0},..Default::default()};
        let e=ItemEnv{data:&data,pad:&pad,frame,hero_moby:0,coll:None,mobys:None,camera:Some(([0.;3],[1.,0.,0.])),camera_up:None,targets:&[]};
        item_tick(h,t,&mut RecordingAnim::default(),&e,s,&mut Rng::new());
    }
    #[test]
    fn config_rejects_unknown_nonfinite_out_of_bounds_and_fractional_ammo(){
        let mut s=Settings::default();for (section,key,value) in [
            ("gadget.swingshot","pull_speed","NaN"),("gadget.trespasser","rotation_degrees_per_second","inf"),
            ("gadget.hydrodisplacer","max_ammo","1.5"),("gadget.hydrodisplacer","freeze_seconds","-1"),
            ("gadget.hydrodisplacer","ice_color","1000000"),("gadget.hologuise","not_a_setting","1")]{assert!(s.parse(section,key,value).is_err());}
        s.parse("gadget.hydrodisplacer","ice_color","default").unwrap();assert_eq!(s.ice_color,0xC0EEFF);
    }
    #[test]
    fn surfaces_allow_walls_ceilings_and_crates_but_not_floor(){
        assert!(!surface_allowed([0.,0.,1.],None));assert!(surface_allowed([1.,0.,0.],None));
        assert!(surface_allowed([0.,0.,-1.],None));assert!(surface_allowed([0.,0.,1.],Some(500)));
    }
    #[test]
    fn native_swingshot_target_wins_and_boss_or_ship_cannot_be_pulled(){
        let mut h=hero(12);let mut t=MobyTable::new(vec![actor(0,0,[0.;3]),actor(1,577,[4.,0.,0.]),actor(2,1106,[3.,0.,0.]),actor(3,660,[2.,0.,0.])],0);
        let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.swing_enabled=true;
        h.swing.swing_ok=true;fire(&mut h,&mut t,&mut sink,1,true);assert!(sink.svc.gadget_combat.cable.is_none());
        h.swing.swing_ok=false;fire(&mut h,&mut t,&mut sink,1,true);
        assert_eq!(sink.svc.gadget_combat.cable.as_ref().unwrap().target.unwrap().id,1);
    }
    #[test]
    fn native_trespasser_pad_wins_and_release_stops_capture(){
        let mut h=hero(26);h.ground_point=from_f32x3([0.;3]);h.ground_moby=Some(2);
        let mut t=MobyTable::new(vec![actor(0,0,[0.;3]),actor(1,577,[3.,0.,0.]),actor(2,615,[0.;3])],0);
        let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.drill_enabled=true;
        fire(&mut h,&mut t,&mut sink,1,true);assert!(sink.svc.gadget_combat.drill.is_none());
        h.ground_point=from_f32x3([0.,0.,-50.]);h.ground_moby=None;fire(&mut h,&mut t,&mut sink,1,true);assert!(h.gadget_combat.holding);
        fire(&mut h,&mut t,&mut sink,2,false);assert!(!h.gadget_combat.holding);assert_eq!(h.gadget_combat.spin,0.);assert!(sink.svc.gadget_combat.drill.is_none());
    }
    #[test]
    fn ice_ammo_ignores_tank_water_state_switching_and_checkpoint_reset(){
        let mut h=hero(22);let mut t=MobyTable::new(vec![actor(0,0,[0.;3])],0);
        let mut sink=Sink{svc:Services::new(),classes:classes()};sink.svc.bomb_evo.gadget_settings.ice_enabled=true;
        sink.svc.bomb_evo.gadget_settings.ice_ammo=2;sink.svc.bomb_evo.gadget_settings.ice_interval=1;
        fire(&mut h,&mut t,&mut sink,1,true);assert_eq!(sink.svc.gadget_combat.ice_ammo,Some(1));
        fire(&mut h,&mut t,&mut sink,2,false);assert_eq!(sink.svc.gadget_combat.ice_ammo,Some(1));
        fire(&mut h,&mut t,&mut sink,3,true);assert_eq!(sink.svc.gadget_combat.ice_ammo,Some(0));
        fire(&mut h,&mut t,&mut sink,4,true);assert_eq!(sink.svc.gadget_combat.ice.len(),2);
        let mut session=crate::game_state::SessionState::default();session.hydro_combat_ammo=Some((1,0));session.hero_init(4);
        assert_eq!(session.hydro_combat_ammo,Some((1,0)));
        let mut runtime=Runtime::default();runtime.sync_ammo(1,Some(0),sink.svc.bomb_evo.gadget_settings);
        assert_eq!(runtime.ice_ammo,Some(0));runtime.sync_ammo(2,None,sink.svc.bomb_evo.gadget_settings);assert_eq!(runtime.ice_ammo,Some(2));
    }
    #[test]
    fn ice_freezes_creature_and_boss_but_never_hostile_projectile_and_expires_exactly(){
        let h=hero(22);let mut t=MobyTable::new(vec![actor(0,0,[0.;3]),actor(1,577,[1.,0.,0.]),actor(2,1106,[2.,0.,0.]),actor(3,409,[1.,0.,0.])],0);
        let mut svc=Services::new();let c=classes();let mut rng=Rng::new();
        let settings=Settings{freeze_seconds:1.,ice_range:0.1,..Default::default()};
        svc.gadget_combat.ice.push(Ice{pos:[0.,0.,1.],vel:[10.,0.,0.],travel:0.,age:0,settings});
        let mut w=World::new(&mut t,&h,&mut rng,&c,&mut svc,10);world_tick(&mut w);
        assert!(frozen(&w,1));assert!(frozen(&w,2));assert!(!frozen(&w,3));assert_eq!(w.svc.gadget_combat.freeze_count(),2);
        w.counter=69;assert!(frozen(&w,1));w.counter=70;world_tick(&mut w);assert!(!frozen(&w,1));
    }
    #[test]
    fn frozen_tint_restores_and_slot_reuse_does_not_inherit_freeze(){
        let h=hero(22);let mut t=MobyTable::new(vec![actor(0,0,[0.;3]),actor(1,577,[2.,0.,0.])],0);
        let mut svc=Services::new();let c=classes();let mut rng=Rng::new();let mut w=World::new(&mut t,&h,&mut rng,&c,&mut svc,1);
        let mut rt=Runtime::default();hold(&mut rt,&w,1,3,Some(0x408CFF),false);w.svc.gadget_combat=rt;
        let original=w.m(1).ambient;draw_status(&mut w);assert_ne!(w.m(1).ambient,original);
        w.counter=3;world_tick(&mut w);assert_eq!(w.m(1).ambient,original);
        let mut rt=std::mem::take(&mut w.svc.gadget_combat);hold(&mut rt,&w,1,100,None,false);w.svc.gadget_combat=rt;
        crate::moby_update::scheduler::rebuild_matrix(w.mm(1),None);assert!(frozen(&w,1));
        w.svc.gadget_lifetimes.spawned(1);assert!(!frozen(&w,1));
    }
    #[test]
    fn crystal_mesh_uses_portable_finite_geometry(){
        let mut rt=Runtime::default();rt.drawn_ice.push(([1.,2.,3.],0.4,0xC0EEFF));let quads=ice_quads(&rt);
        assert_eq!(quads.len(),8);assert!(quads.iter().flat_map(|q|q.corners).flatten().all(f32::is_finite));
    }
    #[test]
    fn drill_hp_and_nanotech_xp_are_based_on_committed_damage_not_pose_frames(){
        let h=hero(26);let mut t=MobyTable::new(vec![actor(0,0,[0.;3]),actor(1,577,[2.,0.,0.])],0);
        let c=classes();let mut svc=Services::new();let mut rng=Rng::new();
        svc.bomb_evo.enabled=true;svc.bomb_evo.real_hp=true;svc.bomb_evo.xp_per_damage=0.1;
        svc.bomb_evo.world_settings.enemy_damage_by_health=true;svc.bomb_evo.world_settings.nanotech_xp_per_upgrade=100000.;
        for frame in 1..=60 {
            let mut w=World::new(&mut t,&h,&mut rng,&c,&mut svc,frame);
            let hit=drill_hit(&w,60.);w.deliver_hit(1,&hit);w.deliver_hit(1,&hit);
            assert!(!creature::damage::frozen_damage(&mut w,1));
        }
        assert!((crate::moby_update::services::pvar::ff(&t.mobys[1].pvars,0x20)-2.4).abs()<0.001);
        assert!((svc.bomb_evo.nanotech.xp-6.).abs()<0.001);
        let xp=svc.bomb_evo.nanotech.xp;
        for frame in 61..=120{let mut w=World::new(&mut t,&h,&mut rng,&c,&mut svc,frame);assert!(!creature::damage::frozen_damage(&mut w,1));}
        assert_eq!(svc.bomb_evo.nanotech.xp,xp);
    }

}

/// Never use generic quiet HP subtraction for a boss. Its own intake decides
/// whether a hit is legal in this shield/phase, even while its pose is frozen.
pub(crate) fn frozen_boss_damage(w:&mut World,id:MobyId)->bool{
    let class=w.m(id).o_class;if !crate::weapon_evo::world::boss(class){return false;}
    let pending=w.svc.hits.real_hp.frozen_next(id).is_some()||w.svc.hits.records.iter().any(|h|h.target==id);
    if !pending&&class!=388{return true;}
    let (position,rotation,animation)=(w.m(id).position,w.m(id).rotation,w.m(id).anim);
    if w.svc.bomb_evo.real_hp_enabled(){w.svc.hits.publish_real_head(w.table,id);}
    use crate::moby_update::classes::units::{blarg_boss,umbris_beast,veldin_boss,qwark_ship};
    match class{1051=>blarg_boss::frozen_hits(w,id),1106=>umbris_beast::frozen_hits(w,id),
        1422=>veldin_boss::intake(w,id),388=>qwark_ship::frozen_hits(w,id),_=>{}}
    if w.m(id).state<0x80&&creature::header(w,id).damage.is_none_or(|o|creature::pf(w,id,o)>0.){
        let m=w.mm(id);m.position=position;m.rotation=rotation;m.anim=animation;
    }
    true
}

/// Native hand chains can be at the hip (or unavailable), unlike a gun muzzle.
/// Use a finite chest-height origin in that case; never launch ice at the feet.
fn launch_origin(hero:&Hero,env:&ItemEnv,direction:V)->V{
    let feet=to_f32x3(hero.pos);
    let chest=add(feet,[0.,0.,0.95]);
    let raw=crate::hero::guns::item_point(hero,env,0);
    let valid=raw.into_iter().all(f32::is_finite)&&length(sub(raw,chest))<=2.5
        &&raw[2]>=feet[2]+0.6&&raw[2]<=feet[2]+2.2;
    if valid{raw}else{
        let mut p=add(chest,scale(direction,0.55));
        p[2]=p[2].max(feet[2]+0.65);p
    }
}
/// Hand gadgets use the native item chain, even when the muzzle is below the
/// chest. The raised fallback belongs only to thrown ice, not to the cable.
fn hand_origin(hero:&Hero,env:&ItemEnv)->V{
    let raw=crate::hero::guns::item_point(hero,env,0);
    let feet=to_f32x3(hero.pos);
    if hero.items.slot.item.is_some()&&raw.into_iter().all(f32::is_finite)&&length(sub(raw,feet))<4.{return raw;}
    let yaw=hero.rot[2].to_f32();add(feet,[yaw.cos()*0.5,yaw.sin()*0.5,0.45])
}
/// Remember a model-space centre once per allocation. Re-reading its rotated
/// world bound each tick would make the pivot chase itself around the drill.
fn drill_bounds(w:&World,id:MobyId)->(V,f32){
    let m=w.m(id);let k=m.scale.abs()/1024.;
    if let Some(s)=w.classes.anim(m.o_class).and_then(|c|c.sequence(0)).map(|q|q.header.sphere){
        if s.into_iter().all(f32::is_finite)&&s[3]>0.{return ([s[0]*k,s[1]*k,s[2]*k],(s[3]*k).max(0.1));}
    }
    let centre=sub([m.bsphere[0]/1024.,m.bsphere[1]/1024.,m.bsphere[2]/1024.],pos(m));
    let rows=[m.rows[0],m.rows[1],m.rows[2]];
    if centre.into_iter().all(f32::is_finite)&&length(centre)<20.&&m.bsphere[3]>0.
        &&rows.iter().all(|r|((r[0]*r[0]+r[1]*r[1]+r[2]*r[2])-1.).abs()<0.02){
        return (rows.map(|r|dot([r[0],r[1],r[2]],centre)),(m.bsphere[3].abs()/1024.).max(0.1));
    }
    ([0.,0.,0.7],0.5)
}
/// Root compensation keeps the same visible centre at the nozzle through a
/// complete turn. Publish the matrix now: the scheduler already ran this frame.
fn place_drill_target(w:&mut World,id:MobyId,centre:V,direction:V,spin:f32,pivot:V){
    let pitch=-direction[2].atan2(direction[0].hypot(direction[1]));
    let yaw=direction[1].atan2(direction[0]);
    let rows=rc_formats::moby_light::instance_rows([spin,pitch,yaw],w.m(id).mode).map(|r|r.map(f32::from_bits));
    let offset:V=std::array::from_fn(|axis|(0..3).map(|row|rows[row][axis]*pivot[row]).sum());
    let class=w.classes.anim(w.m(id).o_class);
    let m=&mut w.table.mobys[id];m.mode&=!mode::KEEP_ROWS;
    m.rotation=[spin,pitch,yaw,0.];
    let root=sub(centre,offset);m.position=[root[0],root[1],root[2],1.];
    crate::moby_update::scheduler::rebuild_matrix(m,class);
}
/// A latch suppresses voluntary translation; surface pull still supplies its
/// own collision-resolved displacement. Extending/retracting cables do not lock.
pub(crate) fn cable_holds_motion(hero:&Hero,pad:&crate::pad::PadState)->bool{
    allowed(hero)&&hero.items.slot.id==12&&hero.gadget_combat.cable_latched
        &&pad.held&(button::CIRCLE|button::R2)!=0
}
/// Filter only the native movement copy, never the camera/menu input record.
/// Releasing fire clears the pose before locomotion, so an unchanged run state
/// cannot keep the custom idle or launch animation for one additional frame.
pub(crate) fn prepare_hero_input(hero:&mut Hero,pad:&mut crate::pad::PadState){
    let fire=pad.held&(button::CIRCLE|button::R2)!=0;
    if !fire{
        hero.gadget_combat.cable_pose=0;hero.gadget_combat.cable_latched=false;
        hero.gadget_combat.grapple_step=None;hero.gadget_combat.holding=false;
    }
    let cable=cable_holds_motion(hero,pad);
    let drill=allowed(hero)&&hero.items.slot.id==26&&hero.gadget_combat.holding&&fire;
    if !(cable||drill){return;}
    let mask=button::UP|button::DOWN|button::LEFT|button::RIGHT|button::FLICK
        |if cable{button::CROSS}else{0};
    pad.lx=Pf::ZERO;pad.ly=Pf::ZERO;pad.analog_copy[2]=Pf::ZERO;pad.analog_copy[3]=Pf::ZERO;
    for bits in [&mut pad.held,&mut pad.pressed,&mut pad.released,&mut pad.prev_held,
        &mut pad.raw,&mut pad.raw_pressed,&mut pad.raw_released,&mut pad.prev_raw,
        &mut pad.held_unmirrored,&mut pad.pressed_unmirrored,&mut pad.released_unmirrored,&mut pad.held_store]{*bits&=!mask;}
    for p in &mut pad.pressure[..4]{*p=Pf::ZERO;}
    if cable{pad.pressure[6]=Pf::ZERO;}
    pad.no_direction=true;pad.stick_active=false;pad.nothing_held=pad.held==0;
}
fn aim_direction(hero:&Hero,env:&ItemEnv)->V{
    let yaw=hero.rot[2].to_f32();
    if hero.fixed_combat_aim&&crate::hero::combat_strafe::active(hero){return [yaw.cos(),yaw.sin(),0.];}
    let aimed=crate::hero::combat_strafe::active(hero)||hero.f13f5!=0
        ||(matches!(hero.state,1|0x1e)&&env.pad.held&button::L1!=0);
    unit(if aimed{env.camera.map(|c|c.1).unwrap_or([yaw.cos(),yaw.sin(),0.])}else{[yaw.cos(),yaw.sin(),0.]})
}
/// Override the velocity *before* native capsule/world collision. Look-stance
/// braking may not consume the grappling cable's vertical displacement.
pub(crate) fn grapple_step(hero:&Hero,pad:&crate::pad::PadState)->Option<V>{
    if allowed(hero)&&hero.items.slot.id==12&&pad.held&(button::CIRCLE|button::R2)!=0{
        hero.gadget_combat.grapple_step
    }else{None}
}
/// Missing per-level sequences must never reach Ratchet or the back-item
/// animation follower: an unavailable pose collapses all attachment joints.
fn valid_pose(anim:&dyn AnimCtl,candidates:&[i32])->Option<u8>{
    candidates.iter().copied().filter(|&s|(0..255).contains(&s))
        .map(|s|s as u8).find(|&s|anim.gadget_sequence_valid(s))
}
fn grounded_drill(hero:&Hero)->bool {
    allowed(hero)&&hero.air_ticks==0&&matches!(hero.group,0|1|0xc)
}
fn cable_animation(hero:&mut Hero,anim:&mut dyn AnimCtl,rng:&mut Rng,pose:u8){
    if pose!=0{
        let fallback=hero.weapons.defs.get(16).map(|d|d.anims[0]).unwrap_or(-1);
        let Some(seq)=valid_pose(anim,&[0x2f,fallback,hero.idle_seq() as i32,0xb])else{return};
        let frame=(if seq==0x2f{2}else{0}).min(anim.frame_count(seq).saturating_sub(1) as i32);
        if !anim.prepare_gadget_pose(seq,frame as u8){return;}
        if hero.gadget_combat.pose_item!=12||anim.view().seq_b!=seq||anim.view().flags&2!=0{
            hero.set_anim(anim,rng,Pf::f(5.),seq,frame);
        }
        hero.gadget_combat.pose_sequence=Some(seq);hero.gadget_combat.pose_item=12;
        if seq==0x2f&&anim.frame_count(seq)>14{anim.set_loop(10,14);}else{anim.clear_loop();}
    }else{restore_motion_pose(hero,anim,rng);}
}
/// Only replace a pose still owned by our gadget. Native jumps, damage and
/// cinematics already chose their own animations and must never be reset here.
fn restore_motion_pose(hero:&mut Hero,anim:&mut dyn AnimCtl,rng:&mut Rng){
    let cable=hero.gadget_combat.pose_item==12;
    let owned=hero.gadget_combat.pose_sequence.take();hero.gadget_combat.pose_item=0;
    let Some(owned)=owned else{return};hero.anim_speed=Pf::ONE;
    let falling=cable&&hero.air_ticks>0&&matches!(hero.state,0|1|2|6|0x1e|0x2d);
    // Native fall may already be sequence B while its source snapshot is absent.
    // Repair that handoff too; preserve independently chosen jump/hit poses.
    if anim.view().seq_b!=owned&&!(falling&&matches!(hero.state,6|0x2d)){return;}
    anim.clear_loop();
    let seq=if falling{Some(0xb)}else{match hero.state{
        2=>Some(3+hero.substate.clamp(0,1) as u8),0x73=>Some(0x60),0x3f=>Some(0x5b),
        0=>Some(hero.idle_seq()),_=>None,
    }};
    if let Some(seq)=seq.filter(|&seq|anim.gadget_sequence_valid(seq)){
        if anim.prepare_gadget_pose(seq,0){hero.set_anim(anim,rng,Pf::f(5.),seq,0);}
    }
}

#[cfg(test)]
#[path="gadget_combat_v134_tests.rs"]
mod v134_tests;

/// Soft camera-facing puffs follow frozen actors without allocating enemy copies
/// or depending on a planet's smoke definitions. Each frame has at most 64 emitters.
pub fn condensation_quads(rt:&Runtime,now:u64,camera:V)->Vec<crate::hero::tesla::BeamQuad>{
    let mut out=Vec::with_capacity(rt.frost_mist.len()*4);
    for &(centre,size) in &rt.frost_mist {for n in 0..4 {
        let phase=((now+n*13)%72) as f32/72.;
        let a=n as f32*std::f32::consts::FRAC_PI_2+now as f32*0.012;
        let p=add(centre,[a.cos()*size*0.3,a.sin()*size*0.3,(phase-0.4)*size*0.9]);
        let view=unit(sub(camera,p));
        let right=if view[0].abs()+view[1].abs()>0.001{unit([view[1],-view[0],0.])}else{[1.,0.,0.]};
        let up=[right[1]*view[2],-right[0]*view[2],right[0]*view[1]-right[1]*view[0]];
        let radius=size*(0.4+phase*0.35);
        let point=|x:f32,y:f32|add(p,add(scale(right,x*radius),scale(up,y*radius)));
        let alpha=(phase.min(1.-phase)*2.*60.) as u8;
        out.push(crate::hero::tesla::BeamQuad{fx:0,corners:[point(-1.,-1.),point(1.,-1.),point(-1.,1.),point(1.,1.)],
            st:[[0.,0.],[1.,0.],[0.,1.],[1.,1.]],rgba:[((alpha as u32)<<24)|0x00FFF6E8;4]});
    }}out
}

#[cfg(test)]
#[path="gadget_combat_v135_tests.rs"]
mod v135_tests;

/// The same final pose must drive the rendered body and the back/hand chains.
/// Apply after native movement transitions, before combat layers and fx::end.
pub(crate) fn apply_hero_pose(hero:&mut Hero,anim:&mut dyn AnimCtl,rng:&mut Rng){
    if !allowed(hero){
        if let Some(seq)=hero.gadget_combat.pose_sequence.take(){
            hero.anim_speed=Pf::ONE;if anim.view().seq_b==seq{anim.clear_loop();}
        }
        hero.gadget_combat.pose_item=0;return;
    }
    match hero.items.slot.id{
        12=>cable_animation(hero,anim,rng,hero.gadget_combat.cable_pose),
        26 if hero.gadget_combat.holding&&grounded_drill(hero)=>{
            let native=hero.weapons.defs.get(16).map(|d|d.anims[0]).unwrap_or(-1);
            if let Some(seq)=valid_pose(anim,&[native,hero.idle_seq() as i32]){
                let key=anim.frame_count(seq)/2;
                if hero.hold_gadget_anim(anim,rng,seq,key){
                    hero.gadget_combat.pose_sequence=Some(seq);hero.gadget_combat.pose_item=26;
                }
            }
        }
        26=>restore_motion_pose(hero,anim,rng),
        _=>restore_motion_pose(hero,anim,rng),
    }
}

#[cfg(test)]
#[path="gadget_combat_v136_tests.rs"]
mod v136_tests;

#[cfg(test)]
#[path="gadget_combat_v137_tests.rs"]
mod v137_tests;

#[cfg(test)]
#[path="gadget_combat_v138_tests.rs"]
mod v138_tests;

#[cfg(test)]
#[path="gadget_combat_v139_tests.rs"]
mod v139_tests;
