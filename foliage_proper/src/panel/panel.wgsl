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
    var offset = vec2<f32>(0.0, 0.0);
    let segment = i32(vertex.vertex_data.z);
    let horizontal_space = vertex.section.z - 2 * vertex.corner_i.x;
    let vertical_space = vertex.section.w - 2 * vertex.corner_i.x;
    let x = vertex.vertex_data.x;
    let y = vertex.vertex_data.y;
    let depth = vertex.corner_i.x;
    if (segment == 0) {
        offset = vec2f(x * depth, y * depth);
    } else if (segment == 1) {
        offset = vec2f(depth + horizontal_space * x, depth * y);
    } else if (segment == 2) {
        offset = vec2f(depth + horizontal_space + depth * x, depth * y);
    } else if (segment == 3) {
        offset = vec2f(depth * x, depth + vertical_space * y);
    } else if (segment == 4) {
        offset = vec2f(depth + horizontal_space * x, depth + vertical_space * y);
    } else if (segment == 5) {
        offset = vec2f(depth + horizontal_space + depth * x, depth + vertical_space * y);
    } else if (segment == 6) {
        offset = vec2f(depth * x, depth + vertical_space + depth * y);
    } else if (segment == 7) {
        offset = vec2f(depth + horizontal_space * x, depth + vertical_space + depth * y);
    } else if (segment == 8) {
        offset = vec2f(depth + horizontal_space + depth * x, depth + vertical_space + depth * y);
    }
    let position = vec4<f32>(
        vertex.section.xy + offset,
        vertex.layer_and_weight.r,
        1.0
    );
    var corner = vertex.corner_i;
    if (segment == 2) {
        corner = vertex.corner_ii;
    } else if (segment == 6) {
        corner = vertex.corner_iii;
    } else if (segment == 8) {
        corner = vertex.corner_iv;
    }
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
    var b = 1.0;
    if (c.w != 0.0) {
        b = smoothstep(c.w - interval, c.w + interval, dist);
    }
    return min(a, b);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let interval = 0.75;
    let half_weight = 0.5 * frag.weight;
    let dist = distance(frag.position.xy, frag.corner.xy);
    let a = step(half_weight, abs(frag.position.y - (frag.section.y + half_weight)));
    let b = step(half_weight, abs(frag.position.x - (frag.section.x + half_weight)));
    let c = step(half_weight, abs(frag.position.y - (frag.section.y + frag.section.w - half_weight)));
    let d = step(half_weight, abs(frag.position.y - (frag.section.x + frag.section.z - half_weight)));
    let e = min(a, min(b, min(c, d)));
    let cor = corner(frag.corner, interval, dist);
    let is_corner = f32(frag.segment == 0 || frag.segment == 2 || frag.segment == 6 || frag.segment == 8);
    let not_corner = f32(is_corner == 0);
    let use_a = f32(frag.segment == 1 && half_weight >= 0);
    let use_b = f32(frag.segment == 3 && half_weight >= 0);
    let use_c = f32(frag.segment == 5 && half_weight >= 0);
    let use_d = f32(frag.segment == 7 && half_weight >= 0);
    let use_e = f32(frag.segment == 4 && half_weight >= 0);
    let no_weight = f32(half_weight < 0);
    let coverage = cor * is_corner + a * use_a + b * use_b + c * use_c + d * use_d + e * use_e + 1.0 * not_corner * no_weight;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}