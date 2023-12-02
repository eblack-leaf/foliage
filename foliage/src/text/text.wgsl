@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
@group(1)
@binding(0)
var<uniform> pos_and_layer: vec4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) vertex_tx: vec2<u32>,
    @location(2) position: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) tx: vec4<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let tx_coord = vec2<f32>(vertex.tx[vertex.vertex_tx.x], vertex.tx[vertex.vertex_tx.y]);
    let position = vec4<f32>(
        pos_and_layer.x + vertex.position.x + vertex.vertex_pos.x * vertex.scale.x,
        pos_and_layer.y + vertex.position.y + vertex.vertex_pos.y * vertex.scale.y,
        pos_and_layer.z,
        1.0
    );
    return Fragment(viewport * position, tx_coord, vertex.color);
}
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
};
@group(1)
@binding(1)
var glyph_texture: texture_2d<f32>;
@group(0)
@binding(1)
var glyph_sampler: sampler;
@fragment
fn fragment_entry(fragment: Fragment) -> @location(0) vec4<f32> {
    let coverage = textureSample(
        glyph_texture,
        glyph_sampler,
        fragment.texture_coordinates,
    ).r;
    return vec4<f32>(fragment.color.rgb, fragment.color.a * coverage);
}