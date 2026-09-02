//! The depth attachment, and the range an element's rank is drawn at.

use wgpu::{
    Extent3d, LoadOp, Operations, RenderPassDepthStencilAttachment, StoreOp, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

/// The depth buffer, sized with the surface.
pub(crate) struct Depth {
    view: TextureView,
}

impl Depth {
    pub(crate) const FORMAT: TextureFormat = TextureFormat::Depth32Float;

    /// Where an element's rank is placed.
    ///
    /// Open at both ends: nothing is drawn at exactly the near or the far plane, so the range
    /// always has room on either side of what is in it. `0.0` is nearest the viewer, which is the
    /// opposite sense to [`ResolvedElevation`](crate::elevation::ResolvedElevation) -- the
    /// front-most element takes the smallest depth.
    pub(crate) const NEAR: f32 = 0.0;
    pub(crate) const FAR: f32 = 1.0;

    pub(crate) fn new(device: &wgpu::Device, configuration: &SurfaceConfiguration) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("depth"),
            size: Extent3d {
                width: configuration.width,
                height: configuration.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[Self::FORMAT],
        });
        Self {
            view: texture.create_view(&TextureViewDescriptor::default()),
        }
    }

    /// The depth `index` of `total` sits at, front-most last.
    ///
    /// Evenly spaced across the range and strictly inside it, so two elements never share a depth
    /// and the ordering the resolver produced is the ordering the depth test enforces. Taken from
    /// the position in the sorted order rather than from the rank itself, because a rank is a pair
    /// of an accumulated elevation and an allocation counter and has no meaningful magnitude to
    /// scale -- only an order, which is exactly what a position is.
    ///
    /// The range therefore holds any number of elements: it subdivides rather than fills, so there
    /// is no budget to exhaust and no value an element can be given that leaves no room beside it.
    /// The one real ceiling is the attachment's precision -- the step is `1 / (total + 1)`, and
    /// [`FORMAT`](Depth::FORMAT) separates two depths until that falls under an `f32` interval,
    /// which is on the order of eight million elements. Assigning from the rank's own magnitude is
    /// what would need a budget, and then a rebalance whenever a gap in it ran out.
    pub(crate) fn of(index: usize, total: usize) -> f32 {
        let step = (index + 1) as f32 / (total + 1) as f32;
        Self::FAR - step * (Self::FAR - Self::NEAR)
    }

    pub(crate) fn attachment(&self) -> RenderPassDepthStencilAttachment<'_> {
        RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(Self::FAR),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
}
