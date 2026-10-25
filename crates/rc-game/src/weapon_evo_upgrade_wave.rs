//! Cosmetic filled upgrade bursts. This module has no access to combat, collision,
//! particles, lights, audio or the gameplay random number generator.
#[derive(Clone, Copy, Debug)]
pub struct Wave {
    pub origin: [f32; 3],
    pub grade: u8,
    pub born: u64,
}
pub const LIFE: u64 = 36;
pub const RADIUS: f32 = 3.;
pub fn radius(grade: u8) -> f32 {
    match grade {
        5 => 3.8,
        8 => 4.6,
        _ => RADIUS,
    }
}
pub fn rgb(grade: u8) -> u32 {
    match grade {
        2 => 0xff40b8,
        3 => 0x3030ff,
        4 => 0x0088ff,
        5 => 0x00e6ff,
        6 => 0x20ffbb,
        7 => 0xffe080,
        8 => 0xffffff,
        _ => 0xffffff,
    }
}
impl Wave {
    pub fn appearance(self, now: u64) -> Option<(f32, u32)> {
        let age = now.checked_sub(self.born)?;
        if age >= LIFE {
            return None;
        }
        let t = age as f32 / LIFE as f32;
        Some((
            0.12 + radius(self.grade) * t,
            (((1. - t) * 78.) as u32) << 24 | rgb(self.grade),
        ))
    }
}
#[derive(Default)]
pub struct State {
    seen: Option<[u8; 37]>,
    last: u64,
    pub waves: Vec<Wave>,
}
impl State {
    pub fn update(&mut self, now: u64, alive: bool, levels: [u8; 37], position: [f32; 3]) {
        if !alive || now < self.last {
            self.seen = Some(levels);
            self.waves.clear();
            self.last = now;
            return;
        }
        if let Some(old) = self.seen {
            let restored = old.iter().zip(levels).any(|(&a, b)| b < a);
            if restored {
                self.waves.clear();
            } else {
                for (&before, &after) in old.iter().zip(&levels) {
                    if before > 0 && after > before {
                        // One earned transition produces one subtle shell, even
                        // if a single attack advances several grades.
                        self.waves.push(Wave {
                            origin: [position[0], position[1], position[2] + 0.8],
                            grade: after,
                            born: now,
                        });
                    }
                }
            }
        }
        self.seen = Some(levels);
        self.last = now;
        self.waves.retain(|w| w.appearance(now).is_some());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_pause_death_and_reload_do_not_replay_rewards() {
        let mut s = State::default();
        let mut levels = [0; 37];
        levels[15] = 2;
        s.update(20, true, levels, [0.; 3]);
        assert!(s.waves.is_empty());
        levels[15] = 3;
        s.update(21, true, levels, [1., 2., 3.]);
        assert_eq!(s.waves.len(), 1);
        for _ in 0..10 {
            s.update(21, true, levels, [1., 2., 3.]);
        }
        assert_eq!(s.waves.len(), 1);
        assert_eq!(s.waves[0].origin, [1., 2., 3.8]);
        s.update(22, false, levels, [0.; 3]);
        assert!(s.waves.is_empty());
        s.update(1, true, levels, [0.; 3]);
        assert!(s.waves.is_empty());
        levels[15] = 4;
        s.update(2, true, levels, [0.; 3]);
        assert_eq!(s.waves.len(), 1);
        s.update(38, true, levels, [0.; 3]);
        assert!(s.waves.is_empty());
    }
    #[test]
    fn colors_expansion_and_transparency_are_bounded() {
        assert_eq!(
            (rgb(2), rgb(3), rgb(4), rgb(5)),
            (0xff40b8, 0x3030ff, 0x0088ff, 0x00e6ff)
        );
        for grade in 2..=8 {
            let w = Wave {
                origin: [0.; 3],
                grade,
                born: 10,
            };
            assert_eq!(w.appearance(9), None);
            assert_eq!(w.appearance(46), None);
            let mut prev = 0.;
            for t in 10..46 {
                let (r, c) = w.appearance(t).unwrap();
                assert!(r >= prev && r <= radius(grade) + 0.12);
                assert!(c >> 24 <= 78);
                prev = r;
            }
        }
    }
}
