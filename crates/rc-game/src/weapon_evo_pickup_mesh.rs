//! Frozen, bounded display geometry for mixed-randomizer world rewards.
//!
//! Native actor palettes are evaluated once, then baked into a PRIVATE class
//! copy. The renderer receives compact vertices and one identity joint. Native
//! actor classes, animations, inventory definitions and collision are untouched.
use rc_formats::moby::{LevelMobyClass, MobySkin, MobyVertex};
use rc_formats::moby_anim::{Rows, IDENTITY};

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum DisplayTexture { Neutral, Moby(usize), Metal(usize) }
pub struct DisplayVertex {pub position:[f32;3],pub normal:[f32;3],pub uv:[f32;2]}
pub struct DisplayBatch {pub texture:DisplayTexture,pub vertices:Vec<DisplayVertex>}

/// Exact native high-LOD/metal triangles and texture routing for a standalone
/// display. No geometry, substitute icon or glow rectangle is generated here.
pub fn display_batches(class:&LevelMobyClass)->Result<Vec<DisplayBatch>,String>{
    let mut batches:std::collections::BTreeMap<DisplayTexture,Vec<DisplayVertex>>=Default::default();
    for sub in class.class.high_lod.iter().chain(&class.class.metal){
        for triangle in &sub.triangles{
            if sub.is_metal&&!matches!(triangle.texture,-2|-3){continue;}
            let texture=match triangle.texture{
                -1=>DisplayTexture::Neutral,
                -2|-3 if sub.is_metal=>DisplayTexture::Metal(if triangle.texture==-2{0}else{1}),
                t if t>=0=>DisplayTexture::Moby(class.texture_table_index(t)
                    .ok_or_else(||format!("class {} missing texture slot {t}",class.o_class))?),
                _=>continue,
            };
            let batch=batches.entry(texture).or_default();
            for index in [triangle.a,triangle.b,triangle.c]{
                let vertex=sub.vertices.get(index as usize).ok_or("reward triangle index out of bounds")?;
                batch.push(DisplayVertex{position:vertex.packed_position().map(|v|v as f32),
                    normal:class.class.normal(vertex),uv:vertex.st.map(|v|v as f32/4096.)});
            }
        }
    }
    Ok(batches.into_iter().map(|(texture,vertices)|DisplayBatch{texture,vertices}).collect())
}

pub const WORLD_EXTENT: f32 = 1.5;
const HALF_PACKED: f32 = 8192.0;

pub struct Prepared {
    pub class: LevelMobyClass,
    pub posed_extent: f32,
    pub bind_fallback: bool,
}

fn matrix(vertex: &MobyVertex, pose: &[Rows], bind: bool) -> Result<Rows, &'static str> {
    let skin = vertex.skin;
    if bind || skin.count == 0 { return Ok(IDENTITY); }
    if skin.count > 3 { return Err("invalid skin influence count"); }
    let count = skin.count as usize;
    if count > 1 && skin.weights[..count].iter().map(|&w| u32::from(w)).sum::<u32>() != 256 {
        return Err("invalid skin weights");
    }
    let mut result = [[0.;4];4];
    for k in 0..count {
        // The native display palette fills slots past the evaluated pose with
        // identity matrices. Preserve that behavior instead of indexing past it.
        let joint = pose.get(skin.joints[k] as usize).unwrap_or(&IDENTITY);
        let weight = if count == 1 { 1. } else { skin.weights[k] as f32 / 256. };
        for r in 0..4 { for c in 0..4 { result[r][c] += joint[r][c] * weight; } }
    }
    if !result.iter().flatten().all(|x| x.is_finite()) { return Err("non-finite pose matrix"); }
    Ok(result)
}

/// Match the vertex shader's ftoi0 and signed 16-bit ppach wrap BEFORE fitting.
fn posed(vertex: &MobyVertex, transform: &Rows) -> Result<[i16;3], &'static str> {
    let p = vertex.packed_position().map(|v| v as f32);
    let mut result = [0;3];
    for axis in 0..3 {
        let value = transform[0][axis]*p[0] + transform[1][axis]*p[1]
            + transform[2][axis]*p[2] + transform[3][axis];
        if !value.is_finite() { return Err("non-finite posed vertex"); }
        result[axis] = (value.trunc().clamp(-2147483520.,2147483520.) as i32) as i16;
    }
    Ok(result)
}

fn normal(vertex: &mut MobyVertex, transform: &Rows) {
    let n = rc_formats::moby::moby_normal(vertex.normal_azimuth,vertex.normal_elevation);
    let transformed = std::array::from_fn::<_,3,_>(|axis|
        transform[0][axis]*n[0] + transform[1][axis]*n[1] + transform[2][axis]*n[2]);
    let length = transformed.iter().map(|v|v*v).sum::<f32>().sqrt();
    if !length.is_finite() || length < 1e-10 { return; }
    let n = transformed.map(|v|v/length);
    let encode = |angle:f32| ((angle/std::f32::consts::TAU*256.).round() as i32).rem_euclid(256) as u8;
    vertex.normal_azimuth = encode(n[1].atan2(n[0]));
    vertex.normal_elevation = encode(n[2].clamp(-1.,1.).asin());
}

fn bake(source: &LevelMobyClass, pose: &[Rows], bind: bool) -> Result<Prepared, &'static str> {
    let mut class = source.clone();
    let mut lo = [f32::INFINITY;3];
    let mut hi = [f32::NEG_INFINITY;3];
    let mut points = 0usize;
    // Only referenced vertices are drawn. Ignore unused cache/epilogue records
    // when measuring; both regular and metal triangles share the SAME bounds.
    for sub in class.class.high_lod.iter_mut().chain(&mut class.class.metal) {
        let used: std::collections::BTreeSet<_> = sub.triangles.iter()
            .flat_map(|t|[t.a,t.b,t.c]).map(|i|i as usize).collect();
        for index in used {
            let vertex = sub.vertices.get_mut(index).ok_or("triangle index out of bounds")?;
            let transform = matrix(vertex,pose,bind)?;
            let point = posed(vertex,&transform)?;
            normal(vertex,&transform);
            [vertex.x,vertex.y,vertex.z] = point;
            vertex.skin = MobySkin::joint(0);
            for axis in 0..3 {
                lo[axis]=lo[axis].min(point[axis] as f32);
                hi[axis]=hi[axis].max(point[axis] as f32);
            }
            points+=1;
        }
    }
    if points==0 { return Err("no display triangles"); }
    let span=(0..3).map(|axis|hi[axis]-lo[axis]).fold(0.0f32,f32::max);
    if !span.is_finite() || span < 1. { return Err("degenerate posed display geometry"); }
    let centre=std::array::from_fn::<_,3,_>(|axis|(lo[axis]+hi[axis])*0.5);
    let factor=HALF_PACKED*2./span;
    let mut radius=0.0f32;
    for sub in class.class.high_lod.iter_mut().chain(&mut class.class.metal) {
        let used:std::collections::BTreeSet<_>=sub.triangles.iter()
            .flat_map(|t|[t.a,t.b,t.c]).map(|i|i as usize).collect();
        for (index,vertex) in sub.vertices.iter_mut().enumerate() {
            if used.contains(&index) {
                let point=vertex.packed_position();
                let compact=std::array::from_fn::<_,3,_>(|axis|
                    ((point[axis] as f32-centre[axis])*factor).round().clamp(-HALF_PACKED,HALF_PACKED) as i16);
                [vertex.x,vertex.y,vertex.z]=compact;
                radius=radius.max(compact.iter().map(|&v|(v as f32).powi(2)).sum::<f32>().sqrt());
            } else {
                [vertex.x,vertex.y,vertex.z]=[0;3];
            }
            vertex.skin=MobySkin::joint(0);
        }
    }
    class.class.header.scale=WORLD_EXTENT*1024./(HALF_PACKED*2.);
    class.class.header.bsphere=[0.,0.,0.,radius];
    // A pickup is a display, not an initialized native actor with its own
    // shadow slab or runtime glow word. Its yellow halo is owned separately.
    class.class.header.shadow=0;
    class.class.header.glow_rgba=0;
    Ok(Prepared{class,posed_extent:span*source.class.header.scale/1024.,bind_fallback:bind})
}

/// Invalid/degenerate native animation falls back to the same model's stored
/// bind pose. Never replace a weapon with an unrelated actor or a blank mesh.
pub fn prepare(source:&LevelMobyClass,pose:&[Rows])->Result<Prepared,&'static str> {
    bake(source,pose,false).or_else(|_|bake(source,&[],true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_formats::moby::*;
    fn fixture()->LevelMobyClass {
        let vertex=|x,y,z|MobyVertex{raw:[0;8],normal_azimuth:0,normal_elevation:0,x,y,z,id:0,
            kind:MobyVertexKind::Single,duplicate:false,skin:MobySkin::joint(0),st:[0,0]};
        let sub=MobySubmesh{entry:Default::default(),vertex_table:Default::default(),metal_header:Default::default(),
            transfers:vec![],duplicates:vec![],raw_vertices:vec![],rgba_multipliers:vec![],
            vertices:vec![vertex(-10,0,0),vertex(10,0,0),vertex(0,10,5)],st:vec![],index_bytes:vec![],
            secret_indices:vec![],texture_indices:vec![],initial_texture:-1,unresolved_duplicates:0,
            triangles:vec![MobyTriangle{a:0,b:1,c:2,texture:-1}],is_metal:false};
        let mut header=MobyClassHeader::default();header.scale=0.125;
        LevelMobyClass{imported_textures:None,o_class:750,
            entry:rc_formats::level::ClassEntry{offset_in_asset_wad:0,o_class:750,unknown_8:0,unknown_c:0,textures:[0xff;16]},
            class:MobyClass{header,sequence_pointers:vec![],high_lod:vec![sub],low_lod:vec![],metal:vec![],skeleton:Default::default()}}
    }
    #[test]fn actual_deformation_is_baked_and_bounded_without_changing_the_source(){
        let source=fixture();let mut pose=IDENTITY;pose[0][0]=1000.;pose[3][2]=30000.;
        let baked=prepare(&source,&[pose]).unwrap();
        assert!(baked.posed_extent>source.class.position(&source.class.high_lod[0].vertices[1])[0]);
        for v in &baked.class.class.high_lod[0].vertices {
            assert_eq!(v.skin,MobySkin::joint(0));
            for c in baked.class.class.position(v){assert!(c.abs()<=WORLD_EXTENT*0.5+0.0001);}
        }
        assert_eq!(source.class.high_lod[0].vertices[0].x,-10);
        assert!(!baked.bind_fallback);
    }
    #[test]fn gpu_signed_wrap_and_identity_padding_are_preserved(){
        let v=fixture().class.high_lod[0].vertices[1];let mut m=IDENTITY;m[3][0]=65536.;
        assert_eq!(posed(&v,&m).unwrap()[0],10);
        let mut v=v;v.skin=MobySkin::joint(200);assert_eq!(matrix(&v,&[],false).unwrap(),IDENTITY);
    }
    #[test]fn unusable_pose_falls_back_to_the_same_bind_model(){
        let source=fixture();let mut pose=IDENTITY;pose[0][0]=f32::NAN;
        let baked=prepare(&source,&[pose]).unwrap();assert!(baked.bind_fallback);assert_eq!(baked.class.o_class,750);
        assert_eq!(baked.class.class.high_lod[0].triangles.len(),1);
    }
    #[test]fn metal_and_body_use_one_bound_and_preserve_texture_mappings(){
        let mut source=fixture();source.imported_textures=Some([Some(500);16]);
        let mut metal=source.class.high_lod[0].clone();metal.is_metal=true;metal.vertices[0].x=-30000;
        source.class.metal.push(metal);
        let baked=prepare(&source,&[IDENTITY]).unwrap();
        assert_eq!(baked.class.imported_textures,source.imported_textures);
        for sub in baked.class.class.high_lod.iter().chain(&baked.class.class.metal){
            for vertex in &sub.vertices{for v in baked.class.class.position(vertex){assert!(v.abs()<=0.7501);}}
        }
    }
    #[test]fn empty_and_bad_topology_are_rejected_instead_of_marked_ready(){
        let mut source=fixture();source.class.high_lod.clear();assert!(prepare(&source,&[]).is_err());
        let mut source=fixture();source.class.high_lod[0].triangles[0].c=999;assert!(prepare(&source,&[]).is_err());
    }
    #[test]fn standalone_display_keeps_real_body_and_metal_topology(){
        let mut source=fixture();source.o_class=1134;
        let mut metal=source.class.high_lod[0].clone();metal.is_metal=true;metal.triangles[0].texture=-2;
        source.class.metal.push(metal);
        let prepared=prepare(&source,&[IDENTITY]).unwrap();
        let batches=display_batches(&prepared.class).unwrap();
        assert_eq!(batches.len(),2);
        assert_eq!(batches.iter().map(|b|b.vertices.len()).sum::<usize>(),6);
        assert!(batches.iter().any(|b|b.texture==DisplayTexture::Metal(0)));
        for vertex in batches.iter().flat_map(|b|&b.vertices){
            assert!(vertex.position.into_iter().all(|p|p.abs()<=8192.));
            assert!(vertex.normal.into_iter().all(f32::is_finite));
        }
    }
    #[test]fn imported_texture_indices_do_not_truncate_and_no_body_is_invented(){
        let mut source=fixture();source.class.high_lod[0].triangles[0].texture=2;
        source.imported_textures=Some([Some(700);16]);
        assert_eq!(display_batches(&source).unwrap()[0].texture,DisplayTexture::Moby(700));
        source.class.high_lod.clear();source.class.metal.clear();
        assert!(display_batches(&source).unwrap().is_empty());
    }
}
