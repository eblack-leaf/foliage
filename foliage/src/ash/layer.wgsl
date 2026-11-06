// Lays a picture drawn apart onto the window: what something drawn together was drawn into while it
// fades, faded once, whole.

@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

// The picture, the size of the window and in its pixels, so a fragment reads the texel under it.
@group(1) @binding(0)
var picture: texture_2d<f32>;

struct Vertex {
    @builtin(vertex_index) index: u32,
    // Per picture: how opaque it is laid down, and where in the stack.
    @location(0) opacity: f32,
    @location(1) depth: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) opacity: f32,
};

// One triangle over the whole target, cut down to what was drawn into the picture by the scissor.
@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let x = f32((vertex.index << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vertex.index & 2u) * 2.0 - 1.0;
    return Fragment(vec4<f32>(x, y, vertex.depth, 1.0), vertex.opacity);
}

// The picture is premultiplied -- each element was blended onto clear, which leaves its colour
// scaled by its own coverage -- so fading it is scaling all four channels, and it is laid down with
// premultiplied blending.
@fragment
fn fragment_entry(fragment: Fragment) -> @location(0) vec4<f32> {
    let texel = textureLoad(picture, vec2<i32>(fragment.position.xy), 0);
    return texel * fragment.opacity;
}
