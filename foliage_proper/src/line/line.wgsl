@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) vertex_pos: vec2<f32>,
    @location(1) start: vec4f,
    @location(2) end: vec4f,
    @location(3) top_edge: vec4f,
    @location(4) bot_edge: vec4f,
    @location(5) percent_and_layer: vec3f,
    @location(6) color: vec4f,
    @location(7) vertex_offsets: vec4f,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4f,
    @location(1) start: vec4f,
    @location(2) end: vec4f,
    @location(3) top_edge: vec4f,
    @location(4) bot_edge: vec4f,
    @location(5) percent: vec2f,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    if (vertex.index == 0 || vertex.index == 5) {
        let start_top = vertex.start.xy;
    } else if (vertex.index == 1) {
        let start_bottom = vertex.start.zw;
    } else if (vertex.index == 2 || vertex.index == 4) {
        let end_bottom = vertex.end.zw;
    } else if (vertex.index == 3) {
        let end_top = vertex.end.xy;
    }
    // viewport * point_derived + send info
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {

}