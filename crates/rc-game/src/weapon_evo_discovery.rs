//! A world reward independent of enemy combat, progression and actor allocation.
//!
//! The engine supplies immutable level data and renders `visual()`. Collection
//! uses the ordinary item grant/save path. No ship or mission flags are changed.
use crate::game_state::{GameState, SessionState};
use crate::moby_update::{interact, services::World};
use rc_formats::save_game::ItemTables;

pub const LEVEL: u32 = 13;
pub const ITEM: usize = 23;
pub const PICKUP_NOTICE: &[u8] = b"RYNO acquired!";
pub const MODEL_CLASS: i32 = crate::hero::ryno::CLASS as i32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Visual {
    pub position: [f32; 3],
    pub yaw: f32,
}

/// The placed gold offers are stable room anchors even after purchases remove
/// their live actors. Use the uppermost cluster, never Ratchet's start position.
pub fn room_centre(offers: &[[f32; 3]]) -> Option<[f32; 3]> {
    let top = offers.iter().filter(|p| p.iter().all(|x| x.is_finite()))
        .map(|p| p[2]).max_by(f32::total_cmp)?;
    let room: Vec<_> = offers.iter().filter(|p| p.iter().all(|x| x.is_finite()) && (p[2] - top).abs() <= 3.).collect();
    if !(5..=10).contains(&room.len()) { return None; }
    let centre: [f32; 3] = std::array::from_fn(|k| room.iter().map(|p| p[k]).sum::<f32>() / room.len() as f32);
    // An unexpected second distant shop must not move the reward between rooms.
    if room.iter().any(|p| (p[0] - centre[0]).hypot(p[1] - centre[1]) > 25.) { return None; }
    Some([centre[0], centre[1], centre[2] + 1.25])
}

#[derive(Clone, Debug, Default)]
pub struct State {
    anchor: Option<[f32; 3]>,
    ready: bool,
    elapsed: u64,
    visible: bool,
    claimed: bool,
    pending: bool,
}
impl State {
    pub fn from_gameplay(level: u32, gameplay: &[u8]) -> Self {
        if level != LEVEL { return Self::default(); }
        let anchor = rc_formats::gameplay::parse_moby_instances(gameplay).ok()
            .zip(rc_formats::gameplay::parse_pvars(gameplay).ok())
            .and_then(|(instances, vars)| {
                let offers: Vec<_> = instances.iter().filter(|m| m.o_class == 304)
                    .filter(|m| m.pvar(&vars).and_then(|p| p.get(0x40..0x44))
                        .map(|p| i32::from_le_bytes(p.try_into().unwrap())).is_some_and(|k| (0..10).contains(&k)))
                    .map(|m| m.position).collect();
                room_centre(&offers)
            });
        if let Some(p) = anchor { eprintln!("world reward: Gemlik RYNO location {p:?}"); }
        else { eprintln!("world reward: Gemlik gold room could not be identified; no pickup installed"); }
        Self { anchor, ..Default::default() }
    }

    pub fn has_location(&self) -> bool { self.anchor.is_some() }
    pub fn location(&self) -> Option<[f32;3]> { self.anchor }
    /// A missing model must never leave an invisible, collectible weapon.
    pub fn set_ready(&mut self) { self.ready = true; }
    pub fn grant_pending(&self) -> bool { self.pending }
    pub fn finish_grant(&mut self) { self.pending = false; }
    pub fn reset_for_reload(&mut self) {
        self.elapsed = 0;
        self.visible = false;
        self.claimed = false;
        self.pending = false;
    }
    pub fn visual(&self) -> Option<Visual> {
        if !self.visible { return None; }
        let mut position = self.anchor?;
        let spin = super::pickup_visuals::spin_angle(self.elapsed);
        let bob = (self.elapsed % 180) as f32 * std::f32::consts::TAU / 180.;
        position[2] += 0.15 * bob.sin();
        Some(Visual { position, yaw: spin })
    }
    fn update(&mut self, w: &mut World) {
        let acquired = w.svc.interact.game.acquired.get(ITEM).is_some_and(|&b| b != 0);
        self.visible = w.svc.bomb_evo.hide_ryno && !w.svc.bomb_evo.golden_settings.enabled && w.svc.level == LEVEL && w.svc.bomb_evo.enabled && self.ready
            && self.anchor.is_some() && !self.claimed && !acquired
            && !w.hero.owned.has(ITEM) && w.hero.health > 0;
        if !self.visible { return; }
        self.elapsed = self.elapsed.wrapping_add(1);
        if w.svc.game_mode != 0 || !matches!(w.hero.mode, 0 | 3) { return; }
        let pos = self.visual().unwrap().position;
        let hero = w.hero.pos.map(|p| p.to_f32());
        if (pos[0] - hero[0]).hypot(pos[1] - hero[1]) > 1.6 || (pos[2] - hero[2]).abs() > 2. {
            return;
        }
        let from = [hero[0], hero[1], hero[2] + 0.75, 0.].map(crate::ps2v::Pf::f);
        let to = [pos[0], pos[1], pos[2], 0.].map(crate::ps2v::Pf::f);
        if w.line(from, to, 0x12, w.hero_moby).is_some() { return; }
        self.claimed = true;
        self.pending = true;
        self.visible = false; // Model and glow stop in the same render frame.
        // The generic GiveItem banner table contains no reliable RYNO message
        // on Gemlik. Keep the native grant, but supply our own literal banner.
        if let Some(acquired) = w.svc.interact.game.acquired.get_mut(ITEM) { *acquired = 1; }
        w.svc.interact.writes.push(interact::GameWrite::GiveItem { item: ITEM, equip: true });
        let duration = w.ticks(300);
        crate::cinematic::show_literal_banner(w, PICKUP_NOTICE, duration);
        crate::cinematic::save(w);
    }
}

pub fn tick(w: &mut World) {
    let mut state = std::mem::take(&mut w.svc.discovery);
    state.update(w);
    w.svc.discovery = state;
}

/// Reuse the native grant and quick-select behavior. Survival mode normally
/// ignores native weapon ammo grants: this physical reward supplies ammo once,
/// then the usual zero-ammo locking rules resume without an exception.
pub fn grant(gs: &mut GameState, session: &mut SessionState, items: &ItemTables,
             settings: &super::ammo_mode::Settings) {
    let mode = gs.weapon_progress.ammo_mode.active;
    gs.give_item(ITEM, true, items, session);
    let native = u16::from_le_bytes(items.records[ITEM].0[0xe..0x10].try_into().unwrap()) as i32;
    let max = super::effective_max_ammo(ITEM, native).max(1);
    let ammo = (items.records[ITEM].grant_ammo() as i32).max(1).min(max);
    gs.global.ammo[ITEM] = gs.global.ammo[ITEM].max(ammo).min(max);
    if mode {
        super::ammo_mode::reconcile_configured(gs, session, true, 0, false, settings);
    }
    session.temp_hand = ITEM as i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moby_update::classes::bomb_water::tests::Bench;
    use rc_formats::save_game::{ChunkTables, ItemRecord, ITEM_COUNT, VENDOR_TABLE_LEN};

    fn state() -> State {
        State { anchor: Some([40., 40., 11.25]), ready: true, ..Default::default() }
    }
    fn items() -> ItemTables {
        let mut t = ItemTables { vendor_items: [0; VENDOR_TABLE_LEN], records: [ItemRecord([0; 0x18]); ITEM_COUNT], slot_type: [0; ITEM_COUNT], item_defs_addr: 0 };
        t.records[ITEM].0[8..10].copy_from_slice(&1u16.to_le_bytes());
        t.records[ITEM].0[0xe..0x10].copy_from_slice(&50u16.to_le_bytes());
        t.records[ITEM].0[0x12..0x14].copy_from_slice(&25u16.to_le_bytes());
        t
    }
    #[test]
    fn room_uses_five_upper_offers_and_rejects_invalid_or_ambiguous_layouts() {
        let mut p = vec![[10.,10.,100.],[14.,10.,100.],[12.,14.,100.],[8.,14.,100.],[16.,14.,100.]];
        assert_eq!(room_centre(&p),Some([12.,12.4,101.25]));
        p.push([500.,500.,10.]); // An unrelated lower shop is not an anchor.
        assert_eq!(room_centre(&p),Some([12.,12.4,101.25]));
        assert_eq!(room_centre(&p[..4]),None);
        p[0] = [1000.,1000.,100.];
        assert_eq!(room_centre(&p),None);
        assert_eq!(room_centre(&[[f32::NAN;3];5]),None);
        assert!(!State::from_gameplay(1, &[]).has_location());
    }
    #[test]
    fn pickup_is_one_shot_removes_glow_and_queues_native_item_and_save() {
        let mut b = Bench::new(crate::hero::testkit::floor(10.,0,20,0,20), &[], [40.,40.,10.]);
        b.svc.level=LEVEL; b.svc.bomb_evo.enabled=true;b.svc.bomb_evo.hide_ryno=true; b.svc.discovery=state();
        let mut w = World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,0);
        tick(&mut w);
        assert!(w.svc.discovery.grant_pending());
        assert!(w.svc.discovery.visual().is_none());
        assert_eq!(w.svc.interact.writes.iter().filter(|x| matches!(x,interact::GameWrite::GiveItem {item:ITEM,equip:true})).count(),1);
        tick(&mut w);
        assert_eq!(w.svc.interact.writes.len(),1);
        assert!(!w.svc.cinematic.requests.is_empty());
    }
    #[test]
    fn dead_hero_missing_model_other_planet_or_cutscene_cannot_collect() {
        for case in 0..4 {
            let mut b = Bench::new(crate::hero::testkit::floor(10.,0,20,0,20), &[], [40.,40.,10.]);
            b.svc.level=LEVEL; b.svc.bomb_evo.enabled=true;b.svc.bomb_evo.hide_ryno=true; b.svc.discovery=state();
            match case {0=>b.hero.health=0,1=>b.svc.discovery.ready=false,2=>b.svc.level=1,_=>b.svc.game_mode=2}
            let mut w = World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,0);
            tick(&mut w);
            assert!(!w.svc.discovery.grant_pending());
            assert!(w.svc.interact.writes.is_empty());
        }
    }
    #[test]
    fn acquired_weapon_does_not_reappear_after_reload_even_when_survival_locks_it() {
        let mut b = Bench::new(crate::hero::testkit::floor(10.,0,20,0,20), &[], [40.,40.,10.]);
        b.svc.level=LEVEL; b.svc.bomb_evo.enabled=true;b.svc.bomb_evo.hide_ryno=true; b.svc.discovery=state();
        b.svc.interact.game.acquired.resize(rc_formats::save_game::ITEM_COUNT, 0);
        b.svc.interact.game.acquired[ITEM]=1;
        b.svc.discovery.reset_for_reload();
        let mut w = World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,0);
        tick(&mut w);
        assert!(w.svc.discovery.visual().is_none());
        assert!(w.svc.interact.writes.is_empty());
    }
    #[test]
    fn normal_grant_uses_native_ownership_ammo_equipment_and_preserves_progress() {
        let mut gs=GameState::zeroed(ChunkTables { global: vec![], level: vec![] });
        let mut session=SessionState::default();
        gs.global.quick_select=[8,10,0,0,0,0,0,0];
        gs.weapon_progress.nanotech.bonus=7;
        gs.weapon_progress.nanotech.xp=123.;
        gs.weapon_progress.tracks[0].level=3;
        gs.weapon_progress.tracks[0].xp=42.;
        let progress=gs.weapon_progress.clone();
        grant(&mut gs,&mut session,&items(),&super::super::ammo_mode::Settings::default());
        assert_eq!((gs.global.owned[ITEM],gs.global.acquired[ITEM],gs.global.ammo[ITEM]),(1,1,25));
        assert_eq!(session.temp_hand,ITEM as i32);
        assert!(gs.global.quick_select.contains(&(ITEM as i32)));
        assert_eq!(gs.weapon_progress,progress);
    }
    #[test]
    fn survival_reward_unlocks_once_then_obeys_ordinary_ammo_locking() {
        let mut gs=GameState::zeroed(ChunkTables { global: vec![], level: vec![] });
        let mut session=SessionState::default();
        let settings=super::super::ammo_mode::Settings::default();
        super::super::ammo_mode::reconcile_configured(&mut gs,&mut session,true,0,false,&settings);
        grant(&mut gs,&mut session,&items(),&settings);
        assert_eq!((gs.global.owned[ITEM],gs.global.acquired[ITEM],gs.global.ammo[ITEM]),(1,1,25));
        assert!(gs.global.quick_select.contains(&(ITEM as i32)));
        assert_eq!(session.temp_hand,ITEM as i32);
        gs.global.ammo[ITEM]=0;
        super::super::ammo_mode::reconcile_configured(&mut gs,&mut session,true,0,false,&settings);
        assert_eq!((gs.global.owned[ITEM],gs.global.acquired[ITEM]),(0,1));
        super::super::ammo_mode::reconcile_configured(&mut gs,&mut session,false,0,false,&settings);
        assert_eq!(gs.global.owned[ITEM],1,"the physical acquisition is restored outside Survival mode");
    }
}

#[cfg(test)]
mod pickup_notice_tests {
    use super::*;
    use crate::moby_update::classes::bomb_water::tests::Bench;
    #[test]
    fn gemlik_pickup_bypasses_wrong_disc_message_and_shows_exact_english_literal_once() {
        let mut b=Bench::new(crate::hero::testkit::floor(10.,0,20,0,20),&[],[40.,40.,10.]);
        b.svc.level=LEVEL;b.svc.bomb_evo.enabled=true;b.svc.bomb_evo.hide_ryno=true;
        b.svc.discovery=State { anchor:Some([40.,40.,11.25]),ready:true,..Default::default() };
        b.svc.interact.game.acquired=vec![0;37];
        // A bad/missing RYNO message from Gemlik must never be looked up.
        std::sync::Arc::make_mut(&mut b.svc.interact.tables).give_banners=vec![999999;74];
        let mut w=World::new(&mut b.table,&b.hero,&mut b.rng,&b.ct,&mut b.svc,0);
        tick(&mut w);
        assert_eq!(w.svc.cinematic.banner.literal,Some(PICKUP_NOTICE));
        assert_eq!(w.svc.cinematic.banner.msg,-1);
        assert_eq!(w.svc.cinematic.banner.ticks,w.ticks(300));
        assert_eq!(w.svc.cinematic.banner.seq,1);
        assert_eq!(w.svc.interact.game.acquired[ITEM],1);
        assert!(w.svc.discovery.visual().is_none());
        tick(&mut w);
        assert_eq!(w.svc.cinematic.banner.seq,1);
        assert_eq!(w.svc.interact.writes.len(),1);
    }
}
