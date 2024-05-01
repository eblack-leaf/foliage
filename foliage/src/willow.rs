use crate::coordinate::DeviceContext;
use crate::Area;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
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
