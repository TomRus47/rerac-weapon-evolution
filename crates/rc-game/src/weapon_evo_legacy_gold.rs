//! Older native Gold purchases entitle their weapon track to V6.
//! Reconciliation grants no items, XP or ammo and never debits the bolt wallet.
use super::{gold_shop, save, Evolution, WEAPON_IDS};

impl Evolution {
    /// Already reconciled V6+ tracks retain their grade, partial XP and spending.
    pub fn migrate_legacy_gold(&mut self, gold: &[u8]) -> usize {
        if !self.enabled {
            return 0;
        }
        let mut count = 0;
        for item in WEAPON_IDS.into_iter().filter(|&id| id != 8) {
            if gold.get(item as usize).copied().unwrap_or(0) == 0
                || !self.weapon_level(item).is_some_and(|level| level < 6)
            {
                continue;
            }
            match item {
                10 => {
                    self.level = 6;
                    self.xp = 0.;
                    self.level_cap = 8;
                }
                16 => {
                    self.pyro.level = 6;
                    self.pyro.xp = 0.;
                    self.pyro.level_cap = 8;
                }
                _ => {
                    let Some(track) = self.extras.iter_mut().find(|t| t.id == item && t.enabled)
                    else {
                        continue;
                    };
                    track.level = 6;
                    track.xp = 0.;
                    track.level_cap = 8;
                }
            }
            // Gold Bolts count cumulative pickups rather than a debited wallet.
            // Current catalog prices leave enough of the forty collectible bolts
            // for the remaining upgrades after migrating original four-bolt buys.
            let cost = gold_shop::GEMLIK
                .iter()
                .find(|p| p.item == item as usize)
                .map_or_else(|| gold_shop::novalis_gold(item as usize), |p| p.gold);
            if !self.golden_settings.enabled {
                self.gold_spent = self.gold_spent.saturating_add(cost);
            }
            count += 1;
        }
        if count > 0 {
            self.publish_max_ammo();
        }
        count
    }

    /// Load another slot before reconciling its flags. Export the migrated result
    /// immediately so a subsequent sync cannot restore its stale V1-V5 tracks.
    pub fn sync_gold_progress(&mut self, state: &mut save::Progress, gold: &[u8]) -> usize {
        self.sync_progress(state);
        let count = self.migrate_legacy_gold(gold);
        if count > 0 {
            self.sync_progress(state);
        }
        count
    }
}
