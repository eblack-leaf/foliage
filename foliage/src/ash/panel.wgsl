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
    );
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    let d = sd_rounded_box(frag.offset - frag.half_extent, frag.half_extent, frag.radii);
    return vec4<f32>(frag.color.rgb, frag.color.a * sd_coverage(d));
}
