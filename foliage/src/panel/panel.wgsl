@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) vertex_tx: vec2<f32>,
    @location(2) vertex_hook: vec2<f32>,
    @location(3) position: vec2<f32>,
    @location(4) area: vec2<f32>,
    @location(5) layer: f32,
    @location(6) color: vec4<f32>,
    @location(7) ring: f32,
};
struct VertexFragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) ring: f32,
};
@vertex
fn vertex_entry(vertex: Vertex) -> VertexFragment {
    let listen = vec2<f32>(vertex.area.x * vertex.vertex_hook.x, vertex.area.y * vertex.vertex_hook.y);
    let pos = vec4<f32>(vertex.position + vertex.vertex_pos + listen, vertex.layer, 1.0);
    return VertexFragment(viewport * pos, vertex.vertex_tx, vertex.color, vertex.ring);
}
@group(0)
@binding(1)
var panel_texture: texture_2d<f32>;
@group(0)
@binding(2)
var panel_ring_texture: texture_2d<f32>;
@group(0)
@binding(3)
var panel_sampler: sampler;
@fragment
fn fragment_entry (vertex_fragment: VertexFragment) -> @location(0) vec4<f32> {
    var coverage = textureSample(panel_texture, panel_sampler, vertex_fragment.texture_coordinates).r;
    if (vertex_fragment.ring != 0.0) {
        coverage = textureSample(panel_ring_texture, panel_sampler, vertex_fragment.texture_coordinates).r;
    }
    if (coverage == 0.0) {
        discard;
    }
    return vec4<f32>(vertex_fragment.color.rgb, vertex_fragment.color.a * coverage);
}