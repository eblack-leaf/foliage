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
    let segment = i32(vertex.vertex_data.z);
    let horizontal_space = vertex.section.z - 2 * vertex.corner_i.x;
    let vertical_space = vertex.section.w - 2 * vertex.corner_i.x;
    let x = vertex.vertex_data.x;
    let y = vertex.vertex_data.y;
    let depth = vertex.corner_i.x;
    let is_a = f32(segment == 0);
    let is_b = f32(segment == 1);
    let is_c = f32(segment == 2);
    let is_d = f32(segment == 3);
    let is_e = f32(segment == 4);
    let is_f = f32(segment == 5);
    let is_g = f32(segment == 6);
    let is_h = f32(segment == 7);
    let is_i = f32(segment == 8);
    let offset = vec2f(x * depth, y * depth) * is_a +
        vec2f(depth + horizontal_space * x, depth * y) * is_b +
        vec2f(depth + horizontal_space + depth * x, depth * y) * is_c +
        vec2f(depth * x, depth + vertical_space * y) * is_d +
        vec2f(depth + horizontal_space * x, depth + vertical_space * y) * is_e +
        vec2f(depth + horizontal_space + depth * x, depth + vertical_space * y) * is_f +
        vec2f(depth * x, depth + vertical_space + depth * y) * is_g +
        vec2f(depth + horizontal_space * x, depth + vertical_space + depth * y) * is_h +
        vec2f(depth + horizontal_space + depth * x, depth + vertical_space + depth * y) * is_i;
    let position = vec4<f32>(
        vertex.section.xy + offset,
        vertex.layer_and_weight.r,
        1.0
    );
    let is_corner_i = f32(segment == 0);
    let is_corner_ii = f32(segment == 2);
    let is_corner_iii = f32(segment == 6);
    let is_corner_iv = f32(segment == 8);
    let corner = vertex.corner_i * is_corner_i +
        vertex.corner_ii * is_corner_ii +
        vertex.corner_iii * is_corner_iii +
        vertex.corner_iv * is_corner_iv +
        vec4f(vertex.section.xy, 0.0, 0.0);
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.layer_and_weight.y,
        corner,
        vertex.section,
        segment
    );
}
fn corner(c: vec4<f32>, interval: f32, dist: f32) -> f32 {
    let a = smoothstep(c.z + interval, c.z - interval, dist);
    let b_valid = f32(c.w > 0.0);
    let b_invalid = f32(c.w <= 0.0);
    let b = 1.0 * b_invalid + smoothstep(c.w - interval, c.w + interval, dist) * b_valid;
    return min(a, b);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let interval = 0.75;
    let half_weight = max(0.5, 0.5 * frag.weight);
    let dist = distance(frag.position.xy, frag.corner.xy);
    let a = 1.0 - step(half_weight, abs(frag.position.y - (frag.section.y + half_weight)));
    let b = 1.0 - step(half_weight, abs(frag.position.x - (frag.section.x + half_weight)));
    let c = 1.0 - step(half_weight, abs(frag.position.x - (frag.section.x + frag.section.z - half_weight)));
    let d = 1.0 - step(half_weight, abs(frag.position.y - (frag.section.y + frag.section.w - half_weight)));
    let e = max(a, max(b, max(c, d)));
    let cor = corner(frag.corner, interval, dist);
    let is_corner = f32(frag.segment == 0 || frag.segment == 2 || frag.segment == 6 || frag.segment == 8);
    let not_corner = f32(is_corner == 0);
    let use_a = f32(frag.segment == 1 && frag.weight >= 0);
    let use_b = f32(frag.segment == 3 && frag.weight >= 0);
    let use_c = f32(frag.segment == 5 && frag.weight >= 0);
    let use_d = f32(frag.segment == 7 && frag.weight >= 0);
    let use_e = f32(frag.segment == 4 && frag.weight >= 0);
    let no_weight = f32(frag.weight < 0);
    let coverage = cor * is_corner +
        a * use_a +
        b * use_b +
        c * use_c +
        d * use_d +
        e * use_e +
        1.0 * not_corner * no_weight;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}