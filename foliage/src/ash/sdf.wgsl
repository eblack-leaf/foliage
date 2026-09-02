// Rounded-rectangle coverage, prepended to every shader that draws one. WGSL has no include, and
// a renderer holding its own copy would be free to drift from the rest -- the point of one file is
// that everything rounding a box rounds it identically.

// Signed distance to a box with independent corner radii. `p` is relative to the box's centre,
// `half_extent` is half its size, and `radii` is (top left, top right, bottom right, bottom left)
// with +y pointing down the screen -- the order `Corners` holds them in. Negative inside, positive
// outside, in `p`'s own units, which are logical pixels.
//
// Each corner is a disc of radius `r` set `r` in from both its edges, so the field is exact while
// `r` is no larger than the shorter half-side. `Rounding` caps it there.
fn sd_rounded_box(p: vec2<f32>, half_extent: vec2<f32>, radii: vec4<f32>) -> f32 {
    let top = select(radii.y, radii.x, p.x < 0.0);
    let bottom = select(radii.z, radii.w, p.x < 0.0);
    let r = select(bottom, top, p.y < 0.0);
    let q = abs(p) - half_extent + r;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

// Distance to alpha, over a band one device pixel wide.
//
// The width is taken from the field's own screen-space derivative rather than assumed, which is
// what makes the edge read the same on a display of any density: the distance is in logical pixels,
// so how many of them one device pixel spans is exactly what the derivative measures.
fn sd_coverage(d: f32) -> f32 {
    let width = max(fwidth(d), 1e-5);
    return 1.0 - smoothstep(-width * 0.5, width * 0.5, d);
}
