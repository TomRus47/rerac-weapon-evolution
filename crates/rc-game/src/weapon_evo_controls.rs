//! Optional saved shortcuts and manual weapon modes. The INI supplies tuning;
//! this state stores only player choices in the existing per-slot XP companion.
use crate::{game_state::{GameState, SessionState}, hero::Hero, inventory::{self, Slot}, pad::{button, PadState}, ps2v::Pf};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8,Ordering};

pub const DPAD_KEY:&str="dpad_quick_select";
pub const R1_KEY:&str="r1_experimental";
pub const DIRECTIONS:[u32;4]=[button::UP,button::RIGHT,button::DOWN,button::LEFT];
pub const DIRECTION_MASK:u32=button::UP|button::RIGHT|button::DOWN|button::LEFT;
pub const COLORS:[u32;4]=[0x8000ff00,0x800000ff,0x80ff0000,0x80d050ff];
/// Mode 0 is native, modes 1/2 select mutually exclusive replacement behaviors.
/// Optional improvements (triple flame, cluster bombs, dual Tesla, crates and
/// companion drones) deliberately remain controlled exclusively by the INI.
pub const MODE_ITEMS:[i32;4]=[16,17,13,23];
pub const MODE_MAX:[u8;4]=[1,1,1,2];
/// Native backpack order: Heli-Pack, Thruster-Pack, Hydro-Pack.
const CLANK_PACKS:[i32;3]=[2,3,4];
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Settings {pub dpad:bool,pub r1:bool}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct State {pub dpad:[u8;4],pub modes:[u8;4]}
impl State {
    pub fn valid(&self)->bool {
        self.dpad.iter().all(|&id|id<=32) && self.modes.iter().zip(MODE_MAX).all(|(&value,max)|value<=max)
            && self.dpad.iter().enumerate().all(|(i,&id)|id==0 || !self.dpad[..i].contains(&id))
    }
    pub fn mode(&self,item:i32)->Option<u8>{MODE_ITEMS.iter().position(|&id|id==item).map(|i|self.modes[i])}
    pub fn cycle(&mut self,item:i32)->Option<u8>{let i=MODE_ITEMS.iter().position(|&id|id==item)?;self.modes[i]=(self.modes[i]+1)%(MODE_MAX[i]+1);Some(self.modes[i])}
    pub fn assign(&mut self,slot:usize,item:i32,owned:bool,item_slot:i32)->bool {
        if slot>=4 || !(1..=32).contains(&item) || !owned || Slot::from_type(item_slot).is_none(){return false;}
        let id=item as u8;
        if self.dpad[slot]==id{return false;}
        for old in &mut self.dpad {if *old==id{*old=0;}}
        self.dpad[slot]=id;true
    }
}
/// Physical buttons only. Analog directions must never trigger a binding.
/// Simultaneous/diagonal presses are ignored rather than equipping two items.
pub fn direction(raw_pressed:u32)->Option<usize>{let mask=raw_pressed&DIRECTION_MASK;(mask.count_ones()==1).then(||DIRECTIONS.iter().position(|&b|b==mask)).flatten()}
static FLAGS:AtomicU8=AtomicU8::new(0);
pub(super) fn publish(settings:Settings){FLAGS.store(1|u8::from(settings.dpad)*2|u8::from(settings.r1)*4,Ordering::Relaxed);}
pub(super) fn disable(){FLAGS.store(0,Ordering::Relaxed);}
pub fn settings()->Settings{let f=FLAGS.load(Ordering::Relaxed);Settings{dpad:f&2!=0,r1:f&4!=0}}
pub fn active()->bool{FLAGS.load(Ordering::Relaxed)&1!=0}

/// Native hold/vehicle/alternate-body states retain complete control of input.
/// Standing still is direct control too. Wearing Grindboots does not block it.
pub fn direct_control(h:&Hero)->bool {
    input_layout_allowed(h) && h.f13ff==0 && h.items.f13fc==0
}
/// Keep native vehicle, turret, alternate-body and scripted inputs unchanged.
/// Weapon extraction can temporarily block equipment changes without changing
/// which physical button fires or crouches midway through a held trigger.
pub fn input_layout_allowed(h:&Hero)->bool {
    h.mode==0 && h.health>0 && h.frozen==0 && h.gadgets.game_mode==0
        && matches!(h.state,0..=9|0xb..=0x16|0x1e|0x23|0x28..=0x2b|0x2f|0x33..=0x37|0x3f|0x42|0x6a|0x70|0x71|0x73)
}
pub fn can_equip(h:&Hero,level:i32,item_slot:i32)->bool {
    if !direct_control(h){return false;}
    match item_slot {
        0=>!super::golden::is_swimming(h) && h.gravity_mode==0 && h.f658==0 && !matches!(h.state,0x3f|0x70|0x71),
        1=>h.gravity_mode==0 && h.f658==0 && !matches!(h.state,0x28..=0x2b|0x42),
        2=>level!=13,
        3=>!matches!(level,0|14),
        _=>false,
    }
}
/// Use the ordinary session requests; never grant items, ammo or progression.
pub fn equip(state:&State,raw_pressed:u32,gs:&GameState,sess:&mut SessionState,h:&Hero,slot_of:impl Fn(i32)->Option<i32>)->bool {
    let Some(dir)=direction(raw_pressed) else{return false;};
    let item=i32::from(state.dpad[dir]);
    if item==0 || !inventory::owns(gs,item){return false;}
    let Some(item_slot)=slot_of(item) else{return false;};
    if !can_equip(h,gs.global.level,item_slot){return false;}
    if item==24 {sess.shortcut_back_cycle=None;sess.drone=true;return true;}
    let Some(slot)=Slot::from_type(item_slot) else{return false;};
    // A second physical press can arrive before a worn-item animation commits.
    // Respect the pending request first, including the native removal sentinel.
    let pending=inventory::request(sess,slot);
    let current=match pending {
        0=>inventory::equipped(gs,slot),
        inventory::NOTHING=>0,
        other=>other,
    };
    let requested=match slot {
        Slot::Feet|Slot::Head=> {
            sess.shortcut_back_cycle=None;
            if current==item {inventory::NOTHING}else{item}
        }
        Slot::Back if CLANK_PACKS.contains(&item)=> {
            // First press selects the assigned pack. Repeating that binding
            // cycles Heli/Thruster/Hydro, skipping unowned or unavailable packs.
            // An external equipment change restarts at the assigned item.
            let repeating=sess.shortcut_back_cycle==Some((dir,item,current));
            let next=if repeating {
                let start=CLANK_PACKS.iter().position(|&id|id==current).unwrap_or(0);
                (1..=CLANK_PACKS.len()).map(|step|CLANK_PACKS[(start+step)%CLANK_PACKS.len()])
                    .find(|&id|inventory::owns(gs,id) && slot_of(id)==Some(3))
                    .unwrap_or(item)
            }else{item};
            sess.shortcut_back_cycle=Some((dir,item,next));
            next
        }
        _=>{sess.shortcut_back_cycle=None;item}
    };
    inventory::set_request(sess,slot,requested);true
}
/// Replace exactly four behavior switches. Every tuning parameter and every
/// excluded weapon enhancement stays at the current grade's original INI value.
pub(super) fn apply_registry(settings:Settings,state:&State,registry:&mut HashMap<(i32,String),f32>){
    if !settings.r1{return;}
    for (item,key,on) in [(17,"flame_mine_enabled",state.mode(17)==Some(1)),
        (13,"grenade_launcher_enabled",state.mode(13)==Some(1)),
        (23,"continuous_missiles_enabled",state.mode(23)==Some(1)),
        (23,"shotgun_enabled",state.mode(23)==Some(2))] {
        registry.insert((item,key.into()),f32::from(u8::from(on)));
    }
}

/// Gameplay-only semantic pad. Physical R2 and Circle are one logical trigger;
/// L1 and R1 share native crouch. Menus/cameras keep the physical pad.
pub fn gameplay_pad(pad:&PadState,item:i32,enabled:bool,dpad:bool,locked:bool)->PadState {
    if !enabled{return pad.clone();}
    let fire=if item==8{button::SQUARE}else{button::CIRCLE};
    let consumed=button::L1|button::R1|button::R2|fire|if locked{button::L2}else{0};
    let alias=|mask:u32| {
        let mut out=mask&!(button::L1|button::R1|button::R2|if locked{button::L2}else{0});
        if mask&(button::L1|button::R1)!=0{out|=button::R1;}
        if mask&button::R2!=0{out|=fire;}
        out
    };
    let mut out=pad.clone();
    out.held=alias(pad.held);out.prev_held=alias(pad.prev_held);out.held_store=alias(pad.held_store);
    out.raw=alias(pad.raw);out.prev_raw=alias(pad.prev_raw);
    let logical=fire|button::R1;
    out.pressed=(pad.pressed&!consumed)|(out.held&!out.prev_held&logical);
    out.released=(pad.released&!consumed)|(out.prev_held&!out.held&logical);
    out.raw_pressed=(pad.raw_pressed&!consumed)|(out.raw&!out.prev_raw&logical);
    out.raw_released=(pad.raw_released&!consumed)|(out.prev_raw&!out.raw&logical);
    out.held_unmirrored=alias(pad.held_unmirrored);
    out.pressed_unmirrored=(pad.pressed_unmirrored&!consumed)|(out.held_unmirrored&!out.prev_held&logical);
    out.released_unmirrored=(pad.released_unmirrored&!consumed)|(out.prev_held&!out.held_unmirrored&logical);
    for mask in &mut out.hist_pressed{*mask=alias(*mask);}
    if out.hist_count>0 {
        let newest=(out.hist_index as usize+crate::pad::HISTORY-1)%crate::pad::HISTORY;
        out.hist_pressed[newest]=(out.hist_pressed[newest]&!logical)|(out.pressed&logical);
    }
    let pressure=if item==8{7}else{5};
    out.pressure[pressure]=out.pressure[pressure].max(pad.pressure[11]);
    out.analog_copy[pressure+4]=out.analog_copy[pressure+4].max(pad.analog_copy[15]);
    out.pressure[9]=pad.pressure[8].max(pad.pressure[9]);out.analog_copy[13]=pad.analog_copy[12].max(pad.analog_copy[13]);
    out.pressure[8]=Pf::ZERO;out.pressure[11]=Pf::ZERO;
    out.analog_copy[12]=Pf::ZERO;out.analog_copy[15]=Pf::ZERO;
    if dpad {
        let stick=(if pad.lx<Pf::ZERO{button::LEFT}else if pad.lx>Pf::ZERO{button::RIGHT}else{0})
            |(if pad.ly<Pf::ZERO{button::UP}else if pad.ly>Pf::ZERO{button::DOWN}else{0});
        out.held=(out.held&!DIRECTION_MASK)|stick;
        out.prev_held&=!DIRECTION_MASK;
        out.pressed&=!DIRECTION_MASK;out.released&=!DIRECTION_MASK;
        out.held_store=(out.held_store&!DIRECTION_MASK)|stick;
        out.raw&=!DIRECTION_MASK;out.prev_raw&=!DIRECTION_MASK;
        out.raw_pressed&=!DIRECTION_MASK;out.raw_released&=!DIRECTION_MASK;
        out.held_unmirrored&=!DIRECTION_MASK;out.pressed_unmirrored&=!DIRECTION_MASK;out.released_unmirrored&=!DIRECTION_MASK;
        for mask in &mut out.hist_pressed{*mask&=!DIRECTION_MASK;}
    }
    out.nothing_held=out.held==0;out.no_direction=out.held&DIRECTION_MASK==0;
    out
}

/// The native raw_pressed compares with stick-merged prev_held. Compare the two
/// raw button snapshots instead, so a held analog direction cannot mask a D-pad
/// press. Binding names remain physical directions even with the mirror cheat.
pub fn physical_edges(pad:&PadState,mirror:bool)->u32 {
    if let Some((current,previous))=pad.physical_buttons{return current&!previous;}
    // Synthetic pads used by native ports/tests may not have a device snapshot.
    let edge=pad.raw&!pad.prev_raw;
    if !mirror{return edge;}
    (edge&!(button::LEFT|button::RIGHT))|((edge&button::LEFT)>>2)|((edge&button::RIGHT)<<2)
}

#[cfg(test)]
#[path="weapon_evo_controls_tests.rs"]
mod tests;
