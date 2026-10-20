//! Per-slot Weapon Evolution state. Content-addressed companions are prepared
//! before the native save is committed, so a failed commit cannot mismatch slots.
use super::Evolution;
use rc_formats::sha1::sha1;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The wrench does not evolve. IDs are part of the versioned disk format.
pub const IDS: [i32; 15] = [9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25];
const MAGIC_V1: &[u8; 8] = b"WEVO\0\x01\0\0";
const MAGIC: &[u8; 8] = b"WEVO\0\x02\0\0";
const SIZE_V1: usize = 8 + 20 + 15 * 12 + 20;
const SIZE: usize = SIZE_V1 + 56;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    pub level: u32,
    pub xp: f32,
}
impl Default for Track { fn default() -> Self { Self { level: 1, xp: 0.0 } } }

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Progress {
    pub tracks: [Track; 15],
    pub ammo_mode: super::ammo_mode::State,
}

fn invalid(message: &str) -> io::Error { io::Error::new(io::ErrorKind::InvalidData, message) }

impl Progress {
    pub fn valid(&self) -> bool {
        self.ammo_mode.valid() && self.tracks.iter().all(|t| (1..=5).contains(&t.level) && t.xp.is_finite()
            && (0.0..=1_000_000.0).contains(&t.xp) && (t.level != 5 || t.xp == 0.0))
    }

    pub fn encode(&self, native: &[u8]) -> io::Result<Vec<u8>> {
        if !self.valid() { return Err(invalid("invalid weapon progress")); }
        let mut out = Vec::with_capacity(SIZE);
        out.extend_from_slice(MAGIC); out.extend_from_slice(&sha1(native));
        for (id, track) in IDS.into_iter().zip(self.tracks) {
            out.extend_from_slice(&id.to_le_bytes());
            out.extend_from_slice(&track.level.to_le_bytes());
            out.extend_from_slice(&track.xp.to_le_bytes());
        }
        let mode = self.ammo_mode;
        for word in [u32::from(mode.active), mode.restore_owned, mode.available, mode.next_quick] {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&mode.normal_bolts.to_le_bytes());
        for item in mode.restore_quick { out.extend_from_slice(&item.to_le_bytes()); }
        let checksum = sha1(&out); out.extend_from_slice(&checksum);
        Ok(out)
    }

    pub fn decode(bytes: &[u8], native: &[u8]) -> io::Result<Self> {
        let legacy = bytes.len() == SIZE_V1 && &bytes[..8] == MAGIC_V1;
        if !legacy && (bytes.len() != SIZE || &bytes[..8] != MAGIC) { return Err(invalid("weapon progress format/version")); }
        if bytes[8..28] != sha1(native) { return Err(invalid("weapon progress belongs to another save")); }
        let end = bytes.len()-20;
        if bytes[end..] != sha1(&bytes[..end]) { return Err(invalid("weapon progress checksum")); }
        let mut out = Self::default();
        for (i, expected) in IDS.into_iter().enumerate() {
            let at = 28 + i * 12;
            let word = |o| u32::from_le_bytes(bytes[o..o+4].try_into().unwrap());
            if word(at) as i32 != expected { return Err(invalid("weapon progress weapon ID")); }
            out.tracks[i] = Track { level: word(at+4), xp: f32::from_bits(word(at+8)) };
        }
        if !legacy {
            let at = SIZE_V1-20;
            let word = |offset| u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap());
            if word(at) > 1 { return Err(invalid("invalid arsenal active flag")); }
            out.ammo_mode = super::ammo_mode::State {
                active: word(at) != 0, restore_owned: word(at+4), available: word(at+8), next_quick: word(at+12),
                normal_bolts: u64::from_le_bytes(bytes[at+16..at+24].try_into().unwrap()),
                restore_quick: std::array::from_fn(|i| word(at+24+i*4) as i32),
            };
        }
        if !out.valid() { return Err(invalid("weapon progress level/XP")); }
        Ok(out)
    }
}

impl Evolution {
    pub fn sync_progress(&mut self, state: &mut Progress) {
        // Arsenal updates must not reset combat epochs or overwrite live XP.
        if state.tracks != self.state_progress.tracks { self.restore_progress(state); }
        let ammo_mode = state.ammo_mode;
        *state = self.progress();
        state.ammo_mode = ammo_mode;
        self.state_progress = *state;
    }

    pub fn progress(&self) -> Progress {
        let mut out = Progress { ammo_mode: self.state_progress.ammo_mode, ..Default::default() };
        for (id, value) in IDS.into_iter().zip(&mut out.tracks) {
            let (level, xp) = match id {
                10 => (self.level, self.xp),
                16 => (self.pyro.level, self.pyro.xp),
                _ => self.extras.iter().find(|t| t.id == id).map_or((1, 0.0), |t| (t.level, t.xp)),
            };
            *value = Track { level: level as u32, xp };
        }
        out
    }

    /// Only progress is restored: balance, colours, ammo caps and Gold options
    /// always come from the current INI. Disabled tracks retain their stored XP.
    pub fn restore_progress(&mut self, progress: &Progress) {
        if !progress.valid() { return; }
        self.combat_epoch = self.combat_epoch.wrapping_add(1);
        self.state_progress = *progress;
        let apply = |level: &mut usize, xp: &mut f32, saved: Track, enabled: bool, thresholds: [f32; 4]| {
            *level = saved.level as usize; *xp = saved.xp;
            if enabled {
                while *level < 5 && *xp >= thresholds[*level-1] {
                    *xp -= thresholds[*level-1]; *level += 1;
                }
                if *level == 5 { *xp = 0.0; }
            }
        };
        for (id, saved) in IDS.into_iter().zip(progress.tracks) {
            match id {
                10 => apply(&mut self.level, &mut self.xp, saved, self.enabled, self.xp_required),
                16 => apply(&mut self.pyro.level, &mut self.pyro.xp, saved, self.pyro.enabled, self.pyro.xp_required),
                _ => if let Some(t) = self.extras.iter_mut().find(|t| t.id == id) {
                    apply(&mut t.level, &mut t.xp, saved, t.enabled, t.xp_required); t.last_hit_xp = 0.0;
                },
            }
        }
        self.last_hit_xp = 0.0; self.pyro.last_hit_xp = 0.0;
        self.applied_level = self.level;
        self.award_once.clear();
        self.terminal_targets.clear();
        if let Ok(mut q) = self.pending_special.0.lock() { q.clear(); }
        self.publish_max_ammo();
    }
}

/// Companions are isolated both by slot filename and by the entire native save.
pub fn companion_path(slot: &Path, native: &[u8]) -> io::Result<PathBuf> {
    let dir = slot.parent().ok_or_else(|| invalid("save has no parent"))?;
    let name = slot.file_name().ok_or_else(|| invalid("save has no filename"))?;
    let hash: String = sha1(native).iter().map(|b| format!("{b:02x}")).collect();
    Ok(dir.join("weapon_evolution").join(name).join(format!("{hash}.wev")))
}

pub fn load(slot: &Path, native: &[u8]) -> io::Result<Progress> {
    let path = companion_path(slot, native)?;
    match std::fs::metadata(&path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Progress::default()),
        Err(e) => return Err(e),
        Ok(m) if m.len() != SIZE as u64 && m.len() != SIZE_V1 as u64 => return Err(invalid("weapon progress size")),
        Ok(_) => {}
    }
    Progress::decode(&std::fs::read(path)?, native)
}

pub fn prepare(slot: &Path, native: &[u8], progress: &Progress) -> io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let data = progress.encode(native)?;
    let path = companion_path(slot, native)?;
    let dir = path.parent().unwrap(); std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".wev-{}-{}.tmp", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        file.write_all(&data)?; file.sync_all()?; drop(file);
        std::fs::rename(&tmp, &path)
    })();
    let _ = std::fs::remove_file(tmp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn sample() -> Progress {
        let mut p = Progress::default();
        for (i, t) in p.tracks.iter_mut().enumerate() {
            t.level = (i % 5 + 1) as u32;
            t.xp = if t.level == 5 { 0.0 } else { i as f32 + 0.125 };
        }
        p
    }

    #[test]
    fn legacy_companions_migrate_and_arsenal_state_round_trips_per_slot() {
        let original = sample();
        let current = original.encode(b"native").unwrap();
        let mut legacy = current[..SIZE_V1-20].to_vec();
        legacy[..8].copy_from_slice(MAGIC_V1);
        let checksum = sha1(&legacy); legacy.extend_from_slice(&checksum);
        assert_eq!(Progress::decode(&legacy,b"native").unwrap(),original);
        let mut mode = original;
        mode.ammo_mode = super::super::ammo_mode::State {
            active:true, restore_owned:1<<10, restore_quick:[10,12,0,0,0,0,0,0],
            available:(1<<10)|(1<<14), next_quick:3, normal_bolts:50_000,
        };
        let bytes = mode.encode(b"native").unwrap();
        assert_eq!(Progress::decode(&bytes,b"native").unwrap(),mode);
        assert!(Progress::decode(&bytes,b"other slot").is_err());
        mode.ammo_mode.next_quick=8;assert!(mode.encode(b"native").is_err());
    }

    #[test]
    fn arsenal_changes_do_not_restore_stale_xp_or_reset_combat() {
        let mut evolution=Evolution::default();
        let mut progress=Progress::default();
        evolution.restore_progress(&progress);
        evolution.xp=4.5;
        progress.ammo_mode.active=true;
        progress.ammo_mode.normal_bolts=75_000;
        let epoch=evolution.combat_epoch;
        evolution.sync_progress(&mut progress);
        assert_eq!(progress.tracks[IDS.iter().position(|&id|id==10).unwrap()].xp,4.5);
        assert_eq!(progress.ammo_mode.normal_bolts,75_000);
        assert_eq!(evolution.combat_epoch,epoch);
    }

    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!("rerac-evo-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
            std::fs::create_dir(&dir).unwrap(); Self(dir)
        }
    }
    impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

    #[test]
    fn progress_round_trips_every_weapon_and_fractional_xp() {
        let p = sample(); let b = p.encode(b"native slot").unwrap();
        assert_eq!(Progress::decode(&b, b"native slot").unwrap(), p);
        assert!(!IDS.contains(&8)); assert_eq!(b.len(), SIZE);
        assert!(Progress::decode(&b, b"different save").is_err());
        assert!(Progress::decode(&b[..SIZE-1], b"native slot").is_err());
        let mut bad = b.clone(); bad[30] ^= 1;
        assert!(Progress::decode(&bad, b"native slot").is_err());
        let mut bad = b; bad[5] = 3;
        assert!(Progress::decode(&bad, b"native slot").is_err());
    }

    #[test]
    fn invalid_levels_nan_infinity_and_max_level_xp_are_refused() {
        for t in [Track { level: 0, xp: 0.0 }, Track { level: 6, xp: 0.0 },
            Track { level: 1, xp: -1.0 }, Track { level: 1, xp: f32::NAN },
            Track { level: 1, xp: f32::INFINITY }, Track { level: 5, xp: 1.0 }] {
            let mut p = sample(); p.tracks[0] = t;
            assert!(p.encode(b"native").is_err());
        }
        // Even a correctly checksummed file cannot introduce unknown IDs or bad XP.
        let p = sample();
        for (at, value) in [(28usize, 8u32), (36, f32::NAN.to_bits())] {
            let mut b = p.encode(b"native").unwrap(); b[at..at+4].copy_from_slice(&value.to_le_bytes());
            let checksum = sha1(&b[..SIZE-20]); b[SIZE-20..].copy_from_slice(&checksum);
            assert!(Progress::decode(&b, b"native").is_err());
        }
    }

    #[test]
    fn separate_slots_and_interrupted_save_do_not_share_or_replace_progress() {
        let tmp = Temp::new(); let slot0 = tmp.0.join("save0.bin"); let slot1 = tmp.0.join("save1.bin");
        let a = sample(); let mut b = a; b.tracks[0].xp = 8.5;
        prepare(&slot0, b"old native", &a).unwrap();
        prepare(&slot1, b"old native", &b).unwrap();
        assert_eq!(load(&slot0, b"old native").unwrap(), a);
        assert_eq!(load(&slot1, b"old native").unwrap(), b);
        // Prepare a companion but simulate a failure before committing its native file.
        prepare(&slot0, b"new native", &b).unwrap();
        assert_eq!(load(&slot0, b"old native").unwrap(), a);
        assert_eq!(load(&slot0, b"new native").unwrap(), b);
        assert_eq!(load(&slot1, b"new native").unwrap(), Progress::default());
        // Same native contents can still capture a later XP-only save atomically.
        prepare(&slot0, b"new native", &a).unwrap();
        assert_eq!(load(&slot0, b"new native").unwrap(), a);
        let path = companion_path(&slot0, b"old native").unwrap(); std::fs::write(path, b"broken").unwrap();
        assert!(load(&slot0, b"old native").is_err());
    }

    #[test]
    fn restore_uses_current_balance_and_preserves_disabled_tracks() {
        let mut e = Evolution::default();
        e.enabled = true; e.xp_required = [2.0; 4];
        e.stats[3].max_ammo = 73;
        let mut p = Progress::default();
        let bomb = IDS.iter().position(|&id| id == 10).unwrap();
        let tesla = IDS.iter().position(|&id| id == 19).unwrap();
        p.tracks[bomb] = Track { level: 2, xp: 5.0 };
        p.tracks[tesla] = Track { level: 3, xp: 12.5 };
        e.restore_progress(&p);
        assert_eq!((e.level, e.xp, e.current().max_ammo), (4, 1.0, 73));
        assert_eq!(e.progress().tracks[tesla], p.tracks[tesla]);
        let mut next_level = Evolution::default(); next_level.restore_progress(&e.progress());
        assert_eq!(next_level.progress(), e.progress());
        next_level.restore_progress(&Progress::default());
        assert_eq!(next_level.progress(), Progress::default());
    }

    #[test]
    fn slot_load_or_new_game_wins_over_the_previous_worlds_live_xp() {
        let mut e = Evolution::default(); e.enabled = true;
        let bomb = IDS.iter().position(|&id| id == 10).unwrap();
        let mut state = Progress::default(); state.tracks[bomb] = Track { level: 2, xp: 1.0 };
        e.restore_progress(&state); e.xp += 2.0;
        e.sync_progress(&mut state);
        assert_eq!(state.tracks[bomb], Track { level: 2, xp: 3.0 });
        // The menu resets GameState while the old Play resource still exists.
        state = Progress::default(); e.xp += 1.0;
        e.sync_progress(&mut state);
        assert_eq!((e.level, e.xp, state), (1, 0.0, Progress::default()));
        state.tracks[bomb] = Track { level: 4, xp: 0.5 };
        e.sync_progress(&mut state);
        assert_eq!((e.level, e.xp), (4, 0.5));
    }

    #[test]
    fn repeated_damage_after_new_game_keeps_accumulating_in_its_own_slot() {
        let mut e = Evolution::default(); e.enabled = true; e.xp_required = [100.0; 4];
        let bomb = IDS.iter().position(|&id| id == 10).unwrap();
        let mut previous = Progress::default(); previous.tracks[bomb] = Track { level: 3, xp: 4.0 };
        e.restore_progress(&previous);
        let mut new_game = Progress::default();
        e.sync_progress(&mut new_game);
        for expected in [0.5, 1.0, 1.5] {
            e.award_damage(0.5, 121); e.sync_progress(&mut new_game);
            assert_eq!(new_game.tracks[bomb], Track { level: 1, xp: expected });
        }
        let bytes = new_game.encode(b"slot B").unwrap();
        let mut loaded = Progress::decode(&bytes, b"slot B").unwrap();
        e.restore_progress(&loaded); e.award_damage(0.5, 121); e.sync_progress(&mut loaded);
        assert_eq!(loaded.tracks[bomb], Track { level: 1, xp: 2.0 });
        assert_eq!(previous.tracks[bomb], Track { level: 3, xp: 4.0 });
    }
}
