//! Seeded Gold Bolt rewards. The full layout belongs to a save, not the INI or
//! the gameplay RNG. Native bolt indices remain the location identity when a
//! planet's actor table changes on a return visit. No mission flags are edited.
use crate::game_state::{GameState, SessionState};
use crate::moby_runtime::MobyId;
use crate::moby_update::{interact, services::World};
use rc_formats::save_game::ItemTables;

pub const ENABLE_KEY: &str = "golden_bolts_to_weapons";
pub const BIAS_KEY: &str = "golden_bolts_early_bias_percent";
pub const ITEMS: [usize; 15] =
    [10, 16, 15, 20, 9, 14, 17, 11, 13, 18, 25, 24, 19, 21, 23];
#[derive(Clone,Copy,Debug,Default,PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub early_bias_percent: u8,
    pub one_weapon_per_planet: bool,
}
impl Settings {
    pub fn parse_field(&mut self, key: &str, value: &str)
        -> Result<bool, String> {
        match key {
            "golden_bolts_one_weapon_per_planet" => {
                self.one_weapon_per_planet=match value.to_ascii_lowercase().as_str(){
                    "true"|"on"|"1"=>true,"false"|"off"|"0"=>false,
                    _=>return Err(format!("{key}: expected on/off")),
                };
            },
            ENABLE_KEY =>
                self.enabled =
                    match value.to_ascii_lowercase().as_str() {
                        "true" | "on" | "1" => true,
                        "false" | "off" | "0" => false,
                        _ => return Err(format!("{key}: expected on/off")),
                    },
            BIAS_KEY => {
                self.early_bias_percent =
                    value.parse::<u8>().ok().filter(|n|
                                    *n <=
                                        100).ok_or_else(||
                                format!("{key}: expected integer 0..100"))?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
#[path = "weapon_evo_golden/layout.rs"]
mod layout;
pub use layout::{State, COUNTS, fresh_seed};
#[path = "weapon_evo_golden/locations.rs"]
pub mod locations;

pub fn enabled() -> bool {
    super::inspect_profile().is_some_and(|p| p.golden_settings.enabled)
}
pub fn initialize_new_game(gs: &mut GameState, settings: Settings,
    seed: u64) {
    gs.weapon_progress.golden =
        if settings.enabled {
            State::generate_with_planet_limit(seed, settings.early_bias_percent, settings.one_weapon_per_planet)
        } else { State::default() };
    if settings.enabled {
        for item in ITEMS {
            gs.global.owned[item]=0;
            gs.global.acquired[item]=0;
            gs.global.ammo[item]=0;
        }
        gs.global.owned[8]=1;
        gs.global.equipped[0]=8;
        gs.global.last_hand_item=8;
        gs.global.quick_select=[0;8];
    }
}
pub fn card_root(root: std::path::PathBuf, on: bool) -> std::path::PathBuf {
    if super::randomizer::enabled(){return root.join("randomizer");}
    if super::gemlik_test::enabled() {return super::gemlik_test::card_root(root);}
    if super::location_test::enabled() {return root.join("weapon_location_test");}
    if on { root.join("golden_bolts_to_weapons") } else { root }
}
/// Reject a copied normal save before any native section is restored. Missing
/// or corrupt companions are never silently regenerated into a different seed.
pub fn accepts_save(slot: &std::path::Path, native: &[u8], on: bool) -> bool {
    match super::save::load(slot, native) {
        Ok(p) => p.golden.active() == on && super::randomizer::accepts(&p,super::randomizer::enabled()),
        Err(_) => !on && !super::randomizer::enabled(),
    }
}
pub fn notice(item: usize) -> &'static [u8] {
    match item {
        10 => b"Bomb Glove acquired!",
        16 => b"Pyrocitor acquired!",
        15 => b"Blaster acquired!",
        20 => b"Glove of Doom acquired!",
        9 => b"Suck Cannon acquired!",
        14 => b"Taunter acquired!",
        17 => b"Mine Glove acquired!",
        11 => b"Devastator acquired!",
        13 => b"Visibomb Gun acquired!",
        18 => b"Walloper acquired!",
        25 => b"Decoy Glove acquired!",
        24 => b"Drone Device acquired!",
        19 => b"Tesla Claw acquired!",
        21 => b"Morph-o-Ray acquired!",
        23 => b"RYNO acquired!",
        _ => b"Weapon acquired!",
    }
}
#[derive(Clone,Debug)]
pub struct Reward {
    pub actor: Option<MobyId>,
    pub index: i32,
    pub item: usize,
    pub position: [f32; 3],
    pub ready: bool,
}
#[derive(Clone,Debug,Default)]
pub struct WorldState {
    pub rewards: Vec<Reward>,
    pending: Vec<usize>,
    elapsed: u64,
    ready_items: u64,
    anchors: Vec<Reward>,
    pub test_mode: bool,
    /// Immutable mission identity, retained even when the native pickup is
    /// removed. Only Oltanis's carrier gate uses this randomizer substitution.
    pub native_morph_mission: Option<i32>,
    test_bolt_sites: Vec<locations::Site>,
    test_spawned: Vec<usize>,
}
impl WorldState {
    /// Prepare every selected native/custom reward before the renderer starts.
    /// Raw actor positions are independent of visited/death filters and retain
    /// their identity when a later visit changes the live moby table.
    pub fn from_gameplay(level:u32, progress:&State, gameplay:&[u8], gemlik:Option<[f32;3]>)->Self {
        if !progress.active() {return Self::default();}
        let mut rewards=Vec::new();
        let test_mode=super::location_test::enabled();
        let mut test_bolt_sites:Vec<locations::Site>=Vec::new();
        if let (Ok(instances),Ok(vars))=(rc_formats::gameplay::parse_moby_instances(gameplay),rc_formats::gameplay::parse_pvars(gameplay)) {
            for m in instances.iter().filter(|m|m.o_class==1134) {
                let Some(index)=m.pvar(&vars).and_then(|p|p.get(..4)).map(|p|i32::from_le_bytes(p.try_into().unwrap())) else {continue;};
                let Some(item)=progress.item(level,index) else {
                    if test_mode && index>=0 && index<COUNTS.get(level as usize).copied().unwrap_or(0) as i32 {
                        let slot=level as usize*4+index as usize;
                        if !test_bolt_sites.iter().any(|s|s.slot==slot) {
                            test_bolt_sites.push(locations::Site{slot,level,name:"Native Gold Bolt",base:m.position,underwater:false});
                        }
                    }
                    continue;
                };
                if rewards.iter().any(|r:&Reward|r.index==index) {continue;}
                rewards.push(Reward {actor:None,index,item,position:m.position,ready:false});
            }
        }
        let native_sites=locations::native_weapon_sites(level,gameplay);
        let native_morph_mission=locations::native_morph_mission(level,gameplay);
        for site in locations::for_level(level,progress,gemlik).into_iter()
            .chain(native_sites.iter().copied().filter(|s|progress.item_at(s.slot).is_some())) {
            rewards.push(Reward {actor:None,index:site.slot as i32,item:progress.item_at(site.slot).unwrap(),position:site.base,ready:false});
        }
        if test_mode {
            test_bolt_sites.extend(locations::all_for_level(level,gemlik).into_iter()
                .chain(native_sites).filter(|s|progress.item_at(s.slot).is_none()));
        }
        if test_mode {
            println!("location test: level {level}: {} weapons and {} Gold Bolt locations prepared",rewards.len(),test_bolt_sites.len());
        }
        Self {anchors:rewards.clone(),rewards,test_mode,test_bolt_sites,native_morph_mission,..Default::default()}
    }
    pub fn pending(&self, item: usize) -> bool {
        self.pending.contains(&item)
    }
    pub fn finish(&mut self, item: usize) {
        self.pending.retain(|&i| i != item);
    }
    pub fn reset_for_reload(&mut self) {
        self.rewards=self.anchors.clone();
        for r in &mut self.rewards {r.ready=self.ready_items&(1u64<<r.item)!=0;}
        self.pending.clear();
        self.elapsed = 0;
        self.test_spawned.clear();
    }
    pub fn set_ready(&mut self, item: usize) {
        self.ready_items |= 1u64 << item;
        for r in &mut self.rewards { if r.item == item { r.ready = true; } }
    }
    pub fn visuals(&self, progress: &State, alive: bool)
        -> Vec<(usize, super::discovery::Visual)> {
        if !alive { return Vec::new(); }
        self.rewards.iter().filter(|r|
                        r.ready && !progress.collected(r.item) &&
                            !self.pending(r.item)).map(|r|
                    {
                        let mut position = r.position;
                        position[2] +=
                            1. +
                                0.15 *
                                    ((self.elapsed % 180) as f32 * std::f32::consts::TAU / 180.).sin();
                        (r.item,
                            super::discovery::Visual {
                                position,
                                yaw: super::pickup_visuals::spin_angle(self.elapsed),
                            })
                    }).collect()
    }
}
/// Replace selected Gold Bolts and delete every unselected Gold Bolt.
/// The hidden native actor retains its position/index but cannot award a bolt,
/// run its cutaway, respawn a glow or touch mission/ship state.
pub fn replace_bolt(w: &mut World, id: MobyId) -> bool {
    if !w.svc.bomb_evo.golden_settings.enabled { return false; }
    let Some(index)=w.m(id).pvars.get(..4).map(|p|i32::from_le_bytes(p.try_into().unwrap())) else {
        w.delete_moby(id);return true;
    };
    let progress = w.svc.bomb_evo.state_progress.golden;
    let Some(item) = progress.item(w.svc.level, index) else {
        if w.svc.golden.test_mode {return false;}
        w.delete_moby(id);return true;
    };
    if progress.collected(item) { w.delete_moby(id); return true; }
    let position = w.m(id).position;
    if let Some(reward)=w.svc.golden.rewards.iter_mut().find(|r|r.index==index) {
        reward.actor=Some(id);
        reward.position=[position[0],position[1],position[2]];
    } else {
        w.svc.golden.rewards.push(Reward {
                actor: Some(id),
                index,
                item,
                position: [position[0], position[1], position[2]],
                ready: w.svc.golden.ready_items & (1u64 << item) != 0,
            });
    }
    let m = w.mm(id);
    m.mode |= 1;
    m.state = 1;
    m.has_collision = false;
    true
}
pub fn tick(w: &mut World) {
    if !w.svc.bomb_evo.golden_settings.enabled { return; }
    w.svc.golden.elapsed = w.svc.golden.elapsed.wrapping_add(1);
    if w.svc.game_mode != 0 || w.hero.health <= 0 ||
            !matches!(w.hero.mode,0|3) {
        return;
    }
    if w.svc.golden.test_mode {
        let sites=w.svc.golden.test_bolt_sites.clone();
        for site in sites {
            if w.svc.golden.test_spawned.contains(&site.slot) {continue;}
            let Some(index)=super::location_test::bolt_index(site) else {continue;};
            let exists=w.table.mobys.iter().any(|m|m.state<0x80 && m.o_class==1134
                && m.pvars.get(..4).is_some_and(|p|i32::from_le_bytes(p.try_into().unwrap())==index as i32));
            if exists || crate::moby_update::classes::gold_bolt::collected(w,index as i32)
                || super::location_test::spawn_bolt(w,site) {
                w.svc.golden.test_spawned.push(site.slot);
            }
        }
    }
    let progress = w.svc.bomb_evo.state_progress.golden;
    let visuals = w.svc.golden.visuals(&progress, true);
    let hero = w.hero.pos.map(|p| p.to_f32());
    for (item, visual) in visuals {
        let p = visual.position;
        if (p[0] - hero[0]).hypot(p[1] - hero[1]) > 1.6 ||
                (p[2] - hero[2]).abs() > 2. {
            continue;
        }
        let from =
            [hero[0], hero[1], hero[2] + 0.75, 0.].map(crate::ps2v::Pf::f);
        if w.line(from, [p[0], p[1], p[2], 0.].map(crate::ps2v::Pf::f), 0x12,
                    w.hero_moby).is_some() {
            continue;
        }
        w.svc.golden.pending.push(item);
        w.svc.interact.writes.push(interact::GameWrite::GiveItem {
                item,
                equip: !is_swimming(w.hero),
            });
        // Save/banner follow successful engine-side inventory grant, so a
        // missing item table cannot permanently consume a physical reward.
    }
}
/// Preserve the public land-grant API used by the ordinary reward integration.
pub fn grant(gs:&mut GameState,session:&mut SessionState,items:&ItemTables,item:usize,settings:&super::ammo_mode::Settings) {
    grant_with_equip(gs,session,items,item,settings,true);
}
pub fn grant_with_equip(gs:&mut GameState,session:&mut SessionState,items:&ItemTables,item:usize,
    settings:&super::ammo_mode::Settings,equip:bool)->bool {
    if !gs.weapon_progress.golden.active() || !ITEMS.contains(&item)
        || gs.weapon_progress.golden.collected(item) {return false;}
    // Claim first: the shared native grant rejects all other weapon sources.
    gs.weapon_progress.golden.collect(item);
    gs.give_item(item,equip,items,session);
    if super::ammo_mode::AMMO_IDS.contains(&item) {
        let native=u16::from_le_bytes(items.records[item].0[0xe..0x10].try_into().unwrap()) as i32;
        let max=super::effective_max_ammo(item,native).max(1);
        let ammo=(items.records[item].grant_ammo() as i32).max(1).min(max);
        gs.global.ammo[item]=gs.global.ammo[item].max(ammo).min(max);
    }
    super::ammo_mode::reconcile_configured(gs,session,gs.weapon_progress.ammo_mode.active,0,false,settings);
    super::quick_select::insert_found(gs,item);
    if equip {
        session.temp_hand=item as i32;
        gs.weapon_progress.golden.pending_equip=0;
    } else {
        gs.weapon_progress.golden.pending_equip=item as u32;
    }
    true
}
pub fn is_swimming(hero:&crate::hero::Hero)->bool {
    hero.in_water!=0 || matches!(hero.state,0x33..=0x37|0x6a|0x73)
        || matches!(hero.group,0x11|0x12)
}
/// The pending item is save-owned. Loading a submerged save or dying there
/// cannot equip a hand weapon in water or lose the permanent discovery.
pub fn finish_deferred_equip(gs:&mut GameState,session:&mut SessionState,hero:&crate::hero::Hero,game_mode:i32)->bool {
    let pending=gs.weapon_progress.golden.pending_equip as usize;
    if pending==0 || !gs.weapon_progress.golden.collected(pending)
        || hero.health<=0 || !matches!(hero.mode,0|3) || game_mode!=0 || is_swimming(hero) {return false;}
    session.temp_hand=pending as i32;
    gs.weapon_progress.golden.pending_equip=0;
    true
}

/// Compose around the disc's localized, font-encoded weapon name. Do not
/// reinterpret those bytes as UTF-8, or guess a level-specific message ID.
pub fn localized_notice(item:usize,lang:u32,native_name:&[u8])->Vec<u8> {
    if native_name.is_empty(){return notice(item).to_vec();}
    let prefix:&[u8]=match lang {
        2=>b"Arme obtenue : ",3=>b"Erhalten: ",4=>b"Arma obtenida: ",5=>b"Arma ottenuta: ",
        6|7=>b"",_=>b"Acquired: ",
    };
    let mut text=prefix.to_vec();text.extend_from_slice(native_name);text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::weapon_evo::save::Progress;
    #[test]
    fn every_seed_has_fifteen_distinct_weapons_and_no_tool_behind_its_own_gate() {
        assert_eq!(layout::locations(false).len(),48);
        for bias in [0, 1, 50, 99, 100] {
            for seed in 1..=500 {
                let s = State::generate(seed, bias);
                assert!(s.valid());
                assert_eq!(s,State::generate(seed,bias));
                assert_eq!(s.placements.iter().filter(|&&i|i!=0).count(),15);
                for (slot, &item) in s.placements.iter().enumerate() {
                    assert!(layout::placement_allowed(slot,item as usize));
                }
                assert!(s.progression_safe(),"Gemlik access must have an available tool; no early category is reserved");
            }
        }
        assert_ne!(State::generate(1,0).placements,State::generate(2,0).placements);
        assert_ne!(fresh_seed(),fresh_seed());
    }
    #[test]
    fn full_bias_uses_earliest_accessible_set_and_zero_keeps_late_bolts_possible() {
        for seed in 1..=100 {
            let s = State::generate(seed, 100);
            assert!(s.placements.iter().enumerate().filter(|(_,i)|**i!=0).all(|(slot,_)|layout::access(slot)<=11));
            assert!(s.placements.iter().enumerate().all(|(slot,&i)|i==0||layout::level(slot).is_some_and(|l|l<12)));
        }
        assert!((1..100).any(|seed|State::generate(seed,0).placements[18*4..].iter().any(|&i|i!=0)));
    }
    #[test]
    fn saved_layout_claims_and_bias_survive_round_trip_and_ini_changes() {
        let mut e = super::super::Evolution::default();
        let mut p = Progress::default();
        p.golden = State::generate(6789, 81);
        p.golden.collect(13);
        let bytes = p.encode(b"slot").unwrap();
        let loaded = Progress::decode(&bytes, b"slot").unwrap();
        assert_eq!(p,loaded);
        assert!(loaded.golden.collected(13));
        e.restore_progress(&loaded);
        e.golden_settings.early_bias_percent = 0;
        e.sync_progress(&mut p);
        assert_eq!(p.golden,loaded.golden);
        let mut fresh = Progress::default();
        fresh.golden = State::generate(123, 0);
        e.sync_progress(&mut fresh);
        assert_eq!(e.golden_progress(),fresh.golden);
        assert!(!fresh.golden.collected(13));
        let mut bad = p;
        bad.golden.placements[0] = 13;
        assert!(bad.encode(b"slot").is_err());
    }
    #[test]
    fn mode_save_guard_rejects_ordinary_missing_and_corrupt_companions() {
        let root =
            std::env::temp_dir().join(format!("golden-guard-{}",fresh_seed()));
        std::fs::create_dir_all(&root).unwrap();
        let slot = root.join("save1");
        let native = b"native";
        assert!(!accepts_save(&slot,native,true));
        assert!(accepts_save(&slot,native,false));
        let mut p = Progress::default();
        p.golden = State::generate(38, 50);
        super::super::save::prepare(&slot, native, &p).unwrap();
        assert!(accepts_save(&slot,native,true));
        assert!(!accepts_save(&slot,native,false));
        assert!(!accepts_save(&slot,b"other",true));
        std::fs::write(super::super::save::companion_path(&slot,
                        native).unwrap(), b"bad").unwrap();
        assert!(!accepts_save(&slot,native,true));
        assert_eq!(card_root(root.clone(),true),root.join("golden_bolts_to_weapons"));
        assert_eq!(card_root(root.clone(),false),root);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn replaced_bolt_waits_for_visible_model_and_only_commits_after_inventory_grant() {
        use crate::moby_update::classes::bomb_water::tests::{Bench, pool};
        let mut b = Bench::new(pool(10., -10.), &[1134], [20., 20., 10.]);
        let p = State::generate(8, 100);
        let slot = p.placements.iter().position(|&i| i != 0).unwrap();
        b.svc.level = (slot / 4) as u32;
        b.svc.bomb_evo.golden_settings.enabled = true;
        b.svc.bomb_evo.state_progress.golden = p;
        let id = b.create(1134);
        b.table.mobys[id].position = [20., 20., 10., 1.];
        crate::moby_update::services::pvar::set_i32(&mut b.table.mobys[id].pvars,
            0, (slot % 4) as i32);
        let item = p.placements[slot] as usize;
        let mut w =
            World::new(&mut b.table, &b.hero, &mut b.rng, &b.ct, &mut b.svc,
                0);
        assert!(replace_bolt(&mut w,id));
        tick(&mut w);
        assert!(!w.svc.golden.pending(item));
        assert!(w.svc.interact.writes.is_empty());
        w.svc.golden.set_ready(item);
        tick(&mut w);
        assert!(w.svc.golden.pending(item));
        assert!(!w.svc.bomb_evo.golden_progress().collected(item));
        assert!(w.svc.golden.visuals(&p,true).is_empty());
        assert!(!w.svc.interact.writes.iter().any(|x|matches!(x,interact::GameWrite::GoldBolt{..})));
        // A model readiness survives a checkpoint reload, but actor IDs do not.
        w.svc.golden.reset_for_reload();
        assert!(replace_bolt(&mut w,id));
        assert_eq!(w.svc.golden.visuals(&p,true).len(),1);
        w.svc.bomb_evo.state_progress.golden.collect(item);
        assert!(replace_bolt(&mut w,id));
        assert!(w.m(id).state>=128);
    }
    #[test]
    fn ini_bounds_and_default_off_are_explicit() {
        let mut s = Settings::default();
        assert!(!s.enabled);
        assert!(s.parse_field(ENABLE_KEY,"on").unwrap());
        assert!(s.enabled);
        assert!(s.parse_field(BIAS_KEY,"100").unwrap());
        for v in ["-1", "101", "NaN", "5.5"] {
            assert!(s.parse_field(BIAS_KEY,v).is_err());
        }
        assert!(s.parse_field(ENABLE_KEY,"maybe").is_err());
    }
    #[test]
    fn physical_ammo_free_rewards_stay_unlocked_when_survival_mode_is_also_enabled() {
        use rc_formats::save_game::{
            ChunkTables, ItemRecord, ITEM_COUNT, VENDOR_TABLE_LEN,
        };
        let items =
            ItemTables {
                vendor_items: [0; VENDOR_TABLE_LEN],
                records: [ItemRecord([0; 0x18]); ITEM_COUNT],
                slot_type: [0; ITEM_COUNT],
                item_defs_addr: 0,
            };
        let mut gs =
            GameState::zeroed(ChunkTables { global: vec![], level: vec![] });
        let mut session = SessionState::default();
        gs.weapon_progress.golden = State::generate(71, 0);
        super::super::ammo_mode::reconcile_configured(&mut gs, &mut session,
            true, 0, false, &Default::default());
        for item in [9, 14, 18, 21] {
            assert_eq!(gs.global.owned[item],0);
            grant(&mut gs, &mut session, &items, item, &Default::default());
            super::super::ammo_mode::reconcile_configured(&mut gs,
                &mut session, true, 0, false, &Default::default());
            assert_eq!(gs.global.owned[item],1);
            assert!(gs.global.quick_select.contains(&(item as i32)));
        }
        assert_eq!(gs.weapon_progress.ammo_mode.normal_bolts,0);
    }

}

#[cfg(test)]
#[path="weapon_evo_golden/tests.rs"]
mod expanded_tests;
