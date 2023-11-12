use crate::gfx::viewport::Viewport;
use crate::gfx::GfxContext;

pub enum RenderPhase {
    Opaque,
    Alpha(i32),
}
pub(crate) struct Renderer {
    pub(crate) phase: RenderPhase,
    pub(crate) render_bundles: Vec<wgpu::RenderBundle>,
}
pub(crate) struct RendererExecutor {
    pub(crate) renderers: Vec<Renderer>,
}
impl RendererExecutor {
    pub(crate) fn new(renderers: Vec<Renderer>) -> Self {
        Self {
            // split to opaque and alpha
            renderers,
        }
    }
    pub(crate) fn render(&mut self, gfx_context: &GfxContext) {}
}
pub struct RenderPassHandle<'a>(pub wgpu::RenderPass<'a>);
pub trait RenderRecorder {
    fn record(&self, render_pass_handle: &mut RenderPassHandle, viewport: &Viewport);
}
