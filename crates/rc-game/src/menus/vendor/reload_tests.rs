use super::*;

fn fixture()->(Vendor,GameState,MenuAssets) {
    let mut gs=GameState::zeroed(rc_formats::save_game::ChunkTables{global:vec![],level:vec![]});
    gs.global.owned[10]=1;gs.global.owned[15]=1;gs.global.ammo[10]=35;gs.global.ammo[15]=190;
    gs.global.vendor[0]=10;
    let mut v=Vendor::open(VendorTables{shop:shop(),..Default::default()},&gs,false,&mut VendorOut::default());
    v.sub=1;v.pre_fade=0;v.power_on=false;v.power_t=0;
    let glyphs=[[rc_formats::font::Glyph::default();rc_formats::font::GLYPHS];3];
    let a=MenuAssets::new(crate::hud::HudAssets{icons:vec![],frame_sizes:vec![],glyphs,messages:vec![]},Overlay::default());
    (v,gs,a)
}
#[test]
fn reload_all_is_atomic_at_the_exact_affordability_boundary() {
    let (mut v,mut gs,a)=fixture();let quote=v.reload_all_quote(&gs);
    assert_eq!(quote.cost,35);assert_eq!(quote.items,vec![(10,40,5),(15,200,10)]);
    gs.global.bolts=34;let ammo=gs.global.ammo;let bought=gs.global.ammo_bought;
    let mut out=VendorOut::default();v.reload_all(&mut gs,&a,&mut out);
    assert_eq!(gs.global.bolts,34);assert_eq!(gs.global.ammo,ammo);assert_eq!(gs.global.ammo_bought,bought);
    assert_eq!(out.sounds,vec![sound::DENIED]);
    gs.global.bolts=35;v.reload_all(&mut gs,&a,&mut VendorOut::default());
    assert_eq!(gs.global.bolts,0);assert_eq!(gs.global.ammo[10],40);assert_eq!(gs.global.ammo[15],200);
    assert_eq!(gs.global.ammo[16],0,"unowned weapon is never purchased or refilled");
    assert_eq!(gs.global.ammo_bought[10],5);assert_eq!(gs.global.ammo_bought[15],10);
    assert_eq!(v.reload_all_quote(&gs).cost,0);
    v.reload_all(&mut gs,&a,&mut VendorOut::default());assert_eq!(gs.global.ammo_bought[10],5);
}
#[test]
fn quote_counts_unlisted_owned_weapons_and_uses_wide_price_arithmetic() {
    let (mut v,mut gs,_)=fixture();
    v.items.clear();assert_eq!(v.reload_all_quote(&gs).cost,35);
    gs.global.owned[16]=1;gs.global.ammo[16]=0;
    assert_eq!(v.reload_all_quote(&gs).cost,275);
    for item in [10,15,16] {v.tables.shop.records[item][8..10].copy_from_slice(&u16::MAX.to_le_bytes());v.tables.shop.records[item][14..16].copy_from_slice(&u16::MAX.to_le_bytes());}
    assert!(v.reload_all_quote(&gs).cost>i64::from(i32::MAX));
    v.remote=true;assert!(v.reload_all_quote(&gs).items.is_empty());
}
#[test]
fn reload_overlay_is_localized_above_strip_and_hidden_during_popup_and_exit() {
    let (mut v,gs,a)=fixture();
    for (lang,label) in [(0,"Buy all ammo"),(1,"Buy all ammo"),(2,"Acheter toutes les munitions"),(3,"Alle Munition auffuellen"),(4,"Comprar toda la municion"),(5,"Compra tutte le munizioni")] {
        v.lang=lang;let d=v.reload_all_draws(&a,&gs,[100.,330.,320.,60.]);
        assert!(d.iter().any(|d|matches!(d,MenuDraw::Hud(crate::hud::Draw::Text{ text,..}) if text.starts_with(label.as_bytes()))));
        assert!(d.iter().any(|d|matches!(d,MenuDraw::Rect{y1,..} if *y1<330)));
    }
    v.buy=2;assert!(v.reload_all_draws(&a,&gs,[100.,330.,320.,60.]).is_empty());
    v.buy=0;let mut textured=a.clone();textured.hud.frame_sizes=vec![(16,16)];
    assert!(v.reload_all_draws(&textured,&gs,[100.,330.,320.,60.]).iter().any(|d|
        matches!(d,MenuDraw::Hud(crate::hud::Draw::Sprite{rot:crate::hud::Rot::R180,..}))));
    v.buy=0;v.exit_req=true;assert!(v.reload_all_draws(&a,&gs,[100.,330.,320.,60.]).is_empty());
}

#[test]
fn empty_vendor_recovers_even_without_exit_input_or_salesman_assets() {
    let (mut vendor,mut gs,assets)=fixture();
    vendor.items.clear(); vendor.salesman_loading=true;
    let items=rc_formats::save_game::ItemTables {
        vendor_items:[0;rc_formats::save_game::VENDOR_TABLE_LEN],
        records:[rc_formats::save_game::ItemRecord([0;0x18]);rc_formats::save_game::ITEM_COUNT],
        slot_type:[0;rc_formats::save_game::ITEM_COUNT],item_defs_addr:0,
    };
    let mut session=SessionState::default();let mut rng=Rng::new();
    let mut closed=false;
    for _ in 0..100 {
        let out=vendor.frame(&MenuInput::default(),&mut gs,&items,&mut session,&assets,&mut rng);
        if out.exit {closed=true;break;}
    }
    assert!(closed,"empty stock must not trap the player in mode 5");
}

#[test]
fn randomized_shop_gate_uses_discoveries_and_prices_but_not_current_ammo_or_bolts() {
    let (_,mut gs,_)=fixture();let prices=shop();
    gs.weapon_progress.golden=crate::weapon_evo::golden::State::generate(700,0);
    let mut progress=gs.weapon_progress.golden;
    assert!(!randomized_ammo_available(progress,&prices,false));
    progress.collect(14); // Taunter is permanent, but has no purchasable ammo.
    assert!(!randomized_ammo_available(progress,&prices,false));
    progress.collect(10);
    assert!(randomized_ammo_available(progress,&prices,false));
    gs.global.ammo[10]=40;gs.global.bolts=0;
    assert!(randomized_ammo_available(progress,&prices,false),"full ammo or no money does not lock the vendor");
    assert!(randomized_ammo_available(crate::weapon_evo::golden::State::default(),&ShopTable::default(),false));
}

#[test]
fn reload_hint_text_is_opaque_even_during_screen_power_on() {
    let (mut vendor,mut gs,assets)=fixture();gs.global.bolts=1000;vendor.power_on=true;vendor.power_t=POWER_TICKS;
    let draws=vendor.reload_all_draws(&assets,&gs,[100.,330.,320.,60.]);
    assert!(draws.iter().any(|d|matches!(d,MenuDraw::Hud(crate::hud::Draw::Text{rgba,text,..})
        if *rgba==0x80ffffff && text.starts_with(b"Buy all ammo - "))));
}

#[test]
fn randomized_vendor_sells_only_found_ammo_and_rejects_weapon_and_unfound_transactions() {
    let (mut vendor,mut gs,assets)=fixture();
    gs.weapon_progress.golden=crate::weapon_evo::golden::State::generate(105,0);
    gs.weapon_progress.golden.collect(10);
    gs.global.bolts=100_000;
    for remote in [false,true] {
        assert_eq!(build_list(&gs,&vendor.tables.shop,remote),vec![Entry {item:10,ammo:true,locked:false}]);
    }
    let quote=vendor.reload_all_quote(&gs);assert_eq!(quote.items,vec![(10,40,5)]);
    let before=gs.clone();let mut rng=crate::rng::Rng::new();let mut session=SessionState::default();
    let items=rc_formats::save_game::ItemTables {
        vendor_items:[0;rc_formats::save_game::VENDOR_TABLE_LEN],
        records:[rc_formats::save_game::ItemRecord([0;0x18]);rc_formats::save_game::ITEM_COUNT],
        slot_type:[0;rc_formats::save_game::ITEM_COUNT],item_defs_addr:0,
    };
    for entry in [Entry {item:15,ammo:false,locked:false},Entry {item:15,ammo:true,locked:false}] {
        let sound=vendor.purchase(entry,&mut gs,&items,&mut session,&assets,&mut rng,&mut VendorOut::default());
        assert_eq!(sound,super::sound::DENIED);
        assert_eq!(gs.global.bolts,before.global.bolts);assert_eq!(gs.global.ammo,before.global.ammo);
    }
    vendor.reload_all(&mut gs,&assets,&mut VendorOut::default());
    assert_eq!(gs.global.ammo[10],40);assert_eq!(gs.global.ammo[15],before.global.ammo[15]);
    assert_eq!(gs.global.bolts,before.global.bolts-quote.cost as i32);
}

#[test]
fn randomized_vendor_preserves_unrelated_gadget_stock_and_purchases() {
    let (mut vendor,mut gs,assets)=fixture();
    gs.weapon_progress.golden=crate::weapon_evo::golden::State::generate(109,0);
    gs.global.vendor=[12,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0xff];
    gs.global.owned[12]=0;gs.global.bolts=5000;
    let gadget=Entry {item:12,ammo:false,locked:false};
    assert_eq!(build_list(&gs,&vendor.tables.shop,false),vec![gadget]);
    assert!(build_list(&gs,&vendor.tables.shop,true).is_empty());
    let mut items=rc_formats::save_game::ItemTables {
        vendor_items:[0;rc_formats::save_game::VENDOR_TABLE_LEN],
        records:[rc_formats::save_game::ItemRecord([0;0x18]);rc_formats::save_game::ITEM_COUNT],
        slot_type:[0;rc_formats::save_game::ITEM_COUNT],item_defs_addr:0,
    };
    items.vendor_items[1]=12;
    let mut out=VendorOut::default();let mut session=SessionState::default();let mut rng=crate::rng::Rng::new();
    vendor.purchase(gadget,&mut gs,&items,&mut session,&assets,&mut rng,&mut out);
    assert_eq!(gs.global.owned[12],1);assert_eq!(gs.global.bolts,4000);
    assert_eq!(out.purchase,Some((12,false,1000,1)));
    assert!(build_list(&gs,&vendor.tables.shop,false).is_empty());
}

#[test]
fn reload_text_changes_at_the_exact_bolt_threshold() {
    let (v,mut gs,a)=fixture();let needed=v.reload_all_quote(&gs).cost as i32;
    for (bolts,rgb) in [(needed-1,0x606060),(needed,0xffffff),(needed+1,0xffffff)] {
        gs.global.bolts=bolts;
        assert!(v.reload_all_draws(&a,&gs,[100.,330.,320.,60.]).iter().any(|d|
            matches!(d,MenuDraw::Hud(crate::hud::Draw::Text{rgba,text,..}) if *rgba==0x80000000|rgb && text.starts_with(b"Buy all ammo - "))));
    }
}
#[test]
fn pickup_and_purchase_use_the_same_optional_native_demo_entry() {
    let mut tables=VendorTables::default();assert_eq!(acquisition_demo_scene(&tables,15),None);
    let mut layout=VendorLayout::default();layout.demo=vec![-1;37];layout.demo[15]=3;
    tables.layout=Some(layout);assert_eq!(acquisition_demo_scene(&tables,15),Some(3));
    assert_eq!(acquisition_demo_scene(&tables,10),None);assert_eq!(acquisition_demo_scene(&tables,100),None);
}
