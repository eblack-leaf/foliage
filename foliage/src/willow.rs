//! Willow -- the window.
//!
//! Everything between the platform's window and the engine: what the window is opened as, the
//! handle the surface is created against, and the translation from window events into the state
//! [`Fern`](crate::fern) reads at intake.
//!
//! It holds no engine state. A window is a platform object with a lifetime the engine does not
//! control -- it does not exist before the loop resumes, and on Android it stops existing again --
//! so the tree is never allowed to depend on one.

use std::sync::Arc;

use tracing::info;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::coordinate::Area;

/// The window, and what it is to be opened as.
///
/// The description is written at boot and the handle appears when the loop resumes. Both live here
/// so there is one place that knows whether there is a window yet.
#[derive(Default)]
pub(crate) struct Willow {
    handle: Option<Arc<Window>>,
    title: Option<String>,
    app_id: Option<String>,
    desktop_size: Option<Area>,
}

impl Willow {
    /// What the window is titled.
    pub(crate) fn title(&mut self, title: String) {
        self.title = Some(title);
    }

    /// How the desktop identifies the application, as against how it labels this window.
    ///
    /// Not a title. A title is prose shown to a person and free to change while the app runs; this
    /// is a stable identifier a shell matches against the entry that launched the program.
    pub(crate) fn app_id(&mut self, app_id: String) {
        self.app_id = Some(app_id);
    }

    /// The size to open at, in logical pixels. Read on desktop only.
    pub(crate) fn desktop_size(&mut self, size: Area) {
        self.desktop_size = Some(size);
    }

    /// Opens the window.
    ///
    /// Called from `resumed`, which is the only point a platform hands out something to open one
    /// against, and which runs again after an Android suspend.
    pub(crate) fn connect(&mut self, event_loop: &ActiveEventLoop) {
        let mut attributes = WindowAttributes::default()
            .with_title(self.title.clone().unwrap_or_default())
            .with_min_inner_size(LogicalSize::new(MINIMUM.width, MINIMUM.height));
        // The other platforms take their size from the canvas or the activity, and asking for one
        // there would be a request nothing reads.
        #[cfg(not(any(target_family = "wasm", target_os = "android", target_os = "ios")))]
        if let Some(size) = self.desktop_size {
            attributes = attributes.with_inner_size(LogicalSize::new(size.width, size.height));
        }
        // One call covers both display servers. The trait is spelled for Wayland, but the field it
        // writes is shared by the whole Linux backend, so the same string becomes the
        // `xdg_toplevel` app_id under Wayland and the `WM_CLASS` pair under X11. Both halves of the
        // pair are given the same value, which is what an application running one kind of window
        // wants.
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        if let Some(app_id) = &self.app_id {
            use winit::platform::wayland::WindowAttributesExtWayland;
            attributes = attributes.with_name(app_id.clone(), app_id.clone());
        }
        #[cfg(any(target_family = "wasm", target_os = "android", target_os = "ios"))]
        let _ = &mut attributes;
        let window = event_loop.create_window(attributes).expect("window");
        info!(
            width = window.inner_size().width,
            height = window.inner_size().height,
            scale = window.scale_factor(),
            "window"
        );
        #[cfg(target_family = "wasm")]
        Self::attach(&window);
        self.handle = Some(Arc::new(window));
    }

    /// Puts the canvas winit made into the page, and lets it take the space it is given.
    ///
    /// The browser is the one platform where a window has to be placed in something. Nothing sizes
    /// the canvas from here: it fills its parent, and the surface follows whatever the page
    /// resolves that to.
    #[cfg(target_family = "wasm")]
    fn attach(window: &Window) {
        use winit::platform::web::WindowExtWebSys;

        window.set_prevent_default(true);
        let canvas = window.canvas().expect("canvas");
        canvas
            .style()
            .set_css_text("width: 100%; height: 100%; display: block;");
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.body())
            .and_then(|body| body.append_child(&canvas).ok())
            .expect("canvas attached");
    }

    /// Whether a window has been opened.
    pub(crate) fn connected(&self) -> bool {
        self.handle.is_some()
    }

    /// The window, which exists once the loop has resumed.
    pub(crate) fn window(&self) -> Arc<Window> {
        self.handle.clone().expect("window")
    }

    /// The surface size in logical pixels, which is the only form the engine sees.
    ///
    /// Never zero on either axis: a minimised window reports a zero extent, and a surface cannot be
    /// configured at one.
    pub(crate) fn area(&self) -> Area {
        let window = self.window();
        let size = window.inner_size();
        let scale = window.scale_factor() as f32;
        Area::new(
            (size.width as f32 / scale).max(1.0),
            (size.height as f32 / scale).max(1.0),
        )
    }

    /// The display's logical-to-physical ratio, applied here and nowhere above.
    pub(crate) fn scale(&self) -> f32 {
        self.window().scale_factor() as f32
    }

    /// Asks the platform to paint.
    pub(crate) fn repaint(&self) {
        if let Some(window) = &self.handle {
            window.request_redraw();
        }
    }
}

/// The smallest window that can be opened, in logical pixels.
///
/// A floor rather than a preference: a window dragged to nothing leaves a surface with no pixels in
/// it, and every extent the layout resolves against would be zero.
const MINIMUM: Area = Area {
    width: 290.0,
    height: 290.0,
};
