//! Native reward scenes staged by the loader, without a foreign planet switch.
//! Only animations/camera tracks are played by the isolated pickup player:
//! source missions, NPC grants, dialogue scripts and travel never execute.
use std::{collections::HashMap,path::{Path,PathBuf},sync::{Arc,Mutex,OnceLock}};
use rc_formats::{scene::{Scene,Region},texture::LevelTexture};

type Key=(PathBuf,u32,usize);
type Cache=HashMap<Key,Arc<Scene>>;
static SCENES:OnceLock<Mutex<Cache>>=OnceLock::new();

/// Move all tracks by the same offset. Retain the native camera and each
/// object's actual streamed animation, rather than substituting a weapon demo.
fn recenter(scene:&mut Scene)->bool {
    // Magneboots are acquired by Clank in the original scene. Prefer Ratchet
    // when present, otherwise use a confirmed Clank actor, never an item/NPC.
    let Some(anchor)=scene.chunks.first().and_then(|chunk|[0,10,87,601].into_iter()
        .find_map(|class|chunk.actors.iter().find(|a|a.class==class)
            .and_then(|a|a.positions.first()).copied())) else {return false;};
    if !anchor[..3].iter().all(|v|v.is_finite()) {return false;}
    for chunk in &mut scene.chunks {
        for camera in &mut chunk.camera {for k in 0..3 {camera.eye[k]-=anchor[k];}}
        for actor in &mut chunk.actors {for position in &mut actor.positions {for k in 0..3 {position[k]-=anchor[k];}}}
    }
    true
}

pub fn cached(root:&Path,level:u32,scene:usize)->Option<Arc<Scene>> {
    SCENES.get()?.lock().unwrap_or_else(|p|p.into_inner()).get(&(root.to_path_buf(),level,scene)).cloned()
}

/// Called only on the level loader worker. Runtime collection reads the cache;
/// an unavailable/corrupt presentation falls back to a banner and one jingle.
pub fn prepare(root:&Path,mobys:&mut crate::moby_render::LevelMobys,textures:&mut Vec<LevelTexture>) {
    if !rc_game::weapon_evo::randomizer::enabled() && !rc_game::weapon_evo::golden::enabled() {return;}
    for &(_,origin,index) in rc_game::weapon_evo::randomizer::ACQUISITION_SCENES {
        let load=||->anyhow::Result<Arc<Scene>> {
            if let Some(scene)=cached(root,origin,index) {return Ok(scene);}
            let bytes=crate::disc_source::level_file(root,origin,"level_header.bin")?;
            let header=rc_formats::toc::parse_level_header(&bytes)?;
            let bytes=crate::disc_source::level_file(root,origin,&format!("scene/{index:02}_ntsc.bin"))?;
            let mut scene=Scene::load(&header,&bytes,index,Region::Ntsc)?;
            anyhow::ensure!(recenter(&mut scene),"item scene has no finite player anchor");
            let scene=Arc::new(scene);
            SCENES.get_or_init(Default::default).lock().unwrap_or_else(|p|p.into_inner())
                .insert((root.to_path_buf(),origin,index),scene.clone());
            Ok(scene)
        };
        match load() {
            Ok(scene)=>for class in scene.actor_classes() {
                if mobys.classes.iter().any(|m|m.o_class==class) {continue;}
                // The ordinary gadget table remains a second resource source
                // in travel_render::spawn_gfx for hand-item classes.
                if let Err(error)=crate::companion_assets::import_models(root,origin,&[class],mobys,textures) {
                    eprintln!("item scene {origin:02}/{index}: class {class}: {error:#}");
                }
            },
            Err(error)=>eprintln!("item scene {origin:02}/{index}: unavailable: {error:#}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_formats::{moby_anim::{MobySequence,MobySequenceHeader},scene::{CamRecord,ChunkHeader,SceneActor,SceneChunk}};

    fn actor(class:i32,positions:Vec<[f32;4]>)->SceneActor {
        SceneActor {class,positions,offset:0,unknown_04:0,unknown_08:0,track_offset:0,frame_offsets:vec![],
            sequence:MobySequence{header:MobySequenceHeader::default(),frames:vec![],triggers:vec![]}}
    }
    #[test]
    fn recenter_preserves_motion_and_camera_offsets_across_chunks() {
        let camera=CamRecord{eye:[101.,205.,307.],angles:[0.1,0.2,0.3],tan_half_fov:0.8,..Default::default()};
        let chunk=SceneChunk{header:ChunkHeader::default(),camera:vec![camera],
            actors:vec![actor(0,vec![[100.,200.,300.,0.],[101.,202.,303.,0.]]),
                actor(618,vec![[103.,204.,305.,0.]])],subtitles:vec![]};
        let mut scene=Scene{index:5,region:Region::Ntsc,chunks:vec![chunk.clone(),chunk]};
        assert!(recenter(&mut scene));
        for c in scene.chunks {
            assert_eq!(c.actors[0].positions,vec![[0.,0.,0.,0.],[1.,2.,3.,0.]]);
            assert_eq!(c.actors[1].positions,vec![[3.,4.,5.,0.]]);
            assert_eq!(c.camera[0].eye,[1.,5.,7.]);
            assert_eq!(c.camera[0].angles,camera.angles);
            assert_eq!(c.camera[0].tan_half_fov,camera.tan_half_fov);
        }
    }
    #[test]
    fn missing_or_nonfinite_anchor_is_rejected_without_mutation() {
        let mut scene=Scene{index:0,region:Region::Ntsc,chunks:vec![]};
        assert!(!recenter(&mut scene));
        scene.chunks.push(SceneChunk{header:ChunkHeader::default(),camera:vec![],subtitles:vec![],
            actors:vec![actor(0,vec![[f32::INFINITY,0.,0.,0.]])]});
        assert!(!recenter(&mut scene));assert!(scene.chunks[0].actors[0].positions[0][0].is_infinite());
    }
    #[test]
    fn clank_item_scenes_do_not_require_a_ratchet_actor() {
        let mut scene=Scene{index:2,region:Region::Ntsc,chunks:vec![SceneChunk {
            header:ChunkHeader::default(),camera:vec![CamRecord{eye:[8.,9.,10.],..Default::default()}],
            actors:vec![actor(18,vec![[90.,90.,90.,0.]]),actor(10,vec![[5.,6.,7.,0.]])],subtitles:vec![]}]};
        assert!(recenter(&mut scene));
        assert_eq!(scene.chunks[0].actors[1].positions[0],[0.,0.,0.,0.]);
        assert_eq!(scene.chunks[0].camera[0].eye,[3.,3.,3.]);
    }
}
