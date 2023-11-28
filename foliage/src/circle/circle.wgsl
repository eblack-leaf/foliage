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
    @location(6) ring: f32,
    @location(7) mip: f32,
    @location(8) prog: f32,
};
struct VertexFragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) ring: f32,
    @location(3) mip: f32,
    @location(4) prog: f32,
};
@vertex
fn vertex_entry(vertex: Vertex) -> VertexFragment {
    let pos = vec4<f32>(vertex.position + vertex.vertex_pos * vertex.area, vertex.layer, 1.0);
    return VertexFragment(viewport * pos, vertex.vertex_tx, vertex.color, vertex.ring, vertex.mip, vertex.prog);
}
@group(0)
@binding(1)
var circle_texture: texture_2d<f32>;
@group(0)
@binding(2)
var circle_ring_texture: texture_2d<f32>;
@group(0)
@binding(3)
var circle_sampler: sampler;
@group(0)
@binding(4)
var circle_progress_texture: texture_2d<f32>;
@fragment
fn fragment_entry (vertex_fragment: VertexFragment) -> @location(0) vec4<f32> {
    var coverage = textureSampleLevel(
        circle_texture,
        circle_sampler,
        vertex_fragment.texture_coordinates,
        vertex_fragment.mip
    ).r;
    if (vertex_fragment.ring != 0.0) {
        coverage = textureSampleLevel(
            circle_ring_texture,
            circle_sampler,
            vertex_fragment.texture_coordinates,
            vertex_fragment.mip
        ).r;
        let prog = textureSampleLevel(
            circle_progress_texture,
            circle_sampler,
            vertex_fragment.texture_coordinates,
            vertex_fragment.mip
        ).r;
        if (prog > vertex_fragment.prog) {
            coverage = 0.0;
        }
    }
    return vec4<f32>(vertex_fragment.color.rgb, vertex_fragment.color.a * coverage);
}