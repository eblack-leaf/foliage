@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) vertex_pos: vec2<f32>,
    @location(1) left: vec4f,
    @location(2) top: vec4f,
    @location(3) right: vec4f,
    @location(4) bot: vec4f,
    @location(5) layer: f32,
    @location(6) color: vec4f,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4f,
    @location(1) left: vec4f,
    @location(2) top: vec4f,
    @location(3) right: vec4f,
    @location(4) bot: vec4f,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    var v = vec2f(0);
    if (vertex.index == 0 || vertex.index == 5) {
        v = vertex.left.xy;
    } else if (vertex.index == 1) {
        v = vertex.left.zw;
    } else if (vertex.index == 2 || vertex.index == 4) {
        v = vertex.right.zw;
    } else if (vertex.index == 3) {
        v = vertex.right.xy;
    }
    // viewport * point_derived + send info
    return Fragment(
        viewport * vec4f(v, vertex.layer, 1.0),
        vertex.color,
        vertex.left,
        vertex.top,
        vertex.right,
        vertex.bot,
    );
}
fn distance_to_edge(edge: vec4f, pt: vec2f) -> f32 {
    let line_dir = edge.zw - edge.xy;
    let perpendicular = vec2f(line_dir.y, -line_dir.x);
    let dir_to_pt = edge.xy - pt;
    return abs(dot(normalize(perpendicular), dir_to_pt));
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let edge_precision = 2.0;
    let left_inclusion = distance_to_edge(frag.left, frag.position.xy);
    let top_inclusion = distance_to_edge(frag.top, frag.position.xy);
    let right_inclusion = distance_to_edge(frag.right, frag.position.xy);
    let bot_inclusion = distance_to_edge(frag.bot, frag.position.xy);
    let inclusion = min(min(min(left_inclusion, top_inclusion), right_inclusion), bot_inclusion);
    let coverage = smoothstep(0.0, edge_precision, inclusion);
    return vec4f(frag.color.rgb, frag.color.a * coverage);
}