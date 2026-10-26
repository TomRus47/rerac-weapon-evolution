//! Standalone rendering for real mixed-randomizer reward meshes.
//!
//! Geometry is the frozen native class, including metal triangles. These
//! entities deliberately have no MeshTag, ExtraMetal, palette/instance storage
//! buffers or DisplayEffect marker. Their only placement is Bevy's transform.
use crate::{game_camera::TfragFog,gs_state::{self,AlphaRange,GsPass}};
use bevy::asset::{load_internal_asset,uuid_handle,RenderAssetUsages};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{ImageAddressMode,ImageFilterMode,ImageSampler,ImageSamplerDescriptor};
use bevy::mesh::{Indices,MeshVertexBufferLayoutRef,PrimitiveTopology};
use bevy::pbr::{MaterialPipeline,MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup,Extent3d,RenderPipelineDescriptor,
    ShaderType,SpecializedMeshPipelineError,TextureDimension,TextureFormat};
use bevy::shader::{Shader,ShaderRef};
use rc_formats::{moby::LevelMobyClass,texture::TextureTable};
use std::collections::HashMap;
use rc_game::weapon_evo::pickup_mesh::{display_batches,DisplayTexture as TextureKey};

const SHADER:Handle<Shader>=uuid_handle!("c9c614e1-eafe-4138-9147-2099de439bf8");

#[derive(Clone,Copy,Debug,Default,ShaderType)]
pub struct Surface { pub values:Vec4 }

#[derive(Asset,TypePath,AsBindGroup,Debug,Clone)]
#[bind_group_data(GsPass)]
pub struct RewardMaterial {
    #[texture(0)] #[sampler(1)] pub texture:Handle<Image>,
    #[uniform(2)] pub fog:TfragFog,
    #[uniform(3)] pub surface:Surface,
    pub pass:GsPass,
}
impl From<&RewardMaterial> for GsPass {
    fn from(m:&RewardMaterial)->Self{m.pass}
}
impl Material for RewardMaterial {
    fn vertex_shader()->ShaderRef{SHADER.clone().into()}
    fn fragment_shader()->ShaderRef{SHADER.clone().into()}
    fn alpha_mode(&self)->AlphaMode{self.pass.alpha_mode()}
    fn depth_bias(&self)->f32{if self.surface.values.x>0.5{0.01}else{0.}}
    fn enable_prepass()->bool{false}
    fn enable_shadows()->bool{false}
    fn specialize(_: &MaterialPipeline,descriptor:&mut RenderPipelineDescriptor,
        layout:&MeshVertexBufferLayoutRef,key:MaterialPipelineKey<Self>)->Result<(),SpecializedMeshPipelineError>{
        descriptor.primitive.cull_mode=None;
        // These passes never switch to the display-encoded full-frame target.
        debug_assert!(!key.bind_group_data.state().display);
        key.bind_group_data.specialize(descriptor);
        descriptor.vertex.buffers=vec![layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
        ])?];
        Ok(())
    }
}

pub struct RewardModelPlugin;
impl Plugin for RewardModelPlugin {
    fn build(&self,app:&mut App){
        load_internal_asset!(app,SHADER,"reward_model.wgsl",Shader::from_wgsl);
        app.add_plugins(MaterialPlugin::<RewardMaterial>::default())
            .add_systems(PostUpdate,refresh_fog);
    }
}
fn refresh_fog(fog:Option<Res<crate::game_camera::GameFog>>,
    mut materials:ResMut<Assets<RewardMaterial>>,mut previous:Local<Option<TfragFog>>){
    let Some(fog)=fog else{return;};
    if previous.as_ref()==Some(&fog.uniform){return;}
    *previous=Some(fog.uniform);
    for (_,material) in materials.iter_mut(){material.fog=fog.uniform;}
}


fn image(level:&crate::level_load::LoadedLevel,key:TextureKey,images:&mut Assets<Image>)
    ->Option<(Handle<Image>,AlphaRange)>{
    let (width,height,mut rgba,mode)=match key{
        TextureKey::Neutral=>(1,1,vec![128,128,128,255],ImageAddressMode::ClampToEdge),
        TextureKey::Moby(index)=>{
            let t=&level.textures.iter().find(|t|t.table==TextureTable::Moby&&t.index==index)?.texture;
            (t.width,t.height,t.rgba.clone(),ImageAddressMode::Repeat)
        }
        TextureKey::Metal(index)=>{
            let t=level.mobys.env_maps.get(index)?.as_ref()?;
            (t.width,t.height,t.rgba.clone(),ImageAddressMode::ClampToEdge)
        }
    };
    // Decoded texture alpha is display-scaled; restore the native 0..128 range.
    for alpha in rgba.iter_mut().skip(3).step_by(4){*alpha=if *alpha==255{128}else{*alpha/2};}
    let range=AlphaRange::of(rgba.iter().skip(3).step_by(4).copied());
    let mut image=Image::new(Extent3d{width,height,depth_or_array_layers:1},TextureDimension::D2,
        rgba,TextureFormat::Rgba8Unorm,RenderAssetUsages::RENDER_WORLD);
    image.sampler=ImageSampler::Descriptor(ImageSamplerDescriptor{
        address_mode_u:mode,address_mode_v:mode,mag_filter:ImageFilterMode::Linear,
        min_filter:ImageFilterMode::Linear,..default()});
    Some((images.add(image),range))
}

/// Draw the actual class topology, never a glow quad or inventory icon.
#[allow(clippy::too_many_arguments)]
pub fn spawn(commands:&mut Commands,level:&crate::level_load::LoadedLevel,
    class:&LevelMobyClass,fog:TfragFog,meshes:&mut Assets<Mesh>,images:&mut Assets<Image>,
    materials:&mut Assets<RewardMaterial>)->Result<Vec<Entity>,String>{
    let batches=display_batches(class)?;
    // Resolve every required image BEFORE spawning anything; a partial body
    // must not mark a collectible ready and become invisible on collection.
    let mut textures=HashMap::new();
    for batch in &batches{
        let key=batch.texture;
        textures.insert(key,image(level,key,images).ok_or_else(||
            format!("reward class {} missing decoded texture {key:?}",class.o_class))?);
    }
    let mut entities=Vec::new();let mut triangles=0;
    for batch in batches{
        let key=batch.texture;
        let metal=matches!(key,TextureKey::Metal(_));
        triangles+=batch.vertices.len()/3;
        let count=batch.vertices.len() as u32;
        let mesh=Mesh::new(PrimitiveTopology::TriangleList,RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,batch.vertices.iter().map(|v|v.position).collect::<Vec<_>>())
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL,batch.vertices.iter().map(|v|v.normal).collect::<Vec<_>>())
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0,batch.vertices.iter().map(|v|v.uv).collect::<Vec<_>>())
            .with_inserted_indices(Indices::U32((0..count).collect()));
        let mesh=meshes.add(mesh);
        let (texture,alpha)=textures[&key].clone();
        for base in gs_state::draws(gs_state::AREF_WORLD,alpha,AlphaRange::OPAQUE){
            let pass=if metal{match base{GsPass::OpaqueTested{aref}=>GsPass::LateTested{aref},p=>p}}else{base};
            let material=materials.add(RewardMaterial{texture:texture.clone(),fog,
                surface:Surface{values:Vec4::new(if metal{1.}else{0.},0.,0.,0.)},pass});
            entities.push(commands.spawn((Mesh3d(mesh.clone()),MeshMaterial3d(material),
                Transform::IDENTITY,Visibility::Hidden,NoFrustumCulling,
                Name::new(format!("Standalone native reward class {} {key:?} {pass:?}",class.o_class)))).id());
        }
    }
    println!("reward standalone model: native_class={} triangles={triangles} parts={} actor_buffers=false",
        class.o_class,entities.len());
    if entities.is_empty(){return Err("reward model contains no renderable triangles".into());}
    Ok(entities)
}
