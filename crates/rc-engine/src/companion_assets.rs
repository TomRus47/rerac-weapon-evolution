//! Import missing ammo pickup models and the optional Sonic Summoner classes.
//! This runs on the level loader worker; no disc reads occur during a summon.
use anyhow::{Context, Result};
use rc_formats::{
    moby, moby_anim,
    texture::{self, LevelTexture, TextureSource, TextureTable},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

type JointData = Vec<(Vec<u8>, Vec<u8>)>;
struct Asset {
    class: moby::LevelMobyClass,
    anim: moby_anim::MobyAnimClass,
    joints: JointData,
    textures: Vec<(usize, LevelTexture)>,
}
static CACHE: OnceLock<Mutex<HashMap<(PathBuf,u32,Vec<i32>), Arc<Vec<Asset>>>>> = OnceLock::new();

fn load(root: &Path, level:u32, requested:&[i32]) -> Result<Arc<Vec<Asset>>> {
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let mut ids=requested.to_vec();ids.sort_unstable();ids.dedup();
    let key=(root.to_path_buf(),level,ids);
    if let Some(cached) = cache.get(&key) {
        return Ok(cached.clone());
    }
    let data = rc_data::level_core_data(root, level).context("reading companion reference core")?;
    let index = crate::disc_source::level_file(root, level, "core_index.bin")?;
    let gs = crate::disc_source::level_file(root, level, "gs_ram.bin")?;
    let mut core = rc_formats::level::parse_level_core(&index, data.len())?;
    core.moby_classes.retain(|c| c.offset_in_asset_wad>0 && requested.contains(&c.o_class));
    let mut assets = Vec::new();
    for class in moby::parse_level_mobys(&core, &data)? {
        let block = core
            .blocks
            .iter()
            .find(|b| b.name == format!("moby_class/{:04}", class.o_class))
            .context("missing companion class block")?;
        let blob = data
            .get(block.offset..block.offset + block.size)
            .context("companion class out of bounds")?;
        let anim = moby_anim::MobyAnimClass::new(
            &class.class,
            moby_anim::parse_sequences(blob, &class.class)?,
        );
        let joints = (0..256)
            .map_while(|i| rc_formats::gadget::joint_list(blob, &class.class.header, i).ok())
            .collect();
        let mut textures = Vec::new();
        for slot in class.entry.textures.into_iter().filter(|&s| s != 0xff) {
            if textures.iter().any(|(s, _)| *s == slot as usize) {
                continue;
            }
            let entry = *core
                .moby_textures
                .get(slot as usize)
                .context("missing companion texture")?;
            let image = texture::entry_image(&core, &data, &gs, &entry)?.decode()?;
            textures.push((
                slot as usize,
                LevelTexture {
                    table: TextureTable::Moby,
                    index: slot as usize,
                    source: TextureSource::Entry(entry),
                    texture: image,
                },
            ));
        }
        assets.push(Asset {
            class,
            anim,
            joints,
            textures,
        });
    }
    anyhow::ensure!(
        requested.iter().all(|id| assets.iter().any(|a| a.class.o_class == *id)),
        "both companion classes must be available in level 01"
    );
    let assets = Arc::new(assets);
    cache.insert(key, assets.clone());
    Ok(assets)
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static GRENADE: OnceLock<bool> = OnceLock::new();
static FLAME_MINE: OnceLock<bool> = OnceLock::new();
static AMMO_MODE: OnceLock<bool> = OnceLock::new();

pub fn install(
    root: &Path,
    mobys: &mut crate::moby_render::LevelMobys,
    textures: &mut Vec<LevelTexture>,
) -> Result<()> {
    let live=std::env::var_os("RC_WEAPON_LIVE_DIR").is_some();
    if rc_game::weapon_evo::randomizer::enabled() || rc_game::weapon_evo::location_test::enabled() {
        import_models(root,1,&[1134,750,479],mobys,textures)?;
        if rc_game::weapon_evo::randomizer::enabled(){
            import_models(root,17,&[1428],mobys,textures)?;
            import_models(root,12,&[287],mobys,textures)?;
        }
    }
    let companion = live || *ENABLED.get_or_init(|| rc_game::weapon_evo::profile_uses_extra(24, "sand_mouse_enabled"));
    let ammo_mode = *AMMO_MODE.get_or_init(|| rc_game::weapon_evo::profile_uses_extra(-1, rc_game::weapon_evo::ammo_mode::KEY));
    let mut requested: Vec<i32> = if ammo_mode {
        rc_game::moby_update::classes::pickup::AMMO_CLASSES.iter().map(|&c| c as i32).collect()
    } else { vec![1438] }; // R.Y.N.O. ammo is now eligible in ordinary play too.
    if companion { requested.extend([1818, 1633]); }
    if live {requested.extend([121,122,1192,74,172,230]);}
    if *GRENADE.get_or_init(||rc_game::weapon_evo::profile_uses_extra(13,"grenade_launcher_enabled")) {requested.extend([121,122,1192,74,172]);}
    if *FLAME_MINE.get_or_init(||rc_game::weapon_evo::profile_uses_extra(17,"flame_mine_enabled")) {requested.extend([121,74,230]);}
    if requested.iter().all(|id| mobys.classes.iter().any(|c| c.o_class == *id))
    {
        return Ok(());
    }
    import_models(root,1,&requested,mobys,textures)
}

/// Import only requested classes, retaining their animations and joint tables.
pub(crate) fn import_models(root:&Path,level:u32,requested:&[i32],mobys:&mut crate::moby_render::LevelMobys,textures:&mut Vec<LevelTexture>)->Result<()> {
    let assets = load(root,level,requested)?;
    for a in assets
        .iter()
        .filter(|a| requested.contains(&a.class.o_class) && !mobys.classes.iter().any(|c| c.o_class == a.class.o_class))
        .collect::<Vec<_>>()
    {
        let mut class = a.class.clone();
        let mut remap = HashMap::new();
        let mut next = textures
            .iter()
            .filter(|t| t.table == TextureTable::Moby)
            .map(|t| t.index + 1)
            .max()
            .unwrap_or(0);
        for (old, image) in &a.textures {
            let mut image = image.clone();
            image.index = next;
            textures.push(image);
            remap.insert(*old, next);
            next += 1;
        }
        // Runtime imports may exceed the disc's byte-sized texture table.
        // Keep their mapping in usize slots rather than truncating/overwriting it.
        class.imported_textures = Some(
            class
                .entry
                .textures
                .map(|slot| remap.get(&(slot as usize)).copied()),
        );
        mobys
            .imported_joints
            .insert(class.o_class as i16, a.joints.clone());
        mobys.classes.push(class);
        mobys.anim.push(a.anim.clone());
    }
    anyhow::ensure!(requested.iter().all(|id| mobys.classes.iter().any(|c| c.o_class == *id)),
        "missing ammo/companion reference class after import");
    Ok(())
}

/// Cache source audio on the loader worker. Never probe/record audio during play.
pub(crate) fn reference_audio(root:&Path,level:u32)->Result<Arc<rc_game::audio::LevelAudio>> {
    static AUDIO:OnceLock<Mutex<HashMap<(PathBuf,u32),Arc<rc_game::audio::LevelAudio>>>>=OnceLock::new();
    let mut cache=AUDIO.get_or_init(Default::default).lock().unwrap_or_else(|p|p.into_inner());
    let key=(root.to_path_buf(),level);if let Some(a)=cache.get(&key){return Ok(a.clone());}
    let read=|name|crate::disc_source::level_file(root,level,name);
    let data=rc_data::level_core_data(root,level)?;let index=read("core_index.bin")?;
    let core=rc_formats::level::parse_level_core(&index,data.len())?;
    let a=rc_game::audio::LevelAudio::from_parts(&read("sound_bank.bin")?,&index,&core,&data,
        &rc_data::level_gameplay(root,level)?,&read("level_header.bin")?,&std::array::from_fn(|_|None),None)?;
    let a=Arc::new(a);cache.insert(key,a.clone());Ok(a)
}
pub(crate) fn install_companion_audio(root:&Path,a:&mut rc_game::audio::LevelAudio)->Result<()> {
    if std::env::var_os("RC_WEAPON_LIVE_DIR").is_none() && !rc_game::weapon_evo::profile_uses_extra(24,"sand_mouse_enabled"){return Ok(());}
    let source=reference_audio(root,1)?;
    rc_game::audio::import::class_sounds(a,&source,&[1818,1633]).map_err(anyhow::Error::msg)?;
    Ok(())
}
