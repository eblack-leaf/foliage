@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
@group(1)
@binding(0)
var<uniform> pos_and_layer: vec4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) vertex_tx: vec2<f32>,
    @location(2) position: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) tx: vec4<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    return Fragment();
}
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coordinates: vec2<f32>,
    @location(1) color: vec4<f32>,
};
@group(1)
@binding(1)
var rectangle_texture: texture_2d<f32>;
@group(0)
@binding(1)
var rectangle_sampler: sampler;
@fragment
fn fragment_entry(fragment: Fragment) -> @location(0) vec4<f32> {
    return vec4<f32>();
}