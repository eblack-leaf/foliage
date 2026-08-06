// Rounded-rectangle coverage, prepended to every shader that draws one (see the
// `format!` in each pipeline's `renderer`). WGSL has no `#include`, and a renderer that
// duplicated these three functions would be free to drift from `Panel`'s rounding -- the
// whole point of the shared file is that `Image` rounds identically, so a full-bleed image
// can sit flush inside a rounded panel.

// Inigo Quilez's rounded-box signed distance field, extended to independent per-corner
// radii. `p` is relative to the box's centre, `b` is its half-extent, and `radii` is
// (top_left, top_right, bottom_left, bottom_right) with +y pointing down the screen.
// Negative inside, positive outside, in `p`'s own units.
//
// Each corner is a disc of radius `r` centred `r` in from both its edges, so the field is
// exact only while `r <= min(b.x, b.y)`; `Rounding::depth` caps it there.
fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, radii: vec4<f32>) -> f32 {
    let pair = select(radii.zw, radii.xy, p.y < 0.0);
    let r = select(pair.y, pair.x, p.x < 0.0);
    let q = abs(p) - b + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

// Re-reads a filled shape's field as a ring `weight` wide lying just inside the boundary.
// One pixel is the floor: a hairline outline still has to cover whole pixels to be seen.
fn sd_outline(d: f32, weight: f32) -> f32 {
    let w = max(weight, 1.0);
    return abs(d + w * 0.5) - w * 0.5;
}

// Distance to alpha. Callers pass distances in physical pixels, so the one-pixel band
// centred on the boundary approximates the fraction of the pixel the shape covers.
fn sd_coverage(d: f32) -> f32 {
    return 1.0 - smoothstep(-0.5, 0.5, d);
}
