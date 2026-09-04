@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

struct Vertex {
    // The unit quad, one corner per vertex.
    @location(0) corner: vec2<f32>,
    // Per instance: the box the shape is inscribed in, its fill, and what shape it is.
    @location(1) section: vec4<f32>,
    @location(2) color: vec4<f32>,
    // Sides, rounding, rotation.
    @location(3) shape: vec3<f32>,
    // Per instance, and the backend's own: where the element's rank placed it in the stack.
    @location(4) depth: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    // Where this fragment falls inside the box, in logical pixels, measured from its centre.
    // Carried rather than recovered from `position`, which is in device pixels -- the field is
    // stated in the units the engine is written in, and `fwidth` converts.
    @location(1) offset: vec2<f32>,
    @location(2) @interpolate(flat) apothem: f32,
    @location(3) @interpolate(flat) shape: vec3<f32>,
};

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let offset = vertex.corner * vertex.section.zw;
    let position = vec4<f32>(vertex.section.xy + offset, vertex.depth, 1.0);
    let half_extent = vertex.section.zw * 0.5;
    return Fragment(
        viewport * position,
        vertex.color,
        offset - half_extent,
        // The shape is inscribed in the largest circle the box holds, so a non-square box leaves
        // room around it rather than distorting it: a regular polygon's rounded corners only stay
        // circular while its own bounds are square.
        min(half_extent.x, half_extent.y),
        vertex.shape,
    );
}

fn floor_mod(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

// Signed distance to a regular polygon of `n` sides with inradius `r`, centred on the origin.
// Quilez's closed form, unrolled from the usual precompute-per-shape version because `n` and `r`
// change per instance here.
fn sd_regular_polygon(at: vec2<f32>, r: f32, n: f32) -> f32 {
    let an = 3.14159265 / n;
    let acs = vec2<f32>(cos(an), sin(an));
    let bn = floor_mod(atan2(at.x, at.y), 2.0 * an) - an;
    var p = length(at) * vec2<f32>(cos(bn), abs(sin(bn)));
    p = p - r * acs;
    p.y = p.y + clamp(-p.y, 0.0, r * acs.y);
    return length(p) * sign(p.x);
}

fn turned(p: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(p.x * c - p.y * s, p.x * s + p.y * c);
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let p = turned(frag.offset, -frag.shape.z);
    // Rounding shrinks the sharp shape's inradius by the round amount and grows the boundary back
    // out by the same -- so at 1.0 the polygon's own inradius degenerates to zero and what is left
    // is a true circle, whatever the side count.
    let round_amount = clamp(frag.shape.y, 0.0, 1.0) * frag.apothem;
    let r = max(frag.apothem - round_amount, 0.0);
    let sides = max(frag.shape.x, 3.0);
    // A fractional side count blends the two whole counts either side. A distance-field blend
    // rather than a vertex-matched morph: cheap, and every endpoint is already rounded, so there is
    // no acute unrounded corner for the blend to make look wrong part way through.
    let d = mix(
        sd_regular_polygon(p, r, floor(sides)),
        sd_regular_polygon(p, r, ceil(sides)),
        fract(sides),
    ) - round_amount;
    let width = max(fwidth(d), 1e-5);
    let coverage = 1.0 - smoothstep(-width * 0.5, width * 0.5, d);
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}
