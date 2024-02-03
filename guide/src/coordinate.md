# Coordinate

The `Coordinate` type is for placing in 3d space. The system is `Orthographic` so the `z` is used for rendering `Layer`
instead. It is clamped from `[0-100)` to allow spacing elements in the `near-far` axis. A `CoordinateUnit` is mapped to
`f32` precision, but as pixels are still integers and cannot describe float values precisely.
The engine needs to handle `x/y` in multiple contexts to ensure correct rendering behavior.
The `InterfaceContext` is used for logical coordinates in screen space. This is the context to use with elements
position and area. The system has a `ScaleFactor` to scale the screen space by to get to the `Device` coordinates.
Smaller screens with more pixels-per-inch will have higher than `1` scale-factors. 
This is used in the conversion `.to_device(scale_factor.factor())` to correctly get the actual px needed to render. 
This is done automatically
to each `Position/Area` that is handed to the predefined `Renderer`s. The `gfx` coordinate system is in `NDC` so 
each `Renderer` will send the `Viewport` matrix to the `wgpu::RenderPipeline` to correctly account for this 
conversion. 

`Viewport` matrix calculation
```rust
fn matrix(section: Section<DeviceContext>, near_far: (Layer, Layer)) -> SMatrix<f32, 4, 4> {
        tracing::trace!("matrix-section: {:?}", section);
        let translation = nalgebra::Matrix::new_translation(&nalgebra::vector![
            section.left(),
            section.top(),
            0f32
        ]);
        let projection = matrix![2f32/(section.right() - section.left()), 0.0, 0.0, -1.0;
                                    0.0, 2f32/(section.top() - section.bottom()), 0.0, 1.0;
                                    0.0, 0.0, 1.0/(near_far.1 - near_far.0).z, 0.0;
                                    0.0, 0.0, 0.0, 1.0];
        projection * translation
    }
```
This adds the `Transform` to move the "Camera" before sending to avoid calculating that in each `shader`.
`wgpu` uses a coordinate system similar to `Vulkan` with z from `0-1` and inverted-y-axis (relative to `OpenGL`).
A `NumericalContext` is also given to denote things that do not prescribe to the screen-coordinates `Interface/Device` 
concerns. 

Each `Position` and `Area` come with helper methods


```rust
position.to_interface(scale_factor.factor());
position.to_device(scale_factor.factor());
position.to_numerical();
position.to_c(); // a #[repr(C)] version
position.normalized(area: Area<_>);
```

A `#[repr(C)]` version of `Position` and `Area` are used to interface with the `C` components used in rendering.
The `gpu` expects a certain format for data and the `C` representation is compatible. 
Thanks [`bytemuck`](https://docs.rs/bytemuck/latest/bytemuck/)!. 

`Section`s are a combination of `Position` and an `Area` of the same `Context`.
This allows more methods such as 
```rust
pub fn width(&self) -> CoordinateUnit {
    self.area.width
}
pub fn height(&self) -> CoordinateUnit {
    self.area.height
}
pub fn left(&self) -> CoordinateUnit {
    self.position.x
}
pub fn right(&self) -> CoordinateUnit {
    self.position.x + self.area.width
}
pub fn top(&self) -> CoordinateUnit {
    self.position.y
}
pub fn bottom(&self) -> CoordinateUnit {
    self.position.y + self.area.height
}
pub fn is_touching(&self, other: Self) -> bool {
    // ...
}
pub fn is_overlapping(&self, other: Self) -> bool {
    // ...
}
pub fn contains(&self, position: Position<Context>) -> bool {
    // ...
}
pub fn intersection(&self, other: Self) -> Option<Self> {
    // ...
}
```
A `Location` is available that combines a `Position` and a `Layer`.
Finally, the `Coordinate` is a combination of a `Section` and `Layer`.