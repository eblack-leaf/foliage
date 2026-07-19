@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @location(0) vertex_pos: vec2<f32>,
    @location(1) section: vec4<f32>,
    @location(2) layer: f32,
    @location(3) color: vec4<f32>,
    @location(4) opacity: f32,
    @location(5) params: vec3<f32>,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4f,
    @location(1) section: vec4f,
    @location(2) params: vec3f,
};
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let world_pos = vertex.section.xy + vertex.vertex_pos * vertex.section.zw;
    return Fragment(
        viewport * vec4f(world_pos, vertex.layer, 1.0),
        vertex.color * vec4f(1.0, 1.0, 1.0, vertex.opacity),
        vertex.section,
        vertex.params,
    );
}
fn floor_mod(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}
// signed distance to a regular polygon with `n` sides and apothem `r`, centered at
// origin -- Quilez's closed-form (2D distance functions), unrolled from the usual
// "precompute per-shape" version since `n`/`r` change every instance here.
fn sd_regular_polygon(p_in: vec2f, r: f32, n: f32) -> f32 {
    let an = 3.14159265 / n;
    let acs = vec2f(cos(an), sin(an));
    let bn = floor_mod(atan2(p_in.x, p_in.y), 2.0 * an) - an;
    var p = length(p_in) * vec2f(cos(bn), abs(sin(bn)));
    p = p - r * acs;
    p.y = p.y + clamp(-p.y, 0.0, r * acs.y);
    return length(p) * sign(p.x);
}
fn rotate(p: vec2f, angle: f32) -> vec2f {
    let c = cos(angle);
    let s = sin(angle);
    return vec2f(p.x * c - p.y * s, p.x * s + p.y * c);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let center = frag.section.xy + frag.section.zw * 0.5;
    let apothem = min(frag.section.z, frag.section.w) * 0.5;
    var p = frag.position.xy - center;
    p = rotate(p, -frag.params.z);
    // rounded-shape trick: shrink the sharp polygon's apothem by the round amount, then
    // grow the boundary back out by that same amount -- at rounding=1 this degenerates
    // sd_regular_polygon's own apothem to 0 (a circle), so full rounding always lands on
    // a true circle regardless of side count.
    let round_amount = clamp(frag.params.y, 0.0, 1.0) * apothem;
    let r = max(apothem - round_amount, 0.0);
    let sides = max(frag.params.x, 3.0);
    let n0 = floor(sides);
    let n1 = ceil(sides);
    let d0 = sd_regular_polygon(p, r, n0);
    let d1 = sd_regular_polygon(p, r, n1);
    // side-count "morph" is a distance-field blend, not a vertex-matched interpolation --
    // cheap, and every endpoint is already rounded, so there's no acute unrounded corner
    // for the blend to make look wrong mid-transition.
    let d = mix(d0, d1, fract(sides)) - round_amount;
    let aa = max(fwidth(d) * 0.5, 0.0001);
    let coverage = smoothstep(aa, -aa, d);
    return vec4f(frag.color.rgb, frag.color.a * coverage);
}
