@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

struct Vertex {
    // The unit quad, one corner per vertex.
    @location(0) corner: vec2<f32>,
    // Per instance: where the panel is, what it is filled with, and how its corners are rounded.
    @location(1) section: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radii: vec4<f32>,
    // Per instance, and the backend's own: where the element's rank placed it in the stack.
    @location(4) depth: f32,
    // Per instance: the line drawn around the inside of the box, and how far in it reaches. Numbered
    // after the depth because it came later, and the depth is where every renderer has it.
    @location(5) outline: vec4<f32>,
    @location(6) outline_width: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    // Where this fragment falls inside the panel, in logical pixels. Carried rather than recovered
    // from `position`, which is in device pixels -- the field is stated in the units the engine is
    // written in, and the derivative in `sd_coverage` converts.
    @location(1) offset: vec2<f32>,
    @location(2) @interpolate(flat) half_extent: vec2<f32>,
    @location(3) @interpolate(flat) radii: vec4<f32>,
    @location(4) @interpolate(flat) outline: vec4<f32>,
    @location(5) @interpolate(flat) outline_width: f32,
};

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let offset = vertex.corner * vertex.section.zw;
    let position = vec4<f32>(vertex.section.xy + offset, vertex.depth, 1.0);
    return Fragment(
        viewport * position,
        vertex.color,
        offset,
        vertex.section.zw * 0.5,
        vertex.radii,
        vertex.outline,
        vertex.outline_width,
    );
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let d = sd_rounded_box(frag.offset - frag.half_extent, frag.half_extent, frag.radii);
    // The box, and the box with the outline taken off its edge: the same field moved in by the
    // outline's width, so the inner edge follows the corners round by what is left of each radius.
    // What lies between the two is the ring. With no outline the two are one, and there is no ring.
    let whole = sd_coverage(d);
    let inside = sd_coverage(d + frag.outline_width);
    let ring = whole - inside;
    // The ring is the outline over the fill, and inside it is the fill alone -- composited here, in
    // the one fragment, as two areas of the pixel, which is what keeps a shared edge from being
    // covered twice. Summed premultiplied, and handed to the blend straight.
    let fill = frag.color.a;
    let line = frag.outline.a;
    let alpha = ring * (line + fill * (1.0 - line)) + inside * fill;
    let premultiplied = ring * (frag.outline.rgb * line + frag.color.rgb * fill * (1.0 - line))
        + inside * frag.color.rgb * fill;
    return vec4<f32>(premultiplied / max(alpha, 1e-5), alpha);
}
