//! Append selected foreign class sounds without overwriting native sound IDs.
//! Work transactionally: invalid offsets leave the original level audio intact.
use super::LevelAudio;
use rc_formats::sound_bank::{grain, Bank};
use std::{collections::HashMap, sync::Arc};
pub fn class_sounds(
    target: &mut LevelAudio,
    source: &LevelAudio,
    classes: &[i32],
) -> Result<usize, String> {
    let mut bank = (*target.bank).clone();
    let mut sounds = target.sounds.clone();
    let mut ids = HashMap::new();
    let mut samples = HashMap::new();
    let mut count = 0;
    for &class in classes {
        if sounds.classes.iter().any(|c| {
            c.o_class == class
                && c.defs
                    .iter()
                    .any(|d| d.index != 0xffff && (d.index as usize) < bank.sounds.len())
        }) {
            continue;
        }
        let Some(c) = source.sounds.classes.iter().find(|c| c.o_class == class) else {
            continue;
        };
        let mut c = c.clone();
        for id in &mut c.bank_ids {
            if *id != 0xffff {
                *id = copy_sound(&source.bank, &mut bank, *id, &mut ids, &mut samples)?;
            }
        }
        for def in &mut c.defs {
            if def.index != 0xffff {
                def.index = copy_sound(&source.bank, &mut bank, def.index, &mut ids, &mut samples)?;
            }
            def.bank_handle = 0;
        }
        sounds.classes.retain(|old| old.o_class != class);
        sounds.classes.push(c);
        count += 1;
    }
    bank.header.n_sounds = bank
        .sounds
        .len()
        .try_into()
        .map_err(|_| "imported sound count overflow")?;
    bank.header.n_grains = bank
        .grains
        .len()
        .try_into()
        .map_err(|_| "imported grain count overflow")?;
    bank.header.n_vags = bank
        .vags
        .len()
        .try_into()
        .map_err(|_| "imported sample count overflow")?;
    bank.header.vag_data_size = bank
        .samples
        .len()
        .try_into()
        .map_err(|_| "imported sample data overflow")?;
    target.bank = Arc::new(bank);
    target.sounds = sounds;
    Ok(count)
}
fn copy_sound(
    src: &Bank,
    dst: &mut Bank,
    id: u16,
    ids: &mut HashMap<u16, u16>,
    samples: &mut HashMap<u32, u32>,
) -> Result<u16, String> {
    if let Some(&new) = ids.get(&id) {
        return Ok(new);
    }
    let mut sound = src
        .sounds
        .get(id as usize)
        .ok_or("foreign sound ID out of range")?
        .clone();
    let new = dst
        .sounds
        .len()
        .try_into()
        .map_err(|_| "sound ID overflow")?;
    ids.insert(id, new);
    dst.sounds.push(sound.clone());
    let mut grains = Vec::new();
    for mut g in src
        .grains
        .get(sound.grains.clone())
        .ok_or("foreign grain range invalid")?
        .iter()
        .copied()
    {
        if matches!(g.kind, grain::TONE | grain::TONE2) {
            let old = u32::from_le_bytes(g.data[16..20].try_into().unwrap());
            let offset = if let Some(&o) = samples.get(&old) {
                o
            } else {
                let ext = src
                    .vags
                    .get(src.vag_at(old).ok_or("foreign sample offset invalid")?)
                    .ok_or("foreign sample invalid")?;
                let bytes = src
                    .samples
                    .get(ext.offset..ext.offset + ext.frames * 16)
                    .ok_or("foreign sample extent invalid")?;
                let o = dst.samples.len() as u32;
                dst.samples.extend_from_slice(bytes);
                let mut ext = ext.clone();
                ext.offset = o as usize;
                dst.vags.push(ext);
                samples.insert(old, o);
                o
            };
            g.data[16..20].copy_from_slice(&offset.to_le_bytes());
        } else if matches!(g.kind, grain::START_CHILD_SOUND | grain::STOP_CHILD_SOUND) {
            let old = i32::from_le_bytes(g.data[12..16].try_into().unwrap());
            if old >= 0 {
                let id = copy_sound(
                    src,
                    dst,
                    old.try_into().map_err(|_| "child sound ID overflow")?,
                    ids,
                    samples,
                )?;
                g.data[12..16].copy_from_slice(&(id as i32).to_le_bytes());
            }
        } else if matches!(g.kind, grain::BRANCH | grain::XREF_ID | grain::XREF_NUM) {
            return Err("foreign branch grain is not supported".into());
        }
        grains.push(g);
    }
    let start = dst.grains.len();
    dst.grains.extend(grains);
    sound.grains = start..dst.grains.len();
    sound.first_grain = (start * 40) as u32;
    dst.sounds[new as usize] = sound;
    Ok(new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_formats::sound_bank::{ClassSounds, Grain, LevelSounds, SfxHeader, Sound, SoundDef};
    fn fixture(class: i32, sample: u8) -> LevelAudio {
        let bank = Bank {
            file_type: 3,
            chunks: [(0, 0); 2],
            header: SfxHeader::default(),
            sounds: vec![Sound {
                vol: 127,
                vol_group: 0,
                pan: 0,
                instance_limit: 0,
                flags: 0,
                first_grain: 0,
                grains: 0..1,
            }],
            grains: vec![Grain {
                kind: grain::TONE,
                delay: 0,
                data: [0; 32],
            }],
            vags: vec![rc_formats::vag::SampleExtent {
                offset: 0,
                frames: 1,
                loop_start: None,
                looped: false,
                next_flags: None,
            }],
            samples: vec![sample; 16],
        };
        LevelAudio {
            bank: Arc::new(bank),
            sounds: LevelSounds {
                classes: vec![ClassSounds {
                    o_class: class,
                    bank_ids: vec![0],
                    defs: vec![SoundDef {
                        index: 0,
                        ..Default::default()
                    }],
                    header_count: Some(1),
                }],
                ..Default::default()
            },
            instances: vec![],
            instance_pvars: vec![],
            music: std::array::from_fn(|_| None),
            env_points: vec![],
            collision: None,
        }
    }
    #[test]
    fn missing_class_appends_real_sample_and_preserves_native_sound_zero() {
        let mut target = fixture(100, 11);
        let source = fixture(1818, 22);
        assert_eq!(class_sounds(&mut target, &source, &[1818]).unwrap(), 1);
        assert_eq!(target.sounds.classes[0].defs[0].index, 0);
        assert_eq!(target.sounds.classes[1].defs[0].index, 1);
        assert_eq!(target.bank.samples[..16], [11; 16]);
        assert_eq!(target.bank.samples[16..], [22; 16]);
        let g = &target.bank.grains[1];
        assert_eq!(u32::from_le_bytes(g.data[16..20].try_into().unwrap()), 16);
        assert_eq!(class_sounds(&mut target, &source, &[1818]).unwrap(), 0);
        assert_eq!(target.bank.samples.len(), 32);
    }
    #[test]
    fn malformed_foreign_sample_cannot_replace_native_audio() {
        let mut target = fixture(100, 11);
        let before = target.bank.clone();
        let mut source = fixture(1818, 22);
        Arc::make_mut(&mut source.bank).grains[0].data[16..20]
            .copy_from_slice(&400u32.to_le_bytes());
        assert!(class_sounds(&mut target, &source, &[1818]).is_err());
        assert_eq!(target.bank.samples, before.samples);
        assert_eq!(target.sounds.classes.len(), 1);
        assert_eq!(target.bank.sounds, before.sounds);
    }
}
