@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer_and_weight: vec2f,
    @location(3) color: vec4<f32>,
    @location(4) corner_i: vec3<f32>,
    @location(5) corner_ii: vec3<f32>,
    @location(6) corner_iii: vec3<f32>,
    @location(7) corner_iv: vec3<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) weight: f32,
    @location(2) corner_i: vec3<f32>,
    @location(3) corner_ii: vec3<f32>,
    @location(4) corner_iii: vec3<f32>,
    @location(5) corner_iv: vec3<f32>,
    @location(6) section: vec4<f32>,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(
        vertex.section.xy + vertex.vertex_pos.xy * vertex.section.zw,
        vertex.layer_and_weight.r,
        1.0
    );
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.layer_and_weight.y,
        vertex.corner_i,
        vertex.corner_ii,
        vertex.corner_iii,
        vertex.corner_iv,
        vertex.section
    );
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let interval = 0.75;
    let in_corner_i: bool = frag.position.x >= frag.corner_i.x && frag.position.y <= frag.corner_i.y
        && frag.corner_i.z != 0.0;
    let in_corner_ii: bool = frag.position.x <= frag.corner_ii.x && frag.position.y <= frag.corner_ii.y
        && frag.corner_ii.z != 0.0;
    let in_corner_iii: bool = frag.position.x <= frag.corner_iii.x && frag.position.y >= frag.corner_iii.y
        && frag.corner_iii.z != 0.0;
    let in_corner_iv: bool = frag.position.x >= frag.corner_iv.x && frag.position.y >= frag.corner_iv.y
        && frag.corner_iv.z != 0.0;
    var actual_i = distance(frag.position.xy, frag.corner_i.xy);
    var actual_ii = distance(frag.position.xy, frag.corner_ii.xy);
    var actual_iii = distance(frag.position.xy, frag.corner_iii.xy);
    var actual_iv = distance(frag.position.xy, frag.corner_iv.xy);
    var start_i = frag.corner_i.z - interval;
    var start_ii = frag.corner_ii.z - interval;
    var start_iii = frag.corner_iii.z - interval;
    var start_iv = frag.corner_iv.z - interval;
    var end_i = frag.corner_i.z + interval;
    var end_ii = frag.corner_ii.z + interval;
    var end_iii = frag.corner_iii.z + interval;
    var end_iv = frag.corner_iv.z + interval;
    if frag.weight != 0 {
        let half_weight = frag.weight * 0.5;
        let center_i = frag.corner_i.z - half_weight;
        let center_ii = frag.corner_ii.z - half_weight;
        let center_iii = frag.corner_iii.z - half_weight;
        let center_iv = frag.corner_iv.z - half_weight;
        if actual_i < center_i {
            start_i = frag.corner_i.z - frag.weight + interval;
            end_i = frag.corner_i.z - frag.weight - interval;
        } else {
            start_i = frag.corner_i.z - interval;
            end_i = frag.corner_i.z + interval;
        }
        if actual_ii < center_ii {
            start_ii = frag.corner_ii.z - frag.weight + interval;
            end_ii = frag.corner_ii.z - frag.weight - interval;
        } else {
            start_ii = frag.corner_ii.z - interval;
            end_ii = frag.corner_ii.z + interval;
        }
        if actual_iii < center_iii {
            start_iii = frag.corner_iii.z - frag.weight + interval;
            end_iii = frag.corner_iii.z - frag.weight - interval;
        } else {
            start_iii = frag.corner_iii.z - interval;
            end_iii = frag.corner_iii.z + interval;
        }
        if actual_iv < center_iv {
            start_iv = frag.corner_iv.z - frag.weight + interval;
            end_iv = frag.corner_iv.z - frag.weight - interval;
        } else {
            start_iv = frag.corner_iv.z - interval;
            end_iv = frag.corner_iv.z + interval;
        }
    }
    let corner_i_adjust = smoothstep(start_i, end_i, actual_i) * f32(in_corner_i);
    let corner_ii_adjust = smoothstep(start_ii, end_ii, actual_ii) * f32(in_corner_ii);
    let corner_iii_adjust = smoothstep(start_iii, end_iii, actual_iii) * f32(in_corner_iii);
    let corner_iv_adjust = smoothstep(start_iv, end_iv, actual_iv) * f32(in_corner_iv);
    var normal_value = 0.0;
    if frag.weight != 0 {
        var edge = 0;
        var least_distance_to_edge = frag.position.y - frag.section.y;
        let half_weight = frag.weight * 0.5;
        let edge_1_diff = frag.position.x - frag.section.x;
        if edge_1_diff < least_distance_to_edge {
            edge = 1;
            least_distance_to_edge = edge_1_diff;
        }
        let edge_2_diff = frag.section.y + frag.section.w - frag.position.y;
        if edge_2_diff < least_distance_to_edge {
            edge = 2;
            least_distance_to_edge = edge_2_diff;
        }
        let edge_3_diff = frag.section.x + frag.section.z - frag.position.x;
        if  edge_3_diff < least_distance_to_edge {
            edge = 3;
        }
        if edge == 0 {
            let weight_center = frag.section.y + half_weight;
            let dist = frag.position.y - weight_center;
            normal_value = step(half_weight, abs(dist));
        } else if edge == 1 {
            let weight_center = frag.section.x + half_weight;
            let dist = frag.position.x - weight_center;
            normal_value = step(half_weight, abs(dist));
        } else if edge == 2 {
            let weight_center = frag.section.y + frag.section.w - half_weight;
            let dist = frag.position.y - weight_center;
            normal_value = step(half_weight, abs(dist));
        } else if edge == 3 {
            let weight_center = frag.section.x + frag.section.z - half_weight;
            let dist = frag.position.x - weight_center;
            normal_value = step(half_weight, abs(dist));
        }
    }
    let normal = normal_value * f32(!in_corner_i) * f32(!in_corner_ii) * f32(!in_corner_iii) * f32(!in_corner_iv);
    let coverage = 1.0 - corner_i_adjust - corner_ii_adjust - corner_iii_adjust - corner_iv_adjust - normal;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}