@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) world_pos: vec2<f32>,
    @location(2) area: vec2<f32>,
    @location(3) layer: f32,
    @location(4) color: vec4<f32>,
    @location(5) corners: vec4<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex: vec3<f32>,
};
@group(0)
@binding(1)
var circle_texture: texture_2d<f32>;
@group(0)
@binding(2)
var circle_sampler: sampler;
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let local_pos: vec2<f32> = vertex.vertex_pos * vertex.area;
    let world_pos = vec4<f32>(vertex.world_pos + local_pos, vertex.layer, 1.0);
    let in_corner_i: bool = local_pos.x > (vertex.area.x - vertex.corners.x) && local_pos.y < vertex.corners.x;
    let in_corner_ii: bool = local_pos.x < vertex.corners.y && local_pos.y < vertex.corners.y;
    let in_corner_iii: bool = local_pos.x < vertex.corners.z && local_pos.y > (vertex.area.y - vertex.corners.z);
    let in_corner_iv: bool = local_pos.x > (vertex.area.x - vertex.corners.w) &&
        local_pos.y > (vertex.area.y - vertex.corners.w);
    let local_i = local_pos - vec2<f32>((vertex.area.x - vertex.corners.x), 0.0);
    let local_ii = local_pos;
    let local_iii = local_pos - vec2<f32>(0.0, (vertex.area.y - vertex.corners.z));
    let local_iv = local_pos - vec2<f32>(
        (vertex.area.x - vertex.corners.w),
        (vertex.area.y - vertex.corners.w)
    );
    let non_zero_divide_factor = 0.00001;
    let normalized_i = local_i / max(vertex.corners.x, non_zero_divide_factor) / 2.0;
    let normalized_ii = local_ii / max(vertex.corners.y, non_zero_divide_factor) / 2.0;
    let normalized_iii = local_iii / max(vertex.corners.z, non_zero_divide_factor) / 2.0;
    let normalized_iv = local_iv / max(vertex.corners.w, non_zero_divide_factor) / 2.0;
    let tex = vec3<f32>(
            1.0 * f32(in_corner_i) +
            1.0 * f32(in_corner_ii) +
            1.0 * f32(in_corner_iii) +
            1.0 * f32(in_corner_iv),
            (normalized_i.x + 0.5) * f32(in_corner_i) +
            (normalized_ii.x) * f32(in_corner_ii) +
            (normalized_iii.x) * f32(in_corner_iii) +
            (normalized_iv.x + 0.5) * f32(in_corner_iv),
            (normalized_i.y) * f32(in_corner_i) +
            (normalized_ii.y) * f32(in_corner_ii) +
            (normalized_iii.y + 0.5) * f32(in_corner_iii) +
            (normalized_iv.y + 0.5) * f32(in_corner_iv)
        );
    return Fragment(viewport * world_pos, vertex.color, tex);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let coverage = textureSample(circle_texture, circle_sampler, frag.tex.yz).r * frag.tex.x + 1.0 * f32(frag.tex.x == 0.0);
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}