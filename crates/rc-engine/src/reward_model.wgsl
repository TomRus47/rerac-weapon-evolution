// Native reward geometry through Bevy's ordinary model transform only.
// No actor slot, joint palette, display-space blending or fullscreen operation.
#import bevy_pbr::{mesh_functions, mesh_view_bindings::view,
    view_transformations::{position_world_to_clip, position_world_to_view}}

struct Fog { color:vec4<f32>, params:vec4<f32> }
struct Surface { values:vec4<f32> }
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var tex:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var tex_sampler:sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> fog:Fog;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> surface:Surface;

struct Input {
    @builtin(instance_index) instance_index:u32,
    @location(0) position:vec3<f32>,
    @location(1) normal:vec3<f32>,
    @location(2) uv:vec2<f32>,
}
struct Output {
    @builtin(position) position:vec4<f32>,
    @location(0) uv:vec2<f32>,
    @location(1) @interpolate(linear) fog:f32,
}
@vertex
fn vertex(v:Input)->Output {
    var out:Output;
    let model=mesh_functions::get_world_from_local(v.instance_index);
    let world=(model*vec4<f32>(v.position,1.0)).xyz;
    out.position=position_world_to_clip(world);
    out.uv=v.uv;
    if(surface.values.x>0.5){
        let world_normal=normalize((model*vec4<f32>(v.normal,0.0)).xyz);
        let view_normal=(view.view_from_world*vec4<f32>(world_normal,0.0)).xyz;
        out.uv=view_normal.xy*vec2<f32>(0.5,-0.5)+vec2<f32>(0.5);
        out.position.z+=1000.0*32.0/1024.0/16777216.0;
    }
    let depth=-position_world_to_view(world).z*1024.0;
    out.fog=trunc(clamp(depth*fog.params.x+fog.params.y,fog.params.z,fog.params.w))/255.0;
    return out;
}
fn srgb_to_linear(c:vec3<f32>)->vec3<f32>{
    return select(pow((c+0.055)/1.055,vec3<f32>(2.4)),c/12.92,c<=vec3<f32>(0.04045));
}
@fragment
fn fragment(v:Output)->@location(0) vec4<f32>{
    let t=textureSample(tex,tex_sampler,v.uv);
    let alpha=round(t.a*255.0);
#ifdef GS_ATEST_PASS
    if(alpha<f32(#{GS_AREF})){discard;}
#endif
#ifdef GS_ATEST_FAIL
    if(alpha>=f32(#{GS_AREF})){discard;}
#endif
    var rgb=t.rgb;
    if(surface.values.x>0.5){rgb*=112.0/128.0;}
    if(fog.color.w>0.5){rgb=mix(fog.color.rgb,rgb,clamp(v.fog,0.0,1.0));}
    return vec4<f32>(srgb_to_linear(clamp(rgb,vec3<f32>(0.0),vec3<f32>(1.0))),clamp(alpha/128.0,0.0,1.0));
}
