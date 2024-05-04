use crate::Elm;
use crate::ginkgo::Ginkgo;

pub(crate) struct Ash {

}
pub struct RenderDirective(pub(crate) wgpu::RenderBundle);
pub struct RenderDirectiveRecorder<'a>(pub(crate) wgpu::RenderBundleEncoder<'a>);
impl<'a> RenderDirectiveRecorder<'a> {
    pub fn new(ginkgo: &'a Ginkgo) -> Self {
        todo!()
    }
    pub fn finish(self) -> RenderDirective {
        todo!()
    }
}
pub trait Render {
    fn create(ginkgo: &Ginkgo) -> Self;
    type Extraction;
    fn extract(elm: &mut Elm) -> Self::Extraction;
    fn prepare(&mut self, extract: Self::Extraction) -> bool;
    fn record(&self, ginkgo: &Ginkgo) -> Vec<RenderDirective>;
}
