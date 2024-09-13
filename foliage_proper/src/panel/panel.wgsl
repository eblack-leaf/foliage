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
    @location(5) corner_iv: vec3<f32>,
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
    let interval = 1.0;
    let in_corner_i: bool = frag.position.x >= frag.corner_i.x && frag.position.y <= frag.corner_i.y
        && frag.corner_i.z != 0.0;
    let in_corner_ii: bool = frag.position.x <= frag.corner_ii.x && frag.position.y <= frag.corner_ii.y
        && frag.corner_ii.z != 0.0;
    let in_corner_iii: bool = frag.position.x <= frag.corner_iii.x && frag.position.y >= frag.corner_iii.y
        && frag.corner_iii.z != 0.0;
    let in_corner_iv: bool = frag.position.x >= frag.corner_iv.x && frag.position.y >= frag.corner_iv.y
        && frag.corner_iv.z != 0.0;
    let actual_i = distance(frag.position.xy, frag.corner_i.xy);
    let actual_ii = distance(frag.position.xy, frag.corner_ii.xy);
    let actual_iii = distance(frag.position.xy, frag.corner_iii.xy);
    let actual_iv = distance(frag.position.xy, frag.corner_iv.xy);
    let start_i = frag.corner_i.z - interval;
    let start_ii = frag.corner_ii.z - interval;
    let start_iii = frag.corner_iii.z - interval;
    let start_iv = frag.corner_iv.z - interval;
    let end_i = frag.corner_i.z;
    let end_ii = frag.corner_ii.z;
    let end_iii = frag.corner_iii.z;
    let end_iv = frag.corner_iv.z;
    let corner_i_adjust = smoothstep(start_i, end_i, actual_i) * f32(in_corner_i);
    let corner_ii_adjust = smoothstep(start_ii, end_ii, actual_ii) * f32(in_corner_ii);
    let corner_iii_adjust = smoothstep(start_iii, end_iii, actual_iii) * f32(in_corner_iii);
    let corner_iv_adjust = smoothstep(start_iv, end_iv, actual_iv) * f32(in_corner_iv);
    let coverage = 1.0 - corner_i_adjust - corner_ii_adjust - corner_iii_adjust - corner_iv_adjust;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}