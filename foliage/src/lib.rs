use std::rc::Rc;

pub struct Foliage {

}
pub(crate) struct EventLoop<R>(pub(crate) Option<winit::event_loop::EventLoop<R>>);
pub(crate) struct Window(pub(crate) Option<Rc<winit::window::Window>>);
pub(crate) struct EventLoopProxy<M>(pub(crate) winit::event_loop::EventLoopProxy<M>);
pub(crate) struct Surface(pub(crate) Option<wgpu::Surface>);
pub(crate) struct Adapter(pub(crate) Option<wgpu::Adapter>);
pub(crate) struct Device(pub(crate) Option<wgpu::Device>);
pub(crate) struct Queue(pub(crate) Option<wgpu::Queue>);
pub(crate) struct
