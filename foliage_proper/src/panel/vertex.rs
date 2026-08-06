use crate::Coordinates;

/// One unit quad. The rounding lives entirely in `panel.wgsl`'s distance field now, so
/// there is no per-corner geometry to carve the mesh into -- the fragment shader decides
/// what is inside the shape.
pub(crate) const VERTICES: [Coordinates; 6] = [
    Coordinates::new(1f32, 0f32),
    Coordinates::new(0f32, 0f32),
    Coordinates::new(0f32, 1f32),
    Coordinates::new(1f32, 0f32),
    Coordinates::new(0f32, 1f32),
    Coordinates::new(1f32, 1f32),
];
