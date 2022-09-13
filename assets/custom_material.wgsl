#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

#import bevy_pbr::mesh_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normals: vec2<u32>,
    @location(2) uvs: vec2<u32>,
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
    out.uvs = vec2<f32>(vertex.uvs);
    if (vertex.uvs.x == u32(0)) {
        out.uvs.x = 0.001;
    } else {
        out.uvs.x -= 0.001;
    }
    if (vertex.uvs.y == u32(0)) {
        out.uvs.y = 0.001;
    } else {
        out.uvs.y -= 0.001;
    }

    out.index = vertex.index;
    return out;
}

@group(1) @binding(0)
var array_texture: texture_2d_array<f32>;
//var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    //var u = f32(input.uvs.x) / 16.0;
    //var v = f32(input.uvs.y) / 16.0;
    //var uv = vec2<f32>(u,v);
    //uv.x += f32(input.index & u32(0x000F)) / 16.0;
    //uv.y += f32(input.index & u32(0x00F0)) / 16.0;
    

    //return vec4<f32>(0.5,0.5,0.5,0.5);

    //return vec4<f32>(f32(input.index & u32(0x00F0)) /16.0, f32(input.uvs.x), 0.0, 1.0);
    return textureSample(array_texture, texture_sampler, input.uvs, i32(input.index));
}