//! Transient upgrade notifications are scoped to one living gameplay timeline.
use std::collections::VecDeque;

pub struct State {
    seen: [u16; 37],
    queue: VecDeque<(i32, u16)>,
    popup: Option<(i32, u16, u64)>,
    last_tick: Option<u64>,
}

impl Default for State {
    fn default()->Self {Self {seen:[0;37],queue:VecDeque::new(),popup:None,last_tick:None}}
}

impl State {
    pub fn popup(&self) -> Option<(i32, u16, u64)> { self.popup }

    /// Return true only when a new popup starts (the HUD may play its sound).
    /// Death, disabled notices and a rewound/reloaded timeline cancel transient
    /// state; persistent weapon levels and XP are owned elsewhere and untouched.
    pub fn update(&mut self, now:u64, alive:bool, enabled:bool, levels:[u16;37], duration:u64)->bool {
        let rewound=self.last_tick.is_some_and(|last|now<last);
        let restored=levels.iter().zip(self.seen).any(|(&level,old)|level<old);
        self.last_tick=Some(now);
        if !alive || !enabled || rewound || restored {
            self.seen=levels; self.queue.clear(); self.popup=None;
            return false;
        }
        for (id,(&level,&old)) in levels.iter().zip(self.seen.iter()).enumerate() {
            if old>0 && level>old {
                for stage in old+1..=level {self.queue.push_back((id as i32,stage));}
            }
        }
        self.seen=levels;
        if self.popup.is_some_and(|(_,_,end)|now>=end) {self.popup=None;}
        if self.popup.is_none() {
            if let Some((id,stage))=self.queue.pop_front() {
                self.popup=Some((id,stage,now.saturating_add(duration.max(1))));
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn levels(level:u16)->[u16;37] {let mut a=[0;37];a[15]=level;a}
    #[test]
    fn dying_during_an_upgrade_clears_the_active_popup_and_queued_grades() {
        let mut s=State::default();s.update(100,true,true,levels(1),180);
        assert!(s.update(101,true,true,levels(4),180));assert!(s.popup().is_some());
        assert!(!s.update(102,false,true,levels(4),180));assert_eq!(s.popup(),None);
        assert!(!s.update(103,true,true,levels(4),180));assert_eq!(s.popup(),None);
        assert!(s.update(104,true,true,levels(5),180));assert_eq!(s.popup().unwrap().1,5);
    }
    #[test]
    fn respawn_rewinds_the_clock_without_resurrecting_the_popup() {
        let mut s=State::default();s.update(900,true,true,levels(1),180);
        s.update(1000,true,true,levels(2),180);assert!(s.popup().is_some());
        assert!(!s.update(0,true,true,levels(2),180));assert_eq!(s.popup(),None);
        assert!(!s.update(1,true,true,levels(2),180));
    }
    #[test]
    fn pause_holds_duration_but_disabled_notices_clear_it() {
        let mut s=State::default();s.update(1,true,true,levels(1),60);
        s.update(2,true,true,levels(2),60);let popup=s.popup();
        for _ in 0..30 {assert!(!s.update(2,true,true,levels(2),60));assert_eq!(s.popup(),popup);}
        assert!(!s.update(2,true,false,levels(2),60));assert_eq!(s.popup(),None);
        assert!(!s.update(3,true,true,levels(2),60));
    }
    #[test]
    fn normal_expiry_and_multi_level_queue_still_work() {
        let mut s=State::default();s.update(1,true,true,levels(1),5);
        assert!(s.update(2,true,true,levels(3),5));assert_eq!(s.popup().unwrap().1,2);
        assert!(s.update(7,true,true,levels(3),5));assert_eq!(s.popup().unwrap().1,3);
        assert!(!s.update(12,true,true,levels(3),5));assert_eq!(s.popup(),None);
    }
}

#[cfg(test)] mod high_nanotech_notice_tests {
    #[test] fn upgrades_above_255_still_notify() {
        let mut state=super::State::default();let mut levels=[0;37];levels[0]=255;
        state.update(1,true,true,levels,1);levels[0]=256;
        assert!(state.update(2,true,true,levels,1));assert_eq!(state.popup().unwrap().1,256);
        levels[0]=300;assert!(state.update(3,true,true,levels,1));
    }
}
