@group(0)
@binding(0)
var<uniform> viewport: mat4x4<f32>;
struct Vertex {
    @builtin(vertex_index) index: u32,
    @location(0) vertex_pos: vec2<f32>,
    @location(1) left: vec4f,
    @location(2) right: vec4f,
    @location(3) layer: f32,
    @location(4) color: vec4f,
    @location(5) opacity: f32,
};
struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4f,
    @location(1) left: vec4f,
    @location(2) right: vec4f,
};
// Physical px the drawn quad is grown by past the line's true edge, on every side. The
// rasterizer only produces a fragment where a pixel *center* lands inside the triangles, so
// without this the fragment stage is never asked about the pixels the outer half of its
// feather covers -- and at weights near 1px, whole runs of pixels along a shallow diagonal
// have no center inside the true quad at all, which is what makes a thin line break into
// dashes and crawl as its endpoints move. One px is comfortably more than the widest feather
// `edge_precision` below ever asks for.
const AA_MARGIN: f32 = 1.0;

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    var v = vec2f(0);
    if (vertex.index == 0 || vertex.index == 5) {
        v = vertex.left.xy;
    } else if (vertex.index == 1) {
        v = vertex.left.zw;
    } else if (vertex.index == 2 || vertex.index == 4) {
        v = vertex.right.zw;
    } else if (vertex.index == 3) {
        v = vertex.right.xy;
    }
    // Grown along the quad's own two axes rather than outward from its center: a thin line's
    // corner is almost straight along its length, so a radial push would put nearly all of
    // the margin on the caps and leave the two long edges -- the ones that need it -- barely
    // moved. `left`/`right` are deliberately *not* adjusted: they travel to the fragment
    // stage as the line's true geometry, and this margin only decides which pixels get asked.
    let center = 0.25 * (vertex.left.xy + vertex.left.zw + vertex.right.xy + vertex.right.zw);
    let across = vertex.left.zw - vertex.left.xy;
    let along = (vertex.right.xy + vertex.right.zw) - (vertex.left.xy + vertex.left.zw);
    let rel = v - center;
    var grown = v;
    if (dot(across, across) > 0.0) {
        let axis = normalize(across);
        grown += axis * sign(dot(rel, axis)) * AA_MARGIN;
    }
    if (dot(along, along) > 0.0) {
        let axis = normalize(along);
        grown += axis * sign(dot(rel, axis)) * AA_MARGIN;
    }
    return Fragment(
        viewport * vec4f(grown, vertex.layer, 1.0),
        vertex.color * vec4f(1.0, 1.0, 1.0, vertex.opacity),
        vertex.left,
        vertex.right,
    );
}
// Signed: positive inside the quad, negative out. Unsigned would do when the rasterizer only
// ever hands over interior pixels, but the vertex stage now grows the drawn quad past the
// true edge on purpose, so the pixels in that margin have to come back negative -- with
// `abs` they would report themselves as being just as far *inside* and paint at full
// coverage, i.e. a line a whole `AA_MARGIN` too fat on every side.
fn signed_distance_to_edge(edge: vec4f, pt: vec2f, center: vec2f, edge_precision: f32) -> f32 {
    let line_dir = edge.zw - edge.xy;
    // a collapsed corner (triangle via a degenerate quad side) makes line_dir zero --
    // normalize(vec2f(0,0)) is NaN, and that NaN would poison the min() below regardless
    // of which of the four sides (left/right from instance data, or top/bot synthesized
    // from adjacent corners) went degenerate. `edge_precision` itself is the right non-
    // constraining value here: a collapsed side isn't a real boundary of the shape, so it
    // should never be the edge that decides coverage (it is where the coverage ramp below
    // already tops out, so this cannot be the min).
    if (dot(line_dir, line_dir) == 0.0) {
        return edge_precision;
    }
    var normal = normalize(vec2f(line_dir.y, -line_dir.x));
    // Turned to face the quad's interior. The four sides are not wound consistently -- two
    // come from instance data and two are synthesized from adjacent corners, and either pair
    // flips with the direction the line was authored in -- so the interior has to be found
    // rather than assumed.
    if (dot(normal, center - edge.xy) < 0.0) {
        normal = -normal;
    }
    return dot(normal, pt - edge.xy);
}
@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let top = vec4f(frag.left.zw, frag.right.zw);
    let bot = vec4f(frag.left.xy, frag.right.xy);
    let center = 0.25 * (frag.left.xy + frag.left.zw + frag.right.xy + frag.right.zw);
    // half of the line's own actual rendered width -- caps the AA feather so a line
    // thinner than the nominal 1px feather still reaches full coverage at its centerline,
    // for any angle. Previously only axis-aligned edges got an escape hatch for this (a
    // flat +1 bias on that edge's distance), which is why a thin diagonal line needed its
    // CPU-side geometry padded out just to read as fully opaque -- see
    // `Line::distill_descriptor`'s (now removed) `angle_bias`.
    let half_weight = distance(frag.left.xy, frag.left.zw) * 0.5;
    // Half the feather's width, so the ramp spans one device pixel -- and it is centered on
    // the true edge now rather than lying wholly inside it, which is what `Polygon` has
    // always done. A line's drawn width is therefore its stated width, not its stated width
    // less a fading px, and `Polyline` no longer has to shrink its joints to match.
    let edge_precision = min(0.5, half_weight);
    let left_inclusion = signed_distance_to_edge(frag.left, frag.position.xy, center, edge_precision);
    let top_inclusion = signed_distance_to_edge(top, frag.position.xy, center, edge_precision);
    let right_inclusion = signed_distance_to_edge(frag.right, frag.position.xy, center, edge_precision);
    let bot_inclusion = signed_distance_to_edge(bot, frag.position.xy, center, edge_precision);
    let inclusion = min(min(min(left_inclusion, top_inclusion), right_inclusion), bot_inclusion);
    // Linear, where every other shape here uses `smoothstep`. A thin line lands across two
    // pixel rows, and what has to stay constant as its centerline drifts between their
    // centers is the *sum* of what those two rows paint -- the line's apparent weight. A
    // linear ramp sums to exactly that; smoothstep's S-curve sums to slightly less mid-drift
    // than at the ends, which is the same shimmer read as thinning and thickening along the
    // length. Nothing else here is thin enough for the difference to be visible.
    let coverage = clamp(inclusion / (2.0 * edge_precision) + 0.5, 0.0, 1.0);
    return vec4f(frag.color.rgb, frag.color.a * coverage);
}