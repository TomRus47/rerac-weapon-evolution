//! One visual style for weapon and Nanotech progress. Alpha is supplied by each
//! owner's visibility envelope; Nanotech opacity never affects the weapon HUD.
use super::Draw;

/// Keep the complete border above the native health panel at every supported
/// weapon-bar height. The top HUD margin fits at most twelve inner pixels.
pub fn nanotech_height(requested:i32)->i32 {
    requested.clamp(2,super::HEALTH_Y+super::ELEMENT_Y+super::BAR_DY-4)
}

pub fn draws(left:i32, top:i32, outer_width:i32, height:i32, fraction:f32, alpha:u32, maxed:bool) -> Vec<Draw> {
    let alpha=alpha.min(128);
    if alpha==0 || outer_width<3 || height<1 {return Vec::new();}
    let right=left+outer_width;
    let width=outer_width-2;
    let rect=|x0,y0,x1,y1,rgba|Draw::Rect16{x0:x0*16,y0:y0*16,x1:x1*16,y1:y1*16,rgba};
    let mut out=vec![
        rect(left,top-1,right,top+height+1,((alpha*3/4)<<24)|0x00202020),
        rect(left+1,top,right-1,top+height,(alpha<<24)|0x00605038),
    ];
    let fraction=if fraction.is_finite(){fraction.clamp(0.,1.)}else{0.};
    let fill=(width as f32*fraction).round() as i32;
    if fill>0 {
        let color=if maxed{0x0020_c6_f1}else{0x00e5_91_28};
        out.push(rect(left+1,top,left+1+fill,top+height,(alpha<<24)|color));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hud::{Call,HudAssets,HudState,Inputs};
    fn hud()->HudState {
        HudState::new(HudAssets{icons:vec![],frame_sizes:vec![(32,32)],
            glyphs:[[rc_formats::font::Glyph::default();rc_formats::font::GLYPHS];3],messages:vec![]})
    }
    fn input(hp:i32,max_hp:i32)->Inputs {Inputs{hp,max_hp,..Default::default()}}
    #[test]
    fn border_fill_and_alpha_match_the_weapon_style_and_remain_inside_the_span() {
        let out=draws(100,8,120,5,0.5,64,false);
        assert_eq!(out.len(),3);
        let rect=|i|match out[i]{Draw::Rect16{x0,y0,x1,y1,rgba}=>(x0/16,y0/16,x1/16,y1/16,rgba),_=>panic!()};
        assert_eq!(rect(0),(100,7,220,14,0x30202020));
        assert_eq!(rect(1),(101,8,219,13,0x40605038));
        assert_eq!(rect(2),(101,8,160,13,0x40e59128));
        assert!(draws(100,8,120,5,1.,0,false).is_empty());
        assert_eq!(draws(100,8,120,5,0.,128,false).len(),2);
    }

    #[test]
    fn damage_healing_l3_and_page_close_do_not_arm_the_progress_bar() {
        let mut h=hud();
        for hp in [4,3,2,4,1,4] {
            h.apply_calls(&[Call::ShowHealthBolts(180)]);
            for _ in 0..30 {
                h.tick(input(hp,4));
                assert!(h.nanotech_xp_anchor(10,5).is_none());
            }
        }
    }

    #[test]
    fn ring_lease_matches_health_fade_and_cannot_reappear_after_expiry_or_reset() {
        let mut h=hud();h.tick(input(4,4));
        h.apply_calls(&[Call::QuickSelectHealth(180)]);
        let mut visible=0;
        for tick in 0..400 {
            h.tick(input(4,4));
            if let Some((_,_,_,fade))=h.nanotech_xp_anchor(10,5) {
                visible+=1;
                assert_eq!(fade,h.slots[1].slide as f32/8.,"tick {tick}");
            }
        }
        assert!(visible>180);
        h.apply_calls(&[Call::ShowHealthBolts(180)]);
        for _ in 0..40 {h.tick(input(3,4));assert!(h.nanotech_xp_anchor(10,5).is_none());}
        h.apply_calls(&[Call::QuickSelectHealth(180)]);
        h.tick(input(3,4));assert!(h.nanotech_xp_anchor(10,5).is_some());
        h.reset_slots();
        for _ in 0..40 {h.tick(input(3,4));assert!(h.nanotech_xp_anchor(10,5).is_none());}
    }

    #[test]
    fn width_and_position_are_fixed_for_every_health_cap_and_screen_width() {
        let mut expected=None;
        for extra in [0,85,200] {
            for max_hp in [4,5,8,9,20,21,40,64] {
                let mut h=hud();h.side_extra=extra;
                h.apply_calls(&[Call::QuickSelectHealth(180)]);
                for _ in 0..30 {h.tick(input(max_hp,max_hp));}
                let (x,y,w,fade)=h.nanotech_xp_anchor(10,5).unwrap();
                assert_eq!(x+w/2,256);assert_eq!(fade,1.);
                if let Some(old)=expected {assert_eq!((x,y,w),old);}else{expected=Some((x,y,w));}
                assert_eq!(y+5+1,super::super::HEALTH_Y+super::super::ELEMENT_Y+super::super::BAR_DY-2);
            }
        }
    }

    #[test]
    fn supported_bar_heights_cannot_cross_the_top_edge_or_health_panel() {
        let mut h=hud();h.apply_calls(&[Call::QuickSelectHealth(180)]);
        for _ in 0..30 {h.tick(input(20,20));}
        for requested in 2..=16 {
            let height=nanotech_height(requested);
            let (_,top,_,_)=h.nanotech_xp_anchor(10,height).unwrap();
            assert!(top-1>=0);
            assert_eq!(top+height+1,super::super::HEALTH_Y+super::super::ELEMENT_Y+super::super::BAR_DY-2);
        }
    }

    #[test]
    fn death_clears_the_ring_lease_without_waiting_for_a_level_reload() {
        let mut h=hud();h.apply_calls(&[Call::QuickSelectHealth(180)]);
        for _ in 0..30 {h.tick(input(4,4));}
        assert!(h.nanotech_xp_anchor(10,5).is_some());
        h.tick(input(0,4));h.tick(input(4,4));
        assert!(h.nanotech_xp_anchor(10,5).is_none());
    }

    #[test]
    fn ratchets_disguise_keeps_the_ring_bar_but_clanks_command_menu_does_not() {
        let mut h=hud();h.apply_calls(&[Call::QuickSelectHealth(180)]);
        let disguised=Inputs{body:3,..input(4,4)};
        for _ in 0..30 {h.tick(disguised.clone());}
        assert!(h.nanotech_xp_anchor(10,5).is_some());
        h.tick(Inputs{body:1,..input(4,4)});
        assert!(h.nanotech_xp_anchor(10,5).is_none());
    }
}

/// Full-opacity BBGGRR colors, shared with the native ammunition text palette.
pub fn weapon_name_rgb(grade:usize)->u32 {if grade>=8{0x00ff_ffff}else if grade>=5{0x0000_ffff}else{0x00e0_8060}}
/// Keep the name at its original top edge; alternative ammo has moved down.
pub fn weapon_rows(ammo_y:i32,height:i32,alternative:bool)->(i32,i32){
    if alternative {(ammo_y-18-height.clamp(2,16)-4,ammo_y-height.clamp(2,16)-2)}
    else {(ammo_y-18,ammo_y+34)}
}
#[cfg(test)]
mod v102_tests {
    use super::*;
    #[test]fn alternate_bar_is_between_name_and_ammo_at_all_heights(){
        for h in 2..=16 {let ammo=18+h+4;let(name,bar)=weapon_rows(ammo,h,true);
            assert_eq!(name,0);assert!(name+16<bar);assert!(bar+h<ammo);
            assert_eq!(weapon_rows(18,h,false),(0,52));}
    }
    #[test]fn native_indigo_then_gold_then_white(){
        for n in 1..=4{assert_eq!(weapon_name_rgb(n),0x00e08060);}
        for n in 5..=7{assert_eq!(weapon_name_rgb(n),0x0000ffff);}
        assert_eq!(weapon_name_rgb(8),0x00ffffff);
    }
}
