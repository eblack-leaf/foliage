use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::{DeviceContext, NumericalContext};

#[derive(Clone, Default)]
pub(crate) struct WindowHandle(pub(crate) Option<Arc<Window>>);

#[derive(Default)]
pub(crate) struct Willow {
    pub(crate) handle: WindowHandle,
    pub(crate) min_size: Option<Area<DeviceContext>>,
    pub(crate) requested_size: Option<Area<DeviceContext>>,
    pub(crate) title: Option<String>,
    pub(crate) max_size: Option<Area<DeviceContext>>,
    pub(crate) resizable: Option<bool>,
    pub(crate) starting_position: Option<Position<NumericalContext>>,
    pub(crate) near_far: Option<NearFarDescriptor>,
}

#[derive(Copy, Clone)]
pub struct NearFarDescriptor {
    pub(crate) near: Layer,
    pub(crate) far: Layer,
}

impl NearFarDescriptor {
    pub fn new<L: Into<Layer>>(near: L, far: L) -> Self {
        Self {
            near: near.into(),
            far: far.into(),
        }
    }
}

impl Default for NearFarDescriptor {
    fn default() -> Self {
        Self {
            near: Layer::new(0f32),
            far: Layer::new(100f32),
        }
    }
}

impl Willow {
    pub(crate) fn connect(&mut self, event_loop: &ActiveEventLoop) {
        let requested_area = self.requested_area();
        let attributes = WindowAttributes::default()
            .with_title(self.title.clone().unwrap_or_default())
            .with_resizable(self.resizable.unwrap_or(true))
            .with_min_inner_size(self.min_size.unwrap_or(Area::device((320, 320))));
        #[cfg(all(
            not(target_family = "wasm"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        let attributes = attributes.with_inner_size(requested_area);
        let window = event_loop.create_window(attributes).unwrap();
        self.handle = WindowHandle(Some(Arc::new(window)));
    }
    pub(crate) fn actual_area(&self) -> Area<DeviceContext> {
        self.handle.0.clone().unwrap().inner_size().into()
    }
    pub(crate) fn window(&self) -> Arc<Window> {
        self.handle.0.clone().unwrap()
    }
    pub(crate) fn requested_area(&self) -> Area<DeviceContext> {
        self.requested_size
            .unwrap_or_default()
            .min(self.max_size.unwrap_or(Area::device((1920, 1080))))
            .max(self.min_size.unwrap_or(Area::device((1, 1))))
    }
}
