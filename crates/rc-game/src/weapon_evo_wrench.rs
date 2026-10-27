//! Wrench grade hooks: native combat retains its animations and reaction types.
//! Collection bonus is passive and adds to the native Ratchet/Bolt Grabber radii.
use crate::hero::{Hero,items::{HitSink,ItemEnv}};
use crate::moby_runtime::MobyTable;
use crate::moby_update::services::HitTemplate;
use crate::ps2v::Pf;
pub const THROWN:u32=0x4000_0000;
pub const BLAST:u32=0x1000_0000;
pub const BURN:u32=0x0800_0000;
#[derive(Clone,Copy,Debug,PartialEq)]
pub struct SlamBlast {pub enabled:bool,pub damage_hp:f32,pub radius:f32,pub color:Option<u32>}
impl Default for SlamBlast {fn default()->Self{Self{enabled:false,damage_hp:200.,radius:0.5,color:None}}}
impl SlamBlast {
    pub fn capture()->Self{Self{enabled:super::extra_flag(8,"slam_explosion_enabled",false),
        damage_hp:super::extra(8,"slam_explosion_damage_hp",200.),radius:super::extra(8,"slam_explosion_radius",0.5),
        color:super::extra_color(8,"slam_explosion_color")}}
}


/// Render-only snapshot of one slam. Its maximum visible radius equals the
/// collision query radius; it cannot create damage, actors, lights or particles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlamVisual {
    pub centre:[f32;3],
    pub radius:f32,
    /// BBGGRR, matching the game's packed explosion palette.
    pub color:u32,
    pub born:u64,
}
impl SlamVisual {
    pub const LIFE:u64=24;
    pub fn appearance(self, now:u64)->Option<(f32,u32)> {
        let age=now.checked_sub(self.born)?;
        if age>=Self::LIFE || !self.radius.is_finite() || self.radius<=0. {return None;}
        // Reach the actual hit boundary quickly, hold it, then fade in place.
        // Never add sprite padding or expand beyond the configured hit radius.
        let expansion=((age+1) as f32/6.).min(1.);
        let fade=1.-age as f32/Self::LIFE as f32;
        Some((self.radius*expansion,((fade*96.).round() as u32)<<24 | (self.color&0xffffff)))
    }
}

/// Cosmetic geometry uses native anchors and the attack's captured hit radius.
/// Color follows the active grade, using the same BBGGRR packing as other FX.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailStyle {
    pub color: u32,
    pub radius_multiplier: f32,
}
impl TrailStyle {
    pub const DEFAULT_COLOR: u32 = 0x00ff_dda5;
    pub fn capture(evolution: &super::Evolution, tuning: AttackTuning) -> Self {
        let color = evolution.extras.iter().find(|t| t.id == 8 && t.enabled)
            .and_then(|t| t.params[t.level.saturating_sub(1).min(7)].get("trail_color"))
            .copied().filter(|v| v.is_finite() && (0.0..=16_777_215.0).contains(v))
            .map_or(Self::DEFAULT_COLOR, |v| v as u32);
        Self { color, radius_multiplier: tuning.hit_radius_multiplier }
    }
    pub fn edge(self, hand: [f32;3], native_tip: [f32;3]) -> [f32;3] {
        if self.radius_multiplier == 1. { return native_tip; }
        std::array::from_fn(|i| hand[i] + (native_tip[i]-hand[i])*self.radius_multiplier)
    }
}

/// Captured once per attack, independently of XP gained during that attack.
/// Identity values preserve native PS2 float results exactly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackTuning {
    pub hit_radius_multiplier:f32,
    pub knockback_multiplier:f32,
}
impl Default for AttackTuning {
    fn default()->Self {Self{hit_radius_multiplier:1.,knockback_multiplier:1.}}
}
impl AttackTuning {
    pub fn capture()->Self {
        Self {hit_radius_multiplier:super::extra(8,"hit_radius_multiplier",1.),
            knockback_multiplier:super::extra(8,"knockback_multiplier",1.)}
    }
    pub fn radius(self,native:Pf)->Pf {
        if self.hit_radius_multiplier==1. {native} else {native*Pf::f(self.hit_radius_multiplier)}
    }
    /// Scale collision endpoints about the hand; never alter animation joints,
    /// trail anchors or the stored native previous-frame hand/head positions.
    pub fn point(self,hand:[Pf;4],native:[Pf;4])->[Pf;4] {
        if self.hit_radius_multiplier==1. {return native;}
        let k=Pf::f(self.hit_radius_multiplier);
        [hand[0]+(native[0]-hand[0])*k,hand[1]+(native[1]-hand[1])*k,
            hand[2]+(native[2]-hand[2])*k,native[3]]
    }
    /// Preserve the native heading, vertical launch factor and exact-push marker.
    pub fn push(self,mut native:[Pf;4])->[Pf;4] {
        if self.knockback_multiplier!=1. {
            let k=Pf::f(self.knockback_multiplier);native[0]=native[0]*k;native[1]=native[1]*k;
        }
        native
    }
}

pub fn native_damage(captured_hp:f32,multiplier:f32)->f32 {
    // Unconfigured worlds/tests keep native 1 HP swings and 2 HP slams.
    let hp=if super::weapon_menu_level(8).is_some() {captured_hp} else {100.};
    hp/100.*multiplier
}
pub fn throw_settings()->(f32,f32,f32) {
    (super::extra(8,"throw_range",10.),super::extra(8,"throw_speed",23.),super::extra(8,"damage_hp",100.))
}
/// Deliver only to actual native crate classes. Their own update performs
/// destruction, rewards/stack release and TNT behavior; no enemy AoE is added.
pub fn break_crates(_hero:&Hero,table:&mut MobyTable,env:&ItemEnv,hits:&mut dyn HitSink,point:[Pf;4]) {
    if !super::extra_flag(8,"crate_breaker_enabled",false) {return;}
    let radius=super::extra(8,"crate_breaker_radius",5.);
    if radius<=0. {return;}
    let targets=crate_targets(table,point,radius);
    let t=HitTemplate {attacker:Some(env.hero_moby),h1a:0x47,flags:0x30000,b18:0,b19:1,
        damage:Pf::f(2.),dir:[Pf::ZERO,Pf::ZERO,Pf::ONE,Pf::ZERO],w20:1};
    for target in targets {hits.deliver(table,target,&t);}
}

fn crate_targets(table:&MobyTable,point:[Pf;4],radius:f32)->Vec<usize> {
    if radius<=0. {return Vec::new();}
    table.mobys.iter().enumerate().filter(|(_,m)|
        !m.is_deleted() && m.state!=crate::moby_update::classes::cannon_crate::CARRIED && crate::moby_update::classes::crate_::CLASSES.contains(&m.o_class) &&
        (0..3).map(|i|(m.position[i]-point[i].to_f32()).powi(2)).sum::<f32>()<=radius*radius)
        .map(|(id,_)|id).collect()

}
pub fn with_bolt_bonus(native:(Pf,Pf),bonus:f32)->(Pf,Pf) {(native.0+Pf::f(bonus),native.1+Pf::f(bonus))}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weapon_evo::Evolution;
    use crate::moby_runtime::Moby;
    #[test]
    fn each_grade_owns_damage_geometry_hud_and_passive_collection_values() {
        let mut raw=String::from("[general]\ndamage_mode=real_hp\nxp_per_damage=0.12\n[weapon.wrench]\nid=8\nxp_v1_to_v2=1000\n");
        for n in 1..=5 {raw+=&format!("[weapon.bomb_glove.level{n}]\n[weapon.wrench.level{n}]\ndamage_hp={}\nthrow_range={}\nthrow_speed={}\nshow_hud={}\ngold_texture={}\ncrate_breaker_enabled={}\ncrate_breaker_radius={}\nmoving_attacks_enabled={}\nbolt_attraction_bonus={}\n",n*100,n*5,20+n,n%2==0,n%2==1,n>=3,n+2,n>=2,n);}
        let mut e=Evolution::parse(&raw).unwrap();
        for grade in 1..=5 {
            let track=e.extras.iter_mut().find(|t|t.id==8).unwrap();track.level=grade;
            assert_eq!(track.params[grade-1]["throw_range"],(grade*5) as f32);
            assert_eq!(track.params[grade-1]["bolt_attraction_bonus"],grade as f32);
            assert_eq!(e.absolute_values()[&8],(grade*100) as f32);
            assert_eq!(e.weapon_hud_enabled(8),grade%2==0);
            assert_eq!(e.weapon_level(8),Some(grade));
            assert!(e.weapon_fraction(8).is_some());
            assert!(e.equipped_title(8,b"OmniWrench").is_some());
        }
        let t=e.extras.iter_mut().find(|t|t.id==8).unwrap();t.level=1;
        e.award_extra_damage(8,2.);
        let progress=e.progress();let bytes=progress.encode(b"slot").unwrap();
        let restored=crate::weapon_evo::save::Progress::decode(&bytes,b"slot").unwrap();
        let index=crate::weapon_evo::save::IDS.iter().position(|&id|id==8).unwrap();
        assert!((restored.tracks[index].xp-24.).abs()<0.0001);
        for (field,value) in [("throw_range","0"),("throw_speed","0"),("bolt_attraction_bonus","101"),("crate_breaker_radius","NaN"),("show_hud","perhaps")] {
            let ini=raw.replace("throw_range=5",&format!("{field}={value}"));assert!(Evolution::parse(&ini).is_err(),"{field}");
        }
    }
    #[test]
    fn crate_breaker_targets_only_native_crates_inside_its_sphere() {
        let make=|class,x|Moby{o_class:class,state:1,position:[x,20.,10.,1.],..Default::default()};
        let mut carried=make(500,21.);carried.state=crate::moby_update::classes::cannon_crate::CARRIED;
        let table=MobyTable::new(vec![carried,make(500,21.),make(502,22.),make(505,23.),make(511,24.),make(501,25.),make(577,21.),make(500,27.)],0);
        let point=[Pf::f(20.),Pf::f(20.),Pf::f(10.),Pf::ONE];
        assert_eq!(crate_targets(&table,point,5.),vec![1,2,3,4,5]);
        assert!(crate_targets(&table,point,0.).is_empty());
    }
    #[test]
    fn collection_bonus_adds_to_both_native_and_bolt_grabber_radii() {
        let plain=with_bolt_bonus((Pf::f(3.),Pf::f(1.75)),2.);
        let grabber=with_bolt_bonus((Pf::f(12.),Pf::f(4.5)),2.);
        assert_eq!(plain,(Pf::f(5.),Pf::f(3.75)));assert_eq!(grabber,(Pf::f(14.),Pf::f(6.5)));
        assert_eq!(with_bolt_bonus((Pf::f(12.),Pf::f(4.5)),0.),(Pf::f(12.),Pf::f(4.5)));
    }
    #[test]
    fn swing_damage_is_captured_and_deduplicated_across_frames_but_next_swing_hits() {
        use crate::moby_update::real_hp::Events;
        let table=MobyTable::new(vec![Moby::default(),Moby{o_class:577,state:1,..Default::default()}],0);
        let mut e=Events::default();e.enabled=true;e.hero=Some(0);e.values.insert(8,900.);e.wrench_serial=1;
        let mut t=HitTemplate{attacker:Some(0),h1a:0x47,damage:Pf::f(1.5),flags:0x10000,..Default::default()};
        assert!(e.queue(&table,1,&t,false,[Pf::ZERO;4],0));
        let first=e.take(1,u32::MAX).unwrap();assert_eq!(first.damage,Pf::f(1.5));assert_eq!(first.evolution.unwrap().weapon,8);
        assert!(e.resolve_once(1,first.evolution.unwrap()));
        e.begin_frame(2);e.queue(&table,1,&t,false,[Pf::ZERO;4],0);assert!(e.take(1,u32::MAX).is_none());
        e.wrench_serial=2;e.begin_frame(3);t.damage=Pf::f(3.);
        e.queue(&table,1,&t,false,[Pf::ZERO;4],0);let slam=e.take(1,u32::MAX).unwrap();assert_eq!(slam.damage,Pf::f(3.));
        e.wrench_serial=3;e.begin_frame(4);t.h1a=1;t.w20=THROWN;t.damage=Pf::f(1.25);
        e.queue(&table,1,&t,false,[Pf::ZERO;4],0);let thrown=e.take(1,u32::MAX).unwrap();assert_eq!(thrown.evolution.unwrap().weapon,8);
        assert_eq!(thrown.h2a,1);assert_eq!(thrown.damage,Pf::f(1.25));
    }
}

#[cfg(test)]
mod tuning_tests {
    use super::*;
    use crate::{hero::{self,items::{HandItem,HandSlot,ItemClass,ItemData},anim::{AnimCtl,AnimView}},moby_runtime::Moby,rng::Rng};
    use rc_formats::moby_anim::{AnimState,MobyAnimClass,Rows,IDENTITY};

    struct Anim { frame:f32 }
    impl AnimCtl for Anim {
        fn set_anim(&mut self,_:Pf,_:u8,_:i32) {}
        fn advance(&mut self,_:Pf) {}
        fn set_loop(&mut self,_:i32,_:i32) {}
        fn clear_loop(&mut self) {}
        fn view(&self)->AnimView {AnimView{frame:self.frame,..Default::default()}}
        fn frame_count(&self,_:u8)->u8 {100}
        fn eval_chains(&self,chains:&[&[u8]])->Vec<Rows> {vec![IDENTITY;chains.len()]}
    }
    #[derive(Default)]
    struct Hits { spheres:Vec<(Pf,[Pf;4],HitTemplate)>, lines:Vec<([Pf;4],[Pf;4])>, target:Option<[f32;3]> }
    impl HitSink for Hits {
        fn sphere(&mut self,_:&mut MobyTable,r:Pf,c:[Pf;4],_:u32,_:Option<usize>,t:&HitTemplate)->Option<usize> {
            self.spheres.push((r,c,t.clone()));
            self.target.filter(|p|(0..3).map(|i|(p[i]-c[i].to_f32()).powi(2)).sum::<f32>()<=r.to_f32().powi(2)).map(|_|1)
        }
        fn line(&mut self,_:&mut MobyTable,a:[Pf;4],b:[Pf;4],_:u32,_:Option<usize>,_:&HitTemplate)->Option<Option<usize>> {
            self.lines.push((a,b));None
        }
    }
    fn data()->ItemData {
        let empty=MobyAnimClass{joint_count:1,skeleton:vec![],rest:vec![[0.;3]],parent_word:vec![0],sequences:vec![]};
        ItemData {defs:vec![Default::default();37],hero_chains:vec![vec![0]],
            classes:vec![ItemClass{o_class:0x47,anim:empty,scale:1.,chains:vec![vec![0],vec![0]]}]}
    }
    fn fixture()->(Hero,MobyTable) {
        let mut h=Hero::default();h.pos=[Pf::f(410.),Pf::f(410.),Pf::f(101.),Pf::ONE];h.state=0x13;h.group=6;
        let anim=AnimState{seq_a:1,frame_a:0,seq_b:1,frame_b:0,t:0.,speed:1.,rate:1.,flags:0,trigger_count:0,skip_advance:false};
        let item=HandItem{o_class:0x47,mstate:0,anim,snapshot:None,scale:1.,position:[412.,410.,101.],rows:[[0;4];3],hit_timer:0,flight:Default::default()};
        h.items.slot=HandSlot{item:Some(item),state:2,id:8,hand_point:[410.,410.,101.],..Default::default()};
        h.melee.damage_hp=100.;h.melee.hand=h.pos;h.melee.tip=[Pf::f(412.17),Pf::f(410.),Pf::f(101.),Pf::ONE];
        let table=MobyTable::new(vec![Moby{position:[410.,410.,101.,1.],..Default::default()},Moby{o_class:577,..Default::default()}],0);
        (h,table)
    }
    fn near(a:f32,b:f32) {assert!((a-b).abs()<0.0005,"{a} != {b}");}
    #[test]
    fn all_combo_swings_and_air_hit_expand_collision_without_changing_pose_or_damage() {
        let d=data();let pad=crate::pad::PadState::default();
        let env=ItemEnv{ mobys: None,data:&d,pad:&pad,frame:0,hero_moby:0,coll:None,camera:None,camera_up:None,targets:&[]};
        for (state,combo) in [(0x13,0),(0x13,1),(0x13,2),(0x70,5),(0x70,6),(0x14,4)] {
            let run=|tuning:AttackTuning| {
                let (mut h,mut table)=fixture();h.state=state;h.melee.combo=combo;h.melee.tuning=tuning;
                let a=Anim{frame:hero::melee::COMBO[combo as usize][7] as f32};
                let mut hits=Hits{target:Some([414.5,410.,101.]),..Default::default()};
                hero::melee::wrench_update(&mut h,&mut table,&a,&env,&mut hits,&mut Rng::new());
                (h,hits)
            };
            let (native,n)=run(AttackTuning::default());
            let (scaled,s)=run(AttackTuning{hit_radius_multiplier:2.,knockback_multiplier:3.});
            assert_eq!(n.spheres.len(),1,"state {state} combo {combo}");assert_eq!(s.lines.len(),5);
            assert_eq!(native.melee.tip,scaled.melee.tip);assert_eq!(native.melee.hand,scaled.melee.hand);
            assert_eq!(native.items.slot.item.as_ref().unwrap().position,scaled.items.slot.item.as_ref().unwrap().position);
            near(s.spheres[0].0.to_f32(),n.spheres[0].0.to_f32()*2.);
            near(s.spheres[0].1[0].to_f32()-410.,(n.spheres[0].1[0].to_f32()-410.)*2.);
            assert!(s.lines[4].1[0]>n.lines[4].1[0]);
            let (nt,st)=(&n.spheres[0].2,&s.spheres[0].2);assert_eq!(nt.damage,st.damage);
            near(st.dir[0].to_f32(),nt.dir[0].to_f32()*3.);near(st.dir[1].to_f32(),nt.dir[1].to_f32()*3.);
            assert_eq!(&nt.dir[2..],&st.dir[2..]);
            assert_eq!(native.melee.hit,0);assert_eq!(scaled.melee.hit,1);
        }
    }
    #[test]
    fn slam_impact_scales_once_and_preserves_vertical_push_and_native_reaction_scaling() {
        let d=data();let pad=crate::pad::PadState::default();
        let env=ItemEnv{ mobys: None,data:&d,pad:&pad,frame:0,hero_moby:0,coll:None,camera:None,camera_up:None,targets:&[]};
        let (mut h,mut table)=fixture();h.melee.tuning=AttackTuning{hit_radius_multiplier:2.,knockback_multiplier:3.};
        h.shockwave=Some((h.pos,[Pf::f(2.),Pf::ZERO,Pf::ZERO,Pf::b(0x45af_df66)]));
        let mut hits=Hits::default();hero::melee::jump_attack_shockwave(&mut h,&mut table,&env,&mut hits,&mut Rng::new());
        hero::melee::jump_attack_shockwave(&mut h,&mut table,&env,&mut hits,&mut Rng::new());
        assert_eq!(hits.spheres.len(),1);let (r,_,t)=&hits.spheres[0];near(r.to_f32(),0.8);near(t.dir[0].to_f32(),6.);
        assert_eq!((t.damage,t.dir[2],t.dir[3]),(Pf::f(2.),Pf::ZERO,Pf::b(0x45af_df66)));
        let mut speed=2.;let mut up=3.;crate::moby_update::creature::knock::aim(t.dir.map(Pf::to_f32),&mut speed,&mut up);
        near(speed,12.);near(up,0.);
    }
    #[test]
    fn thrown_outbound_and_return_use_captured_radius_and_push() {
        let d=data();let pad=crate::pad::PadState::default();
        let env=ItemEnv{ mobys: None,data:&d,pad:&pad,frame:0,hero_moby:0,coll:None,camera:None,camera_up:None,targets:&[]};
        for state in [hero::comet::OUT,hero::comet::BACK] {
            let (mut h,mut table)=fixture();h.items.slot.detached=1;
            let it=h.items.slot.item.as_mut().unwrap();it.mstate=state;
            it.flight=hero::comet::Flight{configured:true,dir:[1.,0.,0.],speed:23./60.,range_left:10.,damage_hp:150.,
                tuning:AttackTuning{hit_radius_multiplier:2.,knockback_multiplier:3.},..Default::default()};
            // Changing the held/melee tuning cannot resize a projectile in flight.
            h.melee.tuning=AttackTuning{hit_radius_multiplier:5.,knockback_multiplier:0.};
            let mut hits=Hits::default();hero::comet::thrown_update(&mut h,&mut table,&Anim{frame:10.},&env,&mut hits,&mut Rng::new());
            assert!(!hits.spheres.is_empty(),"flight state {state}");
            for (r,_,t) in hits.spheres {
                near(r.to_f32(),0.8);let push=(t.dir[0].to_f32().powi(2)+t.dir[1].to_f32().powi(2)).sqrt();
                near(push,3.*if state==hero::comet::OUT {1.2} else {0.37});assert_eq!(t.dir[2],Pf::ONE);
                assert_eq!(t.w20&THROWN,THROWN);
            }
        }
    }
    #[test]
    fn defaults_are_exact_and_each_grade_accepts_independent_bounded_multipliers() {
        let v=[Pf::b(0x43cd_1234),Pf::f(-2.75),Pf::f(0.3),Pf::b(0x45af_df66)];
        let native=AttackTuning::default();assert_eq!(native.point([Pf::f(400.);4],v),v);assert_eq!(native.push(v),v);assert_eq!(native.radius(v[2]),v[2]);
        let zero=AttackTuning{knockback_multiplier:0.,..Default::default()}.push(v);assert_eq!(&zero[2..],&v[2..]);assert_eq!((zero[0],zero[1]),(Pf::ZERO,Pf::ZERO));
        let mut raw=String::new();
        for n in 1..=5 {raw+=&format!("[weapon.bomb_glove.level{n}]\n[weapon.wrench.level{n}]\nhit_radius_multiplier={}\nknockback_multiplier={}\n",n as f32/2.,n-1);}
        let e=crate::weapon_evo::Evolution::parse(&raw).unwrap();let track=e.extras.iter().find(|t|t.id==8).unwrap();
        for n in 0..5 {assert_eq!(track.params[n]["hit_radius_multiplier"],(n+1) as f32/2.);assert_eq!(track.params[n]["knockback_multiplier"],n as f32);}
        for (key,bad) in [("hit_radius_multiplier","0"),("hit_radius_multiplier","10.1"),("knockback_multiplier","-1"),("knockback_multiplier","10.1"),("hit_radius_multiplier","NaN")] {
            let changed=raw.replace("hit_radius_multiplier=0.5",&format!("{key}={bad}"));assert!(crate::weapon_evo::Evolution::parse(&changed).is_err(),"{key}={bad}");
        }
    }
}

#[cfg(test)]
mod trail_tests {
    use super::*;
    #[test]
    fn all_eight_grades_select_their_own_exact_trail_color() {
        let colors=["000000","FF0000","00FF00","0000FF","AABBCC","FFFFFF","012345","default"];
        let expected=[0,0xff,0xff00,0xff0000,0xccbbaa,0xffffff,0x452301,TrailStyle::DEFAULT_COLOR];
        let mut raw=String::from("[general]\ndamage_mode=real_hp\n");
        for grade in 1..=5 {raw+=&format!("[weapon.bomb_glove.level{grade}]\n");}
        for grade in 1..=8 {raw+=&format!("[weapon.wrench.level{grade}]\ntrail_color={}\nhit_radius_multiplier=4\n",colors[grade-1]);}
        let mut e=crate::weapon_evo::Evolution::parse(&raw).unwrap();
        for grade in 1..=8 {
            e.extras.iter_mut().find(|t|t.id==8).unwrap().level=grade;
            let style=TrailStyle::capture(&e,AttackTuning{hit_radius_multiplier:2.,..Default::default()});
            assert_eq!(style.color,expected[grade-1]);
            assert_eq!(style.radius_multiplier,2.,"must use the attack capture, not a later config value");
            assert_eq!(style.edge([10.,20.,30.],[11.,22.,33.]),[12.,24.,36.]);
        }
    }
    #[test]
    fn legacy_default_preserves_native_color_and_identity_geometry() {
        let e=crate::weapon_evo::Evolution::default();
        let style=TrailStyle::capture(&e,AttackTuning::default());
        assert_eq!(style.color,TrailStyle::DEFAULT_COLOR);
        let tip=[1.,-0.,3.];
        assert_eq!(style.edge([10.,20.,30.],tip).map(f32::to_bits),tip.map(f32::to_bits));
    }
}

#[cfg(test)]
mod v102_tests {
    use super::*;
    #[test]
    fn slam_and_blast_are_distinct_once_per_target_and_both_award_wrench_damage(){
        use crate::{moby_runtime::{Moby,MobyTable},moby_update::real_hp::Events};
        let table=MobyTable::new(vec![Moby::default(),Moby{o_class:577,state:1,..Default::default()}],0);
        let mut events=Events::default();events.enabled=true;events.hero=Some(0);events.wrench_serial=1;events.values.insert(8,100.);
        let mut hit=HitTemplate{attacker:Some(0),h1a:0x47,damage:Pf::ONE,flags:0x10000,w20:BURN,..Default::default()};
        events.queue(&table,1,&hit,false,[Pf::ZERO;4],0);
        hit.w20|=BLAST;hit.h1a=121;hit.b18=2;hit.damage=Pf::f(2.);
        for _ in 0..10{events.queue(&table,1,&hit,false,[Pf::ZERO;4],0);}
        let direct=events.take(1,u32::MAX).unwrap();let blast=events.take(1,u32::MAX).unwrap();
        assert_eq!(direct.damage,Pf::ONE);assert_eq!(blast.damage,Pf::f(2.));
        let a=direct.evolution.unwrap();let b=blast.evolution.unwrap();assert_ne!(a.instance,b.instance);
        assert_eq!(b.origin.unwrap().class,121);assert_eq!(b.weapon,8);assert_eq!(b.burn_color,Some(None));assert_eq!(blast.w30&(BLAST|BURN),0);
        events.resolve_once(1,a);events.resolve_once(1,b);events.begin_frame(2);
        events.queue(&table,1,&hit,false,[Pf::ZERO;4],0);assert!(events.take(1,u32::MAX).is_none());
        events.wrench_serial=2;events.queue(&table,1,&hit,false,[Pf::ZERO;4],0);assert!(events.take(1,u32::MAX).is_some());
    }
    #[test]
    fn every_grade_parses_independent_visual_and_explosion_options(){
        let mut raw=include_str!("weapon_evo_release.ini").to_string();
        for grade in 1..=8 {raw+=&format!("\n[weapon.wrench.level{grade}]\nburn_on_hit={}\nhud_quickselect_only={}\nslam_explosion_enabled={}\nslam_explosion_damage_hp={}\nslam_explosion_radius={}\nslam_explosion_color=1234AB\n",grade%2==0,grade%2==1,grade>4,grade*100,grade as f32/2.);}
        raw+="\n[hud]\nweapon_bar_alternative_position=true\n";
        let e=crate::weapon_evo::Evolution::parse(&raw).unwrap();assert!(e.hud.weapon_bar_alternative_position);
        let track=e.extras.iter().find(|t|t.id==8).unwrap();
        for grade in 1..=8 {let p=&track.params[grade-1];assert_eq!(p["slam_explosion_damage_hp"],(grade*100)as f32);assert_eq!(p["burn_on_hit"],if grade%2==0{1.}else{0.});}
        assert!(crate::weapon_evo::Evolution::parse(&raw.replace("slam_explosion_radius=0.5","slam_explosion_radius=0")).is_err());
    }
}

#[cfg(test)]
#[path="weapon_evo_slam_damage_tests.rs"]
mod slam_damage_tests;

#[cfg(test)]
mod slam_visual_tests {
    use super::*;
    #[test]
    fn visual_reaches_exact_hit_radius_and_never_exceeds_it() {
        for radius in [0.1,0.5,1.,3.,8.,30.] {
            let visual=SlamVisual{centre:[10.,20.,0.05],radius,color:0x123456,born:100};
            assert!(visual.appearance(99).is_none());
            let mut maximum=0.0f32;
            for now in 100..100+SlamVisual::LIFE {
                let (visible,rgba)=visual.appearance(now).unwrap();
                assert!(visible>0. && visible<=radius);
                assert_eq!(rgba&0xffffff,0x123456);assert!(rgba>>24<=96);
                maximum=maximum.max(visible);
            }
            assert_eq!(maximum,radius);
            assert!(visual.appearance(100+SlamVisual::LIFE).is_none());
        }
    }
    #[test]
    fn invalid_radius_is_never_rendered() {
        for radius in [0.,-1.,f32::NAN,f32::INFINITY] {
            assert!(SlamVisual{centre:[0.;3],radius,color:0,born:0}.appearance(0).is_none());
        }
    }
}
