@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
@group(0)
@binding(1)
var icon_sampler: sampler;
@group(1)
@binding(0)
var icon_texture: texture_2d<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer: f32,
    @location(3) color: vec4<f32>,
    @location(4) mips: f32,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex: vec2<f32>,
    @location(2) mips: f32,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(vertex.section.xy + vertex.section.zw * vertex.vertex_pos, vertex.layer, 1.0);
    return Fragment(
        viewport * position, vertex.color, vertex.vertex_pos, vertex.mips
    );
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let coverage = textureSampleLevel(icon_texture, icon_sampler, frag.tex, frag.mips).r;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}