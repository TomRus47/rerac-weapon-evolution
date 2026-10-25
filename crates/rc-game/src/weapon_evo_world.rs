//! Optional encounter rules. Keep native HP normalized so AI phase thresholds,
//! shields and Amoeboid splitting remain native; convert at the combat boundary.
use crate::moby_runtime::{Moby, MobyId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Multiplies encounter coefficients in NG+ only, without rewriting the INI.
    pub ngplus_multiplier: f32,
    pub enemy_health_multiplier: f32,
    pub enemy_health_include_bosses: bool,
    pub extra_enemies_per_level: u32,
    pub extra_enemy_prefer_small: bool,
    pub extra_enemy_prefer_medium: bool,
    pub extra_enemy_prefer_large: bool,
    pub enemy_damage_by_health: bool,
    pub enemy_damage_multiplier: f32,
    pub enemy_perception_multiplier: f32,
    pub enemy_chase_range_multiplier: f32,
    pub enemy_speed_multiplier: f32,
    pub enemy_projectile_speed_multiplier: f32,
    pub enemy_projectile_range_multiplier: f32,

    pub nanotech_xp_per_upgrade: f32,
    pub nanotech_max_health: u32,
    pub nanotech_upgrade_sound: bool,
    pub nanotech_random_healing: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            ngplus_multiplier: 1.,
            enemy_health_multiplier: 1.,
            enemy_health_include_bosses: true,
            extra_enemies_per_level: 0,
            extra_enemy_prefer_small: true,
            extra_enemy_prefer_medium: true,
            extra_enemy_prefer_large: true,
            enemy_damage_by_health: false,
            enemy_damage_multiplier: 1.,
            enemy_perception_multiplier: 1.,
            enemy_chase_range_multiplier: 1.,
            enemy_speed_multiplier: 1.,
            enemy_projectile_speed_multiplier: 1.,
            enemy_projectile_range_multiplier: 1.,

            nanotech_xp_per_upgrade: 1000.,
            nanotech_max_health: 20,
            nanotech_upgrade_sound: true,
            nanotech_random_healing: true,
        }
    }
}
pub fn boss(class: i16) -> bool {
    matches!(class, 1051 | 1106 | 388 | 1422)
}
/// Native purchases are independent of XP-earned health, including migrated saves.
pub fn native_nanotech(flags: &[u8]) -> i32 {
    4 + if flags.get(4).is_some_and(|&v|v != 0) {1} else {0}
      + if flags.get(5).is_some_and(|&v|v != 0) {3} else {0}
}
fn scaled_enemy_damage(base: i32, s: &Settings) -> i32 {
    (base as f64 * s.enemy_damage_multiplier as f64).round().clamp(1.,i32::MAX as f64) as i32
}
fn boss_contact_damage(class: i16) -> Option<i32> {
    match class {1051=>Some(3),1106=>Some(4),1422=>Some(5),388=>Some(3),_=>None}
}
impl Settings {
    /// Counts, toggles and Nanotech requirements are not coefficients.
    pub fn for_ng_plus(mut self, ng_plus: bool) -> Self {
        if ng_plus {
            let k = self.ngplus_multiplier;
            self.enemy_health_multiplier *= k;
            self.enemy_damage_multiplier *= k;
            self.enemy_speed_multiplier *= k;
            self.enemy_projectile_speed_multiplier *= k;
            self.enemy_projectile_range_multiplier *= k;
            self.enemy_perception_multiplier *= k;
            self.enemy_chase_range_multiplier *= k;
        }
        self
    }
    pub fn health_scale(&self, class: i16) -> f32 {
        if crate::moby_update::real_hp::donor_class(class)
            && (self.enemy_health_include_bosses || !boss(class))
        {
            self.enemy_health_multiplier
        } else {
            1.
        }
    }
    /// Return false for unrelated keys, an error for malformed recognized keys.
    pub fn parse_field(&mut self, key: &str, value: &str) -> Result<bool, String> {
        let range = match key {
            "ngplus_multiplier" => (0.1, 4.),
            "enemy_health_multiplier" => (0.1, 100.),
            "extra_enemies_per_level" => (0., 100.),
            "enemy_damage_multiplier" => (0.1, 100.),
            "enemy_perception_multiplier" | "enemy_chase_range_multiplier" => (0.1, 10.),
            "enemy_speed_multiplier" | "enemy_projectile_speed_multiplier" => (0.1, 4.),
            "enemy_projectile_range_multiplier" => (0.1, 10.),
            "nanotech_xp_per_upgrade" => (0.1, 1_000_000.),
            "nanotech_max_health" => (0., 64.),
            "enemy_health_include_bosses"
            | "extra_enemy_prefer_small"
            | "extra_enemy_prefer_medium"
            | "extra_enemy_prefer_large"
            | "enemy_damage_by_health"
            | "enemy_damage_include_bosses"
            | "nanotech_upgrade_sound"
            | "nanotech_random_healing" => {
                let v = match value.trim().to_ascii_lowercase().as_str() {
                    "true" | "on" | "1" => true,
                    "false" | "off" | "0" => false,
                    _ => return Err(format!("{key}: expected true/false")),
                };
                match key {
                    "enemy_health_include_bosses" => self.enemy_health_include_bosses = v,
                    "extra_enemy_prefer_small" => self.extra_enemy_prefer_small = v,
                    "extra_enemy_prefer_medium" => self.extra_enemy_prefer_medium = v,
                    "extra_enemy_prefer_large" => self.extra_enemy_prefer_large = v,
                    "enemy_damage_by_health" => self.enemy_damage_by_health = v,
                    // Accept old presets without letting this retired switch bypass boss damage.
                    "enemy_damage_include_bosses" => {},
                    "nanotech_upgrade_sound" => self.nanotech_upgrade_sound = v,
                    "nanotech_random_healing" => self.nanotech_random_healing = v,
                    _ => unreachable!("recognized boolean setting"),
                };
                return Ok(true);
            }
            _ => return Ok(false),
        };
        let v = value
            .parse::<f32>()
            .map_err(|_| format!("{key}: expected a number"))?;
        if !v.is_finite()
            || v < range.0
            || v > range.1
            || (matches!(key, "extra_enemies_per_level" | "nanotech_max_health") && v.fract() != 0.)
        {
            return Err(format!("{key}: outside {}..{}", range.0, range.1));
        }
        match key {
            "ngplus_multiplier" => self.ngplus_multiplier = v,
            "enemy_health_multiplier" => self.enemy_health_multiplier = v,
            "extra_enemies_per_level" => self.extra_enemies_per_level = v as u32,
            "enemy_damage_multiplier" => self.enemy_damage_multiplier = v,
            "enemy_perception_multiplier" => self.enemy_perception_multiplier = v,
            "enemy_chase_range_multiplier" => self.enemy_chase_range_multiplier = v,
            "enemy_speed_multiplier" => self.enemy_speed_multiplier = v,
            "enemy_projectile_speed_multiplier" => self.enemy_projectile_speed_multiplier = v,
            "enemy_projectile_range_multiplier" => self.enemy_projectile_range_multiplier = v,

            "nanotech_xp_per_upgrade" => self.nanotech_xp_per_upgrade = v,
            _ => self.nanotech_max_health = v as u32,
        };
        Ok(true)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NanotechProgress {
    pub bonus: u32,
    pub xp: f32,
}
impl NanotechProgress {
    /// Hidden progression uses one fixed requirement per gained health point.
    /// At the configured cap, a full bar communicates that no upgrade is pending.
    pub fn fraction(self, s: &Settings, _native: i32) -> Option<f32> {
        if !s.enemy_damage_by_health { return None; }
        if self.bonus >= s.nanotech_max_health { return Some(1.0); }
        Some((self.xp / s.nanotech_xp_per_upgrade).clamp(0.0,1.0))
    }
    pub fn valid(self) -> bool {
        self.bonus <= 64 && self.xp.is_finite() && (0. ..=1_000_000.).contains(&self.xp)
    }
    pub fn max_health(self, s: &Settings, native: i32) -> i32 {
        if !s.enemy_damage_by_health {
            return native;
        }
        native + self.bonus.min(s.nanotech_max_health) as i32
    }
    pub fn award(&mut self, s: &Settings, _native: i32, xp: f32) -> u32 {
        if !s.enemy_damage_by_health || !xp.is_finite() || xp <= 0. {
            return 0;
        }
        let before = self.bonus;
        let cap = s.nanotech_max_health;
        if self.bonus >= cap {
            self.xp = 0.;
            return 0;
        }
        self.xp += xp;
        while self.xp >= s.nanotech_xp_per_upgrade && self.bonus < cap {
            self.xp -= s.nanotech_xp_per_upgrade;
            self.bonus += 1;
        }
        if self.bonus >= cap {
            self.xp = 0.;
        }
        self.bonus - before
    }
}
/// Original health belongs to an enemy life, never its current damaged HP.
/// Projectile owners retain a snapshot even after their source is deleted.
#[derive(Clone, Copy, Debug)]
pub struct Enemy {
    pub class: i16,
    pub base_hp: f32,
}
#[derive(Clone, Debug, Default)]
pub struct Enemies {
    pub records: HashMap<MobyId, Enemy>,
    pub updating: Option<MobyId>,
}
impl Enemies {
    pub fn observe(&mut self, id: MobyId, m: &Moby) {
        if m.state == 0 || !crate::moby_update::real_hp::donor(m) {
            return;
        }
        let hp = if crate::moby_update::real_hp::terminal_class(m.o_class) {
            1.
        } else {
            crate::targeting::record_health(m).unwrap_or(1.)
        };
        if hp.is_finite() && hp > 0. {
            self.records.entry(id).or_insert(Enemy {
                class: m.o_class,
                base_hp: hp,
            });
        }
    }
    pub fn points(&self, id: Option<MobyId>, s: &Settings) -> Option<i32> {
        if !s.enemy_damage_by_health {
            return None;
        }
        let e = self.records.get(&id?)?;
        Some(scaled_enemy_damage(boss_contact_damage(e.class).unwrap_or_else(|| e.base_hp.ceil().max(1.) as i32), s))
    }
    /// Pooled boss attacks do not always go through create_moby. Resolve their
    /// native classes with the level as a guard, before ordinary owner tiers.
    pub fn hit_points(&self, hit: &crate::moby_update::services::HitRecord,
        table: &crate::moby_runtime::MobyTable, level: u32, s: &Settings) -> Option<i32> {
        if !s.enemy_damage_by_health { return None; }
        // Resolve the actual projectile before an inherited owner tier: a boss
        // can emit attacks with different damage. Keep unrelated level hazards native.
        let class = hit.attacker_class(table)?;
        let source = hit.attacker.and_then(|id| table.mobys.get(id));
        let base = match (level, class) {
            (7, 1046 | 1049) => Some(4),
            (18, 628) => Some(10), // charging contact aura
            (18, 624 | 983) => Some(5),
            (18, 564) => Some(if source.and_then(|m|m.pvars.get(0x38..0x3c))
                .is_some_and(|b|i32::from_le_bytes(b.try_into().unwrap()) & 2 != 0) {3} else {5}),
            (18, 568) => {
                // Native rolling_mine::throw stores the launcher's index + 1.
                let owner = source.and_then(|m|m.pvars.get(0x198..0x19c))
                    .map(|b|u32::from_le_bytes(b.try_into().unwrap()))
                    .and_then(|n|n.checked_sub(1)).and_then(|n|table.mobys.get(n as usize));
                owner.filter(|m|m.o_class == 1422).map(|_|3)
            },
            _ => boss_contact_damage(class),
        };
        base.map(|n|scaled_enemy_damage(n,s)).or_else(||self.points(hit.attacker,s))
    }

    pub fn created(&mut self, id: MobyId, inherit: bool) {
        let e = if inherit {
            self.updating.and_then(|p| self.records.get(&p).copied())
        } else {
            None
        };
        self.records.remove(&id);
        if let Some(e) = e {
            self.records.insert(id, e);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nanotech_fraction_uses_remaining_xp_and_fills_at_the_effective_cap() {
        let mut s=Settings::default();let mut p=NanotechProgress{bonus:0,xp:250.};
        assert_eq!(p.fraction(&s,4),None);
        s.enemy_damage_by_health=true;
        assert_eq!(p.fraction(&s,4),Some(0.25));
        assert_eq!(p.award(&s,4,1000.),1);assert_eq!(p.fraction(&s,4),Some(0.25));
        p.bonus=20;p.xp=0.;assert_eq!(p.fraction(&s,4),Some(1.));
    }
    #[test]
    fn hp_and_tiers_use_original_health_and_boss_policies() {
        let mut s = Settings::default();
        s.enemy_health_multiplier = 2.;
        assert_eq!(s.health_scale(577), 2.);
        s.enemy_health_include_bosses = false;
        for c in [1051, 1106, 388, 1422] {
            assert_eq!(s.health_scale(c), 1.);
        }
        let mut e = Enemies::default();
        e.records.insert(
            2,
            Enemy {
                class: 577,
                base_hp: 2.,
            },
        );
        assert_eq!(e.points(Some(2), &s), None);
        s.enemy_damage_by_health = true;
        assert_eq!(e.points(Some(2), &s), Some(2));
        s.enemy_damage_multiplier = 3.;
        assert_eq!(e.points(Some(2), &s), Some(6));
    }
    #[test]
    fn fixed_threshold_carries_xp_and_never_exceeds_cap() {
        let s = Settings {
            enemy_damage_by_health: true,
            nanotech_xp_per_upgrade: 100.,
            nanotech_max_health: 4,
            ..Default::default()
        };
        let mut p = NanotechProgress::default();
        assert_eq!(p.award(&s, 4, 250.), 2);
        assert_eq!(p.xp, 50.);
        assert_eq!(p.max_health(&s, 4), 6);
        assert_eq!(p.award(&s, 4, 1000.), 2);
        assert_eq!((p.bonus, p.xp), (4, 0.));
        assert_eq!(p.max_health(&s, 4), 8);
        assert_eq!(p.max_health(&Settings::default(), 4), 4);
    }
    #[test]
    fn invalid_configuration_cannot_create_nan_or_unbounded_spawns() {
        let mut s = Settings::default();
        for (k, v) in [
            ("enemy_health_multiplier", "NaN"),
            ("extra_enemies_per_level", "1.5"),
            ("nanotech_max_health", "1000"),
            ("nanotech_random_healing", "maybe"),
        ] {
            assert!(s.parse_field(k, v).is_err());
        }
    }
}

/// A conservative pool: independent enemies with no mission-controller or
/// cross-moby ownership requirement. Never clone bosses, turrets or scenery.
pub fn population_class(class: i16) -> bool {
    matches!(
        class,
        577 | 459 | 572 | 865 | 866 | 749 | 1440 | 578 | 1231 | 217 | 52 | 1445 | 252 | 573
            | 574
            | 340
            | 427
            | 563
            | 193
            | 623
            | 1246
            | 1202
            | 1048
            | 294
            | 238
            | 1262
            | 1023
            | 638
            | 827
            | 1068
            | 1229
            | 326
            | 77
            | 63
    )
}
pub fn templates(table: &crate::moby_runtime::MobyTable) -> HashMap<MobyId, Moby> {
    table
        .mobys
        .iter()
        .enumerate()
        .filter(|(_, m)| m.state == 0 && population_class(m.o_class))
        .map(|(id, m)| (id, m.clone()))
        .collect()
}
fn v(p: [f32; 3]) -> [crate::ps2v::Pf; 4] {
    [
        crate::ps2v::Pf::f(p[0]),
        crate::ps2v::Pf::f(p[1]),
        crate::ps2v::Pf::f(p[2]),
        crate::ps2v::Pf::ZERO,
    ]
}
pub fn radius(m: &Moby) -> f32 {
    (m.bsphere[3] / 1024.).abs().clamp(0.5, 6.)
}
/// Probe floor, clearance and occupants. A failed placement never overlaps an
/// existing actor or silently replaces a native enemy.
pub fn placement(
    w: &crate::moby_update::services::World,
    p: [f32; 3],
    r: f32,
    ignore: Option<MobyId>,
) -> Option<[f32; 3]> {
    let ground = if w.coll.is_some() {
        let hit = w.line(
            v([p[0], p[1], p[2] + 3.]),
            v([p[0], p[1], p[2] - 5.]),
            2,
            ignore,
        )?;
        if hit.normal[2].to_f32() < 0.5 {
            return None;
        }
        [p[0], p[1], hit.point[2].to_f32() + 0.03]
    } else {
        p
    };
    if w.coll_sphere(
        v([ground[0], ground[1], ground[2] + r + 0.05]),
        crate::ps2v::Pf::f(r),
        2,
        ignore,
    )
    .is_some()
    {
        return None;
    }
    if w.table.mobys.iter().enumerate().any(|(id, m)| {
        Some(id) != ignore
            && m.state > 0
            && m.state < 128
            && m.has_collision
            && m.mode & 1 == 0
            && {
                let d = (m.position[0] - ground[0]).hypot(m.position[1] - ground[1]);
                d < r + radius(m) + 0.35 && (m.position[2] - ground[2]).abs() < r + radius(m) + 1.
            }
    }) {
        return None;
    }
    Some(ground)
}
/// Native initialization runs with a private pvar copy and a fresh allocator
/// identity. The source's mission/group links must not double-award objectives.
pub fn spawn_copy(
    w: &mut crate::moby_update::services::World,
    template: &Moby,
    p: [f32; 3],
) -> Option<MobyId> {
    spawn_copy_prepared(w, template, p, |_, _| {})
}

/// Supply visit-local paths after allocation but before native initialization.
pub fn spawn_copy_prepared(
    w: &mut crate::moby_update::services::World,
    template: &Moby,
    p: [f32; 3],
    prepare: impl FnOnce(&mut crate::moby_update::services::World, MobyId),
) -> Option<MobyId> {
    let id = w.create_moby(template.o_class)?;
    let allocated = w.table.mobys[id].clone();
    let mut m = template.clone();
    m.index = allocated.index;
    m.uid_hi = allocated.uid_hi;
    m.class_slot = allocated.class_slot;
    // Registration belongs to the new allocator identity. A loader template
    // already has a grid rectangle: copying it makes register() skip cells it
    // thinks this actor occupies, causing invisible holes in the copy's hitbox.
    m.ba0 = allocated.ba0;
    m.position = [p[0], p[1], p[2], 1.];
    m.state = 0;
    m.mission = 0xff;
    m.spawn_id = -1;
    m.spawn_flag = 0;
    m.group = -1;
    m.parent = None;
    m.hit_slot = 0xff;
    m.b7c = 0xff;
    m.b7d = 0xff;
    m.joint_mods.clear();
    m.joint_mod_keys.clear();
    w.table.mobys[id] = m;
    prepare(w, id);
    crate::moby_update::scheduler::run_moby(w, id);
    (w.table.mobys[id].state < 128).then_some(id)
}
/// Detection ranges only; callers retain native world coordinates and physics.
pub fn perception_range(w: &crate::moby_update::services::World, id: MobyId, range: f32) -> f32 {
    if crate::moby_update::real_hp::donor(w.m(id))
        && w.classes.info(w.m(id).o_class).is_some_and(|c| c.ty == 5)
    {
        range * w.svc.bomb_evo.world_settings.enemy_perception_multiplier
    } else {
        range
    }
}

/// Additional pursuit allowance, relative to the perception-scaled territory.
/// Only enemy AI opts in; scenery, player items and friendly actors stay native.
pub fn chase_bonus(w: &crate::moby_update::services::World, id: MobyId) -> f32 {
    if motion::enemy(w.m(id)) && w.classes.info(w.m(id).o_class).is_some_and(|c| c.ty == 5) {
        w.svc.bomb_evo.world_settings.enemy_chase_range_multiplier
    } else { 1. }
}

/// Home/territory limits follow perception, then the additional chase factor.
/// Never feed this result back into a native pvar: scaling is read-only.
pub fn chase_range(w: &crate::moby_update::services::World, id: MobyId, native: f32) -> f32 {
    perception_range(w,id,native) * chase_bonus(w,id)
}

/// Expand AI eligibility polygons around their centroid without rewriting the
/// level spline. Physical collision, route graphs and boss arena walls remain
/// native. Querying a transformed point avoids cloning a polygon every tick.
pub fn chase_in_area(w: &crate::moby_update::services::World, id: MobyId, area: usize, mut point: [f32;4]) -> bool {
    let factor = chase_range(w,id,1.);
    if factor != 1. {
        if let Some(points) = w.svc.splines.get(area).filter(|p| !p.is_empty()) {
            let mut center = [0.;2];
            for p in points {for k in 0..2 {center[k] += f32::from_bits(p[k]);}}
            for k in 0..2 {center[k] /= points.len() as f32; point[k] = center[k] + (point[k]-center[k])/factor;}
        }
    }
    crate::moby_update::creature::region::point_in_polygon(w,area,point)
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use crate::moby_runtime::Moby;
    #[test]
    fn damaged_enemies_and_their_projectiles_keep_the_original_tier_until_slot_reuse() {
        let mut m = Moby::zeroed();
        m.o_class = 577;
        m.state = 1;
        m.mode = 0x20;
        m.pvars = vec![0; 0x40];
        m.pvars[0..4].copy_from_slice(&0x20u32.to_le_bytes());
        m.pvars[0x20..0x24].copy_from_slice(&3f32.to_le_bytes());
        let mut enemies = Enemies::default();
        enemies.observe(2, &m);
        m.pvars[0x20..0x24].copy_from_slice(&0.25f32.to_le_bytes());
        enemies.observe(2, &m);
        enemies.updating = Some(2);
        enemies.created(3, true);
        let settings = Settings {
            enemy_damage_by_health: true,
            enemy_health_multiplier: 10.,
            ..Default::default()
        };
        assert_eq!(enemies.points(Some(2), &settings), Some(3));
        assert_eq!(enemies.points(Some(3), &settings), Some(3));
        enemies.created(2, false);
        assert_eq!(enemies.points(Some(2), &settings), None);
        assert_eq!(enemies.points(Some(3), &settings), Some(3));
        enemies.created(3, false);
        assert_eq!(enemies.points(Some(3), &settings), None);
    }
}

#[path = "weapon_evo_population.rs"]
pub mod population;

#[cfg(test)]
mod damage_rule_tests {
    use super::*;
    #[test]
    fn ordinary_damage_is_uncapped_and_uses_original_hp_even_after_health_scaling() {
        let mut s=Settings{enemy_damage_by_health:true,enemy_health_multiplier:100.,..Default::default()};
        let mut e=Enemies::default();
        e.records.insert(1,Enemy{class:574,base_hp:5.});
        assert_eq!(e.points(Some(1),&s),Some(5));
        s.enemy_damage_multiplier=2.;
        assert_eq!(e.points(Some(1),&s),Some(10));
        assert!(s.parse_field("enemy_damage_multiplier","100").unwrap());
        assert_eq!(e.points(Some(1),&s),Some(500));
        s.enemy_damage_by_health=false;
        assert_eq!(e.points(Some(1),&s),None);
    }
    #[test]
    fn boss_contact_profiles_scale_only_when_enabled() {
        let mut s=Settings{enemy_damage_by_health:true,enemy_damage_multiplier:100.,..Default::default()};
        let mut enemies=Enemies::default();
        for (class,base) in [(1051,3),(1106,4),(388,3),(1422,5)] {
            enemies.records.insert(1,Enemy{class,base_hp:100.});
            assert_eq!(enemies.points(Some(1),&s),Some(base*100));
            s.enemy_damage_by_health=false;
            assert_eq!(enemies.points(Some(1),&s),None);s.enemy_damage_by_health=true;
        }

    }
}

#[path="weapon_evo_motion.rs"]
pub mod motion;

#[cfg(test)] mod motion_configuration_tests {
    use super::*;
    #[test] fn motion_defaults_neutral_ranges_and_invalid_values() {
        for key in ["enemy_speed_multiplier","enemy_projectile_speed_multiplier","enemy_projectile_range_multiplier"] {
            let mut s=Settings::default();assert!(s.parse_field(key,"2").unwrap());
            assert!(s.parse_field(key,"0.1").unwrap());
            for bad in ["0","-1","NaN","inf","eleven"] {assert!(s.parse_field(key,bad).is_err());}
        }
        let s=Settings::default();assert_eq!((s.enemy_speed_multiplier,s.enemy_projectile_speed_multiplier,s.enemy_projectile_range_multiplier),(1.,1.,1.));
    }
}

#[cfg(test)]
mod additive_health_tests {
    use super::*;
    use crate::moby_runtime::MobyTable;
    use crate::moby_update::services::HitRecord;
    #[test]
    fn native_purchases_add_slots_without_spending_the_xp_cap() {
        let mut s=Settings{enemy_damage_by_health:true,nanotech_xp_per_upgrade:100.,nanotech_max_health:20,..Default::default()};
        let mut flags=[0u8;8]; let mut p=NanotechProgress{bonus:7,xp:37.};
        for (premium,ultra,native) in [(0,0,4),(1,0,5),(0,1,7),(1,1,8)] {
            flags[4]=premium;flags[5]=ultra;
            assert_eq!(native_nanotech(&flags),native);
            assert_eq!(p.max_health(&s,native),native+7);
            assert_eq!(p.fraction(&s,native),Some(0.37));
        }
        assert_eq!(p.award(&s,8,63.),1);assert_eq!((p.bonus,p.xp),(8,0.));
        assert_eq!(p.award(&s,8,10000.),12);assert_eq!(p.max_health(&s,8),28);
        assert_eq!(p.fraction(&s,8),Some(1.));
        s.nanotech_max_health=0;assert_eq!(p.max_health(&s,8),8);assert_eq!(p.bonus,20);
        s.nanotech_max_health=64;assert_eq!(p.award(&s,8,10000.),44);assert_eq!(p.max_health(&s,8),72);
        let mut save=crate::weapon_evo::save::Progress::default();save.nanotech=p;
        let native=b"native save identity";
        let encoded=save.encode(native).unwrap();let loaded=crate::weapon_evo::save::Progress::decode(&encoded,native).unwrap();
        assert_eq!(loaded.nanotech,p);
        s.enemy_damage_by_health=false;assert_eq!(p.max_health(&s,8),8);assert_eq!(p.fraction(&s,8),None);
        let before=p;assert_eq!(p.award(&s,8,900.),0);assert_eq!(p,before);
    }
    #[test]
    fn partial_progress_and_existing_save_values_survive_purchases_and_cap_changes() {
        let mut p=crate::weapon_evo::save::Progress::default();p.nanotech=NanotechProgress{bonus:12,xp:37.};
        let bytes=p.encode(b"save").unwrap();let loaded=crate::weapon_evo::save::Progress::decode(&bytes,b"save").unwrap();
        assert_eq!(loaded,p);
        let mut s=Settings{enemy_damage_by_health:true,..Default::default()};
        assert_eq!(p.nanotech.max_health(&s,4),16);assert_eq!(p.nanotech.max_health(&s,5),17);assert_eq!(p.nanotech.max_health(&s,8),20);
        s.nanotech_max_health=4;assert_eq!(p.nanotech.max_health(&s,8),12);assert_eq!(p.nanotech.bonus,12);
        s.nanotech_max_health=20;assert_eq!(p.nanotech.max_health(&s,8),20);assert_eq!(p.nanotech.xp,37.);
    }
    #[test]
    fn attack_specific_boss_damage_respects_owner_level_toggle_and_multiplier() {
        let mut actor=Moby::zeroed();actor.o_class=1422;
        let mut table=MobyTable::new(vec![actor,Moby::zeroed()],0);
        let h=HitRecord{attacker:Some(1),..Default::default()};let e=Enemies::default();
        let mut s=Settings{enemy_damage_by_health:true,..Default::default()};
        for (class,level,kind,expected) in [(1051,5,0,3),(1106,7,0,4),(1422,18,0,5),(628,18,0,10),
            (564,18,0,5),(564,18,2,3),(624,18,0,5),(983,18,0,5),(568,18,0,3),(1046,7,0,4),(1049,7,0,4)] {
            table.mobys[1].o_class=class;table.mobys[1].pvars=vec![0;0x1a0];
            table.mobys[1].pvars[0x38..0x3c].copy_from_slice(&(kind as i32).to_le_bytes());
            table.mobys[1].pvars[0x198..0x19c].copy_from_slice(&1u32.to_le_bytes());
            for multiplier in [0.5,1.,2.,100.] {
                s.enemy_damage_multiplier=multiplier;
                assert_eq!(e.hit_points(&h,&table,level,&s),Some(scaled_enemy_damage(expected,&s)),"{class}, {kind}, {multiplier}");
            }
            s.enemy_damage_by_health=false;assert_eq!(e.hit_points(&h,&table,level,&s),None);s.enemy_damage_by_health=true;
        }
        table.mobys[1].o_class=568;table.mobys[0].o_class=577;
        assert_eq!(e.hit_points(&h,&table,18,&s),None);
        table.mobys[0].o_class=1422;assert_eq!(e.hit_points(&h,&table,1,&s),None);
        table.mobys[1].pvars[0x198..0x19c].fill(0);assert_eq!(e.hit_points(&h,&table,18,&s),None);
    }
}
