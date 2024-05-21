@group(0)
@binding(0)
var viewport: mat4x4<f32>;
@group(0)
@binding(1)
var image_sampler: sampler;
@group(1)
@binding(0)
var image_texture: texture_2d<f32>;
struct Vertex{};
struct Fragment {};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    return Fragment();
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}