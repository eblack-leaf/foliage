@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) vertex_pos: vec2<f32>,
    @location(1) left: vec4f,
    @location(2) right: vec4f,
    @location(3) layer: f32,
    @location(4) color: vec4f,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4f,
    @location(1) left: vec4f,
    @location(2) right: vec4f,
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
    return Fragment(
        viewport * vec4f(v, vertex.layer, 1.0),
        vertex.color,
        vertex.left,
        vertex.right,
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
    let edge_precision = 1.0;
    let top = vec4f(frag.left.zw, frag.right.zw);
    let bot = vec4f(frag.left.xy, frag.right.xy);
    let left_inclusion = distance_to_edge(frag.left, frag.position.xy);
    let top_inclusion = distance_to_edge(top, frag.position.xy);
    let right_inclusion = distance_to_edge(frag.right, frag.position.xy);
    let bot_inclusion = distance_to_edge(bot, frag.position.xy);
    let inclusion = min(min(min(left_inclusion, top_inclusion), right_inclusion), bot_inclusion);
    let coverage = smoothstep(0.0, edge_precision, inclusion);
    return vec4f(frag.color.rgb, frag.color.a * coverage);
}