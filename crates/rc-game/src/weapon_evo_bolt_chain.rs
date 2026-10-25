//! NG+ rewards share one exact factor: 30 successful kills grow x1 into x3.
//! Carry fractional bolt values across pickups instead of rounding every 1-bolt
//! coin upward. Only real health loss resets the chain; travel and menus do not.
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
    pub fn factor(self, ng_plus: bool) -> f32 {
        if ng_plus {
            1. + 2. * self.kills as f32 / 30.
        } else {
            1.
        }
    }
    pub fn collect(&mut self, value: i32, ng_plus: bool) -> i32 {
        if !ng_plus || value <= 0 {
            return value;
        }
        let numerator = value as i64 * (30 + 2 * self.kills as i64) + self.remainder as i64;
        self.remainder = (numerator % 30) as u8;
        (numerator / 30).min(i32::MAX as i64) as i32
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!((0..30).map(|_| s.collect(1, true)).sum::<i32>(), 32);
        let before = s;
        s.kill(false);
        assert_eq!(s, before);
        assert_eq!(s.factor(false), 1.);
        assert_eq!(s.collect(50, false), 50);
    }
}
