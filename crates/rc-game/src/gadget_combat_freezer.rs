//! Freezer ammunition uses a simulation clock, not weapon XP or a shop resource.
//! The HUD reads the same capacity/elapsed state that grants recovered shots.
use super::Settings;

#[derive(Clone,Copy,Debug,Default,PartialEq)]
pub struct Hud {pub ammo:u16,pub maximum:u16,pub fraction:f32}
impl Hud {pub fn full(self)->bool {self.ammo>=self.maximum}}

#[derive(Clone,Debug,Default)]
pub struct Recharge {elapsed:u32,last:Option<u64>}
impl Recharge {
    pub fn elapsed(&self)->u32 {self.elapsed}
    pub fn restore(&mut self,elapsed:u32) {self.elapsed=elapsed;self.last=None;}
    fn duration(settings:Settings)->u32 {(settings.recharge_seconds*60.).ceil().max(1.) as u32}
    pub fn hud(&self,ammo:u16,settings:Settings)->Hud {
        let maximum=settings.ice_ammo;
        Hud {ammo:ammo.min(maximum),maximum,fraction:if ammo>=maximum {1.} else {
            (self.elapsed as f32/Self::duration(settings) as f32).clamp(0.,1.)}}
    }
    /// One step per distinct simulation tick. No catch-up from real time;
    /// pauses/death/disabled combat cannot bank recharge at full capacity.
    pub fn tick(&mut self,ammo:&mut Option<u16>,settings:Settings,now:u64,active:bool) {
        if self.last==Some(now) {return;}
        self.last=Some(now);
        let Some(ammo)=ammo else {return};
        *ammo=(*ammo).min(settings.ice_ammo);
        if *ammo>=settings.ice_ammo {self.elapsed=0;return;}
        if !active||!settings.ice_enabled {return;}
        self.elapsed=self.elapsed.saturating_add(1);
        if self.elapsed>=Self::duration(settings) {
            *ammo=(*ammo+1).min(settings.ice_ammo);self.elapsed=0;
        }
    }
}

/// Game language IDs differ from launcher locale codes. Captions use native
/// font bytes, not UTF-8 sequences which would select unrelated glyphs.
pub fn name(language:u32)->&'static [u8] {
    use rc_formats::strings::lang;
    match language {
        lang::ITALIAN=>b"Congelatore",lang::FRENCH=>b"Glaceur",
        lang::GERMAN=>b"Froster",lang::SPANISH=>b"Congelador",_=>b"Freezer",
    }
}

/// A reusable soft alpha mask independent of level-local particle definitions.
/// RGB uses neutral GS 128; the existing FX shader interprets alpha in GS units.
pub fn mist_texture()->Vec<u8> {
    let mut pixels=Vec::with_capacity(32*32*4);
    for y in 0..32 {for x in 0..32 {
        let nx=(x as f32+0.5)/16.-1.;let ny=(y as f32+0.5)/16.-1.;
        let a=(1.-nx*nx-ny*ny).max(0.).powi(2);
        pixels.extend_from_slice(&[128,128,128,(a*128.).round() as u8]);
    }}pixels
}
