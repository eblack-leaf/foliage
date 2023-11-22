@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_data: vec4<f32>,
    @location(1) position: vec2<f32>,
    @location(2) area: vec2<f32>,
    @location(3) layer: f32,
    @location(4) color: vec4<f32>,
};
struct VertexFragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> VertexFragment {
    let pos = vec4<f32>(vertex.position + vertex.vertex_data.xy * vertex.area, vertex.layer, 1.0);
    return VertexFragment(viewport * pos, vertex.vertex_data.zw, vertex.color);
}
@group(0)
@binding(1)
var panel_texture: texture_2d<f32>;
@group(0)
@binding(2)
var panel_sampler: sampler;
@fragment
fn fragment_entry (vertex_fragment: VertexFragment) -> @location(0) vec4<f32> {
    let coverage = textureSample(panel_texture, panel_sampler, vertex_fragment.texture_coordinates).r;
    if (coverage == 0.0) {
        discard;
    }
    return vec4<f32>(vertex_fragment.color.rgb, vertex_fragment.color.a * coverage);
}