//! Engine-facing contracts are tested from outside rc-game's private modules.
use rc_game::{hud::{Call,HudAssets,HudState,Inputs}, weapon_evo::{HudOptions,world},
    menus::{MenuInput,quick_select::QuickSelect,vendor}, pad::button};

// The same public-field expression used by rc-engine/menu_render. Keeping it
// here catches accidental visibility/API changes without a Bevy dependency.
fn ring_health_call(qs:&QuickSelect,inp:&MenuInput,calls:&mut Vec<Call>) {
    if qs.open && !qs.command_page && inp.held_u & button::TRIANGLE != 0 {
        calls.push(Call::QuickSelectHealth(rc_game::hud::scale_ticks(180)));
    }
}

#[test]
fn engine_can_read_progress_opacity_anchor_and_shared_bar_geometry() {
    let mut hud=HudState::new(HudAssets{icons:vec![],frame_sizes:vec![(32,32)],
        glyphs:[[rc_formats::font::Glyph::default();rc_formats::font::GLYPHS];3],messages:vec![]});
    hud.apply_calls(&[Call::QuickSelectHealth(180)]);
    for _ in 0..30 {hud.tick(Inputs{hp:20,max_hp:20,..Default::default()});}
    let ui=HudOptions::default();let mut settings=world::Settings::default();settings.enemy_damage_by_health=true;
    let progress=world::NanotechProgress{bonus:0,xp:500.};
    let (x,y,width,fade)=hud.nanotech_xp_anchor(10,ui.xp_bar_height_px).unwrap();
    let alpha=(HudOptions::alpha(ui.nanotech_xp_bar_opacity_percent) as f32*fade) as u32;
    let draws=rc_game::hud::xp_bar::draws(x,y,width,ui.xp_bar_height_px,progress.fraction(&settings,4).unwrap(),alpha,false);
    assert_eq!(draws.len(),3);assert_eq!(x+width/2,256);
    let _public_ring_contract:fn(&QuickSelect,&MenuInput,&mut Vec<Call>)=ring_health_call;
}

#[test]
fn vendor_gate_is_public_and_rechecks_the_current_randomized_progress() {
    let mut state=rc_game::weapon_evo::golden::State::generate(703,0);
    let mut prices=rc_game::moby_update::interact::ShopTable{records:vec![[0;0x18];43]};
    prices.records[10][8..10].copy_from_slice(&5u16.to_le_bytes());
    assert!(!vendor::randomized_ammo_available(state,&prices,false));
    state.collect(10);
    assert!(vendor::randomized_ammo_available(state,&prices,false));
}
