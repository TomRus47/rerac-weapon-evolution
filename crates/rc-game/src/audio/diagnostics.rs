//! Optional, bounded audio metadata logging. Set RC_AUDIO_DIAGNOSTICS to a file
//! path before starting the launcher. No samples or game data are written.
use std::{collections::HashSet, fs::{File, OpenOptions}, io::Write, sync::{Mutex, OnceLock}};
use super::{LevelAudio, voices::{Emitters, Listener, SoundSlots, state}};
use crate::moby_update::services::SoundEvent;
use rc_formats::sound_bank::{SoundDef, SoundOwner};

struct Log { file: File, seen: HashSet<String> }
fn log() -> &'static Mutex<Option<Log>> {
    static LOG: OnceLock<Mutex<Option<Log>>> = OnceLock::new();
    LOG.get_or_init(|| Mutex::new(std::env::var_os("RC_AUDIO_DIAGNOSTICS").and_then(|path| {
        match OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => Some(Log { file, seen: HashSet::new() }),
            Err(e) => { eprintln!("audio diagnostics: {e}"); None }
        }
    })))
}
fn enabled() -> bool { log().lock().unwrap_or_else(|e| e.into_inner()).is_some() }
fn once(key: String, text: String) {
    let mut guard = log().lock().unwrap_or_else(|e| e.into_inner());
    let Some(log) = guard.as_mut() else { return };
    if log.seen.len() < 4096 && log.seen.insert(key) { let _ = writeln!(log.file, "{text}"); }
}
fn reason(data: &LevelAudio, def: Option<&SoundDef>, flags: u32, slot: i32) -> &'static str {
    let Some(def) = def else { return "missing sound definition" };
    if (flags & 4 != 0) != (def.looped != 0) { return "loop flag mismatch"; }
    let Some(sound) = data.bank.sounds.get(def.index as usize) else { return "invalid bank sound id" };
    if sound.grains.is_empty() { return "empty bank sound"; }
    if slot < 0 { "slot unavailable or initial distance volume below threshold" } else { "slot allocated" }
}
pub(super) fn inventory(data: &LevelAudio, emitters: &Emitters) {
    if !enabled() { return; }
    // Include the bank identity so a level change produces its own inventory.
    let bank = data.bank.header.bank_id;
    once(format!("bank:{bank}"), format!("BANK {bank} sounds={} emitters={}", data.bank.sounds.len(), emitters.instances.len()));
    for class in [816, 1012, 726, 1150, 1151, 1766] {
        let def = data.sounds.def(SoundOwner::Class(class), 0);
        once(format!("classdef:{bank}:{class}"), format!("CLASS bank={bank} class={class} sound0={def:?} reason={}", reason(data, def, 4, 0)));
    }
    for (i, inst) in emitters.instances.iter().enumerate() {
        let pv = emitters.pvars[i];
        let def = usize::try_from(pv.def).ok().and_then(|i| data.sounds.def(SoundOwner::Level, i));
        once(format!("emitterdef:{bank}:{i}"), format!("EMITTER bank={bank} id={i} class={} position={:?} range={} pvars={pv:?} def={def:?}", inst.o_class, inst.position(), inst.range));
    }
}
pub(super) fn class_play(data: &LevelAudio, ev: &SoundEvent, def: Option<&SoundDef>, slot: i32) {
    if !enabled() { return; }
    let why = reason(data, def, ev.flags, slot);
    once(format!("play:{}:{}:{}:{}:{why}", data.bank.header.bank_id, ev.moby, ev.sound_class, ev.index),
        format!("PLAY tick={} owner={} class={} sound={} flags={} slot={slot} position={:?} def={def:?} reason={why}", ev.tick, ev.moby, ev.sound_class, ev.index, ev.flags, ev.pos));
}
pub(super) fn emitters(data: &LevelAudio, emitters: &Emitters, slots: &SoundSlots, l: &Listener, frame: u32) {
    if frame % 60 != 0 || !enabled() { return; }
    for (i, inst) in emitters.instances.iter().enumerate() {
        if !matches!(inst.o_class, 0 | 1 | 2 | 5) { continue; }
        let pv = emitters.pvars[i];
        let slot = usize::try_from(pv.slot).ok().and_then(|k| slots.slots.get(k));
        let playing = slot.is_some_and(|s| s.sndinst == i as i32 && s.state != state::FREE);
        let def = usize::try_from(pv.def).ok().and_then(|i| data.sounds.def(SoundOwner::Level, i));
        let distance = inst.position().into_iter().zip(l.pos).map(|(a,b)| (a-b).powi(2)).sum::<f32>().sqrt();
        let within_range = distance < inst.range;
        let why = reason(data, def, if def.is_some_and(|d| d.looped != 0) { 4 } else { 0 }, if playing { pv.slot } else { -1 });
        once(format!("emitterstate:{}:{i}:{playing}:{within_range}:{why}", data.bank.header.bank_id),
            format!("EMITTER_STATE frame={frame} id={i} class={} playing={playing} distance={distance} range={} camera={:?} local={:?} slot={slot:?} reason={why}", inst.o_class, inst.range, l.pos, inst.to_local(std::array::from_fn(|k| l.pos[k]-inst.position()[k]))));
    }
}
