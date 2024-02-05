@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) vertex_tx: vec2<u32>,
    @location(2) position: vec2<f32>,
    @location(3) area: vec2<f32>,
    @location(4) layer: f32,
    @location(5) tex_coords: vec4<f32>,
};
struct VertexFragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> VertexFragment {
    let pos = vec4<f32>(vertex.position + vertex.vertex_pos * vertex.area, vertex.layer, 1.0);
    let tex = vec2<f32>(vertex.tex_coords[vertex.vertex_tx.x], vertex.tex_coords[vertex.vertex_tx.y]);
    return VertexFragment(viewport * pos, tex);
}
@group(1)
@binding(0)
var image_texture: texture_2d<f32>;
@group(0)
@binding(1)
var image_sampler: sampler;
@fragment
fn fragment_entry (vertex_fragment: VertexFragment) -> @location(0) vec4<f32> {
    return textureSample(image_texture, image_sampler, vertex_fragment.texture_coordinates);
}