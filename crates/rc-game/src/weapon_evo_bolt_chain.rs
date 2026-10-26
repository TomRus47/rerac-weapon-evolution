//! NG+ and optional mixed-randomizer rewards share one exact, capped factor.
//! Thirty successful kills reach the chosen cap; rewards use visible tenths.
//! Carry fractional bolt values across pickups instead of rounding every 1-bolt
//! coin upward. Real health loss or death/reload resets the chain; travel and
//! menus do not. Death triggers can leave HP unchanged, so inspect the flag too.
/// Shared engine boundary, including scripted/void deaths without HP loss.
pub fn broken_by_hero_tick(hero: &crate::hero::Hero, health_before: i32, loss_events_before: u64) -> bool {
    hero.fell_out != 0 || hero.health <= 0 || hero.health < health_before
        || hero.damage.health_loss_events != loss_events_before
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    pub kills: u8,
    pub remainder: u8,
}
impl State {
    pub fn valid(self) -> bool {
        self.kills <= 30 && self.remainder < 30
    }
    pub fn kill(&mut self, ng_plus: bool) {
        if ng_plus {
            self.kills = self.kills.saturating_add(1).min(30);
        }
    }
    pub fn hurt(&mut self) {
        *self = Self::default();
    }
    pub fn factor(self, enabled: bool) -> f32 {self.factor_with_max(enabled,3.)}
    /// Quantize the reward itself to tenths, rather than showing a rounded HUD
    /// number that disagrees with bolt, weapon XP or Nanotech XP accounting.
    pub fn tenths(self,enabled:bool,max:f32)->u16 {
        if !enabled{return 10;}
        let cap=(max*10.).round().clamp(10.,1000.) as u16;
        10+((u32::from(cap-10)*u32::from(self.kills.min(30))+15)/30) as u16
    }
    pub fn factor_with_max(self,enabled:bool,max:f32)->f32 {f32::from(self.tenths(enabled,max))/10.}
    pub fn collect(&mut self, value: i32, ng_plus: bool) -> i32 {
        self.collect_with_max(value,ng_plus,3.)
    }
    pub fn collect_with_max(&mut self,value:i32,enabled:bool,max:f32)->i32 {
        if !enabled || value <= 0 {
            return value;
        }
        let numerator = value as i64 * i64::from(self.tenths(enabled,max)) * 3 + self.remainder as i64;
        self.remainder = (numerator % 30) as u8;
        (numerator / 30).min(i32::MAX as i64) as i32
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scripted_death_breaks_the_chain_even_with_unchanged_health() {
        let mut hero=crate::hero::Hero::new();hero.health=4;
        let previous_events=hero.damage.health_loss_events;
        assert!(!broken_by_hero_tick(&hero,4,previous_events));
        crate::hero::damage::death_fade(&mut hero);
        assert_eq!(hero.health,4);
        assert!(broken_by_hero_tick(&hero,4,previous_events));
        hero.fell_out=0;hero.health=0;
        assert!(broken_by_hero_tick(&hero,0,previous_events));
        hero.health=4;hero.damage.health_loss_events=previous_events.wrapping_add(1);
        assert!(broken_by_hero_tick(&hero,4,previous_events),"healing later in the tick must not undo a real hit");
    }
    #[test]
    fn thirty_kills_reach_three_and_small_pickups_keep_exact_fractional_credit() {
        let mut s = State::default();
        for k in 1..=30 {
            s.kill(true);
            assert_eq!(s.kills, k);
        }
        assert_eq!(s.factor(true), 3.);
        assert_eq!(s.collect(50, true), 150);
        s.kill(true);
        assert_eq!(s.kills, 30);
        s.hurt();
        s.kill(true);
        assert_eq!((0..30).map(|_| s.collect(1, true)).sum::<i32>(), 33);
        let before = s;
        s.kill(false);
        assert_eq!(s, before);
        assert_eq!(s.factor(false), 1.);
        assert_eq!(s.collect(50, false), 50);
    }
}
