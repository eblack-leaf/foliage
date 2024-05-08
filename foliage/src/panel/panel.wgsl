@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) world_pos: vec2<f32>,
    @location(2) area: vec2<f32>,
    @location(3) layer: f32,
    @location(4) color: vec4<f32>,
    @location(5) corner_i: vec4<f32>,
    @location(6) corner_ii: vec4<f32>,
    @location(7) corner_iii: vec4<f32>,
    @location(8) corner_iv: vec4<f32>,
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
    let pos = vec4<f32>(vertex.world_pos + local_pos, vertex.layer, 1.0);
    var tex = vec3<f32>(0.0, 0.0, 0.0);
    if (local_pos.x > vertex.corner_i.x && local_pos.y < vertex.corner_i.w) {
        let localized = local_pos - vertex.corner_i.xy;
        let t = vec2<f32>(localized / vertex.corner_i.zw) / 2.0;
        tex = vec3<f32>(1.0, t.x + 0.5, t.y);
    }
    if (local_pos.x < vertex.corner_ii.x && local_pos.y < vertex.corner_ii.y) {
        let t = vec2<f32>(local_pos.xy / vertex.corner_ii.zw) / 2.0;
        tex = vec3<f32>(1.0, t.x, t.y);
    }
    if (local_pos.x < vertex.corner_iii.z && local_pos.y > vertex.corner_iii.y) {
        let localized = local_pos - vertex.corner_iii.xy;
        let t = vec2<f32>(localized / vertex.corner_iii.zw) / 2.0;
        tex = vec3<f32>(1.0, t.x, t.y + 0.5);
    }
    if (local_pos.x > vertex.corner_iv.x && local_pos.y > vertex.corner_iv.y) {
        let localized = local_pos - vertex.corner_iv.xy;
        let t = vec2<f32>(localized / vertex.corner_iv.zw) / 2.0;
        tex = vec3<f32>(1.0, t.x + 0.5, t.y + 0.5);
    }
    return Fragment(viewport * pos, vertex.color, tex);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    var out = frag.color;
    if (frag.tex.r != 0.0) {
        let coverage = textureSample(circle_texture, circle_sampler, frag.tex.yz).r;
        out = vec4<f32>(frag.color.rgb, frag.color.a * coverage);
    }
    return out;
}