@group(0) @binding(0)
var<uniform> viewport: mat4x4<f32>;

@group(1) @binding(0)
var sheet: texture_2d<f32>;
@group(1) @binding(1)
var sheet_sampler: sampler;

struct Vertex {
    // The unit quad, one corner per vertex.
    @location(0) corner: vec2<f32>,
    // Per instance: where the ink is, what the run it belongs to is filled with, and where the
    // glyph sits on the sheet.
    @location(1) section: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) sheet_rect: vec4<f32>,
    // Per instance, and the backend's own. Every glyph of a run carries its run's depth: a run is
    // one entry in the one stack, and its glyphs are all at that one place in it.
    @location(4) depth: f32,
};

struct Fragment {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    @location(1) sheet_at: vec2<f32>,
};

@vertex
fn vertex_entry(vertex: Vertex) -> Fragment {
    let offset = vertex.corner * vertex.section.zw;
    let position = vec4<f32>(vertex.section.xy + offset, vertex.depth, 1.0);
    return Fragment(
        viewport * position,
        vertex.color,
        vertex.sheet_rect.xy + vertex.corner * vertex.sheet_rect.zw,
    );
}

@fragment
fn fragment_entry(frag: Fragment) -> @location(0) vec4<f32> {
    // The sheet holds coverage and no colour: what a glyph is filled with is the element's, and a
    // glyph's own contribution is how much of the pixel it covers.
    let coverage = textureSample(sheet, sheet_sampler, frag.sheet_at).r;
    return vec4<f32>(frag.color.rgb, frag.color.a * coverage);
}
