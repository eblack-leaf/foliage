@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) vertex_tx: vec2<f32>,
    @location(2) position: vec2<f32>,
    @location(3) area: vec2<f32>,
    @location(4) layer: f32,
    @location(5) color: vec4<f32>,
    @location(6) mip: f32,
};
struct VertexFragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mip: f32,
};
@vertex
fn vertex_entry(vertex: Vertex) -> VertexFragment {
    let pos = vec4<f32>(vertex.position + vertex.vertex_pos * vertex.area, vertex.layer, 1.0);
    return VertexFragment(viewport * pos, vertex.vertex_tx, vertex.color, vertex.mip);
}
@group(1)
@binding(0)
var rectangle_texture: texture_2d<f32>;
@group(0)
@binding(1)
var rectangle_sampler: sampler;
@fragment
fn fragment_entry (vertex_fragment: VertexFragment) -> @location(0) vec4<f32> {
    let coverage = textureSampleLevel(
        rectangle_texture,
        rectangle_sampler,
        vertex_fragment.texture_coordinates,
        vertex_fragment.mip
    ).r;
    return vec4<f32>(vertex_fragment.color.rgb, vertex_fragment.color.a * coverage);
}