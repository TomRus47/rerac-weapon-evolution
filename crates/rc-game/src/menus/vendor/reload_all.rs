//! One atomic, shop-priced refill. A quote is rebuilt at the button edge and
//! never relies on the selected carousel entry or on a previously drawn price.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReloadAllQuote {
    pub cost: i64,
    /// Item, active maximum, missing units. No ownership changes are included.
    pub items: Vec<(usize, i32, i32)>,
}

impl Vendor {
    pub fn reload_all_quote(&self, gs:&GameState) -> ReloadAllQuote {
        let mut quote=ReloadAllQuote {cost:0,items:Vec::new()};
        let golden=gs.weapon_progress.golden;
        if self.remote || (!golden.active() && crate::weapon_evo::ammo_mode::enabled()) {return quote;}
        for item in 0..ITEMS {
            if gs.global.owned[item]==0 {continue;}
            if golden.active() && !golden.collected(item) {continue;}
            let unit=self.ammo_unit(item);
            let max=i32::from(self.tables.shop.max_ammo(item));
            if unit<=0 || max<=0 {continue;}
            let missing=(max-gs.global.ammo[item].max(0)).max(0);
            if missing==0 {continue;}
            quote.cost+=i64::from(unit)*i64::from(missing);
            quote.items.push((item,max,missing));
        }
        quote
    }

    /// Affordability is checked before the first write. A failed transaction
    /// cannot partially fill a weapon or spend a single bolt.
    pub(super) fn reload_all(&mut self, gs:&mut GameState, assets:&MenuAssets, out:&mut VendorOut) {
        let quote=self.reload_all_quote(gs);
        if quote.items.is_empty() {out.sounds.push(sound::SELECT);return;}
        if quote.cost>i64::from(gs.global.bolts) {
            self.set_ticker(assets.msg(msg::CANT_AFFORD_TICKER));
            out.sounds.push(sound::DENIED);
            return;
        }
        for (item,max,missing) in quote.items {
            gs.global.ammo[item]=max;
            gs.global.ammo_bought[item]=gs.global.ammo_bought[item].saturating_add(missing);
        }
        gs.global.bolts-=quote.cost as i32;
        out.selection_changed=true;
        self.set_ticker(assets.msg(msg::THANK_YOU));
        out.sounds.push(sound::PURCHASE);
    }

    /// Overlay immediately above the projected carousel frame. Logical game
    /// pixels keep it aligned in NTSC/PAL and widescreen without screen guesses.
    pub fn reload_all_draws(&self, a:&MenuAssets, gs:&GameState, strip:[f32;4]) -> Vec<MenuDraw> {
        let mut out=Vec::new();
        if self.sub!=1 || self.buy!=0 || self.exit_req || self.remote
            || (!gs.weapon_progress.golden.active() && crate::weapon_evo::ammo_mode::enabled()) {return out;}
        let quote=self.reload_all_quote(gs);
        let label=match self.lang {
            2=>"Acheter toutes les munitions",3=>"Alle Munition auffuellen",4=>"Comprar toda la municion",5=>"Compra tutte le munizioni",
            _=>"Buy all ammo",
        };
        let cost=grouped_price(quote.cost);
        let caption=format!("{label} - {cost}").into_bytes();
        let width=(a.width(Font::Small,&caption)+52).clamp(120,488);
        let x=((strip[0]+strip[2]*0.5) as i32-width/2).clamp(8,504-width);
        let y=(strip[1] as i32-23).clamp(8,389);
        // Text/button remain fully readable. Only the panel uses translucency;
        // this hint is already omitted once the shop starts closing.
        let alpha=128u32;
        let middle=a.frame(crate::hud::icon::BAR,0);
        let cap=a.frame(crate::hud::icon::BAR,1);
        if a.frame_size(middle).0>0 && a.frame_size(cap).0>0 {
            // Reuse the actual ammo/name vignette, including its mirrored cap
            // and the name panel's alpha, rather than approximate its texture.
            for (frame,px,w,rot) in [(cap,x,10,crate::hud::Rot::None),
                (middle,x+10,width-20,crate::hud::Rot::None),(cap,x+width-10,10,crate::hud::Rot::R180)] {
                out.push(MenuDraw::Hud(crate::hud::Draw::Sprite {frame,x:px,y,w,h:20,alpha:(alpha*7/10)as i32,rot}));
            }
        } else {
            out.push(MenuDraw::Rect {x0:x+1,y0:y+1,x1:x+width+1,y1:y+21,rgba:(alpha*7/10)<<24});
        }
        // A self-contained Square button outline; does not assume an atlas
        // frame number that changes between regional HUD texture tables.
        let pink=(alpha<<24)|0x00dc_60dc;
        for (x0,y0,x1,y1) in [(x+9,y+5,x+18,y+6),(x+9,y+13,x+18,y+14),
            (x+9,y+5,x+10,y+14),(x+17,y+5,x+18,y+14)] {
            out.push(MenuDraw::Rect {x0:x0+1,y0:y0+1,x1:x1+1,y1:y1+1,rgba:pink});
        }
        let rgb=if quote.cost>i64::from(gs.global.bolts) {0x00606060}
            else {0x00ffffff};
        text(&mut out,Font::Small,x+27,y+4,alpha<<24,&caption);
        text(&mut out,Font::Small,x+26,y+3,(alpha<<24)|rgb,&caption);
        let bolt=a.frame(crate::hud::icon::BOLT,0);
        if a.frame_size(bolt).0>0 {crate::menus::sprite(&mut out,bolt,x+width-20,y+4,10,12,alpha as i32);}
        out
    }
}

fn grouped_price(cost:i64)->String {
    let s=cost.to_string();
    s.chars().enumerate().flat_map(|(i,c)| {
        let comma=i>0 && (s.len()-i)%3==0;
        comma.then_some(',').into_iter().chain(std::iter::once(c))
    }).collect()
}
