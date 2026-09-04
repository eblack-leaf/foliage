@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

@group(1) @binding(0)
var picture: texture_2d<f32>;
@group(1) @binding(1)
var picture_sampler: sampler;

struct Vertex {
    // The unit quad, one corner per vertex.
    @location(0) corner: vec2<f32>,
    // Per instance: the box the pixels are drawn into, what part of the picture is shown, and how
    // the box's corners are rounded.
    @location(1) section: vec4<f32>,
    @location(2) crop: vec4<f32>,
    @location(3) radii: vec4<f32>,
    // The element's resolved opacity. Carried rather than folded into a colour, because a picture
    // has no colour of its own for it to be folded into.
    @location(4) opacity: f32,
    // Per instance, and the backend's own: where the element's rank placed it in the stack.
    @location(5) depth: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) at: vec2<f32>,
    @location(1) @interpolate(flat) opacity: f32,
    // Where this fragment falls inside the box, in logical pixels. The same field a panel rounds
    // by, in the same units, so a full-bleed picture sits flush inside a rounded card.
    @location(2) offset: vec2<f32>,
    @location(3) @interpolate(flat) half_extent: vec2<f32>,
    @location(4) @interpolate(flat) radii: vec4<f32>,
};

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let offset = vertex.corner * vertex.section.zw;
    let position = vec4<f32>(vertex.section.xy + offset, vertex.depth, 1.0);
    return Fragment(
        viewport * position,
        vertex.crop.xy + vertex.corner * vertex.crop.zw,
        vertex.opacity,
        offset,
        vertex.section.zw * 0.5,
        vertex.radii,
    );
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    // The pixels cover the whole box: the corners are cut by alpha rather than by cropping the
    // sample, so a cropped picture stays full-bleed right up to the curve.
    let d = sd_rounded_box(frag.offset - frag.half_extent, frag.half_extent, frag.radii);
    let color = textureSample(picture, picture_sampler, frag.at);
    return color * vec4<f32>(1.0, 1.0, 1.0, frag.opacity * sd_coverage(d));
}
