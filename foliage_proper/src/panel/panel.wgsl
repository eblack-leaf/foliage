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
    var coverage = 0.0;
    if (frag.segment == 0) {
        coverage = corner(frag.corner, interval, dist);
    } else if (frag.segment == 1) {
        if (half_weight < 0) {
            coverage = 1.0;
        } else {
            let center = frag.section.y + half_weight;
            let actual = abs(frag.position.y - frag.section.y);
            coverage = step(half_weight, actual);
        }
    } else if (frag.segment == 2) {
        coverage = corner(frag.corner, interval, dist);
    } else if (frag.segment == 3) {
        if (half_weight < 0) {
            coverage = 1.0;
        } else {
            let center = frag.section.x + half_weight;
            let actual = abs(frag.position.x - frag.section.x);
            coverage = step(half_weight, actual);
        }
    } else if (frag.segment == 4) {
        // all segments min
    } else if (frag.segment == 5) {
        if (half_weight < 0) {
            coverage = 1.0;
        } else {
            let right = frag.section.x + frag.section.z;
            let center = right - half_weight;
            let actual = abs(frag.position.x - right);
            coverage = step(half_weight, actual);
        }
    } else if (frag.segment == 6) {
        coverage = corner(frag.corner, interval, dist);
    } else if (frag.segment == 7) {
        if (half_weight < 0) {
            coverage = 1.0;
        } else {
            let bottom = frag.section.y + frag.section.w;
            let center = bottom - half_weight;
            let actual = abs(frag.position.y - bottom);
            coverage = step(half_weight, actual);
        }
    } else if (frag.segment == 8) {
        coverage = corner(frag.corner, interval, dist);
    }
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}