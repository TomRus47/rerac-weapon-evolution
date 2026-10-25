//! A compact Weapons-page hint outside panel render targets. Gadget grids show
//! their assigned borders too, but deliberately have no extra text header.
use super::{Data,PageMenu};
use crate::menus::{text,MenuAssets,MenuDraw};
use rc_formats::font::Font;

fn caption(lang:u32)->&'static [u8] {
    match lang {2=>b"Associer",3=>b"Zuweisen",4=>b"Asignar",5=>b"Assegna",_=>b"Assign"}
}
fn dpad(out:&mut Vec<MenuDraw>,x:i32,y:i32,color:u32){
    // The native D-pad silhouette: an unfilled cross, independent of language
    // and a font bank's optional control glyphs. L1 uses the native font glyph.
    let points=[[3,0],[8,0],[8,3],[11,3],[11,8],[8,8],[8,11],[3,11],[3,8],[0,8],[0,3],[3,3],[3,0]];
    for pair in points.windows(2){out.push(MenuDraw::Line{x0:x+pair[0][0],y0:y+pair[0][1],x1:x+pair[1][0],y1:y+pair[1][1],rgba:color});}
}
pub(super) fn draw(menu:&PageMenu,assets:&MenuAssets,out:&mut Vec<MenuDraw>){
    if menu.kind!=3 || !crate::weapon_evo::controls::settings().dpad{return;}
    let Some(page)=menu.pages.get(&menu.current) else{return;};
    // Data::Ammo identifies the real Weapons page, rather than guessing a
    // translated title or a level-specific page address.
    if !page.widgets.iter().filter_map(|id|menu.widgets.get(id)).any(|w|matches!(w.data,Data::Ammo)){return;}
    let Some(grid)=page.widgets.iter().filter_map(|id|menu.widgets.get(id)).find(|w|matches!(w.data,Data::Grid(_))) else{return;};
    let [gx,gy,gw,_]=grid.rect;
    let top=page.widgets.iter().filter_map(|id|menu.widgets.get(id)).map(|w|w.rect)
        .filter(|r|r[2]>0 && r[3]>0 && r[1]+r[3]<=gy && r[0]<gx+gw && r[0]+r[2]>gx)
        .map(|r|r[1]+r[3]).max();
    let Some(top)=top.filter(|&top|gy-top>=12) else{return;};
    let prefix=b"\x14 + ";let label=caption(menu.lang);
    let width=assets.width(Font::Regular,prefix)+16+assets.width(Font::Regular,label);
    let font=if width<=gw-8{Font::Regular}else{Font::Small};
    let prefix_width=assets.width(font,prefix);
    let width=prefix_width+16+assets.width(font,label);
    let x=gx+(gw-width)/2;let y=top+(gy-top-12)/2;
    let color=super::LIGHT_BLUE;
    text(out,font,x,y,color,prefix);dpad(out,x+prefix_width,y,color);
    text(out,font,x+prefix_width+16,y,color,label);
}
