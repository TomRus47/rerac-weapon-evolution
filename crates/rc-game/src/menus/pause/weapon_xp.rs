//! Screen-space progress under the Weapons page's localized name/grade panel.
use crate::{menus::MenuDraw, weapon_evo::Evolution};

/// Port-only label draw metadata; the native panel-blit bits remain unchanged.
pub(super) const NAME_PANEL: u32 = 0x20;

/// A gold flag is valid for all evolving weapons, but the disc has gold-name
/// rows for only ten. Identify the name table by its ordinary row; never repair
/// descriptions, ammo text or unrelated labels by looking at fallback text.
/// Keep localized disc bytes verbatim, including the translated string bank.
pub(super) fn name_id<'a>(ordinary_row: u32, resolved: u32, native: i32, gold: i32,
    message: impl Fn(i32) -> &'a [u8]) -> u32 {
    if native <= 0 || ordinary_row != native as u32 { return resolved; }
    if gold > 0 && resolved == gold as u32 {
        let text = message(gold);
        if !text.is_empty() && text != rc_formats::strings::MISSING_MESSAGE { return resolved; }
    }
    native as u32
}

pub(super) fn draw(out: &mut Vec<MenuDraw>, [x, y, width, panel_height]: [i32; 4],
    evolution: &Evolution, item: i32) {
    let (Some(fraction), Some(_level)) = (evolution.weapon_fraction(item), evolution.weapon_level(item)) else { return };
    if width < 3 || panel_height < 1 { return; }
    // MenuDraw::Rect subtracts one pixel from each supplied corner. These
    // coordinates put the outer span exactly at x..x+width and y+height.
    let (left, top) = (x + 1, y + panel_height + 1);
    let height = evolution.hud.xp_bar_height_px.clamp(2, 16);
    out.push(MenuDraw::Rect { x0: left, y0: top, x1: left + width, y1: top + height + 2, rgba: 0x6020_2020 });
    out.push(MenuDraw::Rect { x0: left + 1, y0: top + 1, x1: left + width - 1, y1: top + height + 1, rgba: 0x8060_5038 });
    let fill = ((width - 2) as f32 * fraction.clamp(0., 1.)).round() as i32;
    if fill > 0 {
        out.push(MenuDraw::Rect { x0: left + 1, y0: top + 1, x1: left + 1 + fill,
            y1: top + height + 1, rgba: if evolution.weapon_at_cap(item) { 0x8020_c6f1 } else { 0x80e5_9128 } });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rects(out: &[MenuDraw]) -> Vec<[i32; 4]> {
        out.iter().filter_map(|d| if let MenuDraw::Rect { x0, y0, x1, y1, .. } = d {
            Some([*x0, *y0, *x1, *y1])
        } else { None }).collect()
    }
    #[test]
    fn progress_is_below_and_exactly_as_wide_as_the_moving_name_panel() {
        let mut e = Evolution::default(); e.enabled = true; e.xp = 25.; e.xp_required = [100.; 7];
        for panel in [[40, 30, 240, 24], [180, 90, 92, 40]] {
            let mut out = vec![]; draw(&mut out, panel, &e, 10); let r = rects(&out);
            assert_eq!(r.len(), 3);
            assert_eq!(r[0][2] - r[0][0], panel[2]);
            assert_eq!(r[0][0] - 1, panel[0]);
            assert_eq!(r[0][1] - 1, panel[1] + panel[3]);
            assert_eq!(r[2][2] - r[2][0], ((panel[2] - 2) as f32 * 0.25).round() as i32);
        }
    }
    #[test]
    fn selection_uses_independent_xp_and_max_grade_fills_the_bar() {
        let mut e = Evolution::default(); e.enabled = true; e.xp = 0.;
        e.pyro.enabled = true; e.pyro.level = 5;
        for (item, count) in [(10, 2), (16, 3), (8, 0)] {
            let mut out = vec![]; draw(&mut out, [0, 0, 102, 20], &e, item);
            let r = rects(&out); assert_eq!(r.len(), count);
            if item == 16 { assert_eq!(r[2][2] - r[2][0], 100); }
        }
        e.enabled = false;
        let mut out = vec![]; draw(&mut out, [0, 0, 102, 20], &e, 10); assert!(out.is_empty());
    }
}

#[cfg(test)]
mod v130_names {
    use super::*;
    #[test]
    fn goldless_names_keep_localized_bytes_and_xp_for_all_high_grades() {
        use crate::weapon_evo::{save::{IDS,Progress},Evolution};
        for text in [b"Drone Device".as_slice(),b"Scudo drone",b"Drone ravageur",b"Drohnen",b"Guante de drones"] {
            for item in IDS { for grade in 6..=8 {
                let lookup=|id|if id==100 {text}else{rc_formats::strings::MISSING_MESSAGE};
                for gold in [0,65535,200] {
                    let id=name_id(100,gold as u32,100,gold,lookup);
                    assert_eq!(id,100);assert_eq!(lookup(id as i32),text);
                }
                assert_eq!(name_id(321,0,100,0,lookup),0,"do not replace descriptions");
                let mut e=Evolution::default();e.enabled=true;e.ng_plus=true;e.level_cap=8;
                e.pyro.enabled=true;for track in &mut e.extras {track.enabled=true;track.level_cap=8;}
                let mut progress=Progress::default();
                progress.tracks[IDS.iter().position(|&id|id==item).unwrap()].level=grade;
                e.restore_progress(&progress);
                let mut draw=vec![];draw_name_xp(&mut draw,&e,item);
                assert!(!draw.is_empty(),"item {item} V{grade} must have a progress panel");
            }}
        }
    }
    fn draw_name_xp(out:&mut Vec<MenuDraw>,e:&Evolution,item:i32) {super::draw(out,[0,0,200,20],e,item);}
    #[test]
    fn real_gold_names_and_translated_bank_are_kept_when_present() {
        let ordinary=b"Inceneritore";let gold=b"Inceneritore d'oro";
        let msg=|id|match id{100=>ordinary.as_slice(),200=>gold.as_slice(),_=>rc_formats::strings::MISSING_MESSAGE};
        assert_eq!(name_id(100,200,100,200,msg),200);
        assert_eq!(name_id(100,0,100,200,msg),100);
    }
}
