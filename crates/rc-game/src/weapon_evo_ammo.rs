//! Optional ammunition-based arsenal. Native save chunks keep their layout;
//! the versioned Weapon Evolution companion stores mode state per save slot.
//! Shops and drop selection consult the published profile flag, while ownership
//! reconciliation takes an explicit flag so it is deterministic and testable.
use crate::game_state::{GameState, SessionState};

pub const KEY: &str = "experimental_ammo_unlock_mode";
/// All ammunition weapons, including R.Y.N.O.; no utility gadgets enter this pool.
pub const AMMO_IDS: [usize; 11] = [10, 15, 16, 11, 17, 19, 20, 24, 13, 25, 23];
pub const PERMANENT: [(usize, u64); 4] = [(18, 0), (14, 25_000), (9, 50_000), (21, 75_000)];
const MASK: u32 = (1 << 9) | (1 << 10) | (1 << 11) | (1 << 13) | (1 << 14)
    | (1 << 15) | (1 << 16) | (1 << 17) | (1 << 18) | (1 << 19)
    | (1 << 20) | (1 << 21) | (1 << 23) | (1 << 24) | (1 << 25);

pub fn enabled() -> bool { super::extra_flag(-1, KEY, false) }
pub fn is_weapon(id: usize) -> bool { id < 32 && MASK & (1 << id) != 0 }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub active: bool,
    /// Ownership and quick select to restore if this experimental mode is disabled.
    pub restore_owned: u32,
    pub restore_quick: [i32; 8],
    pub available: u32,
    pub next_quick: u32,
    /// Collected normal bolts, never current wallet balance or Gold Bolts.
    pub normal_bolts: u64,
}
impl State {
    pub fn valid(&self) -> bool {
        (self.restore_owned | self.available) & !MASK == 0 && self.next_quick < 8
            && self.restore_quick.iter().all(|&i| (0..=38).contains(&i))
    }
    pub fn record_native_grant(&mut self, id: usize) {
        if self.active && is_weapon(id) { self.restore_owned |= 1 << id; }
    }
}

fn quick_insert(quick: &mut [i32; 8], item: usize, state: &mut State, equipped: i32) {
    if quick.contains(&(item as i32)) { return; }
    let slot = quick.iter().position(|&id| id == 0).unwrap_or_else(|| {
        // A finite eight-slot ring cannot contain the entire arsenal. Rotate the
        // replacement slot, avoiding the equipped item whenever possible.
        let start = state.next_quick as usize;
        (0..8).map(|n| (start + n) % 8).find(|&n| quick[n] != equipped).unwrap_or(start)
    });
    quick[slot] = item as i32;
    state.next_quick = ((slot + 1) % 8) as u32;
}

/// Run before the hero's inventory mirror is copied, then after that tick's
/// pickups, ammo use, normal-bolt collection and native story item grants.
pub fn reconcile(gs: &mut GameState, session: &mut SessionState, on: bool, collected: u64) {
    reconcile_with_guidance(gs, session, on, collected, false);
}

/// The last Visibomb must finish its guided flight before a forced hand swap.
/// Ownership and the ring still lock immediately; only the swap is deferred.
pub fn reconcile_with_guidance(gs: &mut GameState, session: &mut SessionState, on: bool, collected: u64, guiding: bool) {
    let mut mode = gs.weapon_progress.ammo_mode;
    if !on {
        if mode.active {
            for id in 0..gs.global.owned.len() {
                if is_weapon(id) { gs.global.owned[id] = u8::from(mode.restore_owned & (1 << id) != 0); }
            }
            gs.global.quick_select = mode.restore_quick;
            mode.active = false;
            mode.available = 0;
            sanitize(gs, session, guiding);
        }
        gs.weapon_progress.ammo_mode = mode;
        return;
    }
    if !mode.active {
        mode.restore_owned = (0..gs.global.owned.len()).filter(|&id| is_weapon(id) && gs.global.owned[id] != 0)
            .fold(0, |mask, id| mask | (1 << id));
        mode.restore_quick = gs.global.quick_select;
        // Existing saves already track collected normal bolts per planet. Seed
        // once; subsequent collections use a monotonic per-slot counter.
        let previous: u64 = gs.levels.iter().map(|l| l.bolts.max(0) as u64).sum();
        mode.normal_bolts = mode.normal_bolts.max(previous);
        mode.available = 0;
        mode.active = true;
    }
    mode.normal_bolts = mode.normal_bolts.saturating_add(collected);
    let mut available = 0;
    for id in AMMO_IDS {
        if gs.global.ammo[id] > 0 { available |= 1 << id; }
    }
    for (id, threshold) in PERMANENT {
        if mode.normal_bolts >= threshold { available |= 1 << id; }
    }
    for id in 0..gs.global.owned.len() {
        if is_weapon(id) { gs.global.owned[id] = u8::from(available & (1 << id) != 0); }
    }
    for item in &mut gs.global.quick_select {
        if *item > 0 && is_weapon(*item as usize) && available & (1 << *item) == 0 { *item = 0; }
    }
    let newly_available = available & !mode.available;
    for id in 0..gs.global.owned.len() {
        if is_weapon(id) && newly_available & (1 << id) != 0 {
            quick_insert(&mut gs.global.quick_select, id, &mut mode, gs.global.equipped[0]);
        }
    }
    mode.available = available;
    sanitize(gs, session, guiding);
    gs.weapon_progress.ammo_mode = mode;
}

fn sanitize(gs: &mut GameState, session: &mut SessionState, guiding: bool) {
    let locked = |id: i32| id > 0 && is_weapon(id as usize) && gs.global.owned[id as usize] == 0;
    if !guiding && (locked(gs.global.equipped[0]) || locked(session.temp_hand)) { session.temp_hand = 8; }
    if gs.global.owned[24] == 0 { session.drone = false; }
    if locked(gs.global.last_hand_item) { gs.global.last_hand_item = 8; }
}

/// Integer tickets give exact endpoint distributions without float rounding:
/// 0 Gold Bolts: 50/40/8/2%; 40+: 25/25/25/25%. Every intermediate count
/// transfers tickets from one/two packs to three/four packs linearly.
pub fn pack_weights(gold_bolts: u32) -> [i32; 4] {
    let g = gold_bolts.min(40) as i32;
    [2000 - 25*g, 1600 - 15*g, 320 + 17*g, 80 + 23*g]
}
pub fn pack_count(gold_bolts: u32, ticket: i32) -> i32 {
    let mut ticket = ticket.rem_euclid(4000);
    for (i, weight) in pack_weights(gold_bolts).into_iter().enumerate() {
        if ticket < weight { return i as i32 + 1; }
        ticket -= weight;
    }
    unreachable!("pack weights always total 4000")
}
pub fn weapon_weight(id: usize) -> i32 {
    match id { 10 | 15 | 16 => 30, 13 | 25 => 3, 23 => 1, _ => 10 }
}
pub fn weighted_item(items: &[usize], ticket: i32) -> Option<usize> {
    let sum: i32 = items.iter().map(|&id| weapon_weight(id)).sum();
    if sum == 0 { return None; }
    let mut ticket = ticket.rem_euclid(sum);
    for &id in items {
        let weight = weapon_weight(id);
        if ticket < weight { return Some(id); }
        ticket -= weight;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn game() -> GameState { GameState::zeroed(rc_formats::save_game::ChunkTables { global: vec![], level: vec![] }) }

    #[test]
    fn pack_distribution_is_exact_and_monotonic_for_every_gold_count() {
        let mut previous = pack_weights(0);
        for gold in 0..=40 {
            let weights = pack_weights(gold);
            let mut observed = [0; 4];
            for ticket in 0..4000 { observed[(pack_count(gold, ticket)-1) as usize] += 1; }
            assert_eq!(weights, observed);
            assert_eq!(weights.iter().sum::<i32>(), 4000);
            assert!(weights[0] <= previous[0] && weights[1] <= previous[1]);
            assert!(weights[2] >= previous[2] && weights[3] >= previous[3]);
            previous = weights;
        }
        assert_eq!(pack_weights(0), [2000,1600,320,80]);
        assert_eq!(pack_weights(40), [1000;4]);
        assert_eq!(pack_weights(u32::MAX), [1000;4]);
    }
    #[test]
    fn every_ammo_weapon_is_possible_with_explicit_rarity() {
        let total: i32 = AMMO_IDS.iter().map(|&i| weapon_weight(i)).sum();
        for id in AMMO_IDS {
            assert_eq!((0..total).filter(|&t| weighted_item(&AMMO_IDS,t)==Some(id)).count(), weapon_weight(id) as usize);
        }
        assert!(weapon_weight(23)<weapon_weight(13) && weapon_weight(13)<weapon_weight(11));
        assert_eq!(weighted_item(&[],0),None);
    }
    #[test]
    fn last_ammo_locks_and_refill_reinserts_even_with_a_full_ring() {
        let mut g=game();let mut s=SessionState::default();
        g.global.quick_select=[12,22,26,27,31,32,10,15];
        g.global.owned[10]=1;g.global.ammo[10]=3;g.global.equipped[0]=10;
        reconcile(&mut g,&mut s,true,0);
        assert!(g.global.quick_select.contains(&10));
        g.global.ammo[10]=0;reconcile(&mut g,&mut s,true,0);
        assert_eq!(g.global.owned[10],0);assert!(!g.global.quick_select.contains(&10));assert_eq!(s.temp_hand,8);
        g.global.quick_select=[12,22,26,27,31,32,15,16];
        g.global.ammo[10]=3;reconcile(&mut g,&mut s,true,0);
        assert_eq!(g.global.owned[10],1);assert!(g.global.quick_select.contains(&10));
        assert_eq!(g.global.acquired[10],0,"ammo does not fabricate story acquisition");
        reconcile(&mut g,&mut s,false,0);
        assert_eq!(g.global.owned[10],1);assert_eq!(g.global.owned[18],0);
        assert_eq!(g.global.quick_select,[12,22,26,27,31,32,10,15]);
    }
    #[test]
    fn collected_bolts_unlock_permanently_spending_does_not_and_slots_are_isolated() {
        let mut g=game();let mut s=SessionState::default();
        reconcile(&mut g,&mut s,true,24_999);assert_eq!(g.global.owned[14],0);
        reconcile(&mut g,&mut s,true,1);assert_eq!(g.global.owned[14],1);
        g.global.bolts=0;reconcile(&mut g,&mut s,true,25_000);assert_eq!(g.global.owned[9],1);
        reconcile(&mut g,&mut s,true,25_000);assert_eq!(g.global.owned[21],1);
        let mut fresh=game();reconcile(&mut fresh,&mut s,true,0);assert_eq!(fresh.global.owned[21],0);
        g.weapon_progress.ammo_mode.record_native_grant(11);
        reconcile(&mut g,&mut s,false,0);assert_eq!(g.global.owned[11],1);
    }
    #[test]
    fn last_visibomb_locks_immediately_without_cancelling_its_guided_shot() {
        let mut g=game();let mut s=SessionState::default();
        g.global.equipped[0]=13;g.global.ammo[13]=1;
        reconcile(&mut g,&mut s,true,0);
        g.global.ammo[13]=0;
        reconcile_with_guidance(&mut g,&mut s,true,0,true);
        assert_eq!(g.global.owned[13],0);assert!(!g.global.quick_select.contains(&13));
        assert_eq!(s.temp_hand,0,"keep the active guided missile's item until flight ends");
        reconcile(&mut g,&mut s,true,0);assert_eq!(s.temp_hand,8);
    }
}
