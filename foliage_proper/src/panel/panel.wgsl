@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_data: vec3<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer_and_weight: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) corner_i: vec4<f32>,
    @location(5) corner_ii: vec4<f32>,
    @location(6) corner_iii: vec4<f32>,
    @location(7) corner_iv: vec4<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) weight: f32,
    @location(2) corner: vec4<f32>,
    @location(3) section: vec4<f32>,
    @location(4) segment: i32,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(
        vertex.section.xy + vertex.vertex_data.xy * vertex.section.zw,
        vertex.layer_and_weight.r,
        1.0
    );
    var corner = vertex.corner_i;
    if (vertex.vertex_data.z == 2) {
        corner = vertex.corner_ii;
    } else if (vertex.vertex_data.z == 6) {
        corner = vertex.corner_iii;
    } else if (vertex.vertex_data.z == 8) {
        corner = vertex.corner_iv;
    }
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.layer_and_weight.y,
        corner,
        vertex.section,
        i32(vertex.vertex_data.z)
    );
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let interval = 0.75;
    let half_weight = 0.5 * frag.weight;
    let dist = distance(frag.position.xy, corner.xy);
    var coverage = 0.0;
    if (frag.segment == 0) {
        let a = smoothstep(frag.corner.z + interval, frag.corner.z - interval, dist);
        var b = 1.0;
        if (frag.corner.w != 0.0) {
            b = smoothstep(frag.corner.w - interval, frag.corner.w + interval, dist);
        }
        coverage = min(a, b);
    } else if (frag.segment == 1) {
        let center = frag.section.y + half_weight;
        let dist = abs(frag.position.y - frag.section.y);
        coverage = step(half_weight, dist);
    }
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}