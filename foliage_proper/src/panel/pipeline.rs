use wgpu::RenderPass;
use crate::ash::render::{Parameters, Render, Renderer};
use crate::Panel;
use crate::ash::differential::RenderQueueHandle;
use crate::ash::node::Nodes;
use crate::ginkgo::Ginkgo;

pub(crate) struct Resources {

}
pub(crate) struct Group {

}
impl Render for Panel {
    type Group = Group;
    type Resources = Resources;

    fn renderer(ginkgo: &Ginkgo) -> Renderer<Self> {
        todo!()
    }

    fn prepare(renderer: &mut Renderer<Self>, queues: &mut RenderQueueHandle, ginkgo: &Ginkgo) -> Nodes {
        todo!()
    }

    fn render(renderer: &mut Renderer<Self>, render_pass: &mut RenderPass, parameters: Parameters) {
        todo!()
    }
}