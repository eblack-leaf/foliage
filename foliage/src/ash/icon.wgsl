@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

@group(1) @binding(0)
var sheet: texture_2d<f32>;
@group(1) @binding(1)
var sheet_sampler: sampler;

struct Vertex {
    // The unit quad, one corner per vertex.
    @location(0) corner: vec2<f32>,
    // Per instance: the square the mark is drawn in, what it is filled with, and where its field
    // sits on the sheet.
    @location(1) section: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) sheet_rect: vec4<f32>,
    // How many screen pixels the field's baked distance range covers at this instance's size.
    @location(4) range: f32,
    // Per instance, and the backend's own: where the element's rank placed it in the stack.
    @location(5) depth: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    @location(1) sheet_at: vec2<f32>,
    @location(2) @interpolate(flat) range: f32,
};

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let position = vec4<f32>(
        vertex.section.xy + vertex.corner * vertex.section.zw,
        vertex.depth,
        1.0,
    );
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.sheet_rect.xy + vertex.corner * vertex.sheet_rect.zw,
        vertex.range,
    );
}

fn median(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    // Multi-channel signed distance: the median of the three channels reconstructs the true
    // distance while keeping the sharp corners a single channel would round off. Half way is the
    // edge, which is what makes the field independent of the size it is drawn at.
    let field = textureSample(sheet, sheet_sampler, frag.sheet_at);
    let distance = median(field.r, field.g, field.b) - 0.5;
    // The distance is in the field's own units, so it is converted to a screen-space edge one pixel
    // wide by how many screen pixels the baked range spans at this instance's size. Below one, the
    // mark is smaller than its own feather and the clamp keeps the edge from vanishing.
    let coverage = clamp(distance * max(frag.range, 1.0) + 0.5, 0.0, 1.0);
    // The field carries shape and no colour, exactly as a glyph's coverage does: what the mark is
    // filled with is the element's.
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}
