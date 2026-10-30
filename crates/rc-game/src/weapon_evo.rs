//! Weapon Evolution for ReRAC v0.3.8-alpha.
//! Per-weapon V1..V8 tracks. Active only with RC_WEAPON_EVOLUTION_INI.
//! XP, grades and bonus Nanotech are stored beside each native save; its format is unchanged.
#[path = "weapon_evo_controls.rs"]
pub mod controls;
#[path = "weapon_evo_live.rs"]
pub mod live;
#[path = "weapon_evo_randomizer.rs"]
pub mod randomizer;
#[path = "weapon_evo_orxon_shortcut.rs"]
pub mod orxon_shortcut;
#[path = "weapon_evo_pickup_visuals.rs"]
pub mod pickup_visuals;
#[path="weapon_evo_pickup_mesh.rs"]
pub mod pickup_mesh;
#[path="weapon_evo_quick_select.rs"]
pub mod quick_select;
#[path = "weapon_evo_bolt_chain.rs"]
pub mod bolt_chain;
#[path = "weapon_evo_gold_shop.rs"]
pub mod gold_shop;
#[path = "weapon_evo_legacy_gold.rs"]
mod legacy_gold;
#[path = "weapon_evo_save.rs"]
pub mod save;
#[path = "weapon_evo_notice.rs"]
pub mod notice;
#[path = "weapon_evo_ammo.rs"]
pub mod ammo_mode;
#[cfg(feature = "weapon-evo-release")]
const RELEASE_PROFILE: &str = include_str!("weapon_evo_release.ini");
#[path="weapon_evo_wrench.rs"]
pub mod wrench;
#[path="weapon_evo_world.rs"]
pub mod world;
#[path = "weapon_evo_burst.rs"]
pub mod burst;
#[path = "weapon_evo_discovery.rs"]
pub mod discovery;
#[path="weapon_evo_golden.rs"]
pub mod golden;
#[path="weapon_evo_location_test.rs"]
pub mod location_test;
#[path="weapon_evo_gemlik_test.rs"]
pub mod gemlik_test;
#[path="weapon_evo_health_test.rs"]
pub mod health_test;
#[cfg(test)]
#[path="weapon_evo_upgrade_tests.rs"]
mod upgrade_tests;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{OnceLock, RwLock, Mutex};
/// A thread-safe queue owned by one Evolution instance, including across clones.
#[derive(Debug, Default)]
struct PendingSpecial(Mutex<Vec<(i32, usize, f32, f32)>>);
impl Clone for PendingSpecial {
    fn clone(&self) -> Self {
        Self(Mutex::new(self.0.lock().map(|q| q.clone()).unwrap_or_default()))
    }
}


// Snapshot of current per-level gameplay values; read by hand items, mobys and HUD.
// The register is republished whenever a weapon levels up; values without a
// corresponding INI entry are handled by their native-code fallback.
static LEVEL_VALUES: OnceLock<RwLock<HashMap<(i32, String), f32>>> = OnceLock::new();
fn level_values() -> &'static RwLock<HashMap<(i32, String), f32>> {
    LEVEL_VALUES.get_or_init(|| RwLock::new(HashMap::new()))
}
/// A per-level absolute value or multiplier. Without an active INI: native behavior.
pub fn extra(item: i32, key: &str, native: f32) -> f32 {
    if !CONFIG_ACTIVE.load(Ordering::Relaxed) { return native; }
    level_values().read().ok().and_then(|g| {
        if key == "gold" {
            g.get(&(item, "gold_effects".to_string())).or_else(|| g.get(&(item, "gold".to_string()))).copied()
        } else { g.get(&(item, key.to_string())).copied() }
    }).unwrap_or(native)
}
pub fn real_hp_active() -> bool { extra(-1, "real_hp", 0.0) >= 0.5 }
pub fn extra_flag(item: i32, key: &str, native: bool) -> bool {
    extra(item, key, if native {1.0} else {0.0}) >= 0.5
}
/// Per-level RGB in native BBGGRR order. All 24-bit colours are exact in f32.
/// -1 is the default sentinel; no custom colour overrides native effects.
pub fn extra_color(item: i32, key: &str) -> Option<u32> {
    let v = extra(item, key, -1.0);
    (v.is_finite() && (0.0..=16_777_215.0).contains(&v)).then_some(v as u32)
}
/// No new configuration key is accepted unless a gameplay hook consumes it.
fn supported_extra_stat(item: i32, key: &str) -> bool {
    if matches!(key, "max_ammo" | "damage_multiplier" | "gold" | "gold_texture" | "gold_effects") { return true; }
    if key == "experimental_max_ammo" { return matches!(item, 9|13|17|19|23|24); }
    let fields: &[&str] = match item {
        8 => &["trail_color", "hit_radius_multiplier", "knockback_multiplier", "throw_range", "throw_speed", "show_hud", "crate_breaker_enabled", "crate_breaker_radius", "moving_attacks_enabled", "move_while_thrown", "burn_on_hit", "hud_quickselect_only", "slam_explosion_enabled", "slam_explosion_damage_hp", "slam_explosion_radius", "slam_explosion_color", "bolt_attraction_bonus"],
        9 => &["capacity", "suck_range_multiplier", "shot_damage_multiplier", "crate_suction_enabled", "suck_wooden_crates", "suck_explosive_crates", "suck_steel_crates", "suck_ammo_crates", "suck_nanotech_crates"],
        11 => &["fire_interval_ticks", "homing_strength_multiplier", "missile_speed_multiplier", "missile_range_multiplier", "explosion_radius_multiplier", "explosion_visual_size_multiplier", "explosion_color", "trail_color"],
        13 => &["duration_multiplier", "turn_multiplier", "damage_multiplier", "invulnerable_while_guiding", "explosion_damage_radius", "grenade_launcher_enabled", "grenade_size_percent", "grenade_arc_height", "grenade_max_range", "grenade_fire_interval_ticks", "grenade_damage_hp", "grenade_explosion_radius", "grenade_explosion_color", "grenade_color", "grenade_model", "grenade_mine_color", "grenade_mine_explosion_color", "grenade_missile_color", "grenade_missile_explosion_color"],
        14 => &["lure_range", "lure_cone_degrees", "xp_attract_fraction", "stun_range", "stun_duration_ticks", "stun_efficiency_percent", "damage_range", "damage_per_second", "wave_speed_multiplier", "wave_size_multiplier", "wave_range_multiplier", "wave_interval_ticks", "wave_alpha", "wave_color", "stun_launch_distance"],
        15 => &["fire_interval_ticks", "projectile_speed_multiplier", "projectile_range_multiplier", "projectile_color", "trail_color", "impact_color"],
        17 => &["throw_range_multiplier", "trajectory_height_multiplier", "activation_range_horizontal", "activation_range_vertical", "explosion_radius_multiplier", "max_active_mines", "mine_lifetime_seconds", "flame_mine_max_active_mines", "flame_mine_lifetime_seconds", "flame_core_color", "flame_outer_color", "burn_flame_color", "explosion_color", "light_color", "flame_mine_enabled", "mine_flame_range", "mine_flame_rotation_degrees_per_second", "mine_flame_damage_per_second", "flame_mine_size_percent", "flame_mine_model", "mine_model_color", "mine_bomb_color", "mine_bomb_core_color", "mine_bomb_outer_color", "mine_bomb_burn_color", "mine_sphere_color", "mine_sphere_core_color", "mine_sphere_outer_color", "mine_sphere_burn_color"],
        18 => &["held_lunge_duration_percent", "knockback_multiplier", "hit_radius_multiplier", "electric_fx_size_multiplier", "gold_electric_fx", "air_dash_enabled", "electric_core_color", "electric_glow_color", "fist_glow_color", "impact_color"],
        19 => &["beam_size_percent", "beam_range_multiplier", "dual_beam_enabled", "target_range_multiplier", "ammo_interval_ticks", "damage_interval_ticks", "lock_cone_multiplier", "beam_core_color", "beam_glow_color", "secondary_core_color", "secondary_glow_color", "arc_core_color", "arc_glow_color", "claw_core_color", "claw_glow_color", "spark_color", "spark_core_color", "impact_color", "light_color", "weapon_glow_color", "secondary_beam_damage_percent"],
        20 => &["robot_count", "robot_max_alive", "robot_detection_range", "explosion_radius_multiplier"],
        21 => &["fire_in_air", "morph_speed_multiplier", "explode_small_chickens", "chicken_damage_multiplier", "chicken_explosion_radius_multiplier", "beam_range_multiplier", "tube_color", "beam_core_color", "beam_glow_color", "secondary_core_color", "secondary_glow_color", "pulse_color", "muzzle_core_color", "muzzle_glow_color", "tip_color", "tip_core_color", "spark_color", "spark_core_color", "light_color"],
        23 => &["missiles_per_salvo", "fire_interval_ticks", "missile_speed_multiplier", "missile_range_multiplier", "homing_strength_multiplier", "missile_max_range", "explosion_damage_radius", "explosion_visual_size_multiplier", "continuous_missiles_enabled", "shotgun_enabled", "shotgun_damage_hp", "shotgun_speed", "shotgun_range", "shotgun_size_percent", "shotgun_spread_degrees", "shotgun_horizontal_spread_degrees", "shotgun_vertical_spread_degrees", "shotgun_projectile_color", "shotgun_trail_color", "shotgun_impact_color"],
        24 => &["drone_count", "detection_range_multiplier", "drone_size_multiplier", "blast_radius_multiplier", "sand_mouse_enabled", "sand_mouse_mode", "sand_mouse_lifetime_seconds", "sand_mouse_max_shots", "sand_mouse_fire_interval_ticks", "sand_mouse_shot_range", "sand_mouse_shot_damage_hp", "sand_mouse_projectile_color"],
        25 => &["decoy_health_multiplier", "explosion_enabled", "explosion_radius_multiplier", "explosion_damage_multiplier", "throw_range_multiplier", "xp_attract_fraction", "lure_range"],
        _ => &[],
    };
    fields.contains(&key)
}


const BOMB: usize = 10;
const PYRO: usize = 16;
static CONFIG_ACTIVE: AtomicBool = AtomicBool::new(false);
/// Levels visible in the weapon menus. 0 means no active Weapon Evolution
/// profile (use the native icon, including its native gold variant).
static ICON_LEVELS: [std::sync::atomic::AtomicU8; 37] =
    [const { std::sync::atomic::AtomicU8::new(0) }; 37];
/// -1: keep original gold map; 0: normal texture; 1: gold texture.
/// Does not affect gameplay's original gold bonus flags.
static GOLD_TEXTURE: [std::sync::atomic::AtomicI8; 37] =
    [const { std::sync::atomic::AtomicI8::new(-1) }; 37];
pub fn gold_texture(item: i32, original: bool) -> bool {
    let Some(v) = usize::try_from(item).ok().and_then(|i| GOLD_TEXTURE.get(i)) else { return original };
    match v.load(Ordering::Relaxed) { 0 => false, 1 => true, _ => original }
}


/// Real XP level for weapon-name labels, unaffected by icon-only preview modes.
/// Weapons without an enabled evolution track return None.
pub fn weapon_menu_level(item: i32) -> Option<u8> {
    if !CONFIG_ACTIVE.load(Ordering::Relaxed) { return None; }
    let i = usize::try_from(item).ok()?;
    let level = ICON_LEVELS.get(i)?.load(Ordering::Relaxed);
    (1..=8).contains(&level).then_some(level)
}

/// Level used by coloured icons; the visual test launcher can preview other colours.
pub fn weapon_icon_level(item: i32) -> Option<u8> {
    let level = weapon_menu_level(item)?;
    // Only for the visual test launcher. Never changes weapon XP or actual levels.
    // V1..V5 can be previewed with RC_WEAPON_EVO_ICON_TEST_LEVEL=1..5.
    static VISUAL_TEST_LEVEL: std::sync::OnceLock<Option<u8>> = std::sync::OnceLock::new();
    if let Some(test_level) = *VISUAL_TEST_LEVEL.get_or_init(|| {
        std::env::var("RC_WEAPON_EVO_ICON_TEST_LEVEL").ok()
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|&n| (1..=8).contains(&n))
    }) { return Some(test_level); }
    Some(level)
}

/// Native GS MODULATE is 0x7f per channel (RGB byte order is BBGGRR).
/// V1 stays unmodified; subsequent levels tint ONLY the 2-D weapon icon.
pub fn weapon_icon_tint(item: i32) -> Option<u32> {
    icon_grade_tint(weapon_icon_level(item)?)
}

/// Bright GS modulation on a neutral source frame, shared by every weapon.
pub fn icon_grade_tint(level:u8)->Option<u32> {
    match level {
        2 => Some(0x80ef_28ff), // bright fuchsia (approximately #FF20C8)
        3 => Some(0x8038_38ff), // bright red (approximately #FF5050)
        4 => Some(0x8000_78ff), // orange
        5 => Some(0x8000_e8ff), // golden yellow
        6 => Some(0x8020_ffb0), // luminous lime, distinct from gadget green
        7 => Some(0x80ff_e080), // pale cyan
        8 => Some(0x80ff_ffff), // white
        _ => None,
    }
}
static BOMB_AMMO_CAP: AtomicI32 = AtomicI32::new(0);
static PYRO_AMMO_CAP: AtomicI32 = AtomicI32::new(0);
/// FFFFFF in the atomic marks "use game defaults"; RGB itself is stored as BBGGRR.
static PYRO_CORE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
static PYRO_OUTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
static PYRO_BURN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
pub fn pyro_burn_color() -> Option<u32> { value_or_none(PYRO_BURN.load(Ordering::Relaxed)) }


pub fn pyro_core_color() -> Option<u32> { value_or_none(PYRO_CORE.load(Ordering::Relaxed)) }
pub fn pyro_outer_color() -> Option<u32> { value_or_none(PYRO_OUTER.load(Ordering::Relaxed)) }
fn value_or_none(c: u32) -> Option<u32> { (c != u32::MAX).then_some(c) }

/// UI syntax #RRGGBB -> native little-endian low-R RGB, alpha separate.
pub fn parse_rgb(s: &str) -> Result<Option<u32>, String> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("default") { return Ok(None); }
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() != 6 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid RGB color {s:?}; expected #RRGGBB or default"));
    }
    let v = u32::from_str_radix(hex, 16).map_err(|_| format!("invalid RGB {s}"))?;
    Ok(Some(((v >> 16) & 0xff) | (v & 0x00ff00) | ((v & 0xff) << 16)))
}

pub fn rgb_float(v: u32) -> [f32; 3] {
    [(v & 0xff) as f32 / 255.0, ((v >> 8) & 0xff) as f32 / 255.0, ((v >> 16) & 0xff) as f32 / 255.0]
}

/// Tint native particle RGBA without changing its alpha or time envelope.
pub fn tint_rgba(base: u32, color: Option<u32>) -> u32 {
    let Some(rgb) = color else { return base; };
    let strength = ((base & 0xff).max((base >> 8) & 0xff).max((base >> 16) & 0xff)).min(128) as f32 / 255.0;
    let r = ((rgb & 0xff) as f32 * strength).round() as u32;
    let g = (((rgb >> 8) & 0xff) as f32 * strength).round() as u32;
    let b = (((rgb >> 16) & 0xff) as f32 * strength).round() as u32;
    (base & 0xff00_0000) | (b << 16) | (g << 8) | r
}


/// Shared single-source-of-truth ammo capacity: shop, pickups, hero HUD.
pub fn effective_max_ammo(item: usize, vanilla: i32) -> i32 {
    let cap = match item {
        BOMB => BOMB_AMMO_CAP.load(Ordering::Relaxed),
        PYRO => PYRO_AMMO_CAP.load(Ordering::Relaxed),
        _ => return extra(item as i32, "max_ammo", vanilla as f32).round().clamp(0.0, 9999.0) as i32,
    };
    if cap > 0 { cap } else { vanilla }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClusterStats {
    pub enabled: bool,
    pub count: u8,
    pub model_percent: f32,
    pub explosion_percent: f32,
    pub damage_percent: f32,
    pub launch_speed: f32,
    pub jump_height: f32,
    pub jump_speed: Option<f32>,
}

#[cfg(test)]
mod v44_configuration_tests {
    use super::*;
    #[test]
    fn new_geometry_and_crate_options_are_independent_for_all_five_grades() {
        let mut text=String::from("[general]\ndamage_mode=real_hp\n");
        for grade in 1..=5 {
            text+=&format!("[weapon.bomb_glove.level{grade}]\ncluster_jump_speed={grade}\ncluster_launch_speed={}\ncluster_bomb_size_percent={}\n[weapon.pyrocitor.level{grade}]\nside_flame_left_angle_degrees={}\nside_flame_right_angle_degrees={}\n[weapon.suck_cannon.level{grade}]\ncrate_suction_enabled=true\nsuck_steel_crates=false\nsuck_wooden_crates=true\nsuck_ammo_crates=false\nsuck_nanotech_crates=true\nsuck_explosive_crates=false\n[weapon.tesla_claw.level{grade}]\nbeam_size_percent={}\nbeam_range_multiplier={}\ndual_beam_enabled=true\n",grade*2,grade*10,grade*5,grade*7,grade*50,grade);
        }
        let e=Evolution::parse(&text).unwrap();
        for i in 0..5 {
            let n=i as f32+1.0;
            assert_eq!(e.stats[i].cluster.jump_speed,Some(n));
            assert_eq!(e.stats[i].cluster.model_percent,n*10.0);
            assert_eq!(e.stats[i].cluster.launch_speed,n*2.0);
            assert_eq!(e.pyro.stats[i].side_flame_left_angle_degrees,Some(n*5.0));
            assert_eq!(e.pyro.stats[i].side_flame_right_angle_degrees,Some(n*7.0));
            let cannon=&e.extras.iter().find(|t|t.id==9).unwrap().params[i];
            assert_eq!(cannon["crate_suction_enabled"],1.0);assert_eq!(cannon["suck_steel_crates"],0.0);
            let tesla=&e.extras.iter().find(|t|t.id==19).unwrap().params[i];
            assert_eq!(tesla["beam_size_percent"],n*50.0);assert_eq!(tesla["beam_range_multiplier"],n);
            assert_eq!(tesla["dual_beam_enabled"],1.0);
        }
    }
    #[test]
    fn malformed_experimental_and_standard_geometry_is_rejected() {
        for (weapon,key,value) in [("bomb_glove","cluster_jump_speed","NaN"),("bomb_glove","cluster_jump_speed","0"),
            ("pyrocitor","side_flame_left_angle_degrees","-1"),("pyrocitor","side_flame_right_angle_degrees","81"),
            ("tesla_claw","beam_size_percent","0"),("tesla_claw","beam_range_multiplier","9"),
            ("tesla_claw","dual_beam_enabled","maybe"),("suck_cannon","suck_ammo_crates","maybe")] {
            assert!(Evolution::parse(&format!("[general]\ndamage_mode=real_hp\n[weapon.{weapon}.level1]\n{key}={value}\n")).is_err(),"{weapon} {key} {value}");
        }
        let stats=ClusterStats::default();assert_eq!(stats.model_percent,40.0);assert_eq!(stats.launch_speed,3.0);
        assert_eq!(PyroStats::default().side_flame_angle_degrees,22.0);
    }
}
impl Default for ClusterStats {
    fn default() -> Self { Self { enabled: false, count: 3, model_percent: 40.0,
        explosion_percent: 15.0, damage_percent: 10.0, launch_speed: 3.0, jump_height: 0.8, jump_speed: None } }
}
impl ClusterStats {
    fn valid(self) -> bool {
        (1..=12).contains(&self.count) && (1.0..=100.0).contains(&self.model_percent)
            && (1.0..=100.0).contains(&self.explosion_percent) && (0.1..=100.0).contains(&self.damage_percent)
            && (0.0..=20.0).contains(&self.launch_speed) && (0.1..=3.0).contains(&self.jump_height)
            && self.jump_speed.is_none_or(|v| (0.1..=10.0).contains(&v))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub cluster: ClusterStats,
    pub throw_range: f32,
    /// Absolute auto-aim range; None follows throw reach, zero disables snapping.
    pub auto_lock_range: Option<f32>,
    /// 1.0 = native arc; greater means higher parabola (gravity and launch correction).
    pub trajectory_height: f32,
    pub explosion_radius: f32,
    /// RGB written as #RRGGBB in the INI; None = native particles.
    pub explosion_color: Option<u32>,
    pub cooldown_seconds: f32,
    pub max_ammo: i32,
    pub experimental_max_ammo: Option<i32>,
    pub damage_multiplier: f32,
    pub gold: bool,
    pub gold_texture: bool,
}
impl Default for Stats {
    fn default() -> Self {
        Self { cluster: ClusterStats::default(), throw_range: 1.0, auto_lock_range: None, trajectory_height: 1.0, explosion_radius: 1.0, explosion_color: None, cooldown_seconds: 20.0/60.0,
            max_ammo: 40, experimental_max_ammo: None, damage_multiplier: 1.0, gold: false, gold_texture: false }
    }
}

impl Stats {
    fn ammo_capacity(self)->i32 {
        if self.cluster.enabled {self.experimental_max_ammo.unwrap_or(self.max_ammo)} else {self.max_ammo}
    }

    /// A lower arc shortens the native free throw; a configured auto-lock range
    /// can narrow that reach but cannot advertise a target beyond the throw.
    pub fn auto_lock_distance(self) -> f32 {
        let reach = 8.5 * self.throw_range * self.trajectory_height.min(1.0);
        self.auto_lock_range.unwrap_or(reach).min(reach)
    }
}

/// The flame width scales BOTH particle dispersion and the hit spheres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PyroStats {
    pub lava: crate::hero::pyro_lava::Settings,
    pub triple_flame_enabled: bool,
    pub side_flame_angle_degrees: f32,
    pub side_flame_left_angle_degrees: Option<f32>,
    pub side_flame_right_angle_degrees: Option<f32>,
    pub ammo_interval_ticks: i32,
    pub damage_interval_ticks: i32,
    pub flame_range: f32,
    pub flame_width: f32,
    pub damage_multiplier: f32,
    pub flame_core_color: Option<u32>,
    pub flame_outer_color: Option<u32>,
    pub light_color: Option<u32>,
    pub max_ammo: i32,
    pub experimental_max_ammo: Option<i32>,
    pub gold: bool,
    pub gold_texture: bool,
    pub burn_flame_color: Option<u32>,
}
impl Default for PyroStats {
    fn default() -> Self { Self { lava: Default::default(), triple_flame_enabled: false, side_flame_angle_degrees: 22.0, side_flame_left_angle_degrees: None, side_flame_right_angle_degrees: None, ammo_interval_ticks: 10, damage_interval_ticks: 1, flame_range: 1.0, flame_width: 1.0, damage_multiplier: 1.0, flame_core_color: None, flame_outer_color: None, light_color: None, max_ammo: 100, experimental_max_ammo: None, gold: false, gold_texture: false, burn_flame_color: None } }
}

impl PyroStats {
    fn ammo_capacity(self)->i32 {
        if self.triple_flame_enabled || self.lava.enabled {self.experimental_max_ammo.unwrap_or(self.max_ammo)} else {self.max_ammo}
    }
}

/// Global UI settings. Full opacity is 100% (GS alpha 0x80).
#[derive(Clone, Copy, Debug)]
pub struct HudOptions {
    pub name_opacity_percent: u8,
    pub weapon_bar_alternative_position:bool,
    pub weapon_bar_behind_hud:bool,
    /// Independent of weapon-name opacity and the legacy weapon XP setting.
    pub nanotech_xp_bar_opacity_percent: u8,
    pub xp_bar_height_px: i32,
    pub xp_bar_horizontal_inset_px: i32,
    pub show_for_weapons_without_ammo: bool,
    pub level_up_popup: bool,
    pub level_up_popup_duration_seconds: f32,
    pub level_up_sound: bool,
}
impl Default for HudOptions {
    fn default() -> Self {
        Self { name_opacity_percent: 100, weapon_bar_alternative_position:false, weapon_bar_behind_hud:true, nanotech_xp_bar_opacity_percent: 100,
            xp_bar_height_px: 5, xp_bar_horizontal_inset_px: 2,
            show_for_weapons_without_ammo: true, level_up_popup: true,
            level_up_popup_duration_seconds: 3.0, level_up_sound: true }
    }
}
impl HudOptions {
    pub fn alpha(percent: u8) -> u32 { ((percent as u32 * 128 + 50) / 100).min(128) }
}
fn parse_percent(v: &str, key: &str) -> Result<u8, String> {
    let n: u8 = v.parse().map_err(|_| format!("invalid {key}: {v}"))?;
    if n > 100 { return Err(format!("{key} must be 0..100")); }
    Ok(n)
}

/// IDs 8..25: weapons and combat gadgets, excluding unrelated utility items.
pub const MAX_LEVEL: usize = 8;
pub const BASE_LEVEL_CAP: usize = 5;
pub const WEAPON_IDS: [i32; 16] = [8, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24, 25];

#[derive(Clone, Debug)]
pub struct ExtraTrack {
    pub section: &'static str,
    pub id: i32,
    pub enabled: bool,
    pub level: usize,
    pub level_cap: usize,
    pub xp: f32,
    pub xp_required: [f32; 7],
    pub last_hit_xp: f32,
    /// Per-level properties are absolute values or multipliers according to their name.
    pub params: [BTreeMap<String, f32>; 8],
}
impl ExtraTrack {
    /// The same active capacity feeds the published HUD/pickup registry and
    /// the actual inventory. Suck Cannon uses stored objects rather than ammo.
    fn ammo_capacity(&self) -> Option<i32> {
        if !self.enabled { return None; }
        let params = &self.params[self.level.saturating_sub(1).min(7)];
        let standard = if self.id == 9 { "capacity" } else { "max_ammo" };
        let value = if experimental_ammo_enabled(self.id, params) {
            params.get("experimental_max_ammo").or_else(|| params.get(standard))
        } else { params.get(standard) };
        value.map(|cap| cap.round().clamp(0.0, 9999.0) as i32)
    }
    fn new(section: &'static str, id: i32) -> Self {
        Self { section, id, enabled: false, level: 1, level_cap: 5,
            xp: 0.0, xp_required: [10.0, 20.0, 35.0, 55.0, 70.0, 85.0, 110.0], last_hit_xp: 0.0, params: std::array::from_fn(|_| BTreeMap::new()) }
    }
    pub fn required(&self) -> f32 { if self.level >= self.level_cap { 0.0 } else { self.xp_required[self.level-1] } }
    pub fn fraction(&self) -> f32 { percent(self.xp, self.required(), self.level >= self.level_cap) }
    pub fn award(&mut self, amount: f32) {
        self.last_hit_xp = 0.0;
        if !self.enabled || self.level >= self.level_cap || !amount.is_finite() || amount <= 0.0 { return; }
        self.last_hit_xp = amount;
        self.xp += amount;
        while self.level < self.level_cap && self.xp >= self.required() {
            self.xp -= self.required();
            self.level += 1;
            println!("weapon evolution: {} -> V{} (remaining XP {:.1})", self.section, self.level, self.xp);
        }
        if self.level >= self.level_cap { self.xp = 0.0; }
    }
}
fn extra_tracks() -> Vec<ExtraTrack> {
    vec![
        ExtraTrack::new("wrench", 8),
        ExtraTrack::new("suck_cannon", 9),
        ExtraTrack::new("devastator", 11),
        ExtraTrack::new("visibomb", 13),
        ExtraTrack::new("taunter", 14),
        ExtraTrack::new("blaster", 15),
        ExtraTrack::new("mine_glove", 17),
        ExtraTrack::new("walloper", 18),
        ExtraTrack::new("tesla_claw", 19),
        ExtraTrack::new("glove_of_doom", 20),
        ExtraTrack::new("morph_ray", 21),
        ExtraTrack::new("ryno", 23),
        ExtraTrack::new("drone_device", 24),
        ExtraTrack::new("decoy_glove", 25),
    ]
}

fn valid_xp(values: &[f32; 7], weapon: &str) -> Result<(), String> {
    for (i, &v) in values.iter().enumerate() {
        if !v.is_finite() || !(0.1..=1_000_000.0).contains(&v) {
            return Err(format!("{weapon}: invalid XP threshold V{}->V{}", i+1, i+2));
        }
    }
    Ok(())
}
fn percent(xp: f32, needed: f32, maxed: bool) -> f32 {
    if maxed { 1.0 } else { (xp / needed.max(0.001)).clamp(0.0, 1.0) }
}
fn xp_transition(key: &str) -> Option<usize> {
    match key {
        "xp_v1_to_v2" => Some(0), "xp_v2_to_v3" => Some(1),
        "xp_v3_to_v4" => Some(2), "xp_v4_to_v5" => Some(3), "xp_v5_to_v6" => Some(4),
        "xp_v6_to_v7" => Some(5), "xp_v7_to_v8" => Some(6), _ => None,
    }
}
fn parse_number(value: &str, key: &str) -> Result<f32, String> {
    let n: f32 = value.parse().map_err(|_| format!("invalid {key}: {value}"))?;
    if !n.is_finite() { return Err(format!("nonfinite {key}: {value}")); }
    Ok(n)
}
fn parse_tick_interval(value: &str, key: &str, max: i32) -> Result<i32, String> {
    let n: i32 = value.parse().map_err(|_| format!("{key} must be an integer 1..{max}"))?;
    if !(1..=max).contains(&n) { return Err(format!("{key} must be an integer 1..{max}")); }
    Ok(n)
}
fn parse_ammo(value: &str) -> Result<i32, String> {
    value.parse().map_err(|_| format!("invalid max_ammo: {value}"))
}
fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" => Ok(true), "false" | "no" | "0" => Ok(false),
        _ => Err(format!("invalid boolean: {value}")),
    }
}
// Accepted only for compatibility with old profiles; never used for display.
fn legacy_name_key(key: &str) -> bool {
    matches!(key, "name_en" | "name_fr" | "name_de" | "name_es" | "name_it")
}

#[derive(Clone, Debug)]
pub struct PyroEvolution {
    pub enabled: bool,
    pub level: usize,
    pub level_cap: usize,
    pub xp: f32,
    pub xp_required: [f32; 7],
    pub stats: [PyroStats; 8],
    pub last_hit_xp: f32,
}
impl Default for PyroEvolution {
    fn default() -> Self {
        Self { enabled: false, level: 1, level_cap: 5, xp: 0.0,
            xp_required: [10.0, 16.0, 24.0, 32.0, 40.0, 48.0, 64.0], stats: [PyroStats::default(); 8], last_hit_xp: 0.0 }
    }
}
impl PyroEvolution {
    pub fn current(&self) -> PyroStats { self.stats[self.level.saturating_sub(1).min(7)] }
    pub fn required(&self) -> f32 { if self.level >= self.level_cap { 0.0 } else { self.xp_required[self.level-1] } }
    pub fn fraction(&self) -> f32 { percent(self.xp, self.required(), self.level >= self.level_cap) }
    fn award(&mut self, amount: f32) {
        self.last_hit_xp = 0.0;
        if !self.enabled || self.level >= self.level_cap || !amount.is_finite() { return; }
        self.last_hit_xp = amount.max(0.0);
        self.xp += self.last_hit_xp;
        while self.level < self.level_cap && self.xp >= self.required() {
            self.xp -= self.required();
            self.level += 1;
            println!("weapon evolution: PYROCITOR -> V{} (remaining XP {:.1})", self.level, self.xp);
        }
        if self.level >= self.level_cap { self.xp = 0.0; }
    }
}

#[derive(Clone, Debug)]
pub struct Evolution {
    pub randomizer_mode:bool,
    pub randomizer_bolt_multiplier:bool,
    pub bolt_multiplier_max:f32,
    pub control_settings:controls::Settings,
    pub r1_experimental_max:[[u8;8];37],
    /// Opt-in reward for an earned upgrade; loading progress never grants ammo.
    pub golden_settings:golden::Settings,
    pub hide_ryno: bool,
    pub ng_plus: bool,
    /// Transient accepted wrench XP events; HUD timing is independent of damage/stun.
    pub wrench_xp_feedback_serial: u64,
    pub bolt_chain: bolt_chain::State,
    pub gold_spent: u32,
    pub refill_ammo_on_upgrade: bool,
    upgrade_refills: u64,
    pub world_settings:world::Settings,
    /// Unscaled encounter settings; NG+ snapshots never compound.
    encounter_base: Option<world::Settings>,
    pub nanotech:world::NanotechProgress,
    pub nanotech_native_max:i32,
    pub nanotech_pending_heal:u32,
    pub ammo_unlock_mode: bool,
    pub ammo_mode_settings: ammo_mode::Settings,
    pub wrench_gold_texture: bool,
    pub enabled: bool,
    pub level: usize,
    pub level_cap: usize,
    pub applied_level: usize,
    pub xp: f32,
    pub xp_per_damage: f32,
    /// v37: public HP units; native AI keeps its original normalized health.
    pub real_hp: bool,
    pub(crate) combat_epoch: u64,
    pub damage_hp: [[Option<f32>; 8]; 37],
    pub stats: [Stats; 8],
    /// Four transitions: V1 -> V2, V2 -> V3, V3 -> V4, V4 -> V5.
    pub xp_required: [f32; 7],
    pub last_hit_xp: f32,
    pub pyro: PyroEvolution,
    pub extras: Vec<ExtraTrack>,
    pub hud: HudOptions,
    /// Once per moby and XP mechanism (not once per render/tick).
    award_once: HashSet<(i32, usize)>,
    pending_special: PendingSpecial,
    terminal_targets: HashSet<usize>,
    /// Last state exchanged with GameState; a menu load/reset takes precedence
    /// over the previous world's live XP, even during a level transition.
    state_progress: save::Progress,
}
impl Default for Evolution {
    fn default() -> Self {
        Self { randomizer_bolt_multiplier:false,bolt_multiplier_max:3.,r1_experimental_max:controls::default_mode_limits(),randomizer_mode:false,control_settings:Default::default(),golden_settings:Default::default(),hide_ryno:false,ng_plus:false,wrench_xp_feedback_serial:0,bolt_chain:Default::default(),gold_spent:0,refill_ammo_on_upgrade:false,upgrade_refills:0,world_settings:Default::default(),encounter_base:None,nanotech:Default::default(),nanotech_native_max:4,nanotech_pending_heal:0,ammo_unlock_mode: false, ammo_mode_settings: Default::default(), wrench_gold_texture: false, enabled: false, level: 1, level_cap: 5, applied_level: 0,
            xp: 0.0, xp_per_damage: 1.0, real_hp: false, combat_epoch: 0, damage_hp: [[None; 8]; 37], stats: [Stats::default(); 8],
            xp_required: [6.0; 7], last_hit_xp: 0.0, pyro: PyroEvolution::default(), extras: extra_tracks(), hud: HudOptions::default(), award_once: HashSet::new(), pending_special: Default::default(), terminal_targets: HashSet::new(), state_progress: Default::default() }
    }
}
impl Evolution {
    pub fn reward_multiplier_active(&self)->bool {
        self.ng_plus || (self.randomizer_mode && self.randomizer_bolt_multiplier && self.state_progress.randomizer.active())
    }
    pub fn reward_factor(&self)->f32 {self.bolt_chain.factor_with_max(self.reward_multiplier_active(),self.bolt_multiplier_max)}
    pub fn reward_factor_tenths(&self)->u16 {self.bolt_chain.tenths(self.reward_multiplier_active(),self.bolt_multiplier_max)}
    pub fn reward_chain_kill(&mut self){self.bolt_chain.kill(self.reward_multiplier_active());}
    /// Commit the reset to both live and saved progress before reload/respawn.
    /// Updating only the live chain lets a later progress sync restore the old
    /// saved bonus. XP, inventory, seed and other progression are untouched.
    pub fn reset_reward_chain(&mut self, persisted: Option<&mut save::Progress>) {
        self.bolt_chain.hurt();
        self.state_progress.bolt_chain = self.bolt_chain;
        if let Some(progress) = persisted { progress.bolt_chain = self.bolt_chain; }
    }
    pub fn collect_reward_bolts(&mut self,value:i32)->i32 {
        self.bolt_chain.collect_with_max(value,self.reward_multiplier_active(),self.bolt_multiplier_max)
    }
    pub fn controls(&self)->controls::State{self.state_progress.controls}
    pub fn set_controls(&mut self,state:controls::State){
        if state.valid() && state!=self.state_progress.controls {self.state_progress.controls=state;self.publish_max_ammo();}
    }
    /// R1 limits belong to the current profile/grade, never to the save format.
    pub fn r1_mode_max(&self,item:i32)->u8 {
        if !self.control_settings.r1{return 0;}
        let level=match item {
            10=>self.level,16=>self.pyro.level,
            _=>self.extras.iter().find(|t|t.id==item).map_or(1,|t|t.level),
        };
        usize::try_from(item).ok().and_then(|id|self.r1_experimental_max.get(id))
            .map_or(0,|limits|limits[level.saturating_sub(1).min(7)])
            .min(controls::supported_mode_max(item))
    }
    pub fn effective_pyro(&self)->PyroStats {
        let mut stats=self.pyro.current();
        if self.control_settings.r1 {stats.lava.enabled=self.state_progress.controls.mode_limited(16,self.r1_mode_max(16))==Some(1);}
        stats
    }
    fn track_ammo_capacity(&self,track:&ExtraTrack)->Option<i32>{
        if !self.control_settings.r1 || !controls::MODE_ITEMS.contains(&track.id){return track.ammo_capacity();}
        if !track.enabled{return None;}
        let params=&track.params[track.level.saturating_sub(1).min(7)];
        let alternate=self.state_progress.controls.mode_limited(track.id,self.r1_mode_max(track.id)).is_some_and(|m|m>0);
        let cap=if alternate {params.get("experimental_max_ammo").or_else(||params.get("max_ammo"))}else{params.get("max_ammo")};
        cap.map(|v|v.round().clamp(0.,9999.) as i32)
    }
    pub fn configure_ng_plus(&mut self, ng_plus: bool) {
        if !self.ng_plus || self.encounter_base.is_none() {
            self.encounter_base = Some(self.world_settings);
        }
        self.world_settings = self.encounter_base.unwrap().for_ng_plus(ng_plus);
        self.ng_plus=ng_plus;
        self.level_cap=if self.level>=6 {8}else{5};
        self.pyro.level_cap=if self.pyro.level>=6 {8}else{5};
        for t in &mut self.extras {t.level_cap=if t.id==8 {if ng_plus {8}else{5}} else if t.level>=6 {8}else{5};}
    }
    pub fn purchase_gold_upgrade(&mut self,item:i32)->bool {
        if !self.ng_plus || self.weapon_level(item)!=Some(5) || item==8 {return false;}
        match item {10=>{self.level=6;self.level_cap=8;self.xp=0.;},16=>{self.pyro.level=6;self.pyro.level_cap=8;self.pyro.xp=0.;},
            _=>{let Some(t)=self.extras.iter_mut().find(|t|t.id==item && t.enabled) else{return false;};t.level=6;t.level_cap=8;t.xp=0.;}}
        self.earned_upgrade(item,5);self.publish_max_ammo();true
    }
    pub fn weapon_at_cap(&self,item:i32)->bool {
        match item {10=>self.level>=self.level_cap,16=>self.pyro.level>=self.pyro.level_cap,
            _=>self.extras.iter().find(|t|t.id==item).is_some_and(|t|t.level>=t.level_cap)}
    }
    fn earned_upgrade(&mut self, weapon:i32, before:usize) {
        if self.refill_ammo_on_upgrade && self.weapon_level(weapon).is_some_and(|after| after>before)
            && matches!(weapon,10|11|13|15|16|17|19|20|23|24|25) {
            self.upgrade_refills |= 1u64 << weapon;
        }
    }
    /// Called after inventory synchronization, before firing. The pending mask
    /// is transient, cleared on restore and spent once even if the gun is empty.
    /// Suck Cannon contents cannot be fabricated from its capacity setting.
    pub fn apply_upgrade_refills(&mut self, weapons:&mut crate::hero::weapons::Weapons, owned:&[u8]) {
        let pending=std::mem::take(&mut self.upgrade_refills);
        if !self.refill_ammo_on_upgrade {return;}
        for item in 0..weapons.ammo.len().min(64) {
            if pending & (1u64<<item)==0 || owned.get(item).copied().unwrap_or(0)==0 {continue;}
            let cap=match item {
                BOMB=>Some(self.current().ammo_capacity()),
                PYRO=>Some(self.effective_pyro().ammo_capacity()),
                _=>self.extras.iter().find(|t|t.id==item as i32 && t.enabled).and_then(|t|self.track_ammo_capacity(t)),
            };
            if let Some(cap)=cap {weapons.ammo[item]=cap.max(0);}
        }
    }
    /// Restrict current ammunition to the active grade/mode without refilling.
    /// Call after restoring inventory and before weapon updates each tick.
    pub fn apply_ammo_limits(&self, weapons: &mut crate::hero::weapons::Weapons) {
        if self.enabled { weapons.ammo[BOMB] = weapons.ammo[BOMB].clamp(0, self.current().ammo_capacity()); }
        if self.pyro.enabled { weapons.ammo[PYRO] = weapons.ammo[PYRO].clamp(0, self.effective_pyro().ammo_capacity()); }
        for track in self.extras.iter().filter(|track| track.id != 9 && track.id != 8) {
            if let Some(cap) = self.track_ammo_capacity(track) {
                if let Some(ammo) = weapons.ammo.get_mut(track.id as usize) { *ammo = (*ammo).clamp(0, cap); }
            }
        }
    }
    pub fn real_hp_enabled(&self) -> bool { self.enabled && self.real_hp }
    fn hp_units(&self, n: f32) -> f32 { n * if self.real_hp_enabled() { 100.0 } else { 1.0 } }
    pub fn stream_intervals(&self) -> [u32; 2] {
        let pyro = if self.pyro.enabled { self.effective_pyro().damage_interval_ticks as u32 } else { 1 };
        let tesla = self.extras.iter().find(|t| t.id == 19 && t.enabled)
            .and_then(|t| t.params[t.level.saturating_sub(1).min(7)].get("damage_interval_ticks"))
            .copied().unwrap_or(1.0) as u32;
        [pyro.max(1), tesla.max(1)]
    }
    pub fn absolute_values(&self) -> HashMap<i32, f32> {
        let mut values = HashMap::new();
        if !self.real_hp_enabled() { return values; }
        for item in 0..37 {
            let level = if item == 10 { self.level } else if item == 16 { self.pyro.level }
                else { self.extras.iter().find(|t| t.id == item as i32).map_or(1, |t| t.level) };
            if let Some(hp) = self.damage_hp[item][level.saturating_sub(1).min(7)] { values.insert(item as i32, hp); }
        }
        values
    }
    fn publish_max_ammo(&self) {
        if CONFIG_ACTIVE.load(Ordering::Relaxed) {
            controls::publish(self.control_settings);
            for icon in &ICON_LEVELS { icon.store(0, Ordering::Relaxed); }
            for flag in &GOLD_TEXTURE { flag.store(-1, Ordering::Relaxed); }
            GOLD_TEXTURE[8].store(i8::from(self.wrench_gold_texture), Ordering::Relaxed);
            if self.enabled { GOLD_TEXTURE[BOMB].store(i8::from(self.current().gold_texture), Ordering::Relaxed); }
            if self.pyro.enabled { GOLD_TEXTURE[PYRO].store(i8::from(self.effective_pyro().gold_texture), Ordering::Relaxed); }
            if self.enabled { ICON_LEVELS[BOMB].store(self.level as u8, Ordering::Relaxed); }
            if self.pyro.enabled { ICON_LEVELS[PYRO].store(self.pyro.level as u8, Ordering::Relaxed); }
            for track in &self.extras {
                if track.enabled {
                    if let Ok(i) = usize::try_from(track.id) {
                        if let Some(icon) = ICON_LEVELS.get(i) { icon.store(track.level as u8, Ordering::Relaxed); }
                        if let Some(flag) = GOLD_TEXTURE.get(i) {
                            let p = &track.params[track.level.saturating_sub(1).min(7)];
                            let texture = p.get("gold_texture").or_else(|| p.get("gold_effects")).or_else(|| p.get("gold"));
                            if let Some(&v) = texture { flag.store(i8::from(v >= 0.5), Ordering::Relaxed); }
                        }
                    }
                }
            }
            if let Ok(mut registry) = level_values().write() {
                registry.clear();
                registry.insert((-1, ammo_mode::KEY.to_string()), f32::from(u8::from(self.ammo_unlock_mode)));
                if self.real_hp {
                    registry.insert((-1, "real_hp".to_string()), 1.0);
                    for (item, hp) in self.absolute_values() {
                        registry.insert((item, if item == 16 || item == 19 {"damage_per_second"} else {"damage_hp"}.to_string()), hp);
                    }
                }
                for track in &self.extras {
                    if !track.enabled { continue; }
                    let params=&track.params[track.level.saturating_sub(1).min(7)];
                    for (name, &value) in params { registry.insert((track.id, name.clone()), value); }
                    if let Some(cap) = self.track_ammo_capacity(track) {
                        registry.insert((track.id, if track.id==9 {"capacity"} else {"max_ammo"}.into()), cap as f32);
                    }
                }
            }
            if let Ok(mut registry)=level_values().write(){controls::apply_registry(self.control_settings,&self.state_progress.controls,&mut registry,|item|self.r1_mode_max(item));}
            BOMB_AMMO_CAP.store(if self.enabled { self.current().ammo_capacity() } else { 0 }, Ordering::Relaxed);
            PYRO_AMMO_CAP.store(if self.pyro.enabled { self.effective_pyro().ammo_capacity() } else { 0 }, Ordering::Relaxed);
            let p = self.effective_pyro();
            PYRO_CORE.store(p.flame_core_color.unwrap_or(u32::MAX), Ordering::Relaxed);
            PYRO_OUTER.store(p.flame_outer_color.unwrap_or(u32::MAX), Ordering::Relaxed);
            PYRO_BURN.store(p.burn_flame_color.or(p.flame_outer_color).or(p.flame_core_color).unwrap_or(u32::MAX), Ordering::Relaxed);
        }
    }
    pub fn current(&self) -> Stats { self.stats[self.level.saturating_sub(1).min(7)] }
    pub fn required(&self) -> f32 { if self.level >= self.level_cap { 0.0 } else { self.xp_required[self.level-1] } }
    pub fn fraction(&self) -> f32 { percent(self.xp, self.required(), self.level >= self.level_cap) }
    pub fn weapon_fraction(&self, weapon: i32) -> Option<f32> {
        match weapon {
            10 if self.enabled => Some(self.fraction()),
            16 if self.pyro.enabled => Some(self.pyro.fraction()),
            _ => self.extras.iter().find(|t| t.id == weapon && t.enabled).map(ExtraTrack::fraction),
        }
    }
    pub fn weapon_hud_enabled(&self,weapon:i32)->bool {
        weapon!=8 || self.extras.iter().find(|t|t.id==8 && t.enabled).is_some_and(|t|
            t.params[t.level.saturating_sub(1).min(7)].get("show_hud").copied().unwrap_or(1.)!=0.)
    }
    pub fn wrench_quickselect_only(&self)->bool {
        self.extras.iter().find(|t|t.id==8&&t.enabled).is_some_and(|t|
            t.params[t.level.saturating_sub(1).min(7)].get("hud_quickselect_only").copied().unwrap_or(0.)!=0.)
    }
    pub fn weapon_level(&self, weapon: i32) -> Option<usize> {
        match weapon { 10 if self.enabled => Some(self.level), 16 if self.pyro.enabled => Some(self.pyro.level),
            _ => self.extras.iter().find(|t| t.id == weapon && t.enabled).map(|t| t.level) }
    }
    /// The renderer supplies the disc's localized bytes (including its font encoding).
    pub fn equipped_title(&self, weapon: i32, native_name: &[u8]) -> Option<Vec<u8>> {
        let level = self.weapon_level(weapon)?;
        if native_name.is_empty() { return None; }
        let mut title = native_name.to_vec();
        title.extend_from_slice(format!(" V{level}").as_bytes());
        Some(title)
    }
    pub fn level_notice(&self, weapon: i32, level: usize, lang: u32, native_name: &[u8]) -> Option<Vec<u8>> {
        self.weapon_level(weapon)?;
        if native_name.is_empty() { return None; }
        let prefix = match lang { 2 => "Amelioration de", 3 => "Verbessert:",
            4 => "Has mejorado", 5 => "Hai potenziato", _ => "Upgraded" };
        let mut message = format!("{prefix} ").into_bytes();
        message.extend_from_slice(native_name);
        message.extend_from_slice(format!(" - V{level}").as_bytes());
        Some(message)
    }
    fn award_nanotech(&mut self, amount:f32) {
        self.nanotech_pending_heal += self.nanotech.award(&self.world_settings,self.nanotech_native_max,amount);
    }
    pub fn award_extra_damage(&mut self, weapon:i32, damage:f32) {
        self.award_extra_xp(weapon,damage*self.reward_factor()*self.xp_per_damage*if self.real_hp {100.}else{1.});
    }
    fn award_extra_xp(&mut self,weapon:i32,amount:f32) {
        self.award_extra_xp_split(weapon,amount,amount);
    }
    fn award_extra_xp_split(&mut self,weapon:i32,amount:f32,nanotech_amount:f32) {
        let before=self.weapon_level(weapon).unwrap_or(1);
        // Taunter/Decoy attraction and offensive auras never level Nanotech.
        if !matches!(weapon,14|25) && self.extras.iter().any(|t|t.id==weapon&&t.enabled) {self.award_nanotech(nanotech_amount);}
        if let Some(track)=self.extras.iter_mut().find(|t|t.id==weapon&&t.enabled) {
            track.award(amount);
            if weapon == 8 && track.last_hit_xp > 0. {
                self.wrench_xp_feedback_serial = self.wrench_xp_feedback_serial.wrapping_add(1);
            }
            self.publish_max_ammo();
        }
        self.earned_upgrade(weapon,before);
    }
    /// An explicit weapon XP budget must not bypass the configured Nanotech rate.
    /// `native_loss` is confirmed HP loss with the encounter health scale restored.
    /// Keep reward-chain bonuses and the zero-XP switch common to both tracks.
    pub(crate) fn award_fixed_xp(&mut self,weapon:i32,amount:f32,native_loss:f32) {
        if self.xp_per_damage<=0. || !amount.is_finite() || amount<=0.
            || !native_loss.is_finite() || native_loss<=0. {return;}
        let bonus=self.reward_factor();
        let nanotech_amount=self.hp_units(native_loss)*self.xp_per_damage*bonus;
        let amount=amount*bonus;
        match weapon {
            10=>self.award_bomb_xp_split(amount,nanotech_amount),
            16=>self.award_pyro_xp_split(amount,nanotech_amount),
            _=>self.award_extra_xp_split(weapon,amount,nanotech_amount),
        }
    }
    /// Swallow/morph share the per-life key with other special rewards. Their
    /// fixed weapon budget still uses ordinary HP conversion for Nanotech.
    pub(crate) fn award_fixed_special_once(&mut self,weapon:i32,target:usize,amount:f32,native_loss:f32) {
        if !amount.is_finite() || amount<=0. || !native_loss.is_finite() || native_loss<=0.
            || !self.extras.iter().any(|t|t.id==weapon&&t.enabled) {return;}
        if self.award_once.insert((weapon,target)) {self.award_fixed_xp(weapon,amount,native_loss);}
    }
    pub fn drain_special(&mut self) {
        let messages = self.pending_special.0.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
        for (weapon, target, hp, fraction) in messages {
            self.award_special_once(weapon, target, hp, fraction);
        }
    }
    /// Read-only target searches post to this world's queue, never another slot.
    pub fn enqueue_special(&self, weapon: i32, target: usize, hp: f32, fraction: f32) {
        if self.extras.iter().any(|t| t.id == weapon && t.enabled) {
            if let Ok(mut q) = self.pending_special.0.lock() { q.push((weapon, target, hp, fraction)); }
        }
    }
    /// Boss attraction is capped independently of the ordinary enemy reward.
    /// Reuse the per-life award key so switching decoys or reapplying a lure
    /// cannot farm the same boss. A configured zero still disables the reward.
    pub fn enqueue_attraction(&self, weapon: i32, target: usize, class: i16, hp: f32, fraction: f32) {
        self.enqueue_special(weapon, target, hp, Self::attraction_fraction(weapon, class, fraction));
    }
    pub fn award_attraction_once(&mut self, weapon: i32, target: usize, class: i16, hp: f32, fraction: f32) {
        self.award_special_once(weapon, target, hp, Self::attraction_fraction(weapon, class, fraction));
    }
    fn attraction_fraction(weapon: i32, class: i16, fraction: f32) -> f32 {
        if matches!(weapon, 14 | 25) && world::boss(class) && fraction.is_finite() {
            fraction.min(0.1)
        } else { fraction }
    }
    pub fn award_special_once(&mut self, weapon: i32, moby: usize, native_hp: f32, fraction: f32) {
        if native_hp <= 0.0 || !native_hp.is_finite() || !fraction.is_finite() || fraction <= 0.0 { return; }
        if !self.extras.iter().any(|t| t.id == weapon && t.enabled) { return; }
        if self.award_once.insert((weapon, moby)) { self.award_extra_damage(weapon, native_hp * fraction); }
    }
    /// A newly created enemy in a reused slot is a new target for special XP.
    /// Matrix rebuilds, temporary stuns and animation changes never call this.
    pub fn forget_special_target(&mut self, moby: usize) {
        self.award_once.retain(|&(_, target)| target != moby);
        self.terminal_targets.remove(&moby);
        if let Ok(mut q) = self.pending_special.0.lock() { q.retain(|&(_, target, _, _)| target != moby); }
    }
    pub fn mark_terminal_target(&mut self, moby: usize) -> bool { self.terminal_targets.insert(moby) }
    pub fn target_is_terminal(&self, moby: usize) -> bool { self.terminal_targets.contains(&moby) }
    pub fn from_env() -> Self {
        CONFIG_ACTIVE.store(false, Ordering::Relaxed);
        controls::disable();
        for icon in &ICON_LEVELS { icon.store(0, Ordering::Relaxed); }
        BOMB_AMMO_CAP.store(0, Ordering::Relaxed);
        PYRO_AMMO_CAP.store(0, Ordering::Relaxed);
        PYRO_CORE.store(u32::MAX, Ordering::Relaxed);
        PYRO_OUTER.store(u32::MAX, Ordering::Relaxed);
        PYRO_BURN.store(u32::MAX, Ordering::Relaxed);
        for f in &GOLD_TEXTURE { f.store(-1, Ordering::Relaxed); }
        if let Ok(mut g) = level_values().write() { g.clear(); }
        #[cfg(all(feature = "weapon-evo-release", not(test)))]
        {
            let profile = Self::parse(RELEASE_PROFILE).expect("validated embedded Weapon Evolution profile");
            CONFIG_ACTIVE.store(true, Ordering::Relaxed);
            profile.publish_max_ammo();
            println!("weapon evolution: embedded fixed profile v0.1.6.32");
            return profile;
        }
        // Unit-test worlds retain their explicit native/configured fixtures.
        // The embedded profile itself is parsed and checked in the test below.
        #[cfg(any(not(feature = "weapon-evo-release"), test))]
        {
            let Ok(path) = std::env::var("RC_WEAPON_EVOLUTION_INI") else { return Self::default() };
            match std::fs::read_to_string(&path).and_then(|s| Self::parse(&s)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))) {
                Ok(mut s) => {
                    location_test::configure(&mut s);
                    gemlik_test::configure(&mut s);
                    health_test::configure(&mut s);
                    CONFIG_ACTIVE.store(true, Ordering::Relaxed);
                    s.publish_max_ammo();
                    println!("weapon evolution: config loaded from {path}");
                    s
                }
                Err(e) => { eprintln!("weapon evolution: disabled: {e}"); Self::default() }
            }
        }
    }
    pub fn parse(ini: &str) -> Result<Self, String> {
        let mut evo = Self::default();
        let mut progression_cap = None;
        let mut section = String::new();
        let mut bomb_seen = [false; 8];
        let mut pyro_seen = [false; 8];
        let mut r1_limit_seen=[[false;8];37];
        for (lineno, raw) in ini.lines().enumerate() {
            let line = raw.trim_start_matches('\u{feff}').split(';').next().unwrap_or("").trim();
            // Hex RGB values legitimately begin with '#'; only full lines are comments.
            if line.is_empty() || line.starts_with('#') { continue; }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len()-1].trim().to_ascii_lowercase();
                // Backwards compatibility with [levelN] and [weapon.bomb_glove.levelN].
                if let Some(n) = section.strip_prefix("weapon.bomb_glove.level")
                    .or_else(|| section.strip_prefix("level"))
                    .and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=8).contains(n)) { bomb_seen[n-1] = true; }
                if let Some(n) = section.strip_prefix("weapon.pyrocitor.level")
                    .and_then(|n| n.parse::<usize>().ok()).filter(|n| (1..=8).contains(n)) { pyro_seen[n-1] = true; }
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| format!("line {} missing '='", lineno+1))?;
            let key = key.trim().to_ascii_lowercase();
            let val = value.trim();
            if let Some((name, number)) = section.strip_prefix("weapon.").and_then(|s| s.rsplit_once(".level")) {
                let item = if name == "bomb_glove" { Some(10) } else if name == "pyrocitor" { Some(16) }
                    else { evo.extras.iter().find(|t| t.section == name).map(|t| t.id as usize) };
                if key == controls::R1_MAX_KEY {
                    let item=item.ok_or_else(||format!("unknown weapon in [{section}]"))?;
                    let grade=number.parse::<usize>().ok().filter(|n|(1..=8).contains(n))
                        .ok_or_else(||format!("invalid level [{section}]"))?;
                    let max=controls::supported_mode_max(item as i32);
                    let limit=val.parse::<u8>().ok().filter(|&n|n<=max)
                        .ok_or_else(||format!("{key} must be an integer 0..{max} in [{section}]"))?;
                    evo.r1_experimental_max[item][grade-1]=limit;
                    r1_limit_seen[item][grade-1]=true;
                    continue;
                }
                if key == "damage_hp" || (key == "damage_per_second" && matches!(item, Some(16 | 19)))
                    || (key == "chicken_damage_hp" && item == Some(21)) {
                    let item = item.ok_or_else(|| format!("unknown weapon in [{section}]"))?;
                    let level = number.parse::<usize>().ok().filter(|n| (1..=8).contains(n))
                        .ok_or_else(|| format!("invalid level [{section}]"))?;
                    if (item == 16 || item == 19) && key != "damage_per_second" {
                        return Err(format!("use damage_per_second for stream weapon [{section}]"));
                    }
                    let hp = parse_number(val, &key)?;
                    if !(0.0..=1_000_000.0).contains(&hp) { return Err(format!("{key} must be 0..1000000")); }
                    evo.damage_hp[item][level-1] = Some(hp);
                    continue;
                }
            }
            if section == "hud" {
                match key.as_str() {
                    "weapon_bar_alternative_position" => evo.hud.weapon_bar_alternative_position=parse_bool(val)?,
                    "weapon_bar_behind_hud" => evo.hud.weapon_bar_behind_hud=parse_bool(val)?,
                    "name_opacity_percent" => evo.hud.name_opacity_percent = parse_percent(val, &key)?,
                    "nanotech_xp_bar_opacity_percent" => evo.hud.nanotech_xp_bar_opacity_percent = parse_percent(val, &key)?,
                    // Compatibility only: this obsolete key has no field or effect.
                    // New editable INIs remove it; old embedded profiles still load.
                    "xp_bar_opacity_percent" => {},
                    "xp_bar_height_px" => {
                        let n: i32 = val.parse().map_err(|_| format!("invalid {key}: {val}"))?;
                        if !(2..=16).contains(&n) { return Err("xp_bar_height_px must be 2..16".into()); }
                        evo.hud.xp_bar_height_px = n;
                    }
                    "xp_bar_horizontal_inset_px" => {
                        let n: i32 = val.parse().map_err(|_| format!("invalid {key}: {val}"))?;
                        if !(0..=24).contains(&n) { return Err("xp_bar_horizontal_inset_px must be 0..24".into()); }
                        evo.hud.xp_bar_horizontal_inset_px = n;
                    }
                    "show_for_weapons_without_ammo" => evo.hud.show_for_weapons_without_ammo = parse_bool(val)?,
                    "level_up_popup" => evo.hud.level_up_popup = parse_bool(val)?,
                    "level_up_popup_duration_seconds" => {
                        let n = parse_number(val, &key)?;
                        if !(0.5..=15.0).contains(&n) { return Err("popup duration must be 0.5..15 seconds".into()); }
                        evo.hud.level_up_popup_duration_seconds = n;
                    }
                    "level_up_sound" => {
                        evo.hud.level_up_sound = match val.to_ascii_lowercase().as_str() {
                            "help_notice" | "true" => true,
                            "off" | "false" => false,
                            _ => return Err(format!("invalid level_up_sound: {val}")),
                        };
                    }
                    _ => return Err(format!("unknown {key} in [hud] (line {})", lineno+1)),
                }
                continue;
            }
            // Other weapons: common XP track only. Gameplay-specific settings are
            // deliberately not accepted until the corresponding weapon hook exists.
            if let Some(t) = evo.extras.iter_mut().find(|t| section == format!("weapon.{}", t.section)) {
                t.enabled = true;
                if key == "id" {
                    if val.parse::<i32>().ok() != Some(t.id) { return Err(format!("invalid id in [{section}]")); }
                } else if legacy_name_key(&key) {
                    // Old name overrides are ignored: display always comes from the disc.
                } else if let Some(i) = xp_transition(&key) {
                    t.xp_required[i] = parse_number(val, &key)?;
                } else {
                    return Err(format!("unknown {key} in [{section}] at line {}; not implemented", lineno+1));
                }
                continue;
            }
            if let Some((track_name, level)) = section.strip_prefix("weapon.").and_then(|v| v.rsplit_once(".level")) {
                if let Some(t) = evo.extras.iter_mut().find(|t| t.section == track_name) {
                    let n: usize = level.parse().map_err(|_| format!("invalid level in [{section}]"))?;
                    if !(1..=8).contains(&n) { return Err(format!("invalid level in [{section}]")); }
                    if !supported_extra_stat(t.id, &key) {
                        return Err(format!("unsupported {key} in [{section}] at line {}", lineno+1));
                    }
                    if t.id == 25 && key == "lure_range" && val.eq_ignore_ascii_case("default") {
                        // Each enemy has its own native detection range. Absence
                        // in the published registry preserves that per-call value.
                        t.params[n-1].remove(&key);
                        t.enabled = true;
                        continue;
                    }
                    if key.ends_with("_color") || matches!(key.as_str(), "explosion_color" | "trail_color" | "wave_color" | "projectile_color" | "impact_color" | "electric_core_color" | "electric_glow_color" | "fist_glow_color" | "beam_core_color" | "beam_glow_color" | "secondary_core_color" | "secondary_glow_color" | "arc_core_color" | "arc_glow_color" | "claw_core_color" | "claw_glow_color" | "spark_color" | "spark_core_color" | "light_color" | "weapon_glow_color" | "tube_color" | "pulse_color" | "muzzle_core_color" | "muzzle_glow_color" | "tip_color" | "tip_core_color") {
                        t.params[n-1].insert(key.clone(), parse_rgb(val)?.map_or(-1.0, |c| c as f32));
                        t.enabled = true;
                        continue;
                    }
                    let v = if matches!(key.as_str(), "show_hud" | "crate_breaker_enabled" | "moving_attacks_enabled" | "move_while_thrown" | "burn_on_hit" | "hud_quickselect_only" | "slam_explosion_enabled" | "grenade_launcher_enabled" | "continuous_missiles_enabled" | "shotgun_enabled" | "flame_mine_enabled" | "sand_mouse_enabled" | "crate_suction_enabled" | "suck_wooden_crates" | "suck_explosive_crates" | "suck_steel_crates" | "suck_ammo_crates" | "suck_nanotech_crates" | "dual_beam_enabled" | "gold" | "gold_texture" | "gold_effects" | "fire_in_air" | "air_dash_enabled" | "allow_large_enemies" | "invulnerable_while_guiding" | "gold_electric_fx" | "explode_small_chickens" | "explosion_enabled") {
                        if parse_bool(val)? { 1.0 } else { 0.0 }
                    } else { parse_number(val, &key)? };
                    validate_v45_extra(t.id, &key, v).map_err(|e| format!("{e} in [{section}]"))?;
                    if v < 0.0 || v > 9999.0 { return Err(format!("{key} out of range in [{section}]")); }
                    if (matches!(key.as_str(), "wave_speed_multiplier" | "wave_size_multiplier" | "wave_range_multiplier" | "explosion_visual_size_multiplier")
                        || (t.id == 11 && key == "explosion_radius_multiplier")) && !(0.1..=8.0).contains(&v) {
                        return Err(format!("{key} must be 0.1..8 in [{section}]"));
                    }
                    if t.id == 19 && matches!(key.as_str(), "ammo_interval_ticks" | "damage_interval_ticks") {
                        let max = if key == "damage_interval_ticks" { 600.0 } else { 9999.0 };
                        if !(1.0..=max).contains(&v) || v.fract() != 0.0 {
                            return Err(format!("{key} must be an integer 1..{max} in [{section}]"));
                        }
                    }
                    if key == "beam_size_percent" && !(1.0..=500.0).contains(&v) {
                        return Err(format!("beam_size_percent must be 1..500 in [{section}]"));
                    }
                    if t.id == 19 && key == "beam_range_multiplier" && !(0.1..=8.0).contains(&v) {
                        return Err(format!("beam_range_multiplier must be 0.1..8 in [{section}]"));
                    }
                    if key == "held_lunge_duration_percent" && !(100.0..=1000.0).contains(&v) {
                        return Err(format!("held_lunge_duration_percent must be 100..1000 in [{section}]"));
                    }
                    if key == "wave_interval_ticks" && (!(1.0..=600.0).contains(&v) || v.fract() != 0.0) {
                        return Err(format!("wave_interval_ticks must be an integer 1..600 in [{section}]"));
                    }
                    if key == "wave_alpha" && (v > 128.0 || v.fract() != 0.0) {
                        return Err(format!("wave_alpha must be an integer 0..128 in [{section}]"));
                    }
                    if key == "stun_efficiency_percent" && v > 100.0 {
                        return Err(format!("stun_efficiency_percent must be 0..100 in [{section}]"));
                    }
                    if key == "lure_cone_degrees" && !(1.0..=180.0).contains(&v) {
                        return Err(format!("lure_cone_degrees must be 1..180 in [{section}]"));
                    }
                    if key == "stun_duration_ticks" && v > 600.0 {
                        return Err(format!("stun_duration_ticks must be 0..600 in [{section}]"));
                    }
                    if key == "capacity" && v > 10.0 { return Err("Suck Cannon capacity above 10 needs slot-array changes".into()); }
                    if key == "drone_count" && (v > 10.0 || v.fract() != 0.0) { return Err("drone_count must be an integer 0..10".into()); }
                    t.params[n-1].insert(key.clone(), v);
                    t.enabled = true;
                    continue;
                }
            }
            if section.starts_with("weapon.") && section != "weapon.bomb_glove" && section != "weapon.pyrocitor"
                && !section.starts_with("weapon.bomb_glove.level")
                && !section.starts_with("weapon.pyrocitor.level") {
                return Err(format!("unsupported or unknown section [{section}] at line {}; gameplay hooks for this level are not active yet", lineno+1));
            }
            if section == "general" {
                if key==randomizer::KEY {evo.randomizer_mode=parse_bool(val)?;continue;}
                if key=="randomizer_bolt_multiplier" {evo.randomizer_bolt_multiplier=parse_bool(val)?;continue;}
                if key=="bolt_multiplier_max" {
                    let max=parse_number(val,&key)?;
                    if !(1.0..=100.0).contains(&max)||(max*10.-(max*10.).round()).abs()>0.001 {
                        return Err("bolt_multiplier_max must be 1..100 in steps of 0.1".into());
                    }
                    evo.bolt_multiplier_max=max;continue;
                }
                if key==controls::DPAD_KEY {evo.control_settings.dpad=match val.to_ascii_lowercase().as_str(){"on"=>true,"off"=>false,_=>parse_bool(val)?};continue;}
                if key==controls::R1_KEY {evo.control_settings.r1=match val.to_ascii_lowercase().as_str(){"on"=>true,"off"=>false,_=>parse_bool(val)?};continue;}
                if key == "hide_ryno" { evo.hide_ryno=parse_bool(val)?; continue; }
                if key=="refill_ammo_on_upgrade" {evo.refill_ammo_on_upgrade=match val.to_ascii_lowercase().as_str(){"on"=>true,"off"=>false,_=>parse_bool(val)?};}
                else if evo.golden_settings.parse_field(&key,val)? {}
                else if evo.world_settings.parse_field(&key,val)? {
                    if key == "nanotech_progression_max_health" {
                        progression_cap = Some(evo.world_settings.nanotech_max_health);
                    }
                }
                else if key == ammo_mode::KEY { evo.ammo_unlock_mode = parse_bool(val)?; }
                else if evo.ammo_mode_settings.parse_field(&key, val)? { /* validated global mode parameter */ }
                else if key == "wrench_gold_texture" { evo.wrench_gold_texture = parse_bool(val)?; }
                else if key == "xp_per_damage" { evo.xp_per_damage = parse_number(val, &key)?; }
                else if key == "damage_mode" { evo.real_hp = match val {
                    "real_hp" => true, "legacy" => false,
                    _ => return Err("damage_mode must be real_hp or legacy".into()),
                }; }
                continue;
            }
            if section == "weapon.bomb_glove" || section == "weapon.pyrocitor" {
                let is_bomb = section == "weapon.bomb_glove";
                if key == "id" {
                    if val != (if is_bomb { "10" } else { "16" }) { return Err(format!("invalid id in [{section}]")); }
                } else if legacy_name_key(&key) {
                    // Old name overrides are ignored: display always comes from the disc.
                } else if let Some(i) = xp_transition(&key) {
                    let amount = parse_number(val, &key)?;
                    if is_bomb { evo.xp_required[i] = amount; } else { evo.pyro.xp_required[i] = amount; }
                } else { return Err(format!("unknown {key} in [{section}]")); }
                continue;
            }
            let bomb_n = section.strip_prefix("weapon.bomb_glove.level")
                .or_else(|| section.strip_prefix("level"))
                .and_then(|x| x.parse::<usize>().ok()).filter(|n| (1..=8).contains(n));
            if let Some(n) = bomb_n {
                let stat = &mut evo.stats[n-1];
                match key.as_str() {
                    "xp_to_next" if n <= 7 => evo.xp_required[n-1] = parse_number(val, &key)?,
                    "xp_to_next" => {},
                    "cluster_bombs_enabled" => stat.cluster.enabled = parse_bool(val)?,
                    "cluster_bomb_count" => stat.cluster.count = parse_tick_interval(val, &key, 12)? as u8,
                    "cluster_bomb_size_percent" => stat.cluster.model_percent = parse_number(val, &key)?,
                    "cluster_explosion_size_percent" => stat.cluster.explosion_percent = parse_number(val, &key)?,
                    "cluster_damage_percent" => stat.cluster.damage_percent = parse_number(val, &key)?,
                    "cluster_launch_speed" => stat.cluster.launch_speed = parse_number(val, &key)?,
                    "cluster_jump_height" => stat.cluster.jump_height = parse_number(val, &key)?,
                    "cluster_jump_speed" => stat.cluster.jump_speed = Some(parse_number(val, &key)?),
                    "throw_range" => stat.throw_range = parse_number(val, &key)?,
                    "auto_lock_range" => stat.auto_lock_range = if val.eq_ignore_ascii_case("default") { None } else { Some(parse_number(val, &key)?) },
                    "trajectory_height" => stat.trajectory_height = parse_number(val, &key)?,
                    "explosion_color" => stat.explosion_color = parse_rgb(val)?,
                    "explosion_radius" => stat.explosion_radius = parse_number(val, &key)?,
                    "cooldown_seconds" => stat.cooldown_seconds = parse_number(val, &key)?,
                    "max_ammo" => stat.max_ammo = parse_ammo(val)?,
                    "experimental_max_ammo" => { let cap=parse_ammo(val)?; if !(1..=999).contains(&cap) { return Err("experimental_max_ammo must be 1..999".into()); } stat.experimental_max_ammo=Some(cap); },
                    "damage_multiplier" => stat.damage_multiplier = parse_number(val, &key)?,
                    "gold" => { let v = parse_bool(val)?; stat.gold = v; stat.gold_texture = v; },
                    "gold_effects" => stat.gold = parse_bool(val)?,
                    "gold_texture" => stat.gold_texture = parse_bool(val)?,
                    "name" => {}, // Legacy display name (only localized short name is used in HUD).
                    _ => return Err(format!("unknown {key} in [{section}]")),
                }
                continue;
            }
            let pyro_n = section.strip_prefix("weapon.pyrocitor.level")
                .and_then(|x| x.parse::<usize>().ok()).filter(|n| (1..=8).contains(n));
            if let Some(n) = pyro_n {
                let stat = &mut evo.pyro.stats[n-1];
                match key.as_str() {
                    "xp_to_next" if n <= 7 => evo.pyro.xp_required[n-1] = parse_number(val, &key)?,
                    "xp_to_next" => {},
                    "triple_flame_enabled" => stat.triple_flame_enabled = parse_bool(val)?,
                    "lava_stream_enabled" => stat.lava.enabled = parse_bool(val)?,
                    "lava_damage_bonus_percent" => stat.lava.damage_bonus_percent = parse_number(val, &key)?,
                    "lava_launch_speed" => stat.lava.speed = parse_number(val, &key)?,
                    "lava_launch_angle_degrees" => stat.lava.launch_angle_degrees = parse_number(val, &key)?,
                    "lava_gravity" => stat.lava.gravity = parse_number(val, &key)?,
                    "lava_max_range" => stat.lava.range = parse_number(val, &key)?,
                    "lava_size_percent" => stat.lava.size_percent = parse_number(val, &key)?,
                    "lava_core_color" => stat.lava.core_color = parse_rgb(val)?,
                    "lava_outer_color" => stat.lava.outer_color = parse_rgb(val)?,
                    "lava_impact_color" => stat.lava.impact_color = parse_rgb(val)?,
                    "lava_light_color" => stat.lava.light_color = parse_rgb(val)?,
                    "side_flame_angle_degrees" => stat.side_flame_angle_degrees = parse_number(val, &key)?,
                    "side_flame_left_angle_degrees" => stat.side_flame_left_angle_degrees = Some(parse_number(val, &key)?),
                    "side_flame_right_angle_degrees" => stat.side_flame_right_angle_degrees = Some(parse_number(val, &key)?),
                    "ammo_interval_ticks" => stat.ammo_interval_ticks = parse_tick_interval(val, &key, 9999)?,
                    "damage_interval_ticks" => stat.damage_interval_ticks = parse_tick_interval(val, &key, 600)?,
                    "flame_range" => stat.flame_range = parse_number(val, &key)?,
                    "flame_width" => stat.flame_width = parse_number(val, &key)?,
                    "damage_multiplier" => stat.damage_multiplier = parse_number(val, &key)?,
                    "flame_core_color" => stat.flame_core_color = parse_rgb(val)?,
                    "flame_outer_color" => stat.flame_outer_color = parse_rgb(val)?,
                    "light_color" => stat.light_color = parse_rgb(val)?,
                    "max_ammo" => stat.max_ammo = parse_ammo(val)?,
                    "experimental_max_ammo" => { let cap=parse_ammo(val)?; if !(1..=999).contains(&cap) { return Err("experimental_max_ammo must be 1..999".into()); } stat.experimental_max_ammo=Some(cap); },
                    "gold" => { let v = parse_bool(val)?; stat.gold = v; stat.gold_texture = v; },
                    "gold_effects" => stat.gold = parse_bool(val)?,
                    "gold_texture" => stat.gold_texture = parse_bool(val)?,
                    "burn_flame_color" => stat.burn_flame_color = parse_rgb(val)?,
                    _ => return Err(format!("unknown {key} in [{section}]")),
                }
            }
        }
        for i in 5..8 {
            if !bomb_seen[i] {evo.stats[i]=evo.stats[i-1];}
            if !pyro_seen[i] {evo.pyro.stats[i]=evo.pyro.stats[i-1];}
            for item in 0..37 {
                if !r1_limit_seen[item][i]{evo.r1_experimental_max[item][i]=evo.r1_experimental_max[item][i-1];}
                if evo.damage_hp[item][i].is_none(){evo.damage_hp[item][i]=evo.damage_hp[item][i-1];}
            }
            for track in &mut evo.extras {
                if track.params[i].is_empty(){track.params[i]=track.params[i-1].clone();}
            }
        }
        if bomb_seen[..5].iter().any(|&s| !s) { return Err("all Bomb Glove levels 1..5 required".into()); }
        if !(evo.xp_per_damage > 0.0 && evo.xp_per_damage <= 100.0) { return Err("xp_per_damage out of range".into()); }
        valid_xp(&evo.xp_required, "bomb_glove")?;
        for (i, s) in evo.stats.iter().enumerate() {
            if !s.cluster.valid() || !(0.1..=5.0).contains(&s.throw_range) || !(0.4..=3.0).contains(&s.trajectory_height) || !(0.1..=8.0).contains(&s.explosion_radius)
                || !(0.1..=10.0).contains(&s.cooldown_seconds) || !(1..=999).contains(&s.max_ammo)
                || !(0.1..=20.0).contains(&s.damage_multiplier)
                || s.auto_lock_range.is_some_and(|r| !r.is_finite() || !(0.0..=9999.0).contains(&r)) {
                return Err(format!("invalid Bomb Glove level {}", i+1));
            }
        }
        for t in &evo.extras {
            if t.enabled { valid_xp(&t.xp_required, t.section)?; }
        }
        evo.enabled = true;
        if pyro_seen.iter().any(|&s| s) {
            if pyro_seen[..5].iter().any(|&s| !s) { return Err("all Pyrocitor levels 1..5 required".into()); }
            valid_xp(&evo.pyro.xp_required, "pyrocitor")?;
            for (i, s) in evo.pyro.stats.iter().enumerate() {
                if !s.side_flame_left_angle_degrees.is_none_or(|v| (0.0..=80.0).contains(&v))
                    || !s.side_flame_right_angle_degrees.is_none_or(|v| (0.0..=80.0).contains(&v))
                    || !(1.0..=80.0).contains(&s.side_flame_angle_degrees) || !(0.1..=3.0).contains(&s.flame_range) || !(0.1..=3.0).contains(&s.flame_width)
                    || !(0.1..=20.0).contains(&s.damage_multiplier)
                    || !s.lava.valid() || !(1..=999).contains(&s.max_ammo) { return Err(format!("invalid Pyrocitor level {}", i+1)); }
            }
            evo.pyro.enabled = true;
        }
        if evo.real_hp {
            for i in 0..8 {
                if let Some(hp) = evo.damage_hp[10][i] { evo.stats[i].damage_multiplier = hp / 200.0; }
                if let Some(hp) = evo.damage_hp[16][i] {
                    evo.pyro.stats[i].damage_multiplier = hp / (100.0 * (1.0 + if evo.pyro.stats[i].gold {1.0} else {0.0}));
                }
                for t in &mut evo.extras {
                    if let Some(hp) = evo.damage_hp[t.id as usize][i] {
                        let gold = t.params[i].get("gold_effects").or_else(|| t.params[i].get("gold")).copied().unwrap_or(0.0) >= 0.5;
                        let (key, base) = match t.id {
                            8 => ("damage_multiplier", 1.0), 9 => ("shot_damage_multiplier", 2.0), 15 => ("damage_multiplier", 1.0),
                            11 | 17 | 18 | 20 | 23 => ("damage_multiplier", 3.0),
                            13 => ("damage_multiplier", 6.0), 19 => ("damage_multiplier", if gold {4.0} else {2.0}),
                            24 => ("damage_multiplier", 1.0), 25 => ("explosion_damage_multiplier", if gold {4.0} else {2.0}),
                            21 => ("chicken_damage_multiplier", 1.0), _ => continue,
                        };
                        t.params[i].insert(key.to_string(), hp / (100.0 * base));
                    }
                }
            }
        }
        if let Some(track)=evo.extras.iter().find(|t|t.id==23) {
            for (i,p) in track.params.iter().enumerate() {
                if p.get("continuous_missiles_enabled").copied().unwrap_or(0.0)!=0.0 && p.get("shotgun_enabled").copied().unwrap_or(0.0)!=0.0 {
                    return Err(format!("R.Y.N.O. grade {}: continuous missiles and shotgun are mutually exclusive",i+1));
                }
            }
        }
        if let Some(cap) = progression_cap { evo.world_settings.nanotech_max_health = cap; }
        evo.ammo_mode_settings.validate()?;
        if evo.randomizer_mode {evo.golden_settings.enabled=false;evo.ammo_unlock_mode=false;evo.hide_ryno=false;}
        Ok(evo)
    }
    /// Called by creature hit handlers only after their native damage resolution.
    pub fn award_damage(&mut self, effective_damage: f32, attack_class: u16) {
        self.last_hit_xp = 0.0;
        if !effective_damage.is_finite() || effective_damage <= 0.0 { return; }
        // v0.1.5 phase A: Blaster projectile (class 305) and nozzle (168).
        // Uses exactly the same effective-damage call site as the Bomb Glove.
        let weapon = match attack_class {
            305 | 168 => Some(15), // Blaster bullet/nozzle
            153 => Some(11), // Devastator missile
            172 => Some(13), // Visibomb missile
            74 => Some(17), // Mine
            180 => Some(18), // Walloper
            177 => Some(19), // Tesla Claw
            186 => Some(20), // Glove of Doom robot
            457 => Some(23), // RYNO missile
            479 => Some(24), // Drone shield
            203 | 1900 => Some(25), // Decoy explosion
            0x351 => Some(9), // Suck Cannon expelled creature
            _ => None,
        };
        if let Some(id) = weapon { self.award_extra_damage(id, effective_damage); }
        if !self.enabled || attack_class != 121 { return; }
        let amount = self.hp_units(effective_damage.max(0.0)) * self.xp_per_damage * self.reward_factor();
        self.award_bomb_xp(amount);
    }
    fn award_bomb_xp(&mut self,amount:f32) {
        self.award_bomb_xp_split(amount,amount);
    }
    fn award_bomb_xp_split(&mut self,amount:f32,nanotech_amount:f32) {
        self.last_hit_xp=0.;
        if !self.enabled {return;}
        let before=self.level;
        self.award_nanotech(nanotech_amount);
        if self.level >= self.level_cap { return; }
        self.last_hit_xp = amount;
        self.xp += amount;
        while self.level < self.level_cap && self.xp >= self.required() {
            self.xp -= self.required();
            self.level += 1;
            println!("weapon evolution: BOMB GLOVE -> V{} (remaining XP {:.1})", self.level, self.xp);
        }
        if self.level >= self.level_cap { self.xp = 0.0; }
        self.earned_upgrade(10,before);
        self.publish_max_ammo();
    }
    /// The Pyrocitor's hit templates have type 5; kept flame has class 176,
    /// direct nozzle hits class 0. Caller additionally checks attacker is Ratchet.
    pub fn award_pyro_damage(&mut self, effective_damage: f32, attack_type: u8, attack_class: u16, from_ratchet: bool) {
        if !from_ratchet || attack_type != 5 || !matches!(attack_class, 0 | 176) { return; }
        let amount = self.hp_units(effective_damage.max(0.0)) * self.xp_per_damage * self.reward_factor();
        self.award_pyro_xp(amount);
    }
    fn award_pyro_xp(&mut self,amount:f32) {
        self.award_pyro_xp_split(amount,amount);
    }
    fn award_pyro_xp_split(&mut self,amount:f32,nanotech_amount:f32) {
        if self.pyro.enabled { self.award_nanotech(nanotech_amount); }
        let before=self.pyro.level;
        self.pyro.award(amount);
        self.earned_upgrade(16,before);
        self.publish_max_ammo();
    }
}

#[cfg(test)]
mod v01629_special_tests {
    use super::*;
    fn enabled() -> Evolution {
        let mut e = Evolution::default();
        for t in &mut e.extras { t.enabled = true; t.xp_required = [1000.0; 7]; }
        e
    }
    fn xp(e: &Evolution, id: i32) -> f32 { e.extras.iter().find(|t| t.id == id).unwrap().xp }

    #[test]
    fn fractions_are_per_weapon_and_duplicate_events_do_not_farm_one_enemy() {
        let mut e = enabled();
        for (weapon, fraction) in [(9,1.0), (14,0.5), (21,1.0), (25,0.25)] {
            for _ in 0..4 { e.enqueue_special(weapon, 42, 0.75, fraction); }
        }
        e.drain_special();
        assert_eq!((xp(&e,9),xp(&e,14),xp(&e,21),xp(&e,25)), (0.75,0.375,0.75,0.1875));
        e.drain_special(); assert_eq!(xp(&e,25),0.1875);
        // A disabled/zero-fraction action must not consume the once-only key.
        e.award_special_once(25, 50, 1.0, 0.0); e.award_special_once(25,50,1.0,0.25);
        assert_eq!(xp(&e,25),0.4375);
    }

    #[test]
    fn queues_are_isolated_between_worlds_and_clones_remain_send_sync() {
        fn send_sync<T: Send + Sync>() {} send_sync::<Evolution>();
        let mut a = enabled(); let mut b = enabled();
        a.enqueue_special(25, 42, 2.0, 0.25);
        b.drain_special(); assert_eq!(xp(&b,25),0.0);
        let mut snapshot = a.clone(); snapshot.drain_special(); assert_eq!(xp(&snapshot,25),0.5);
        a.drain_special(); assert_eq!(xp(&a,25),0.5, "clone has its own queue");
    }

    #[test]
    fn new_game_discards_pending_attractions_and_resets_per_target_state() {
        let mut e = enabled();
        e.enqueue_special(25,42,2.0,0.25); e.mark_terminal_target(42);
        e.restore_progress(&save::Progress::default()); e.drain_special();
        assert_eq!(xp(&e,25),0.0); assert!(!e.target_is_terminal(42));
        e.enqueue_special(25,42,2.0,0.25); e.drain_special();
        assert_eq!(xp(&e,25),0.5);
    }

    #[test]
    fn recreated_target_forgets_old_queue_but_preserves_other_targets() {
        let mut e = enabled();
        e.enqueue_special(25,42,2.0,0.25); e.enqueue_special(25,43,2.0,0.25);
        e.forget_special_target(42); e.drain_special(); assert_eq!(xp(&e,25),0.5);
        e.award_special_once(25,42,2.0,0.25); assert_eq!(xp(&e,25),1.0);
    }
}

#[cfg(all(test, feature = "weapon-evo-release"))]
mod fixed_release_tests {
    use super::*;
    #[test]
    fn embedded_profile_has_all_tracks_explicit_decoy_ammo_and_empty_progress() {
        let e = Evolution::parse(RELEASE_PROFILE).expect("approved fixed profile must parse");
        assert!(e.enabled && e.pyro.enabled);
        assert_eq!(e.extras.iter().filter(|t| t.enabled).count(), 13);
        let decoy = e.extras.iter().find(|t| t.id == 25).unwrap();
        let ammo: Vec<f32> = decoy.params.iter().map(|p| p["max_ammo"]).collect();
        // The author's build input may change ammo; require all five explicit
        // grade values without locking tests to a previous balance revision.
        assert_eq!(ammo.len(), 5);
        assert!(ammo.iter().all(|&a| a.is_finite() && (1.0..=999.0).contains(&a)));
        assert_eq!(e.progress(), save::Progress::default());
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;
    #[test]
    fn hex_rgb_and_native_alpha() {
        assert_eq!(parse_rgb("#000000").unwrap(), Some(0));
        assert_eq!(parse_rgb("#FFFFFF").unwrap(), Some(0x00ff_ffff));
        assert_eq!(parse_rgb("#278BFF").unwrap(), Some(0x00ff_8b27));
        assert!(parse_rgb("#12XXX0").is_err());
        let output = tint_rgba(0x7f80_4060, parse_rgb("#FF0000").unwrap());
        assert_eq!(output & 0xff00_0000, 0x7f00_0000);
        assert_eq!(output & 0x00ff_ff00, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_weapon_levels_and_xp() {
        let mut e = Evolution::default(); e.enabled = true; e.pyro.enabled = true;
        e.award_damage(6.0, 121);
        assert_eq!(e.level, 2);
        assert_eq!(e.pyro.level, 1);
        e.award_pyro_damage(10.0, 5, 176, true);
        assert_eq!(e.pyro.level, 2);
        assert_eq!(e.level, 2);
    }
    #[test]
    fn no_pyro_xp_without_ratchet_or_real_hit_type() {
        let mut e = Evolution::default(); e.pyro.enabled = true;
        e.award_pyro_damage(100.0, 5, 176, false);
        e.award_pyro_damage(100.0, 2, 176, true);
        assert_eq!(e.pyro.level, 1);
    }
}

#[cfg(test)]
mod common_track_tests {
    use super::*;
    #[test]
    fn all_ids_unique_and_all_tracks_present() {
        let e = Evolution::default();
        assert_eq!(e.extras.len(), 14);
        for (i, id) in WEAPON_IDS.iter().enumerate() {
            assert!(WEAPON_IDS[..i].iter().all(|n| n != id));
        }
    }
    #[test]
    fn blaster_damage_levels_only_blaster() {
        let mut e = Evolution::default();
        e.extras.iter_mut().find(|t| t.id == 15).unwrap().enabled = true;
        e.enabled = true;
        e.award_damage(10.0, 305);
        assert_eq!(e.weapon_level(15), Some(2));
        assert_eq!(e.level, 1);
        assert_eq!(e.weapon_fraction(15), Some(0.0));
        assert_eq!(e.equipped_title(15, b"Native name").unwrap(), b"Native name V2");
    }
    #[test]
    fn opacity_bounds_and_v015_ini() {
        assert_eq!(HudOptions::alpha(100), 128);
        assert_eq!(HudOptions::alpha(50), 64);
        assert_eq!(HudOptions::alpha(0), 0);
        assert!(parse_percent("101", "opacity").is_err());
    }

    #[test]
    fn retired_xp_opacity_has_no_field_or_effect_and_old_profiles_still_load() {
        let mut profile = String::new();
        for n in 1..=5 { profile.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        let e = Evolution::parse(&(profile.clone() + "[hud]\nname_opacity_percent = 65\n")).unwrap();
        let old = Evolution::parse(&(profile + "[hud]\nname_opacity_percent = 65\nxp_bar_opacity_percent = 0\n")).unwrap();
        assert_eq!(e.hud.name_opacity_percent, old.hud.name_opacity_percent);
        assert_eq!(e.hud.xp_bar_height_px, old.hud.xp_bar_height_px);
    }
}

#[cfg(test)]
mod v01613_tests {
    use super::*;
    fn profile(extra: &str) -> String {
        let mut s = String::new();
        for n in 1..=5 { s.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        s.push_str(extra);
        s
    }
    #[test]
    fn localized_disc_bytes_replace_legacy_names() {
        let e = Evolution::parse(&profile("[weapon.bomb_glove]\nname_it = OLD\nname_en = WRONG\n")).unwrap();
        // The game font is byte encoded: a non-UTF8 accent must remain intact.
        let name = b"Arme \xe9";
        assert_eq!(e.equipped_title(10, name).unwrap(), b"Arme \xe9 V1");
        assert_eq!(e.level_notice(10, 2, 5, name).unwrap(), b"Hai potenziato Arme \xe9 - V2");
        assert!(e.equipped_title(8, name).is_none());
    }
    #[test]
    fn colours_round_trip_all_24_bits_and_default() {
        for color in ["#000000", "#FFFFFF", "#123456", "default"] {
            let e = Evolution::parse(&profile(&format!("[weapon.devastator.level1]\nexplosion_color = {color}\ntrail_color = {color}\n[weapon.taunter.level1]\nwave_color = {color}\n"))).unwrap();
            let want = parse_rgb(color).unwrap().map_or(-1.0, |c| c as f32);
            for (id, key) in [(11, "explosion_color"), (11, "trail_color"), (14, "wave_color")] {
                let t = e.extras.iter().find(|t| t.id == id).unwrap();
                assert_eq!(t.params[0][key], want);
                if want >= 0.0 { assert_eq!(want as u32, parse_rgb(color).unwrap().unwrap()); }
            }
        }
    }
    #[test]
    fn each_grade_keeps_its_own_air_setting() {
        let e = Evolution::parse(&profile("[weapon.morph_ray.level1]\nfire_in_air = false\n[weapon.morph_ray.level5]\nfire_in_air = true\n")).unwrap();
        let t = e.extras.iter().find(|t| t.id == 21).unwrap();
        assert_eq!(t.params[0]["fire_in_air"], 0.0);
        assert_eq!(t.params[4]["fire_in_air"], 1.0);
    }
    #[test]
    fn bad_effect_parameters_are_rejected() {
        for (section, value) in [
            ("devastator", "explosion_color = #GG0000"),
            ("devastator", "explosion_radius_multiplier = 0"),
            ("devastator", "explosion_visual_size_multiplier = 9"),
            ("taunter", "wave_interval_ticks = 0"),
            ("taunter", "wave_interval_ticks = 2.5"),
            ("taunter", "wave_speed_multiplier = NaN"),
            ("taunter", "wave_alpha = 129"),
            ("morph_ray", "fire_in_air = maybe"),
        ] {
            assert!(Evolution::parse(&profile(&format!("[weapon.{section}.level1]\n{value}\n"))).is_err(), "{section}: {value}");
        }
    }
}

#[cfg(test)]
mod v01632_tests {
    use super::*;

    fn profile(extra: &str) -> String {
        let mut text = String::new();
        for n in 1..=5 { text.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        text.push_str(extra);
        text
    }

    #[test]
    fn bomb_auto_lock_range_is_independent_per_grade_and_accepts_old_profiles() {
        let text = "[weapon.bomb_glove.level1]\nauto_lock_range = default\n\
            [weapon.bomb_glove.level2]\nauto_lock_range = 0\n\
            [weapon.bomb_glove.level3]\nauto_lock_range = 3\n\
            [weapon.bomb_glove.level4]\nauto_lock_range = 7.5\n\
            [weapon.bomb_glove.level5]\nauto_lock_range = 9999\n";
        let evo = Evolution::parse(&profile(text)).unwrap();
        assert_eq!(evo.stats[..5].iter().map(|s| s.auto_lock_range).collect::<Vec<_>>(), [None, Some(0.0), Some(3.0), Some(7.5), Some(9999.0)]);
        let legacy = Evolution::parse(&profile("")).unwrap();
        assert!(legacy.stats.iter().all(|s| s.auto_lock_range.is_none()));
        assert_eq!(legacy.stats[0].auto_lock_distance(), 8.5);
    }

    #[test]
    fn bomb_auto_lock_range_rejects_invalid_values() {
        for value in ["-1", "10000", "NaN", "inf", "red"] {
            let text = format!("[weapon.bomb_glove.level1]\nauto_lock_range = {value}\n");
            assert!(Evolution::parse(&profile(&text)).is_err(), "accepted {value}");
        }
    }

}

#[cfg(test)]
mod blaster_color_tests {
    use super::*;
    fn profile(extra: &str) -> String {
        let mut text = String::new();
        for n in 1..=5 { text.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        text.push_str(extra);
        text
    }
    #[test]
    fn blaster_colors_are_independent_for_all_five_grades() {
        let values = ["#000000", "#FF0000", "8CC3EA", "#FFFFFF", "default"];
        let mut text = String::new();
        for (n, color) in values.iter().enumerate() {
            text.push_str(&format!("[weapon.blaster.level{}]\nprojectile_color = {}\ntrail_color = #00FF00\nimpact_color = #0000FF\n", n + 1, color));
        }
        let evo = Evolution::parse(&profile(&text)).unwrap();
        let track = evo.extras.iter().find(|t| t.id == 15).unwrap();
        for (n, color) in values.iter().enumerate() {
            assert_eq!(track.params[n]["projectile_color"], parse_rgb(color).unwrap().map_or(-1.0, |c| c as f32));
            assert_eq!(track.params[n]["trail_color"], 0x00ff00 as f32);
            assert_eq!(track.params[n]["impact_color"], 0xff0000 as f32);
        }
    }
    #[test]
    fn blaster_rejects_bad_colors_without_accepting_them_for_other_weapons() {
        for key in ["projectile_color", "trail_color", "impact_color"] {
            for value in ["#GG0000", "#12345", "16777216", "NaN"] {
                let text = format!("[weapon.blaster.level1]\n{key} = {value}\n");
                assert!(Evolution::parse(&profile(&text)).is_err(), "{key} = {value}");
            }
        }
        assert!(Evolution::parse(&profile("[weapon.taunter.level1]\nprojectile_color = #FF0000\n")).is_err());
        let evo = Evolution::parse(&profile("[weapon.blaster.level1]\ndamage_multiplier = 1\n")).unwrap();
        let track = evo.extras.iter().find(|t| t.id == 15).unwrap();
        assert!(!track.params[0].contains_key("projectile_color"), "old INIs remain valid");
    }
}

#[cfg(test)]
mod gold_walloper_tests {
    use super::*;
    fn profile(extra: &str) -> String {
        let mut text = String::new();
        for n in 1..=5 { text.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        text.push_str(extra); text
    }
    #[test]
    fn devastator_gold_behavior_and_texture_are_independent_per_grade() {
        let mut text = String::new();
        for (n, (effects, texture)) in [(false, false), (false, true), (true, false), (true, true), (false, true)].into_iter().enumerate() {
            text.push_str(&format!("[weapon.devastator.level{}]\ngold_effects = {effects}\ngold_texture = {texture}\n", n + 1));
        }
        let e = Evolution::parse(&profile(&text)).unwrap();
        let track = e.extras.iter().find(|t| t.id == 11).unwrap();
        for (n, (effects, texture)) in [(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].into_iter().enumerate() {
            assert_eq!(track.params[n]["gold_effects"], effects);
            assert_eq!(track.params[n]["gold_texture"], texture);
        }
    }
    #[test]
    fn walloper_color_keys_accept_black_white_rgb_and_default_per_grade() {
        let values = ["000000", "FFFFFF", "8CC3EA", "#FF0000", "default"];
        let keys = ["electric_core_color", "electric_glow_color", "fist_glow_color", "impact_color"];
        let mut text = String::new();
        for (n, color) in values.iter().enumerate() {
            text.push_str(&format!("[weapon.walloper.level{}]\n", n + 1));
            for key in keys { text.push_str(&format!("{key} = {color}\n")); }
        }
        let e = Evolution::parse(&profile(&text)).unwrap();
        let track = e.extras.iter().find(|t| t.id == 18).unwrap();
        for (n, color) in values.iter().enumerate() {
            for key in keys { assert_eq!(track.params[n][key], parse_rgb(color).unwrap().map_or(-1.0, |c| c as f32)); }
        }
        for key in keys {
            assert!(Evolution::parse(&profile(&format!("[weapon.walloper.level1]\n{key} = #GG0000\n"))).is_err());
        }
        assert!(Evolution::parse(&profile("[weapon.blaster.level1]\nelectric_core_color = FF0000\n")).is_err());
    }
    #[test]
    fn all_tesla_color_parameters_are_per_grade_and_validated() {
        let keys = ["beam_core_color", "beam_glow_color", "secondary_core_color", "secondary_glow_color",
            "arc_core_color", "arc_glow_color", "claw_core_color", "claw_glow_color", "spark_color",
            "spark_core_color", "impact_color", "light_color", "weapon_glow_color"];
        let values = ["default", "000000", "FFFFFF", "8CC3EA", "#FF0000"];
        let mut text = String::new();
        for (n, value) in values.iter().enumerate() {
            text.push_str(&format!("[weapon.tesla_claw.level{}]\n", n + 1));
            for key in keys { text.push_str(&format!("{key} = {value}\n")); }
        }
        let e = Evolution::parse(&profile(&text)).unwrap();
        let t = e.extras.iter().find(|t| t.id == 19).unwrap();
        for (n, value) in values.iter().enumerate() {
            for key in keys { assert_eq!(t.params[n][key], parse_rgb(value).unwrap().map_or(-1.0, |c| c as f32)); }
        }
        for key in keys {
            assert!(Evolution::parse(&profile(&format!("[weapon.tesla_claw.level1]\n{key} = #12ZZ00\n"))).is_err());
        }
    }
}

#[cfg(test)]
mod morph_color_tests {
    use super::*;
    const KEYS: [&str; 13] = ["tube_color", "beam_core_color", "beam_glow_color", "secondary_core_color",
        "secondary_glow_color", "pulse_color", "muzzle_core_color", "muzzle_glow_color", "tip_color",
        "tip_core_color", "spark_color", "spark_core_color", "light_color"];
    fn profile(extra: &str) -> String {
        let mut text = String::new();
        for n in 1..=5 { text.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        text.push_str(extra); text
    }
    #[test]
    fn morph_colors_keep_all_24_bits_independently_per_grade() {
        let mut text = String::new();
        let values = ["000000", "FFFFFF", "8CC3EA", "#FF0000", "default"];
        for (i, color) in values.iter().enumerate() {
            text.push_str(&format!("[weapon.morph_ray.level{}]\n", i + 1));
            for key in KEYS { text.push_str(&format!("{key} = {color}\n")); }
        }
        let e = Evolution::parse(&profile(&text)).unwrap();
        let t = e.extras.iter().find(|t| t.id == 21).unwrap();
        for (i, color) in values.iter().enumerate() {
            let expected = parse_rgb(color).unwrap().map_or(-1.0, |v| v as f32);
            for key in KEYS { assert_eq!(t.params[i][key], expected, "{key}, grade {}", i + 1); }
        }
    }
    #[test]
    fn morph_gold_behavior_and_texture_do_not_enable_each_other() {
        let choices = [(false, false), (true, false), (false, true), (true, true), (false, true)];
        let mut text = String::new();
        for (i, (effects, texture)) in choices.iter().enumerate() {
            text.push_str(&format!("[weapon.morph_ray.level{}]\ngold_effects = {effects}\ngold_texture = {texture}\n", i + 1));
        }
        let e = Evolution::parse(&profile(&text)).unwrap();
        let t = e.extras.iter().find(|t| t.id == 21).unwrap();
        for (i, (effects, texture)) in choices.into_iter().enumerate() {
            assert_eq!(t.params[i]["gold_effects"], if effects { 1.0 } else { 0.0 });
            assert_eq!(t.params[i]["gold_texture"], if texture { 1.0 } else { 0.0 });
        }
    }
    #[test]
    fn morph_rejects_invalid_colors_and_keeps_older_profiles_valid() {
        for key in KEYS {
            for value in ["12345", "CCCCCCCC", "GG2200", "16777216", "NaN"] {
                assert!(Evolution::parse(&profile(&format!("[weapon.morph_ray.level1]\n{key} = {value}\n"))).is_err());
            }
        }
        let e = Evolution::parse(&profile("[weapon.morph_ray.level1]\nfire_in_air = true\n")).unwrap();
        let t = e.extras.iter().find(|t| t.id == 21).unwrap();
        assert!(!t.params[0].contains_key("tube_color"));
        assert!(Evolution::parse(&profile("[weapon.blaster.level1]\ntube_color = FF0000\n")).is_err());
    }
}

#[cfg(test)]
mod decoy_range_tests {
    use super::*;
    fn profile(extra: &str) -> String {
        let mut text = String::new();
        for n in 1..=5 { text.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        text.push_str(extra); text
    }
    #[test]
    fn decoy_each_grade_has_its_own_radius_or_native_default() {
        let text = "[weapon.decoy_glove.level1]\nlure_range = default\n\
            [weapon.decoy_glove.level2]\nlure_range = 0\n\
            [weapon.decoy_glove.level3]\nlure_range = 12.5\n\
            [weapon.decoy_glove.level4]\nlure_range = 30\n\
            [weapon.decoy_glove.level5]\nlure_range = 100\n";
        let e = Evolution::parse(&profile(text)).unwrap();
        let t = e.extras.iter().find(|t| t.id == 25).unwrap();
        assert!(!t.params[0].contains_key("lure_range"));
        for (i, value) in [0.0, 12.5, 30.0, 100.0].into_iter().enumerate() {
            assert_eq!(t.params[i+1]["lure_range"], value);
        }
        // Last occurrence wins, including a return to the per-enemy default.
        let e = Evolution::parse(&profile("[weapon.decoy_glove.level1]\nlure_range = 30\nlure_range = DEFAULT\n")).unwrap();
        let t = e.extras.iter().find(|t| t.id == 25).unwrap();
        assert!(!t.params[0].contains_key("lure_range"));
    }
    #[test]
    fn decoy_rejects_invalid_radii_and_keeps_old_profiles_valid() {
        for value in ["-1", "10000", "NaN", "inf", "purple"] {
            assert!(Evolution::parse(&profile(&format!("[weapon.decoy_glove.level1]\nlure_range = {value}\n"))).is_err());
        }
        assert!(Evolution::parse(&profile("[weapon.decoy_glove.level1]\nlure_range = 9999\n")).is_ok());
        assert!(Evolution::parse(&profile("[weapon.decoy_glove.level1]\ndecoy_health_multiplier = 1\n")).is_ok());
        assert!(Evolution::parse(&profile("[weapon.taunter.level1]\nlure_range = default\n")).is_err());
    }
}


#[cfg(test)]
mod v37_real_hp_tests {
    use super::*;
    fn profile() -> String {
        let mut s="[general]\ndamage_mode = real_hp\nxp_per_damage = 0.1\n".to_string();
        for n in 1..=5 {
            s.push_str(&format!("[weapon.bomb_glove.level{n}]\ndamage_hp = 250\n"));
            s.push_str(&format!("[weapon.pyrocitor.level{n}]\ndamage_per_second = 130\n"));
            s.push_str(&format!("[weapon.blaster.level{n}]\ndamage_hp = 75\n"));
            s.push_str(&format!("[weapon.tesla_claw.level{n}]\ndamage_per_second = 800\ngold_effects = true\n"));
        }
        s
    }
    #[test]
    fn parses_absolute_hp_and_reconstructs_only_the_native_object_multiplier() {
        let e=Evolution::parse(&profile()).unwrap();
        assert!(e.real_hp_enabled()); assert_eq!(e.current().damage_multiplier,1.25);
        assert_eq!(e.absolute_values()[&15],75.0); assert_eq!(e.absolute_values()[&16],130.0);
        let tesla=e.extras.iter().find(|t|t.id==19).unwrap();
        assert_eq!(tesla.params[0]["damage_multiplier"],2.0);
    }
    #[test]
    fn absolute_profile_accepts_a_utf8_bom_without_disabling_evolution() {
        assert!(Evolution::parse(&(String::from("\u{feff}") + &profile())).unwrap().real_hp_enabled());
    }
    #[test]
    fn absolute_hp_rejects_invalid_values_and_bad_stream_units() {
        for change in ["damage_hp = NaN","damage_hp = -1","damage_hp = 1000001"] {
            assert!(Evolution::parse(&profile().replacen("damage_hp = 250",change,1)).is_err());
        }
        assert!(Evolution::parse(&profile().replace("damage_per_second = 130","damage_hp = 130")).is_err());
    }
    #[test]
    fn stream_intervals_keep_each_grade_and_old_profiles_keep_defaults() {
        let mut text = profile();
        for n in 1..=5 {
            for weapon in ["pyrocitor", "tesla_claw"] {
                let header = format!("[weapon.{weapon}.level{n}]\n");
                text = text.replace(&header, &format!("{header}ammo_interval_ticks = {}\ndamage_interval_ticks = {}\n", n * 3, n * 2));
            }
        }
        let mut e = Evolution::parse(&text).unwrap();
        for n in 1..=5 {
            e.pyro.level = n;
            e.extras.iter_mut().find(|t| t.id == 19).unwrap().level = n;
            assert_eq!(e.pyro.current().ammo_interval_ticks, (n * 3) as i32);
            assert_eq!(e.stream_intervals(), [(n * 2) as u32; 2]);
            assert_eq!(e.extras.iter().find(|t| t.id == 19).unwrap().params[n-1]["ammo_interval_ticks"], (n * 3) as f32);
        }
        let old = Evolution::parse(&profile()).unwrap();
        assert_eq!(old.stream_intervals(), [1, 1]); assert_eq!(old.pyro.current().ammo_interval_ticks, 10);
    }
    #[test]
    fn stream_intervals_reject_zero_fractional_nonfinite_and_oversized_values() {
        for weapon in ["pyrocitor", "tesla_claw"] {
            for (key, invalid) in [("ammo_interval_ticks", "10000"), ("damage_interval_ticks", "601")] {
                for value in ["0", "-1", "1.5", "NaN", "inf", invalid] {
                    let header = format!("[weapon.{weapon}.level1]\n");
                    assert!(Evolution::parse(&profile().replace(&header, &format!("{header}{key} = {value}\n"))).is_err());
                }
            }
        }
    }
}

#[cfg(test)]
mod v43_tests {
    use super::*;

    fn profile(extra: &str) -> String {
        let mut s=String::from("[general]\ndamage_mode = real_hp\n");
        for n in 1..=5 { s.push_str(&format!("[weapon.bomb_glove.level{n}]\n")); }
        s+extra
    }
    #[test]
    fn experimental_settings_are_optional_and_independent_for_every_grade() {
        let base=Evolution::parse(&profile("")).unwrap();
        assert!(!base.stats[0].cluster.enabled);assert_eq!(base.stats[0].cluster.model_percent,40.0);
        let mut text=String::new();
        for n in 1..=5 { text+=&format!("[weapon.bomb_glove.level{n}]\ncluster_bombs_enabled = true\ncluster_bomb_count = {n}\n[weapon.pyrocitor.level{n}]\ntriple_flame_enabled = true\nside_flame_angle_degrees = {}\n[weapon.walloper.level{n}]\nheld_lunge_duration_percent = {}\n",n*10,n*100); }
        let e=Evolution::parse(&profile(&text)).unwrap();
        for i in 0..5 {
            assert_eq!(e.stats[i].cluster.count,i as u8+1);
            assert!(e.pyro.stats[i].triple_flame_enabled);
            assert_eq!(e.pyro.stats[i].side_flame_angle_degrees,(i+1) as f32*10.0);
            assert_eq!(e.extras.iter().find(|t|t.id==18).unwrap().params[i]["held_lunge_duration_percent"],(i+1) as f32*100.0);
        }
    }
    #[test]
    fn invalid_experimental_geometry_and_duration_are_rejected() {
        for (weapon,field,value) in [("bomb_glove","cluster_bomb_count","13"),("bomb_glove","cluster_bomb_count","2.5"),
            ("bomb_glove","cluster_jump_height","NaN"),("bomb_glove","cluster_damage_percent","0"),
            ("bomb_glove","cluster_bomb_size_percent","101"),("pyrocitor","side_flame_angle_degrees","90"),
            ("walloper","held_lunge_duration_percent","99"),("walloper","held_lunge_duration_percent","1001")] {
            assert!(Evolution::parse(&profile(&format!("[weapon.{weapon}.level1]\n{field} = {value}\n"))).is_err(),"{weapon}: {field}");
        }
    }

}

fn validate_v45_extra(item:i32, key: &str, v: f32) -> Result<(), String> {
    if matches!(key, "grenade_model"|"flame_mine_model") && (v.fract()!=0.0 || !(0.0..=2.0).contains(&v)) {return Err("projectile model must be 0, 1 or 2".into());}
    if item==8 && matches!(key,"hit_radius_multiplier"|"knockback_multiplier") {
        let low=if key=="hit_radius_multiplier" {0.1} else {0.0};
        if !(low..=10.0).contains(&v) {return Err(format!("{key} must be {low}..10"));}
    }
    if key == "experimental_max_ammo" && (v.fract()!=0.0 || !(0.0..=if item==9 {10.0} else {9999.0}).contains(&v)) {
        return Err("experimental_max_ammo must be an integer within this weapon's capacity limit".into());
    }
    if key=="homing_strength_multiplier" && item!=23 { return Ok(()); }
    let bounds = match key {
        "homing_strength_multiplier" => Some((0.0, 8.0)),
        "missile_max_range" => Some((0.0, 500.0)),
        "explosion_damage_radius" => Some((0.0, 100.0)),
        "explosion_visual_size_multiplier" => Some((0.1, 8.0)),
        "max_active_mines" | "flame_mine_max_active_mines" => Some((1.0, 32.0)),
        "mine_lifetime_seconds" | "flame_mine_lifetime_seconds" => Some((0.5, 600.0)),
        "stun_launch_distance" => Some((0.0, 50.0)),
        "shotgun_damage_hp" => Some((0.0, 9999.0)),
        "shotgun_speed" => Some((1.0, 200.0)),
        "shotgun_range" => Some((0.5, 500.0)),
        "shotgun_size_percent" => Some((1.0, 500.0)),
        "shotgun_spread_degrees" | "shotgun_horizontal_spread_degrees" | "shotgun_vertical_spread_degrees" => Some((0.0, 60.0)),
        "mine_flame_range" => Some((0.5, 100.0)),
        "mine_flame_rotation_degrees_per_second" => Some((0.0, 720.0)),
        "mine_flame_damage_per_second" => Some((0.0, 9999.0)),
        "sand_mouse_mode" => Some((1.0, 3.0)),
        "sand_mouse_lifetime_seconds" => Some((0.5, 600.0)),
        "sand_mouse_max_shots" => Some((1.0, 9999.0)),
        "sand_mouse_fire_interval_ticks" => Some((1.0, 9999.0)),
        "sand_mouse_shot_range" => Some((0.5, 500.0)),
        "sand_mouse_shot_damage_hp" => Some((0.0, 9999.0)),
        "secondary_beam_damage_percent" => Some((0.0, 100.0)),
        "throw_range" if item==8 => Some((0.5,100.)),
        "slam_explosion_radius" if item==8 => Some((0.1,30.)),
        "slam_explosion_damage_hp" if item==8 => Some((0.,9999.)),
        "throw_speed" if item==8 => Some((1.,200.)),
        "crate_breaker_radius" if item==8 => Some((0.,100.)),
        "bolt_attraction_bonus" if item==8 => Some((0.,100.)),
        "grenade_size_percent" => Some((1.,1000.)),
        "grenade_arc_height" => Some((0.1,50.)),
        "grenade_max_range" => Some((0.5,500.)),
        "grenade_fire_interval_ticks" => Some((1.,600.)),
        "grenade_damage_hp" => Some((0.,9999.)),
        "grenade_explosion_radius" => Some((0.1,8.)),
        "flame_mine_size_percent" => Some((1.,500.)),
        _ => None,
    };
    if let Some((lo, hi)) = bounds {
        if !(lo..=hi).contains(&v) { return Err(format!("{key} must be {lo}..{hi}")); }
    }
    if matches!(key, "grenade_fire_interval_ticks" | "max_active_mines" | "flame_mine_max_active_mines" | "sand_mouse_mode" | "sand_mouse_max_shots" | "sand_mouse_fire_interval_ticks") && v.fract() != 0.0 {
        return Err(format!("{key} must be an integer"));
    }
    Ok(())
}

/// Loader-only inspection. Unlike from_env this never publishes or resets the
/// active registry, icon levels, ammo caps, XP or gameplay state.
pub fn inspect_profile() -> Option<Evolution> {
    #[cfg(feature="weapon-evo-release")]
    let raw=RELEASE_PROFILE.to_string();
    #[cfg(not(feature="weapon-evo-release"))]
    let raw=std::env::var("RC_WEAPON_EVOLUTION_INI").ok().and_then(|p|std::fs::read_to_string(p).ok())?;
    Evolution::parse(&raw).ok().map(|mut e| {location_test::configure(&mut e);gemlik_test::configure(&mut e);health_test::configure(&mut e);e})
}

/// Reserve actors on the loader thread without publishing gameplay state.
pub fn profile_extra_enemy_count() -> usize {
    inspect_profile().map_or(0, |e| e.world_settings.extra_enemies_per_level as usize)
}

pub fn profile_uses_extra(item:i32,key:&str)->bool {
    inspect_profile().is_some_and(|e| {
        if item == -1 && key == ammo_mode::KEY { return e.ammo_unlock_mode; }
        e.extras.iter().find(|t|t.id==item)
            .is_some_and(|t|t.params.iter().any(|p|p.get(key).is_some_and(|&v|v!=0.0)))
    })
}

/// The disc template may contain Bomb Glove ammunition. Apply the opt-in
/// new-game policy before native level-start grants, without touching loads.
pub fn initialize_ammo_mode_new_game(gs: &mut crate::game_state::GameState) {
    if let Some(e) = inspect_profile() {
        ammo_mode::initialize_new_game(gs, e.ammo_unlock_mode, &e.ammo_mode_settings);
        golden::initialize_new_game(gs,e.golden_settings,golden::fresh_seed());
        randomizer::initialize(gs,e.randomizer_mode,golden::fresh_seed());
        if location_test::enabled() { location_test::prepare_new_game(gs); }
    }
}

#[cfg(test)]
mod v45_configuration_tests {
    use super::*;
    fn base()->String {
        let mut s="[general]\ndamage_mode=real_hp\n".to_owned();
        for grade in 1..=5 {s+=&format!("[weapon.bomb_glove.level{grade}]\n");}
        s
    }
    #[test]
    fn companion_mine_ryno_and_tesla_fields_remain_independent_at_every_grade() {
        let mut s=base();
        for grade in 1..=5 {
            s+=&format!("[weapon.drone_device.level{grade}]\nsand_mouse_enabled=true\nsand_mouse_mode={}\nsand_mouse_shot_damage_hp={}\nsand_mouse_lifetime_seconds={}\nsand_mouse_max_shots={}\nsand_mouse_fire_interval_ticks={}\nsand_mouse_shot_range={}\nsand_mouse_projectile_color=000000\n[weapon.mine_glove.level{grade}]\nmax_active_mines={grade}\nmine_lifetime_seconds={}\nflame_mine_enabled=true\nmine_flame_range={}\nmine_flame_rotation_degrees_per_second={}\nmine_flame_damage_per_second={}\nflame_core_color=FF0000\nflame_outer_color=00FF00\nburn_flame_color=0000FF\nexplosion_color=default\nlight_color=FFFFFF\n[weapon.ryno.level{grade}]\ncontinuous_missiles_enabled={}\nshotgun_enabled={}\nshotgun_damage_hp={}\nshotgun_range={}\nshotgun_speed={}\nshotgun_size_percent={}\nshotgun_spread_degrees={}\nshotgun_projectile_color=123456\nshotgun_trail_color=234567\nshotgun_impact_color=345678\nhoming_strength_multiplier={}\nmissile_max_range={}\nexplosion_damage_radius={}\nexplosion_visual_size_multiplier={}\n[weapon.tesla_claw.level{grade}]\nsecondary_beam_damage_percent={}\n[weapon.taunter.level{grade}]\nstun_launch_distance={}\n",
                (grade-1)%3+1,grade*25,grade*30,grade*10,grade*3,grade*5,
                grade*20,grade*2,grade*30,grade*50,grade%2==1,grade%2==0,
                grade*25,grade*5,grade*10,grade*20,grade*5,grade,grade*20,grade,grade,grade*10,grade);
        }
        let e=Evolution::parse(&s).unwrap();
        for i in 0..5 {
            let n=i as f32+1.0;
            let track=|item|&e.extras.iter().find(|t|t.id==item).unwrap().params[i];
            assert_eq!(track(24)["sand_mouse_shot_damage_hp"],n*25.0);
            assert_eq!(track(24)["sand_mouse_projectile_color"],0.0);
            assert_eq!(track(17)["mine_flame_range"],n*2.0);
            assert_eq!(track(17)["mine_flame_damage_per_second"],n*50.0);
            assert_eq!(track(23)["shotgun_speed"],n*10.0);
            assert_eq!(track(23)["missile_max_range"],n*20.0);
            assert_eq!(track(19)["secondary_beam_damage_percent"],n*10.0);
            assert_eq!(track(14)["stun_launch_distance"],n);
        }
    }
    #[test]
    fn shotgun_axes_are_independent_for_all_five_grades() {
        let mut s=base();
        for grade in 1..=5 {
            s+=&format!("[weapon.ryno.level{grade}]\nshotgun_horizontal_spread_degrees={}\nshotgun_vertical_spread_degrees={}\n",grade*7,grade);
        }
        let e=Evolution::parse(&s).unwrap();let ryno=e.extras.iter().find(|t|t.id==23).unwrap();
        for grade in 0..5 {
            assert_eq!(ryno.params[grade]["shotgun_horizontal_spread_degrees"],(grade+1) as f32*7.0);
            assert_eq!(ryno.params[grade]["shotgun_vertical_spread_degrees"],(grade+1) as f32);
        }
    }
    #[test]
    fn shotgun_axes_accept_zero_but_reject_invalid_bounds() {
        for key in ["shotgun_horizontal_spread_degrees","shotgun_vertical_spread_degrees"] {
            for value in ["-1","61","NaN","inf"] {
                assert!(Evolution::parse(&(base()+&format!("[weapon.ryno.level1]\n{key}={value}\n"))).is_err());
            }
            assert!(Evolution::parse(&(base()+&format!("[weapon.ryno.level1]\n{key}=0\n"))).is_ok());
        }
    }

    #[test]
    fn malformed_new_fields_and_conflicting_ryno_modes_are_rejected() {
        for (weapon,key,value) in [
            ("drone_device","sand_mouse_mode","4"),("drone_device","sand_mouse_mode","1.5"),
            ("drone_device","sand_mouse_max_shots","0"),("drone_device","sand_mouse_fire_interval_ticks","0"),
            ("drone_device","sand_mouse_lifetime_seconds","NaN"),("drone_device","sand_mouse_shot_damage_hp","-1"),
            ("tesla_claw","secondary_beam_damage_percent","101"),("mine_glove","max_active_mines","33"),
            ("mine_glove","mine_flame_range","0"),("mine_glove","mine_flame_damage_per_second","inf"),
            ("ryno","shotgun_speed","0"),("ryno","shotgun_spread_degrees","61"),
        ] {
            let s=base()+&format!("[weapon.{weapon}.level1]\n{key}={value}\n");
            assert!(Evolution::parse(&s).is_err(),"{weapon}.{key}={value}");
        }
        assert!(Evolution::parse(&(base()+"[weapon.ryno.level1]\ncontinuous_missiles_enabled=true\nshotgun_enabled=true\n")).is_err());
    }
}

#[cfg(test)]
mod v48_tests {
    use super::*;
    #[test]
    fn each_drone_grade_accepts_ten_and_rejects_fractional_or_excess_capacity() {
        for level in 1..=5 {
            for count in [0, 6, 7, 10] {
                let ini = format!("[weapon.bomb_glove.level1]\n[weapon.bomb_glove.level2]\n[weapon.bomb_glove.level3]\n[weapon.bomb_glove.level4]\n[weapon.bomb_glove.level5]\n[weapon.drone_device]\nid = 24\n[weapon.drone_device.level{level}]\ndrone_count = {count}\n");
                let e = Evolution::parse(&ini).unwrap();
                let track = e.extras.iter().find(|t| t.id == 24).unwrap();
                assert_eq!(track.params[level-1]["drone_count"], count as f32);
            }
            for count in ["-1", "10.5", "11", "nan"] {
                let ini = format!("[weapon.bomb_glove.level1]\n[weapon.bomb_glove.level2]\n[weapon.bomb_glove.level3]\n[weapon.bomb_glove.level4]\n[weapon.bomb_glove.level5]\n[weapon.drone_device]\nid = 24\n[weapon.drone_device.level{level}]\ndrone_count = {count}\n");
                assert!(Evolution::parse(&ini).is_err());
            }
        }
    }
}

#[cfg(test)]
mod v51_configuration_tests {
    use super::*;
    #[test]
    fn gold_appearance_is_independent_for_new_weapons_and_global_mode_is_opt_in() {
        let mut ini=String::from("[general]\nexperimental_ammo_unlock_mode=true\nwrench_gold_texture=true\n");
        for grade in 1..=5 { ini+=&format!("[weapon.bomb_glove.level{grade}]\n"); }
        for weapon in ["visibomb","taunter","walloper","ryno"] {
            for grade in 1..=5 { ini+=&format!("[weapon.{weapon}.level{grade}]\ngold_texture={}\ngold_effects=false\n",grade%2==0); }
        }
        let evo=Evolution::parse(&ini).unwrap();
        assert!(evo.ammo_unlock_mode && evo.wrench_gold_texture);
        for weapon in ["visibomb","taunter","walloper","ryno"] {
            let track=evo.extras.iter().find(|t|t.section==weapon).unwrap();
            for grade in 0..5 {
                assert_eq!(track.params[grade]["gold_texture"], if grade%2==1 {1.0}else{0.0});
                assert_eq!(track.params[grade]["gold_effects"],0.0);
            }
        }
        assert!(!Evolution::default().ammo_unlock_mode);
        assert!(Evolution::parse(&ini.replace("unlock_mode=true","unlock_mode=maybe")).is_err());
        assert!(Evolution::parse(&ini.replace("wrench_gold_texture=true","wrench_gold_texture=2")).is_err());
    }
}

#[cfg(test)]
mod v53_tests {
    use super::*;
    #[test]
    fn per_grade_geometry_and_grenade_hp_are_independent() {
        let mut ini=String::new();for grade in 1..=5 {ini+=&format!("[weapon.bomb_glove.level{grade}]\n");}
        for grade in 1..=5 {ini+=&format!("[weapon.visibomb.level{grade}]\ngrenade_launcher_enabled={}\ngrenade_damage_hp={}\nexplosion_damage_radius={}\ngrenade_color=FF4400\n[weapon.mine_glove.level{grade}]\nflame_mine_size_percent={}\n",grade%2==0,grade*125,grade+2,100+grade*10);}
        let e=Evolution::parse(&ini).unwrap();let v=e.extras.iter().find(|t|t.id==13).unwrap();let m=e.extras.iter().find(|t|t.id==17).unwrap();
        for grade in 0..5 {assert_eq!(v.params[grade]["grenade_damage_hp"],(grade+1) as f32*125.);assert_eq!(v.params[grade]["grenade_launcher_enabled"],if grade%2==1 {1.} else {0.});assert_eq!(m.params[grade]["flame_mine_size_percent"],110.+grade as f32*10.);}
        for (key,value) in [("grenade_size_percent","0"),("grenade_arc_height","0"),("grenade_max_range","NaN"),("grenade_fire_interval_ticks","2.5"),("grenade_explosion_radius","9"),("grenade_launcher_enabled","maybe")] {assert!(Evolution::parse(&ini.replace("grenade_color=FF4400",&format!("{key}={value}"))).is_err(),"{key}");}
    }
    #[test]
    fn vivid_orange_and_gold_remain_distinct() {
        assert_eq!(icon_grade_tint(1),None);let orange=icon_grade_tint(4).unwrap();let gold=icon_grade_tint(5).unwrap();
        assert_eq!(orange&255,255);assert_eq!(gold&255,255);assert_eq!((orange>>16)&255,0);assert_eq!((gold>>16)&255,0);
        assert!(((gold>>8)&255)>((orange>>8)&255)+80);assert_ne!(icon_grade_tint(2),icon_grade_tint(3));
    }
}


/// Mode switches only: crate filters and presentation controls never change capacity.
/// Keep in sync with launcher/schema.py EXPERIMENTAL_ENABLERS.
fn experimental_ammo_enabled(item:i32, values:&BTreeMap<String,f32>)->bool {
    let keys:&[&str]=match item {
        9=>&["crate_suction_enabled"],13=>&["grenade_launcher_enabled"],
        17=>&["flame_mine_enabled"],19=>&["dual_beam_enabled"],
        23=>&["continuous_missiles_enabled","shotgun_enabled"],24=>&["sand_mouse_enabled"],_=>&[],
    };
    keys.iter().any(|key|values.get(*key).is_some_and(|&v|v>=0.5))
}


#[cfg(test)]
mod v56_ammo_tests {
    use super::*;
    #[test]
    fn every_grade_selects_independent_capacity_only_with_its_mode_enabled() {
        let mut ini=String::from("[general]\ndamage_mode=real_hp\n");
        for n in 1..=5 {
            for (name,key) in [("bomb_glove","cluster_bombs_enabled"),("pyrocitor","triple_flame_enabled"),
                ("suck_cannon","crate_suction_enabled"),("visibomb","grenade_launcher_enabled"),
                ("mine_glove","flame_mine_enabled"),("tesla_claw","dual_beam_enabled"),
                ("ryno","shotgun_enabled"),("drone_device","sand_mouse_enabled")] {
                let standard=if name=="suck_cannon" {"capacity=5"} else {"max_ammo=40"};
                ini+=&format!("[weapon.{name}.level{n}]\n{standard}\n{key}={}\nexperimental_max_ammo={}\n",n%2==0,n+5);
            }
        }
        let e=Evolution::parse(&ini).unwrap();
        for i in 0..5 {
            let active=(i+1)%2==0;let alt=(i+6) as i32;
            assert_eq!(e.stats[i].ammo_capacity(),if active {alt} else {40});
            assert_eq!(e.pyro.stats[i].ammo_capacity(),if active {alt} else {40});
            for t in e.extras.iter().filter(|t|matches!(t.id,9|13|17|19|23|24)) {
                assert_eq!(experimental_ammo_enabled(t.id,&t.params[i]),active);
                assert_eq!(t.params[i]["experimental_max_ammo"],alt as f32);
            }
        }
        // Legacy profiles and disabled modes preserve native/standard values.
        assert_eq!(Stats{cluster:ClusterStats{enabled:true,..Default::default()},..Default::default()}.ammo_capacity(),40);
        assert_eq!(PyroStats{triple_flame_enabled:true,..Default::default()}.ammo_capacity(),100);
        let mut params=BTreeMap::new();params.insert("continuous_missiles_enabled".into(),1.0);
        assert!(experimental_ammo_enabled(23,&params));
        assert!(!experimental_ammo_enabled(19,&params));
    }
    fn complete_profile(extra:&str)->String {
        let mut base=String::new();
        for n in 1..=5 {base+=&format!("[weapon.bomb_glove.level{n}]\n[weapon.pyrocitor.level{n}]\n");}
        base+extra
    }
    #[test]
    fn capacity_limits_match_standard_and_reject_invalid_numbers() {
        for name in ["bomb_glove","pyrocitor","suck_cannon","visibomb","mine_glove","tesla_claw","ryno","drone_device"] {
            for value in ["-1","1.5","NaN","inf","10000"] {
                let ini=format!("[weapon.{name}.level1]\nexperimental_max_ammo={value}\n");
                assert!(Evolution::parse(&complete_profile(&ini)).is_err(),"{name}: {value}");
            }
        }
        for name in ["bomb_glove","pyrocitor"] {
            assert!(Evolution::parse(&complete_profile(&format!("[weapon.{name}.level1]\nexperimental_max_ammo=0"))).is_err());
        }
        for name in ["suck_cannon","visibomb","mine_glove","tesla_claw","ryno","drone_device"] {
            assert!(Evolution::parse(&complete_profile(&format!("[weapon.{name}.level1]\nexperimental_max_ammo=0"))).is_ok());
        }
        assert!(Evolution::parse(&complete_profile("[weapon.suck_cannon.level1]\nexperimental_max_ammo=11")).is_err());
        assert!(Evolution::parse(&complete_profile("[weapon.pyrocitor.level1]\nexperimental_max_ammo=1000")).is_err());
        assert!(Evolution::parse(&complete_profile("[weapon.bomb_glove.level1]\nexperimental_max_ammo=1000")).is_err());
    }
}

#[cfg(test)]
mod v57_inventory_tests {
    use super::*;
    #[test]
    fn all_experimental_ammo_counters_match_the_active_capacity_and_keep_consuming() {
        let mut e=Evolution::parse(include_str!("weapon_evo_release.ini")).unwrap();
        // Exercise configured alternates, independently of saved manual modes
        // and whichever switches are enabled in the public balance profile.
        e.control_settings.r1=false;
        let counters=[10,16,13,17,19,23,24];
        let mut w=crate::hero::weapons::Weapons::default();
        for id in counters {w.uses_ammo[id]=true;}
        for grade in 1..=5 {
            e.level=grade;e.pyro.level=grade;
            e.stats[grade-1].max_ammo=40;e.stats[grade-1].experimental_max_ammo=Some(5);
            e.stats[grade-1].cluster.enabled=true;
            e.pyro.stats[grade-1].max_ammo=40;e.pyro.stats[grade-1].experimental_max_ammo=Some(5);
            e.pyro.stats[grade-1].triple_flame_enabled=true;
            e.pyro.stats[grade-1].lava.enabled=false;
            for track in &mut e.extras {
                track.level=grade;
                if !matches!(track.id,13|17|19|23|24) {continue;}
                let p=&mut track.params[grade-1];
                p.insert("max_ammo".into(),40.);p.insert("experimental_max_ammo".into(),5.);
                let key=match track.id {13=>"grenade_launcher_enabled",17=>"flame_mine_enabled",19=>"dual_beam_enabled",23=>"shotgun_enabled",_=>"sand_mouse_enabled"};
                p.insert(key.into(),1.);
                assert_eq!(track.ammo_capacity(),Some(5));
            }
            w.ammo[9]=7; // Suck Cannon's stored-object queue has its own capacity path.
            for id in counters {w.ammo[id]=40;}
            e.apply_ammo_limits(&mut w);
            for remaining in (0..5).rev() {
                for id in counters {assert!(w.use_ammo(id as i32,1));}
                e.apply_ammo_limits(&mut w);
                for id in counters {assert_eq!(w.ammo[id],remaining,"grade {grade}, weapon {id}");}
            }
            for id in counters {assert!(!w.use_ammo(id as i32,1));}
            assert_eq!(w.ammo[9],7);
            e.stats[grade-1].cluster.enabled=false;e.pyro.stats[grade-1].triple_flame_enabled=false;
            for track in &mut e.extras {for key in ["grenade_launcher_enabled","flame_mine_enabled","dual_beam_enabled","shotgun_enabled","continuous_missiles_enabled","sand_mouse_enabled"] {track.params[grade-1].insert(key.into(),0.);}}
            e.apply_ammo_limits(&mut w);
            for id in counters {assert_eq!(w.ammo[id],0,"changing mode must not refill");w.ammo[id]=60;}
            e.apply_ammo_limits(&mut w);
            for id in counters {assert_eq!(w.ammo[id],40,"standard cap restored for weapon {id}");}
        }
    }
}

#[cfg(test)]
mod v58_model_tests {
    use super::*;
    #[test]
    fn every_grade_keeps_model_and_palettes_separate_and_rejects_invalid_codes() {
        let mut ini=String::new();
        for grade in 1..=5 {
            ini+=&format!("[weapon.bomb_glove.level{grade}]\n[weapon.visibomb.level{grade}]\ngrenade_model={}\ngrenade_color=FF0000\ngrenade_mine_color=00FF00\ngrenade_missile_color=0000FF\n[weapon.mine_glove.level{grade}]\nflame_mine_model={}\nmine_model_color=FFFFFF\nmine_bomb_color=112233\nmine_sphere_color=AABBCC\nmine_bomb_core_color=FF8800\nmine_sphere_core_color=44CCFF\n",grade%3,(grade+1)%3);
        }
        let e=Evolution::parse(&ini).unwrap();
        let v=e.extras.iter().find(|t|t.id==13).unwrap();let m=e.extras.iter().find(|t|t.id==17).unwrap();
        for grade in 0..5 {
            assert_eq!(v.params[grade]["grenade_model"],((grade+1)%3) as f32);
            assert_eq!(m.params[grade]["flame_mine_model"],((grade+2)%3) as f32);
            assert_eq!(v.params[grade]["grenade_mine_color"],65280.);
            assert_eq!(v.params[grade]["grenade_missile_color"],parse_rgb("0000FF").unwrap().unwrap() as f32);
            assert_eq!(m.params[grade]["mine_sphere_color"],parse_rgb("AABBCC").unwrap().unwrap() as f32);
        }
        for bad in ["-1","3","1.5","NaN","inf"] {
            let invalid=ini.replace("grenade_model=1",&format!("grenade_model={bad}"));
            assert!(Evolution::parse(&invalid).is_err(),"{bad}");
            let invalid=ini.replace("flame_mine_model=2",&format!("flame_mine_model={bad}"));
            assert!(Evolution::parse(&invalid).is_err(),"{bad}");
        }
        Evolution::parse(include_str!("weapon_evo_release.ini")).unwrap();
    }
}

#[cfg(test)]
mod v64_lava_tests {
    use super::*;
    fn profile(extra: &str) -> String {
        let mut ini=String::from("[general]\ndamage_mode=real_hp\n");
        for n in 1..=5 { ini+=&format!("[weapon.bomb_glove.level{n}]\n[weapon.pyrocitor.level{n}]\n"); }
        ini+extra
    }
    #[test]
    fn triple_lava_reuses_existing_independent_grade_switches_and_angles() {
        let mut ini=String::new();
        for n in 1..=5 {
            ini+=&format!("[weapon.pyrocitor.level{n}]\nlava_stream_enabled=true\ntriple_flame_enabled={}\nside_flame_left_angle_degrees={}\nside_flame_right_angle_degrees={}\nlava_launch_angle_degrees={}\n",n%2==1,n*4,n*6,n*2);
        }
        let e=Evolution::parse(&profile(&ini)).unwrap();
        for (i,s) in e.pyro.stats.into_iter().take(5).enumerate() {
            let n=i+1;
            assert!(s.lava.enabled);
            assert_eq!(s.triple_flame_enabled,n%2==1);
            assert_eq!(s.side_flame_left_angle_degrees,Some((n*4) as f32));
            assert_eq!(s.side_flame_right_angle_degrees,Some((n*6) as f32));
            assert_eq!(s.lava.launch_angle_degrees,(n*2) as f32);
        }
    }
    #[test]
    fn independent_grades_select_lava_capacity_without_changing_dps_or_xp() {
        let mut ini=String::new();
        for n in 1..=5 {
            ini+=&format!("[weapon.pyrocitor.level{n}]\nlava_stream_enabled={}\nlava_launch_speed={}\nlava_gravity={}\nlava_max_range={}\nlava_size_percent={}\nlava_core_color=FF0000\nlava_outer_color=00FF00\nlava_impact_color=0000FF\nlava_light_color=F0A008\nmax_ammo=50\nexperimental_max_ammo={}\ndamage_per_second={}\ndamage_interval_ticks={}\nammo_interval_ticks={}\n",n%2==1,n*10,n*2,n*3,n*20,n+5,n*100,n,n+1);
        }
        let e=Evolution::parse(&profile(&ini)).unwrap();
        for i in 0..5 {
            let n=i+1;let s=e.pyro.stats[i];
            assert_eq!(s.lava.enabled,n%2==1);
            assert_eq!(s.ammo_capacity(),if n%2==1 {(n+5) as i32} else {50});
            assert_eq!(s.lava.speed,(n*10) as f32);
            assert_eq!(s.lava.gravity,(n*2) as f32);
            assert_eq!(s.lava.range,(n*3) as f32);
            assert_eq!(s.lava.size_percent,(n*20) as f32);
            assert_eq!(s.lava.core_color,Some(0x0000FF));
            assert_eq!(s.lava.outer_color,Some(0x00FF00));
            assert_eq!(s.lava.impact_color,Some(0xFF0000));
            assert_eq!(s.lava.light_color,Some(0x08A0F0));
            assert_eq!(e.damage_hp[16][i],Some((n*100) as f32));
            assert_eq!(s.damage_interval_ticks,n as i32);
            assert_eq!(s.ammo_interval_ticks,(n+1) as i32);
        }
        let old=Evolution::parse(&profile("")).unwrap();
        assert!(old.pyro.stats.iter().all(|s| !s.lava.enabled));
        assert_eq!(old.pyro.xp_required,e.pyro.xp_required);
    }
    #[test]
    fn lava_rejects_invalid_and_nonfinite_tuning_even_when_disabled() {
        for (key,value) in [("lava_launch_speed","0"),("lava_launch_speed","121"),
            ("lava_gravity","-1"),("lava_gravity","NaN"),("lava_max_range","inf"),
            ("lava_max_range","101"),("lava_size_percent","9"),("lava_size_percent","501"),
            ("lava_core_color","RED"),("lava_stream_enabled","maybe")] {
            assert!(Evolution::parse(&profile(&format!("[weapon.pyrocitor.level3]\n{key}={value}\n"))).is_err(),"{key}={value}");
        }
        assert!(Evolution::parse(&profile("[weapon.pyrocitor.level2]\nlava_gravity=0\nlava_core_color=default\n")).is_ok());
    }
}

#[cfg(test)]
mod v65_lava_angle_tests {
    use super::*;
    fn profile(extra:&str)->String {
        let mut ini=String::new();
        for grade in 1..=5 {ini+=&format!("[weapon.bomb_glove.level{grade}]\n[weapon.pyrocitor.level{grade}]\n");}
        ini+extra
    }
    #[test]
    fn signed_elevation_is_per_grade_and_legacy_profiles_get_ten_degrees() {
        let mut ini=String::new();
        for (n,angle) in [-45.0,-10.0,0.0,10.0,45.0].into_iter().enumerate() {
            ini+=&format!("[weapon.pyrocitor.level{}]\nlava_launch_angle_degrees={angle}\n",n+1);
        }
        let parsed=Evolution::parse(&profile(&ini)).unwrap();
        assert_eq!(parsed.pyro.stats[..5].iter().map(|s|s.lava.launch_angle_degrees).collect::<Vec<_>>(),[-45.0,-10.0,0.0,10.0,45.0]);
        assert!(Evolution::parse(&profile("")).unwrap().pyro.stats.iter().all(|s|s.lava.launch_angle_degrees==10.0));
        for invalid in ["-45.1","45.1","NaN","inf","oops"] {
            assert!(Evolution::parse(&profile(&format!("[weapon.pyrocitor.level1]\nlava_launch_angle_degrees={invalid}\n"))).is_err());
        }
    }
}

#[cfg(test)]
mod attraction_reward_tests {
    use super::*;

    fn enabled() -> Evolution {
        let mut evo = Evolution::default();
        evo.real_hp = true;
        evo.xp_per_damage = 0.1;
        evo.world_settings.enemy_damage_by_health = true;
        evo.world_settings.nanotech_xp_per_upgrade = 1.0;
        for track in &mut evo.extras {
            track.enabled = true;
            track.xp_required = [1_000_000.0; 7];
        }
        evo
    }
    fn xp(evo: &Evolution, item: i32) -> f32 {
        evo.extras.iter().find(|t| t.id == item).unwrap().xp
    }

    #[test]
    fn boss_attraction_is_ten_percent_once_per_weapon_and_life() {
        for class in [1051, 1106, 388, 1422] {
            let mut evo = enabled();
            for _ in 0..10 {
                evo.award_attraction_once(14, 42, class, 20.0, 1.0 / 3.0);
                evo.enqueue_attraction(25, 42, class, 20.0, 0.25);
            }
            evo.drain_special();
            assert_eq!(xp(&evo, 14), 20.0);
            assert_eq!(xp(&evo, 25), 20.0);
            assert_eq!(evo.nanotech, world::NanotechProgress::default());
            assert_eq!(evo.nanotech_pending_heal, 0);
            evo.forget_special_target(42);
            evo.award_attraction_once(14, 42, class, 20.0, 1.0 / 3.0);
            assert_eq!(xp(&evo, 14), 40.0, "a recreated boss is a new life");
        }
    }

    #[test]
    fn ordinary_rewards_and_disabled_or_smaller_fractions_are_preserved() {
        let mut evo = enabled();
        evo.award_attraction_once(14, 1, 577, 2.0, 0.5);
        evo.enqueue_attraction(25, 1, 577, 2.0, 0.25);
        evo.drain_special();
        assert_eq!((xp(&evo, 14), xp(&evo, 25)), (10.0, 5.0));
        evo.award_attraction_once(14, 2, 1051, 20.0, 0.0);
        assert_eq!(xp(&evo, 14), 10.0);
        evo.award_attraction_once(14, 2, 1051, 20.0, 0.05);
        assert_eq!(xp(&evo, 14), 20.0);
        for fraction in [f32::NAN, f32::INFINITY, -1.0] {
            evo.award_attraction_once(14, 3, 1051, 20.0, fraction);
        }
        assert_eq!(xp(&evo, 14), 20.0);
    }

    #[test]
    fn direct_taunter_damage_and_capped_weapon_tracks_never_grant_nanotech_xp() {
        let mut evo = enabled();
        for item in [14, 25] {
            evo.award_extra_damage(item, 10.0);
            assert_eq!(xp(&evo, item), 100.0);
            evo.extras.iter_mut().find(|t| t.id == item).unwrap().level = 5;
            evo.award_extra_damage(item, 10.0);
        }
        assert_eq!(evo.nanotech, world::NanotechProgress::default());
        assert_eq!(evo.nanotech_pending_heal, 0);
        evo.award_extra_damage(15, 0.1);
        assert_eq!(evo.nanotech.bonus, 1, "other weapons still level Ratchet");
    }
}

#[cfg(test)]
mod nanotech_bar_opacity_tests {
    use super::*;
    fn ini(value:&str)->String {
        let mut text=String::new();
        for n in 1..=5 {text+=&format!("[weapon.bomb_glove.level{n}]\n[weapon.pyrocitor.level{n}]\n");}
        text+&format!("[hud]\nname_opacity_percent=35\nxp_bar_opacity_percent=0\nnanotech_xp_bar_opacity_percent={value}\n")
    }
    #[test]
    fn nanotech_opacity_is_independent_and_rejects_invalid_percentages() {
        for value in [0u8,50,100] {
            let evo=Evolution::parse(&ini(&value.to_string())).unwrap();
            assert_eq!(evo.hud.nanotech_xp_bar_opacity_percent,value);
            assert_eq!(evo.hud.name_opacity_percent,35);
        }
        assert_eq!(HudOptions::default().nanotech_xp_bar_opacity_percent,100);
        for value in ["101","-1","50.5","NaN","off"] {assert!(Evolution::parse(&ini(value)).is_err());}
    }
}

#[path="weapon_evo_upgrade_wave.rs"]
pub mod upgrade_wave;

#[cfg(test)]
#[path="weapon_evo_v76_tests.rs"]
mod v76_tests;

#[cfg(test)]
#[path = "weapon_evo_v77_tests.rs"]
mod v77_tests;

#[cfg(test)]
mod alternate_override_tests {
    use super::*;
    fn profile() -> String {
        let mut text=String::from("[general]\ndamage_mode=real_hp\nr1_experimental=true\n");
        for n in 1..=8 {
            text+=&format!("[weapon.bomb_glove.level{n}]\nmax_ammo=40\n");
            text+=&format!("[weapon.pyrocitor.level{n}]\ndamage_per_second={}\nlava_stream_enabled=true\nlava_damage_bonus_percent={}\n",n*100,n*10);
            text+=&format!("[weapon.mine_glove.level{n}]\ndamage_hp=300\nflame_mine_enabled=true\nmax_active_mines=2\nmine_lifetime_seconds=40\nflame_mine_max_active_mines={}\nflame_mine_lifetime_seconds={}\n",n+8,n+2);
        }
        text
    }
    #[test]
    fn all_grade_overrides_survive_parsing_and_r1_changes_only_mode_enablers() {
        let mut evo=Evolution::parse(&profile()).unwrap();
        for n in 1..=8 {
            evo.pyro.level=n;
            assert_eq!(evo.effective_pyro().lava.damage_bonus_percent,(n*10) as f32);
            assert!(!evo.effective_pyro().lava.enabled,"R1 native mode ignores the INI enabler");
            evo.state_progress.controls.cycle(16);
            assert!(evo.effective_pyro().lava.enabled);
            evo.state_progress.controls.cycle(16);
            let mine=evo.extras.iter().find(|t|t.id==17).unwrap();
            let params=&mine.params[n-1];
            for active in [false,true] {
                let mut registry:HashMap<_,_>=params.iter().map(|(k,&v)|((17,k.clone()),v)).collect();
                let mut state=controls::State::default();
                if active {state.cycle(17);}
                controls::apply_registry(evo.control_settings,&state,&mut registry,|_|1);
                assert_eq!(registry[&(17,"flame_mine_enabled".into())],if active {1.}else{0.});
                assert_eq!(registry[&(17,"flame_mine_max_active_mines".into())],(n+8) as f32);
                assert_eq!(registry[&(17,"flame_mine_lifetime_seconds".into())],(n+2) as f32);
                assert_eq!(registry[&(17,"max_active_mines".into())],2.);
            }
        }
        evo.control_settings.r1=false;
        assert!(evo.effective_pyro().lava.enabled,"INI activation still works without R1 switching");
    }
    #[test]
    fn override_ranges_are_validated_even_with_the_mode_disabled() {
        for (weapon,key,value) in [("pyrocitor","lava_damage_bonus_percent","-1"),
            ("pyrocitor","lava_damage_bonus_percent","1001"),("pyrocitor","lava_damage_bonus_percent","NaN"),
            ("mine_glove","flame_mine_max_active_mines","0"),("mine_glove","flame_mine_max_active_mines","33"),
            ("mine_glove","flame_mine_max_active_mines","2.5"),("mine_glove","flame_mine_lifetime_seconds","0.4"),
            ("mine_glove","flame_mine_lifetime_seconds","601")] {
            let original=match weapon {"pyrocitor"=>"10",_=>if key=="flame_mine_max_active_mines" {"9"} else {"3"}};
            let invalid=profile().replacen(&format!("{key}={original}"),&format!("{key}={value}"),1)
                .replace("enabled=true","enabled=false");
            assert!(Evolution::parse(&invalid).is_err(),"{key}={value}");
        }
        let text=profile().lines().filter(|line|!line.starts_with("lava_damage_bonus_percent=")).collect::<Vec<_>>().join("\n");
        let evo=Evolution::parse(&text).unwrap();
        assert_eq!(evo.effective_pyro().lava.damage_bonus_percent,25.);
    }
}

#[cfg(test)]
mod fixed_xp_nanotech_tests {
    use super::*;
    #[test]
    fn explicit_weapon_budgets_keep_chain_bonus_zero_xp_and_nanotech_switch() {
        for weapon in [8,9,10,14,16,21,25] {
            let mut e=Evolution::default();
            e.enabled=true;e.real_hp=true;e.xp_per_damage=0.12;e.xp_required=[10000.;7];
            e.pyro.enabled=true;e.pyro.xp_required=[10000.;7];
            for t in &mut e.extras {t.enabled=true;t.xp_required=[10000.;7];}
            e.world_settings.enemy_damage_by_health=true;
            e.world_settings.nanotech_xp_per_upgrade=10000.;
            e.configure_ng_plus(true);e.bolt_chain.kills=30;
            e.award_fixed_xp(weapon,300.,6.);
            let xp=match weapon {10=>e.xp,16=>e.pyro.xp,_=>e.extras.iter().find(|t|t.id==weapon).unwrap().xp};
            assert!((xp-900.).abs()<0.001);
            let expected=if matches!(weapon,14|25) {0.}else{216.};
            assert!((e.nanotech.xp-expected).abs()<0.001);
            e.xp_per_damage=0.;e.award_fixed_xp(weapon,300.,6.);
            assert!((e.nanotech.xp-expected).abs()<0.001);
            e.xp_per_damage=0.12;e.world_settings.enemy_damage_by_health=false;
            e.award_fixed_xp(weapon,300.,6.);
            assert!((e.nanotech.xp-expected).abs()<0.001);
        }
    }
}

#[cfg(test)]
mod wrench_feedback_tests {
    use super::*;
    #[test]
    fn only_positive_accepted_wrench_xp_signals_hud_and_caps_follow_game_mode() {
        let mut e=Evolution::default();
        let wrench=e.extras.iter_mut().find(|t|t.id==8).unwrap();wrench.enabled=true;wrench.xp_required=[100.;7];
        for n in [0.,-1.,f32::NAN] {e.award_extra_xp(8,n);}
        assert_eq!(e.wrench_xp_feedback_serial,0);
        e.award_extra_xp(8,1.);assert_eq!(e.wrench_xp_feedback_serial,1);
        e.award_extra_xp(15,1.);assert_eq!(e.wrench_xp_feedback_serial,1);
        e.extras.iter_mut().find(|t|t.id==8).unwrap().level=5;
        e.award_extra_xp(8,1.);assert_eq!(e.wrench_xp_feedback_serial,1);
        e.configure_ng_plus(true);e.award_extra_xp(8,1.);assert_eq!(e.wrench_xp_feedback_serial,2);
        e.extras.iter_mut().find(|t|t.id==8).unwrap().level=8;
        e.award_extra_xp(8,1.);assert_eq!(e.wrench_xp_feedback_serial,2);
        e.restore_progress(&save::Progress::default());assert_eq!(e.wrench_xp_feedback_serial,0);
    }
}
