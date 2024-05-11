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
    @location(1) section: vec4<f32>,
    @location(2) corners: vec4<f32>,
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
    let final_pos = vec4<f32>(vertex.world_pos + local_pos, vertex.layer, 1.0);
    return Fragment(viewport * final_pos, vertex.color, vec4<f32>(vertex.world_pos, vertex.area), vertex.corners);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let in_corner_i: bool = frag.position.x > (frag.section.z - frag.corners.x) && frag.position.y < frag.corners.x;
    let in_corner_ii: bool = frag.position.x < frag.corners.y && frag.position.y < frag.corners.y;
    let in_corner_iii: bool = frag.position.x < frag.corners.z && frag.position.y > (frag.section.w - frag.corners.z);
    let in_corner_iv: bool = frag.position.x > (frag.section.z - frag.corners.w) &&
        frag.position.y > (frag.section.w - frag.corners.w);
    let local_i = frag.position.xy - vec2<f32>((frag.section.z - frag.corners.x), 0.0);
    let local_ii = frag.position.xy;
    let local_iii = frag.position.xy - vec2<f32>(0.0, (frag.section.w - frag.corners.z));
    let local_iv = frag.position.xy - vec2<f32>(
        (frag.section.z - frag.corners.w),
        (frag.section.w - frag.corners.w)
    );
    let non_zero_divide_factor = 0.00001;
    let normalized_i = local_i / max(frag.corners.x, non_zero_divide_factor) / 2.0;
    let normalized_ii = local_ii / max(frag.corners.y, non_zero_divide_factor) / 2.0;
    let normalized_iii = local_iii / max(frag.corners.z, non_zero_divide_factor) / 2.0;
    let normalized_iv = local_iv / max(frag.corners.w, non_zero_divide_factor) / 2.0;
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
    let coverage = textureSample(circle_texture, circle_sampler, tex.yz).r * tex.x + 1.0 * f32(tex.x == 0.0);
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}