@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer: f32,
    @location(3) color: vec4<f32>,
    @location(4) corner_i: vec3<f32>,
    @location(5) corner_ii: vec3<f32>,
    @location(6) corner_iii: vec3<f32>,
    @location(7) corner_iv: vec3<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) section: vec4<f32>,
    @location(2) corner_i: vec3<f32>,
    @location(3) corner_ii: vec3<f32>,
    @location(4) corner_iii: vec3<f32>,
    @location(5) corner_iV: vec3<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(
        vertex.section.xy + vertex.vertex_pos * vertex.section.zw,
        vertex.layer,
        1.0
    );
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.section,
        vertex.corner_i,
        vertex.corner_ii,
        vertex.corner_iii,
        vertex.corner_iv
    );
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let coverage = 1.0;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}