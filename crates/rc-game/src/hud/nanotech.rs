//! Logical HP packing and native-art recoloring. No extra protection or damage
//! system is introduced: one accepted damage point still removes exactly one HP.
pub const MAX_VISIBLE: usize = 30;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Orb { pub current: u8, pub capacity: u8 }
/// Extra HP belongs to the first orbs; damage drains the last occupied orb.
/// Therefore a 31-HP bar ends with one purple two-HP orb, then one blue HP.
pub fn orbs(maximum: i32, current: i32) -> Vec<Orb> {
    let maximum=maximum.clamp(1,crate::weapon_evo::world::MAX_NANOTECH) as usize;
    let count=maximum.min(MAX_VISIBLE);
    let mut remaining=current.clamp(0,maximum as i32) as usize;
    (0..count).map(|i| {
        let capacity=maximum/count+usize::from(i<maximum%count);
        let used=remaining.min(capacity); remaining-=used;
        Orb {current:used as u8,capacity:capacity as u8}
    }).collect()
}
/// Native blue at one HP. The seven layers end at white.
pub fn color(layer: u8) -> Option<[u8;3]> {
    match layer {2=>Some([192,64,255]),3=>Some([255,48,48]),
        4=>Some([255,152,32]),5=>Some([255,229,48]),6=>Some([92,255,69]),7=>Some([255,255,255]),_=>None}
}
/// Recolor existing vortex/shell texels without changing alpha, shape or white
/// highlights. Multiplication alone cannot turn blue artwork into bright red.
pub fn recolor(pixel: &mut [u8], layer: u8) {
    let Some(rgb)=color(layer) else{return;};
    if pixel.len()!=4 {return;}
    let low=*pixel[..3].iter().min().unwrap() as u16;
    let high=*pixel[..3].iter().max().unwrap() as u16;
    for c in 0..3 {pixel[c]=(low+(high-low)*rgb[c] as u16/255) as u8;}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn every_hp_point_is_represented_exactly_once_at_every_maximum() {
        for maximum in 1..=210 {for current in 0..=maximum {
            let a=orbs(maximum,current);
            assert_eq!(a.len(),(maximum as usize).min(30));
            assert_eq!(a.iter().map(|o|o.current as i32).sum::<i32>(),current);
            assert_eq!(a.iter().map(|o|o.capacity as i32).sum::<i32>(),maximum);
            assert!(a.iter().all(|o|o.current<=o.capacity&&o.capacity<=7));
        }}
    }
    #[test] fn extra_hp_stays_in_the_first_orb_and_reverts_color_on_damage() {
        let full=orbs(31,31);assert_eq!(full[0],Orb{current:2,capacity:2});
        assert!(full[1..].iter().all(|o|o.current==1));
        assert_eq!(orbs(31,2)[0].current,2);assert!(color(2).is_some());
        assert_eq!(orbs(31,1)[0].current,1);assert_eq!(color(1),None);
        for (max,layer) in [(30,1),(60,2),(90,3),(120,4),(150,5),(180,6),(210,7)] {
            assert!(orbs(max,max).iter().all(|o|o.current==layer));
        }
        assert_eq!(orbs(500,500),orbs(210,210));
    }
    #[test] fn recoloring_preserves_native_alpha_and_white_highlights() {
        for layer in 2..=7 {
            let mut blue=[0,0,255,91];recolor(&mut blue,layer);
            assert_eq!(&blue[..3],&color(layer).unwrap());assert_eq!(blue[3],91);
            let mut white=[255,255,255,128];recolor(&mut white,layer);
            assert_eq!(white,[255,255,255,128]);
        }
    }
}
