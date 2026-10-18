//! Absolute HP are a unit conversion: 1 native HP = 100 displayed/configured HP.
//! Keep native records normalized so AI phase thresholds, regeneration and stun
//! code retain their original units. Only the confirmed XP donors below opt in.
use std::collections::{HashMap, HashSet, VecDeque};
use crate::moby_runtime::{Moby, MobyId, MobyTable};
use super::services::{HitRecord, HitTemplate};
use crate::ps2v::Pf;

pub const HP_SCALE: f32 = 100.0;

/// The source at impact time. Its moby can be deleted before the target updates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Origin {
    pub class: i16,
    pub position: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attack {
    pub origin: Option<Origin>,
    pub instance: u64,
    pub weapon: i32,
    /// Native units. Multiplying by HP_SCALE gives actual HP.
    pub damage: f32,
    /// A hit can hurt without restarting an existing native hurt animation.
    pub damage_only: bool,
}

#[derive(Clone, Debug, Default)]
struct StreamPulse {
    contact_ticks: u32,
    damage: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Events {
    pub enabled: bool,
    pub epoch: u64,
    pub frame: u64,
    pub hero: Option<MobyId>,
    pub melee_serial: u64,
    pub values: HashMap<i32, f32>,
    /// Pyrocitor, Tesla: number of actual contact ticks per damage pulse.
    pub stream_intervals: [u32; 2],
    streams: HashMap<(MobyId, MobyId, i32), StreamPulse>,
    next: u64,
    muzzle_bound: bool,
    sources: HashMap<(MobyId, i32, u64), u64>,
    queued: HashSet<(MobyId, u64)>,
    resolved: HashSet<(MobyId, u64)>,
    terminal_seen: HashSet<(MobyId, u64)>,
    pending: HashMap<MobyId, VecDeque<HitRecord>>,
    /// Confirmed partial HP for enemies whose native class only has a kill branch.
    logical: HashMap<MobyId, f32>,
}

pub fn donor_class(class: i16) -> bool {
    matches!(class, 8 | 28 | 29 | 30 | 31 | 41 | 44 | 49 | 50 | 52 | 63 | 77 | 79 | 81 | 101 | 111 | 170 | 193 | 217 | 221 | 231 | 233 | 238 | 248 | 252 | 253 | 294 | 325 | 326 | 331 | 333 | 336 | 340 | 347 | 388 | 403 | 409 | 425 | 427 | 435 | 438 | 441 | 442 | 443 | 444 | 445 | 448 | 449 | 450 | 451 | 452 | 455 | 459 | 462 | 463 | 464 | 465 | 466 | 480 | 485 | 486 | 488 | 490 | 491 | 493 | 494 | 495 | 498 | 541 | 545 | 555 | 563 | 568 | 572 | 573 | 574 | 577 | 578 | 580 | 612 | 621 | 623 | 625 | 627 | 631 | 638 | 643 | 660 | 668 | 686 | 688 | 696 | 697 | 698 | 700 | 722 | 749 | 819 | 827 | 833 | 835 | 848 | 865 | 866 | 871 | 882 | 908 | 921 | 922 | 923 | 934 | 1023 | 1041 | 1048 | 1051 | 1059 | 1068 | 1106 | 1110 | 1112 | 1172 | 1193 | 1196 | 1199 | 1202 | 1212 | 1213 | 1218 | 1229 | 1231 | 1233 | 1238 | 1246 | 1261 | 1262 | 1269 | 1292 | 1297 | 1319 | 1356 | 1382 | 1401 | 1422 | 1440 | 1445 | 1471 | 1516 | 1634 | 1805 | 1843 | 1885 | 1906 | 1973 | 933)
}

pub fn terminal_class(class: i16) -> bool {
    matches!(class, 686 | 688 | 696 | 697 | 698 | 700 | 660 | 221 | 1445 | 621 | 835 | 77 | 438 | 435
        | 643 | 79 | 1212 | 1213 | 1973 | 923 | 1401 | 466 | 480 | 485 | 486 | 488 | 490 | 493 | 494
        | 495 | 498 | 555 | 336 | 441 | 442 | 443 | 444 | 445 | 448 | 449 | 450 | 451 | 462 | 463 | 464
        | 465 | 568 | 452 | 1319 | 1843 | 908 | 921 | 922 | 871 | 1110 | 933)
}

pub fn donor(m: &Moby) -> bool {
    m.state < 0x80 && m.o_class != 1126 && donor_class(m.o_class)
}

pub fn weapon(t: &HitTemplate, hero: Option<MobyId>) -> Option<i32> {
    Some(match t.h1a {
        121 => 10, 153 => 11, 172 => 13, 305 | 168 => 15,
        176 if t.b18 == 5 && t.attacker == hero => 16,
        0 if t.b18 == 5 && hero.is_some() && t.attacker == hero => 16,
        175 if t.attacker == hero => 14,
        177 => 19, 74 => 17, 180 => 18, 186 if t.b18 == 2 && t.b19 == 3 => 20,
        0x351 => 9, 457 => 23, 479 => 24, 203 | 1900 => 25, 270 => 21,
        _ => return None,
    })
}

impl Events {
    pub fn on_spawn(&mut self, id: MobyId, class: i16) {
        if !self.enabled || class != 305 { return; }
        let Some(hero) = self.hero else { return; };
        let key=(hero,15,self.frame);
        let reserved=if self.muzzle_bound { None } else { self.sources.get(&key).copied() };
        let instance=match reserved {
            Some(n)=>n, None=>{ self.next+=1; if !self.muzzle_bound { self.sources.insert(key,self.next); } self.next }
        };
        self.muzzle_bound=true;
        // The muzzle hit and the emitted bullet belong to this same shot.
        self.sources.insert((id,15,u64::MAX),instance);
    }
    /// True means delivery was handled, including a duplicate or contact-only sweep.
    pub fn queue(&mut self, table: &MobyTable, target: MobyId, t: &HitTemplate, sweep: bool, pos: [Pf; 4], prim: i32) -> bool {
        if !self.enabled || !table.mobys.get(target).is_some_and(donor) { return false; }
        let Some(item) = weapon(t, self.hero) else {
            // A Doom bot's contact never becomes HP damage; detonation owns it.
            return t.h1a == 186;
        };
        // Direct missile/grenade contact triggers the blast but must not double it.
        if sweep && matches!(item, 10 | 11 | 13 | 20 | 23) { return true; }
        let Some(attacker) = t.attacker else { return false; };
        let stream = item == 16 || item == 19;
        let phase = if item == 18 { self.melee_serial } else if stream || item == 14 || t.h1a == 168 { self.frame } else { u64::MAX };
        let key = (attacker, item, phase);
        let instance = match self.sources.get(&key).copied() {
            Some(id) => id,
            None => {
                self.next = self.next.checked_add(1).expect("weapon attack id overflow");
                self.sources.insert(key, self.next); self.next
            }
        };
        if !self.queued.insert((target, instance)) {
            // Tesla's targeted electric subtype takes precedence over the
            // near-nozzle sphere, without creating another HP event.
            if item == 19 {
                if let Some(record)=self.pending.get_mut(&target).and_then(|q| q.iter_mut().find(|r| r.evolution.is_some_and(|a| a.instance == instance))) {
                    if t.b19 > record.b29 { record.b29=t.b19;record.dir=t.dir; }
                }
            }
            return true;
        }
        let hp = self.values.get(&item).copied().unwrap_or_else(|| t.damage.to_f32() * HP_SCALE);
        let mut damage = hp / HP_SCALE / if stream { 60.0 } else { 1.0 };
        if !damage.is_finite() || damage <= 0.0 { return true; }
        if stream {
            let interval = self.stream_intervals[usize::from(item == 19)].clamp(1, 600);
            let pulse = self.streams.entry((target, attacker, item)).or_default();
            pulse.damage += damage;
            pulse.contact_ticks += 1;
            // Only real contact contributes: no full-interval burst on a tap,
            // and no damage or XP is delivered while the target is out of reach.
            if pulse.contact_ticks < interval { return true; }
            damage = pulse.damage;
            *pulse = StreamPulse::default();
        }
        // The port uses Ratchet's moby as the source of both continuous weapons.
        // Preserve his position/ownership, but expose the weapon class to native
        // enemy filters: otherwise fire/electricity is mistaken for the wrench.
        let stream_class = match item { 16 => Some(176), 19 => Some(177), _ => None };
        let r = HitRecord { pos, dir: t.dir, attacker: t.attacker, flags: t.flags,
            b28: t.b18, b29: t.b19, h2a: stream_class.unwrap_or(t.h1a as i16) as u16, damage: Pf::f(damage), w30: t.w20,
            target, prim, doom_explosion: 0,
            evolution: Some(Attack { instance, weapon: item, damage, damage_only: false,
                origin: table.mobys.get(attacker).map(|m| Origin { class: stream_class.unwrap_or(m.o_class), position: m.position }) }) };
        self.pending.entry(target).or_default().push_back(r);
        true
    }

    pub fn frozen_next(&self, target: MobyId) -> Option<HitRecord> {
        self.pending.get(&target)?.iter().find(|r| r.evolution.is_some_and(|a|
            !self.resolved.contains(&(target,a.instance)) && !self.terminal_seen.contains(&(target,a.instance)))).copied()
    }
    pub fn finished(&self, target: MobyId, attack: Attack) -> bool {
        self.resolved.contains(&(target,attack.instance)) || self.terminal_seen.contains(&(target,attack.instance))
    }
    pub fn requeue(&mut self, target: MobyId, mut record: HitRecord) {
        let Some(a) = record.evolution else { return; };
        if self.finished(target,a) || !self.queued.insert((target,a.instance)) { return; }
        record.target=target; self.pending.entry(target).or_default().push_back(record);
    }
    pub fn take(&mut self, target: MobyId, mask: u32) -> Option<HitRecord> {
        let q = self.pending.get_mut(&target)?;
        while q.front().is_some_and(|r| r.flags & mask == 0 || r.evolution.is_some_and(|a|
            self.resolved.contains(&(target,a.instance)) || self.terminal_seen.contains(&(target,a.instance)))) { q.pop_front(); }
        let r = q.pop_front();
        if q.is_empty() { self.pending.remove(&target); }
        r
    }

    pub fn resolve_once(&mut self, target: MobyId, attack: Attack) -> bool {
        self.resolved.insert((target, attack.instance))
    }

    pub fn terminal_once(&mut self, target: MobyId, instance: u64) -> bool { self.terminal_seen.insert((target, instance)) }
    pub fn logical_health(&self, target: MobyId) -> f32 { self.logical.get(&target).copied().unwrap_or(1.0) }
    /// Called only in an already accepted, native weapon-destruction branch.
    pub fn terminal_loss(&mut self, target: MobyId, damage: f32) -> (f32, bool) {
        let hp = self.logical.entry(target).or_insert(1.0);
        let loss = damage.max(0.0).min(*hp);
        *hp = (*hp - loss).max(0.0);
        (loss, *hp <= 0.000_001)
    }
    pub fn forget(&mut self, id: MobyId) {
        self.pending.remove(&id); self.logical.remove(&id);
        // Keep confirmed hits on other targets, but never let their source pointer
        // follow a recycled slot. Origin and weapon attribution remain immutable.
        for queue in self.pending.values_mut() {
            for hit in queue {
                if hit.attacker == Some(id) { hit.attacker = None; }
            }
        }
        self.streams.retain(|(target, source, _), _| *target != id && *source != id);
        self.queued.retain(|(t, _)| *t != id); self.resolved.retain(|(t, _)| *t != id);
        self.terminal_seen.retain(|(t, _)| *t != id);
        self.sources.retain(|(source, _, _), _| *source != id);
    }
    pub fn begin_frame(&mut self, frame: u64) {
        if self.frame != frame {
            // Per-frame flow identities need not survive after their records were copied.
            self.sources.retain(|(_, item, phase), _| *phase == u64::MAX || (*item == 18 && *phase == self.melee_serial));
            self.frame = frame;
            self.muzzle_bound=false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impact_origin_survives_source_deletion_and_reuse_by_a_different_class() {
        let mut t = table();
        t.mobys.push(Moby { o_class: 305, state: 1, position: [10.0, 20.0, 30.0, 1.0], ..Moby::default() });
        let mut e = Events { enabled: true, ..Default::default() };
        e.queue(&t, 0, &template(305, 1), true, [Pf::ZERO; 4], 0);
        e.forget(1);
        t.mobys[1] = Moby { o_class: 1049, state: 1, position: [90.0; 4], ..Moby::default() };
        let hit = e.take(0, u32::MAX).unwrap();
        assert_eq!(hit.attacker, None);
        assert_eq!(hit.attacker_class(&t), Some(305));
        assert_eq!(hit.attacker_position(&t), Some([10.0, 20.0, 30.0, 1.0]));
        assert_eq!(hit.evolution.unwrap().weapon, 15);
    }

    #[test]
    fn independent_hit_origin_survives_removal_of_the_source_slot() {
        let mut t = table();
        t.mobys.push(Moby { o_class: 153, state: 1, ..Moby::default() });
        let mut e = Events { enabled: true, ..Default::default() };
        e.queue(&t, 0, &template(153, 1), false, [Pf::ZERO; 4], 0);
        e.forget(1);
        t.mobys.pop();
        let hit = e.take(0, u32::MAX).unwrap();
        assert_eq!(hit.attacker_class(&t), Some(153));
        assert_eq!(hit.evolution.unwrap().weapon, 11);
    }

    #[test]
    fn native_hit_identity_keeps_its_pointer_based_semantics() {
        let mut t = table();
        let hit = HitRecord { attacker: Some(0), ..Default::default() };
        assert_eq!(hit.attacker_class(&t), Some(577));
        t.mobys[0].o_class = 749;
        assert_eq!(hit.attacker_class(&t), Some(749));
        assert_eq!(HitRecord::default().attacker_class(&t), None);
    }

    fn table() -> MobyTable {
        let mut t = MobyTable::default();
        t.mobys.push(Moby { o_class: 577, state: 1, ..Moby::default() }); t
    }
    fn template(item: u16, source: usize) -> HitTemplate {
        HitTemplate { attacker: Some(source), flags: 0x83_0000, h1a: item,
            b18: 3, b19: 1, damage: Pf::ONE, ..Default::default() }
    }
    #[test]
    fn one_projectile_per_enemy_and_independent_second_projectile() {
        let t = table(); let mut e = Events { enabled: true, ..Default::default() };
        e.values.insert(11, 300.0);
        e.queue(&t, 0, &template(153, 20), true, [Pf::ZERO;4], 0);
        assert!(e.take(0, u32::MAX).is_none(), "contact only triggers detonation");
        for _ in 0..20 { e.queue(&t, 0, &template(153,20), false,[Pf::ZERO;4],0); }
        e.queue(&t,0,&template(153,21),false,[Pf::ZERO;4],0);
        let a=e.take(0,u32::MAX).unwrap(); let b=e.take(0,u32::MAX).unwrap();
        assert_eq!(a.damage.to_f32(),3.0); assert_ne!(a.evolution.unwrap().instance,b.evolution.unwrap().instance);
        assert!(e.take(0,u32::MAX).is_none());
    }
    #[test]
    fn continuous_flow_counts_once_per_target_per_tick() {
        let t=table(); let mut e=Events { enabled:true,hero:Some(10),..Default::default() };
        e.values.insert(19,240.0); let mut damage=0.0;
        for tick in 0..60 { e.begin_frame(tick); for _ in 0..20 { e.queue(&t,0,&template(177,10),false,[Pf::ZERO;4],0); }
            damage+=e.take(0,u32::MAX).unwrap().damage.to_f32(); assert!(e.take(0,u32::MAX).is_none()); }
        assert!((damage*HP_SCALE-240.0).abs()<0.001);
    }
    #[test]
    fn continuous_weapon_identity_is_not_ratchets_body_or_the_wrench() {
        for (item, class) in [(16, 176), (19, 177)] {
            let mut t = table();
            t.mobys.push(Moby { o_class: 0, state: 1, position: [10.0; 4], ..Default::default() });
            let mut e = Events { enabled: true, hero: Some(1), stream_intervals: [1; 2], ..Default::default() };
            let mut tmpl = template(class, 1); tmpl.b18 = 5;
            if item == 16 { tmpl.h1a = 0; } // direct nozzle hit
            e.queue(&t, 0, &tmpl, false, [Pf::ZERO; 4], 0);
            let hit = e.take(0, u32::MAX).unwrap();
            assert_eq!(hit.h2a, class);
            assert_eq!(hit.attacker_class(&t), Some(class as i16));
            assert_eq!(hit.attacker_position(&t), Some([10.0; 4]));
            assert_eq!(hit.evolution.unwrap().weapon, item);
        }
    }
    fn flow_template(class: u16) -> HitTemplate {
        let mut t = template(class, 10);
        t.b18 = 5;
        t
    }
    #[test]
    fn both_streams_keep_dps_when_contact_is_batched() {
        for (item, class) in [(16, 176), (19, 177)] {
            for interval in [1, 6, 10, 60] {
                let t = table();
                let mut e = Events { enabled: true, hero: Some(10), stream_intervals: [interval; 2], ..Default::default() };
                e.values.insert(item, 240.0);
                let mut hp = 0.0;
                let mut pulses = 0;
                for frame in 0..60 {
                    e.begin_frame(frame);
                    for _ in 0..20 { e.queue(&t, 0, &flow_template(class), false, [Pf::ZERO; 4], 0); }
                    if let Some(hit) = e.take(0, u32::MAX) { hp += hit.damage.to_f32() * HP_SCALE; pulses += 1; }
                    assert!(e.take(0, u32::MAX).is_none());
                }
                assert_eq!(pulses, 60 / interval);
                assert!((hp - 240.0).abs() < 0.001, "interval {interval}: {hp}");
            }
        }
    }
    #[test]
    fn short_contacts_only_contribute_their_actual_damage() {
        let t = table();
        let mut e = Events { enabled: true, hero: Some(10), stream_intervals: [6; 2], ..Default::default() };
        e.values.insert(19, 600.0);
        for frame in [0, 1, 2, 30, 31] {
            e.begin_frame(frame); e.queue(&t, 0, &flow_template(177), false, [Pf::ZERO; 4], 0);
            assert!(e.take(0, u32::MAX).is_none());
        }
        e.begin_frame(100);
        assert!(e.take(0, u32::MAX).is_none(), "no damage away from the target");
        e.queue(&t, 0, &flow_template(177), false, [Pf::ZERO; 4], 0);
        assert!((e.take(0, u32::MAX).unwrap().damage.to_f32() * HP_SCALE - 60.0).abs() < 0.001);
    }
    #[test]
    fn stream_budgets_are_separate_for_targets_and_weapons_and_reset_on_reuse() {
        let mut t = table(); t.mobys.push(t.mobys[0].clone());
        let mut e = Events { enabled: true, hero: Some(10), stream_intervals: [2, 3], ..Default::default() };
        e.values.insert(16, 120.0); e.values.insert(19, 600.0);
        for (target, class) in [(0, 176), (0, 177), (1, 176)] {
            e.queue(&t, target, &flow_template(class), false, [Pf::ZERO; 4], 0);
        }
        assert!(e.take(0, u32::MAX).is_none()); assert!(e.take(1, u32::MAX).is_none());
        e.forget(1); e.begin_frame(1);
        for (target, class) in [(0, 176), (0, 177), (1, 176)] {
            e.queue(&t, target, &flow_template(class), false, [Pf::ZERO; 4], 0);
        }
        let hit = e.take(0, u32::MAX).unwrap(); assert_eq!(hit.evolution.unwrap().weapon, 16);
        assert!((hit.damage.to_f32() * HP_SCALE - 4.0).abs() < 0.001);
        assert!(e.take(0, u32::MAX).is_none()); assert!(e.take(1, u32::MAX).is_none());
        e.begin_frame(2); e.queue(&t, 0, &flow_template(177), false, [Pf::ZERO; 4], 0);
        assert!((e.take(0, u32::MAX).unwrap().damage.to_f32() * HP_SCALE - 30.0).abs() < 0.001);
    }
    #[test]
    fn temporary_turret_and_scenery_stay_native() {
        let mut t=table(); let mut e=Events { enabled:true,..Default::default() };
        for class in [1126,1,1416] { t.mobys[0].o_class=class; assert!(!e.queue(&t,0,&template(153,20),false,[Pf::ZERO;4],0)); }
        t.mobys[0].o_class=30; assert!(e.queue(&t,0,&template(153,20),false,[Pf::ZERO;4],0));
    }
    #[test]
    fn logical_lives_and_reused_sources_are_independent() {
        let mut e=Events::default(); assert_eq!(e.terminal_loss(0,0.4),(0.4,false));
        assert!((e.logical_health(0)-0.6).abs()<0.00001);
        let (loss,dead)=e.terminal_loss(0,2.0); assert!(dead); assert!((loss-0.6).abs()<0.00001);
        e.forget(0); assert_eq!(e.logical_health(0),1.0);
    }
    #[test]
    fn muzzle_and_bullet_share_one_identity_but_the_next_shot_is_independent() {
        let t=table(); let mut e=Events { enabled:true,hero:Some(10),..Default::default() };
        e.values.insert(15,75.0);
        e.queue(&t,0,&template(168,10),false,[Pf::ZERO;4],0);
        e.on_spawn(20,305); e.queue(&t,0,&template(305,20),false,[Pf::ZERO;4],0);
        assert!(e.take(0,u32::MAX).is_some()); assert!(e.take(0,u32::MAX).is_none());
        e.begin_frame(1);e.on_spawn(21,305);e.queue(&t,0,&template(305,21),false,[Pf::ZERO;4],0);
        assert!(e.take(0,u32::MAX).is_some());
    }
    #[test]
    fn walloper_spheres_across_frames_belong_to_one_lunge() {
        let t=table();let mut e=Events { enabled:true,..Default::default() };
        e.queue(&t,0,&template(180,10),false,[Pf::ZERO;4],0);assert!(e.take(0,u32::MAX).is_some());
        e.begin_frame(1);e.queue(&t,0,&template(180,10),false,[Pf::ZERO;4],0);assert!(e.take(0,u32::MAX).is_none());
        e.melee_serial=1;e.begin_frame(2);e.queue(&t,0,&template(180,10),false,[Pf::ZERO;4],0);assert!(e.take(0,u32::MAX).is_some());
    }

}
