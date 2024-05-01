use crate::ginkgo::Ginkgo;

pub trait Render {
    fn create(ginkgo: &Ginkgo) -> Self
    where
        Self: Sized;
}
pub(crate) struct Renderer {
    pub(crate) object: Box<dyn Render>,
}
#[derive(Default)]
pub(crate) struct Ash {
    pub(crate) renderers: Vec<Renderer>,
}
