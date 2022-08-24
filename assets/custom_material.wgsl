#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

//struct CustomMaterial {
//};
//@group(1) @binding(0)
//var<uniform> material: CustomMaterial;

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normals: vec3<f32>,
    @location(2) uvs: vec2<f32>,
    @location(3) index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uvs: vec2<f32>,
    @location(1) index: u32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(vertex.position, 1.0));
    out.uvs = vertex.uvs;
    out.index = vertex.index;
    return out;
}

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    var uv = input.uvs / 16.0;
    uv.x += f32(input.index & u32(0x000F)) / 16.0;
    uv.y += f32(input.index & u32(0x00F0)) / 16.0;

    //return vec4<f32>(input.index, 0.0, 0.0, 1.0);
    return textureSample(texture, texture_sampler, uv);
}