use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::coordinate::area::Area;
use crate::coordinate::elevation::ResolvedElevation;
use crate::coordinate::position::Position;
use crate::coordinate::{Numerical, Physical};

#[derive(Clone, Default)]
pub(crate) struct WindowHandle(pub(crate) Option<Arc<Window>>);

#[derive(Default, Clone)]
pub(crate) struct Willow {
    pub(crate) handle: WindowHandle,
    pub(crate) min_size: Option<Area<Physical>>,
    pub(crate) requested_size: Option<Area<Physical>>,
    pub(crate) title: Option<String>,
    pub(crate) max_size: Option<Area<Physical>>,
    pub(crate) resizable: Option<bool>,
    pub(crate) starting_position: Option<Position<Numerical>>,
    pub(crate) near_far: Option<NearFarDescriptor>,
}

#[derive(Copy, Clone)]
pub struct NearFarDescriptor {
    pub(crate) near: ResolvedElevation,
    pub(crate) far: ResolvedElevation,
}

impl NearFarDescriptor {
    pub fn new<L: Into<ResolvedElevation>>(near: L, far: L) -> Self {
        Self {
            near: near.into(),
            far: far.into(),
        }
    }
}

impl Default for NearFarDescriptor {
    fn default() -> Self {
        Self {
            near: ResolvedElevation::new(0f32),
            far: ResolvedElevation::new(100f32),
        }
    }
}

impl Willow {
    pub(crate) fn connect(&mut self, event_loop: &ActiveEventLoop) {
        let requested_area = self.requested_area();
        let attributes = WindowAttributes::default()
            .with_title(self.title.clone().unwrap_or_default())
            .with_resizable(self.resizable.unwrap_or(true))
            .with_min_inner_size(self.min_size.unwrap_or(Area::physical((290, 290))));
        #[cfg(all(
            not(target_family = "wasm"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        let attributes = attributes.with_inner_size(requested_area);
        let window = event_loop.create_window(attributes).unwrap();
        #[cfg(target_family = "wasm")]
        {
            use winit::platform::web::WindowExtWebSys;
            window.set_prevent_default(true);
            let canvas = window.canvas().expect("window-canvas");
            canvas.style().set_css_text("height: 100%; width: 100%;");
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&canvas).ok())
                .expect("append-canvas");
        }
        self.handle = WindowHandle(Some(Arc::new(window)));
    }
    pub(crate) fn actual_area(&self) -> Area<Physical> {
        self.handle.0.clone().unwrap().inner_size().into()
    }
    pub(crate) fn window(&self) -> Arc<Window> {
        self.handle.0.clone().unwrap()
    }
    pub(crate) fn requested_area(&self) -> Area<Physical> {
        self.requested_size
            .unwrap_or_default()
            .min(self.max_size.unwrap_or(Area::physical((1920, 1080))))
            .max(self.min_size.unwrap_or(Area::physical((1, 1))))
    }
}
