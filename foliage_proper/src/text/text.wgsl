@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
@group(0)
@binding(1)
var text_sampler: sampler;
@group(1)
@binding(0)
var text_texture: texture_2d<f32>;
@group(1)
@binding(1)
var<uniform> pos_and_layer: vec4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2f,
    @location(1) tx_index: vec2<u32>,
    @location(2) section: vec4f,
    @location(3) color: vec4f,
    @location(4) tex_coords: vec4f
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex: vec2<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let tex_coord = vec2f(vertex.tex_coords[vertex.tx_index.x], vertex.tex_coords[vertex.tx_index.y]);
    let position = vec4f(pos_and_layer.xy + vertex.vertex_pos * vertex.section.zw
                         + vertex.section.xy, pos_and_layer.z, 1.0);
    return Fragment(
        viewport * position,
        vertex.color,
        tex_coord
    );
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let coverage = textureSample(text_texture, text_sampler, frag.tex).r;
    return vec4f(frag.color.rgb, frag.color.a * coverage);
}