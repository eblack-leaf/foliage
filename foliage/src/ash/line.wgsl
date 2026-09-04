@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

struct Vertex {
    // The unit quad. `x` runs along the stroke, `y` across it.
    @location(0) corner: vec2<f32>,
    // Per instance: the two ends, the fill, and how thick and how capped.
    @location(1) segment: vec4<f32>,
    @location(2) color: vec4<f32>,
    // Half the weight, whether the ends are round, then half the coverage ramp.
    @location(3) stroke: vec3<f32>,
    // Per instance, and the backend's own: where the element's rank placed it in the stack.
    @location(4) depth: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    // Where this fragment is, in logical pixels. Carried rather than taken from `position`, which
    // is in device pixels: the segment is logical, and the two have to be compared in one unit.
    @location(1) at: vec2<f32>,
    @location(2) @interpolate(flat) segment: vec4<f32>,
    @location(3) @interpolate(flat) stroke: vec3<f32>,
};

// How far past the stroke's own edge the drawn quad reaches, in logical pixels.
//
// The rasteriser only produces a fragment where a pixel *centre* lands inside the triangles, so
// without this the fragment stage is never asked about the pixels the outer half of the feather
// covers -- and at weights near one pixel, whole runs along a shallow diagonal have no centre
// inside the true quad at all, which is what makes a thin stroke break into dashes and crawl as its
// ends move. One logical pixel is at least one device pixel on any display, and comfortably more
// than the widest feather the fragment stage asks for.
const MARGIN: f32 = 1.0;

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let a = vertex.segment.xy;
    let b = vertex.segment.zw;
    let half = vertex.stroke.x;
    let along = b - a;
    let length_of = length(along);
    // A stroke of no length has no direction to be thick in. Across the horizontal keeps a round
    // one a dot of its own weight rather than nothing, which is what a degenerate segment should
    // look like while it is being dragged into existence.
    let direction = select(vec2<f32>(1.0, 0.0), along / length_of, length_of > 0.0);
    let across = vec2<f32>(-direction.y, direction.x);
    // A round cap reaches half a weight past each end, so the quad has to cover it. A butt cap does
    // not, and gets only the margin.
    let reach = half * vertex.stroke.y + MARGIN;
    let run = vertex.corner.x * length_of + (vertex.corner.x * 2.0 - 1.0) * reach;
    let side = (vertex.corner.y * 2.0 - 1.0) * (half + MARGIN);
    let at = a + direction * run + across * side;
    return Fragment(
        viewport * vec4<f32>(at, vertex.depth, 1.0),
        vertex.color,
        at,
        vertex.segment,
        vertex.stroke,
    );
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let a = frag.segment.xy;
    let b = frag.segment.zw;
    let half = frag.stroke.x;
    let along = b - a;
    // Named for what it is rather than where it starts, because `from` is a reserved word in WGSL.
    let offset = frag.at - a;
    let squared = dot(along, along);
    var d: f32;
    if (squared <= 0.0) {
        // No length: a round cap is a disc and a butt cap is nothing.
        d = select(1.0, length(offset) - half, frag.stroke.y > 0.5);
    } else {
        let at = dot(offset, along) / squared;
        if (frag.stroke.y > 0.5) {
            // Distance to the segment, offset by the radius. Exact, and its own cap: the field is
            // round at both ends because a clamped projection is.
            d = length(offset - along * clamp(at, 0.0, 1.0)) - half;
        } else {
            // The same field cut off at each end by the plane through it, which is exactly an
            // oriented box -- and exact, so there is no side to be wound the wrong way and no
            // collapsed edge to guard against.
            let beyond = (abs(at - 0.5) - 0.5) * sqrt(squared);
            d = max(length(offset - along * at) - half, beyond);
        }
    }
    // Half the coverage ramp, stated by the backend rather than taken from a screen-space
    // derivative: the density is the whole of the answer and only the backend knows it, while
    // `fwidth` over this field reports a width that varies with the angle the stroke runs at. The
    // ramp is one device pixel wide, centred on the true edge rather than lying wholly inside it --
    // so a stroke's drawn width is the width it was asked for rather than that less a fading pixel.
    let feather = frag.stroke.z;
    // Linear, where every other shape here uses `smoothstep`. A thin stroke lands across two pixel
    // rows, and what has to stay constant as its centreline drifts between their centres is the
    // *sum* of what those rows paint -- the stroke's apparent weight. A linear ramp sums to exactly
    // that; smoothstep's S-curve sums to slightly less mid-drift than at the ends, which reads as
    // the stroke thinning and thickening along its own length. Nothing else here is thin enough for
    // the difference to show.
    let coverage = clamp(-d / (2.0 * feather) + 0.5, 0.0, 1.0);
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}
