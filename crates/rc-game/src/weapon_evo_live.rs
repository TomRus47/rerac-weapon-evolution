//! Replace configuration without replacing combat identity or save-owned state.
//! Parsing runs on a worker. This commit runs only at a gameplay tick boundary.
use super::Evolution;

impl Evolution {
    pub fn apply_live_configuration(&mut self, next: &Evolution) {
        // Seed, world population, HP normalization and save-directory policies
        // belong to the startup profile. Never reinterpret a running save.
        self.enabled = next.enabled;
        self.stats = next.stats;
        self.xp_required = next.xp_required;
        self.xp_per_damage = next.xp_per_damage;
        self.damage_hp = next.damage_hp;
        self.hud = next.hud;
        self.wrench_gold_texture = next.wrench_gold_texture;
        self.refill_ammo_on_upgrade = next.refill_ammo_on_upgrade;
        self.control_settings = next.control_settings;
        self.r1_experimental_max = next.r1_experimental_max;
        self.randomizer_bolt_multiplier=next.randomizer_bolt_multiplier;
        self.bolt_multiplier_max=next.bolt_multiplier_max;
        self.pyro.enabled = next.pyro.enabled;
        self.pyro.stats = next.pyro.stats;
        self.pyro.xp_required = next.pyro.xp_required;
        for track in &mut self.extras {
            if let Some(source) = next.extras.iter().find(|t| t.id == track.id) {
                track.enabled = source.enabled;
                track.params = source.params.clone();
                track.xp_required = source.xp_required;
            }
        }
        let mut base=self.encounter_base.unwrap_or(self.world_settings);
        base.enemy_damage_multiplier=next.world_settings.enemy_damage_multiplier;
        base.enemy_speed_multiplier=next.world_settings.enemy_speed_multiplier;
        base.enemy_perception_multiplier=next.world_settings.enemy_perception_multiplier;
        base.enemy_chase_range_multiplier=next.world_settings.enemy_chase_range_multiplier;
        base.enemy_projectile_speed_multiplier=next.world_settings.enemy_projectile_speed_multiplier;
        base.enemy_projectile_range_multiplier=next.world_settings.enemy_projectile_range_multiplier;
        base.enemy_burst_tracking=next.world_settings.enemy_burst_tracking;
        base.nanotech_upgrade_sound=next.world_settings.nanotech_upgrade_sound;
        base.nanotech_random_healing=next.world_settings.nanotech_random_healing;
        self.encounter_base=Some(base);self.world_settings=base.for_ng_plus(self.ng_plus);
        // HP normalization, population and Nanotech capacity stay staged: a
        // health-scale change must not reinterpret accepted damage or XP.
        // Do not use restore_progress: it clears terminal/damage identities.
        self.publish_max_ammo();
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn live_balance_preserves_awards_progress_and_startup_policy() {
        let mut e=Evolution::default();e.enabled=true;e.level=3;e.xp=12.;
        e.award_once.insert((14,77));e.terminal_targets.insert(77);
        e.combat_epoch=19;e.upgrade_refills=4;
        let mut next=e.clone();next.level=1;next.xp=0.;next.stats[2].max_ammo=42;
        next.golden_settings.enabled=true;next.world_settings.enemy_health_multiplier=9.;
        e.apply_live_configuration(&next);
        assert_eq!((e.level,e.xp,e.combat_epoch,e.upgrade_refills),(3,12.,19,4));
        assert_eq!(e.current().max_ammo,42);
        assert!(e.award_once.contains(&(14,77)));assert!(e.terminal_targets.contains(&77));
        assert!(!e.golden_settings.enabled);assert_eq!(e.world_settings.enemy_health_multiplier,1.);
    }
}
