//! Accepted wrench XP feedback. The lease is transient and measured in gameplay
//! ticks, so pause does not consume it. It never changes ammo or equipment.
#[derive(Default, Debug)]
pub struct Lease {
    serial: u64,
    tick: Option<u64>,
    until: u64,
    fade: u64,
    hand_item: Option<i32>,
}
impl Lease {
    pub fn update(&mut self, tick:u64, serial:u64, level:usize, ng_plus:bool, alive:bool) {
        let dt=self.tick.map_or(0,|old|tick.saturating_sub(old));
        let reset=self.tick.is_some_and(|old|tick<old) || serial<self.serial || !alive;
        let fresh=serial!=self.serial && !reset;
        self.serial=serial;
        self.tick=Some(tick);
        if reset {self.hand_item=None;}
        if reset || level>=if ng_plus {8}else{5} {
            self.until=0;self.fade=0;return;
        }
        if fresh {self.until=tick.saturating_add(crate::hud::scale_ticks(480) as u64);}
        let steps=crate::hud::scale_ticks(8).max(1) as u64;
        if self.until>tick {self.fade=self.fade.saturating_add(dt).min(steps);}
        else {self.fade=self.fade.saturating_sub(dt);}
    }
    pub fn alpha(&self)->f32 {
        self.fade as f32 / crate::hud::scale_ticks(8).max(1) as f32
    }
    /// Observe the actual hand model after processing this frame's XP event.
    /// Circle/R2 recalls the saved weapon without a Quick Select request. Such
    /// a completed swap must retire melee feedback, including its fade tail.
    /// Keep observing an outgoing wrench while it holsters, even when it is no
    /// longer ready to fire. An empty draw gap is not a new selection.
    pub fn observe_hand(&mut self, item:Option<i32>) {
        let Some(item)=item.filter(|&id|id>0) else {return;};
        let changed=self.hand_item.replace(item).is_some_and(|old|old!=item);
        if changed && item!=8 {self.until=0;self.fade=0;}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepted_awards_refresh_eight_seconds_and_fade() {
        let mut h=Lease::default(); h.update(0,0,1,false,true);
        h.update(1,1,1,false,true);assert!(h.alpha()>0. && h.alpha()<1.);
        h.update(20,1,1,false,true);assert_eq!(h.alpha(),1.);
        h.update(400,2,1,false,true);
        let until=400+crate::hud::scale_ticks(480) as u64;
        h.update(until-1,2,1,false,true);assert_eq!(h.alpha(),1.);
        h.update(until,2,1,false,true);assert!(h.alpha()>0.&&h.alpha()<1.);
        h.update(until+20,2,1,false,true);assert_eq!(h.alpha(),0.);
    }
    #[test]
    fn grade_caps_death_reload_and_pause() {
        for (grade,ng,show) in [(4,false,true),(5,false,false),(5,true,true),(7,true,true),(8,true,false)] {
            let mut h=Lease::default();h.update(0,0,grade,ng,true);h.update(8,1,grade,ng,true);
            assert_eq!(h.alpha()>0.,show);
            let alpha=h.alpha();h.update(8,1,grade,ng,true);assert_eq!(h.alpha(),alpha);
            h.update(9,1,grade,ng,false);assert_eq!(h.alpha(),0.);
            h.update(10,1,grade,ng,true);assert_eq!(h.alpha(),0.);
            h.update(0,0,grade,ng,true);assert_eq!(h.alpha(),0.);
        }
    }

    fn frame(h:&mut Lease,tick:u64,serial:u64,item:Option<i32>) {
        h.update(tick,serial,1,false,true);
        h.observe_hand(item);
    }

    #[test]
    fn recalling_the_saved_weapon_retires_wrench_feedback_without_a_ring_request() {
        for item in crate::weapon_evo::WEAPON_IDS.into_iter().filter(|&id|id!=8).chain([22]) {
            let mut h=Lease::default();
            frame(&mut h,0,0,Some(item));
            frame(&mut h,1,1,Some(8));
            frame(&mut h,8,1,Some(8));
            assert_eq!(h.alpha(),1.);
            frame(&mut h,9,1,None); // Native put-away/draw gap, no selection yet.
            assert_eq!(h.alpha(),1.);
            frame(&mut h,10,1,Some(item));
            assert_eq!(h.alpha(),0.,"recalled item {item} must own its HUD immediately");
            frame(&mut h,11,1,Some(item));
            frame(&mut h,12,1,Some(8));
            assert_eq!(h.alpha(),0.,"a canceled timer must not reappear on a later swap");
            frame(&mut h,13,2,Some(8));
            assert!(h.alpha()>0.,"new accepted wrench XP still starts its normal fade");
        }
    }

    #[test]
    fn hand_changes_also_retire_a_natural_fade_tail_or_a_same_tick_award() {
        let mut h=Lease::default();
        frame(&mut h,0,0,Some(8));
        frame(&mut h,8,1,Some(8));
        let until=8+crate::hud::scale_ticks(480) as u64;
        frame(&mut h,until-1,1,Some(8));
        frame(&mut h,until,1,Some(8));
        assert!(h.alpha()>0.&&h.alpha()<1.);
        frame(&mut h,until+1,2,Some(15));
        assert_eq!(h.alpha(),0.,"equipping wins over feedback earned in the same tick");
        frame(&mut h,until+2,2,Some(15));
        assert_eq!(h.alpha(),0.);
    }

    #[test]
    fn late_thrown_wrench_xp_still_shows_without_an_actual_equipment_change() {
        let mut h=Lease::default();
        frame(&mut h,0,0,Some(15));
        frame(&mut h,1,1,Some(15));
        frame(&mut h,8,1,Some(15));
        assert_eq!(h.alpha(),1.,"a delayed hit is valid feedback, not a stale swap");
        frame(&mut h,8,1,None);
        frame(&mut h,8,1,Some(15));
        assert_eq!(h.alpha(),1.,"pause and empty slot gaps must not consume feedback");
        frame(&mut h,9,1,Some(13));
        assert_eq!(h.alpha(),0.,"a real subsequent equipment change retires it");
    }
}
