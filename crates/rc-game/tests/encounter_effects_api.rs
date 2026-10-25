//! Public contracts used directly by the Bevy world-effect and pickup adapters.
use rc_game::weapon_evo::{Evolution,golden,upgrade_wave,world};
#[test]
fn engine_can_draw_an_upgrade_without_combat_or_hud_notice_state() {
    let mut evo=Evolution::default();
    let levels=std::array::from_fn(|i|evo.weapon_level(i as i32).unwrap_or(0) as u8);
    let mut effect=upgrade_wave::State::default();effect.update(100,true,levels,[0.;3]);
    assert!(effect.waves.is_empty());
    let _settings=world::Settings{enemy_speed_multiplier:2.,enemy_projectile_speed_multiplier:3.,enemy_projectile_range_multiplier:1.5,..Default::default()};
    evo.golden_settings=golden::Settings{enabled:true,one_weapon_per_planet:true,..Default::default()};
    let layout=golden::State::generate_with_planet_limit(73,30,evo.golden_settings.one_weapon_per_planet);
    assert!(layout.valid());
    let _demo:fn(&rc_game::menus::vendor::VendorTables,usize)->Option<i32>=rc_game::menus::vendor::acquisition_demo_scene;
}
