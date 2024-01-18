use bevy_ecs::prelude::Resource;
use std::sync::Arc;

#[cfg(all(not(target_os = "android"), not(target_family = "wasm")))]
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, DeviceContext};

#[derive(Default, Clone)]
pub struct WindowDescriptor {
    desktop_dimensions: Option<Area<DeviceContext>>,
    title: Option<&'static str>,
}

impl WindowDescriptor {
    pub fn new() -> Self {
        Self {
            desktop_dimensions: None,
            title: None,
        }
    }
    pub fn with_desktop_dimensions<A: Into<Area<DeviceContext>>>(mut self, dims: A) -> Self {
        self.desktop_dimensions.replace(dims.into());
        self
    }
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title.replace(title);
        self
    }
}

#[derive(Clone)]
pub(crate) struct WindowHandle(pub(crate) Option<Arc<Window>>);

impl WindowHandle {
    #[allow(unused)]
    pub(crate) fn none() -> Self {
        Self(None)
    }
    pub(crate) fn value(&self) -> &Window {
        self.0.as_ref().expect("window handle value")
    }
    pub(crate) fn some(
        event_loop: &EventLoopWindowTarget,
        window_descriptor: &WindowDescriptor,
    ) -> Self {
        #[allow(unused_mut)]
        let mut builder = WindowBuilder::new()
            .with_resizable(true)
            .with_title(window_descriptor.title.unwrap_or_default());
        #[cfg(all(
            not(target_family = "wasm"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        if let Some(dims) = window_descriptor.desktop_dimensions {
            builder =
                builder.with_inner_size(PhysicalSize::new(dims.width as i32, dims.height as i32));
        }
        let window = builder.build(event_loop).expect("window creation");
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas().expect("Couldn't get canvas");
            canvas.style().set_css_text("height: 100%; width: 100%;");
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&canvas).ok())
                .expect("couldn't append canvas to document body");
        }
        Self(Some(Arc::new(window)))
    }
    pub(crate) fn area(&self) -> Area<DeviceContext> {
        let width = self.0.as_ref().unwrap().inner_size().width as CoordinateUnit;
        let height = self.0.as_ref().unwrap().inner_size().height as CoordinateUnit;
        Area::new(width, height)
    }
    pub(crate) fn scale_factor(&self) -> CoordinateUnit {
        self.value().scale_factor() as CoordinateUnit
    }
}
#[derive(Resource)]
pub struct ScaleFactor(CoordinateUnit);
impl ScaleFactor {
    pub(crate) fn new(factor: CoordinateUnit) -> Self {
        Self(factor.round())
    }
    pub fn factor(&self) -> CoordinateUnit {
        self.0
    }
}
