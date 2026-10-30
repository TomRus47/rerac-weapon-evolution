//! Accepted wrench XP feedback. The lease is transient and measured in gameplay
//! ticks, so pause does not consume it. It never changes ammo or equipment.
#[derive(Default, Debug)]
pub struct Lease {
    serial: u64,
    tick: Option<u64>,
    until: u64,
    fade: u64,
}
impl Lease {
    pub fn update(&mut self, tick:u64, serial:u64, level:usize, ng_plus:bool, alive:bool) {
        let dt=self.tick.map_or(0,|old|tick.saturating_sub(old));
        let reset=self.tick.is_some_and(|old|tick<old) || serial<self.serial || !alive;
        let fresh=serial!=self.serial && !reset;
        self.serial=serial;
        self.tick=Some(tick);
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
}
