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
    @location(0) vertex_pos: vec2f,
    @location(1) section: vec4f,
    @location(2) layer: f32,
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
    return Fragment();
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    return vec4f(0.0);
}