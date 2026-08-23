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
    /// How the desktop identifies this *application*, as against how it labels this *window*.
    ///
    /// Not interchangeable with [`Willow::title`]. A title is prose shown to a person and free to
    /// change while the app runs; this is a stable identifier a shell matches against the
    /// `.desktop` file that launched it, to tie a running window back to its entry.
    ///
    /// Left unset no identity is published at all, and the consequence is not a missing name --
    /// it is that a shell has no way to recognise the window as the application the user
    /// launched, so it draws a second, generic icon beside the one they clicked.
    pub(crate) app_id: Option<String>,
    /// Only read by `requested_area`, which is desktop-only.
    #[allow(dead_code)]
    pub(crate) max_size: Option<Area<Physical>>,
    pub(crate) resizable: Option<bool>,
    pub(crate) starting_position: Option<Position<Numerical>>,
    pub(crate) near_far: Option<NearFarDescriptor>,
}

#[derive(Copy, Clone)]
/// The near and far depth planes the renderer maps [`ResolvedElevation`] into.
pub struct NearFarDescriptor {
    pub(crate) near: ResolvedElevation,
    pub(crate) far: ResolvedElevation,
}

impl NearFarDescriptor {
    /// A depth range from `near` to `far`.
    pub fn new(near: ResolvedElevation, far: ResolvedElevation) -> Self {
        Self { near, far }
    }
}

impl Default for NearFarDescriptor {
    fn default() -> Self {
        // purely internal headroom for `ash::assign_elevations`'s gapped/fractional-index
        // scheme (more room between adjacent entities before a gap needs renormalizing) --
        // not an author-facing budget. Nothing outside that scheme ever reads a specific
        // `ResolvedElevation` value directly, so this is free to widen with no migration cost.
        Self::new(ResolvedElevation(0f32), ResolvedElevation(300f32))
    }
}

impl Willow {
    pub(crate) fn connect(&mut self, event_loop: &ActiveEventLoop) {
        // only consumed by `with_inner_size` below, which is desktop-only -- the platforms
        // without it size the surface from the canvas/activity instead.
        #[cfg(all(
            not(target_family = "wasm"),
            not(target_os = "android"),
            not(target_os = "ios")
        ))]
        let requested_area = self.requested_area();
        let attributes = WindowAttributes::default()
            .with_title(self.title.clone().unwrap_or_default())
            .with_resizable(self.resizable.unwrap_or(true))
            .with_min_inner_size(self.min_size.unwrap_or(Area::physical((290, 290))));
        // Publish the application identity, where the platform has one.
        //
        // One call covers both display servers. The trait is spelled `...ExtWayland`, but the
        // field it writes -- `platform_specific.name` -- is shared by the whole Linux backend, so
        // the same value becomes the `xdg_toplevel` app_id under Wayland and the `WM_CLASS` pair
        // under X11. Calling the X11 trait as well would overwrite it with itself.
        //
        // Winit publishes nothing here unless asked, and an absent app_id is not a cosmetic gap:
        // it is what makes a desktop unable to match the window to the `.desktop` entry that
        // started it. `general` and `instance` are given the same string, which is what a
        // single-window application wants -- the distinction only matters to apps that run several
        // kinds of window under one identity.
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        let attributes = match &self.app_id {
            Some(app_id) => {
                use winit::platform::wayland::WindowAttributesExtWayland;
                attributes.with_name(app_id.clone(), app_id.clone())
            }
            None => attributes,
        };
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
    /// Desktop-only, like its one caller: the other platforms take their surface size from
    /// the canvas or activity rather than requesting one.
    #[cfg(all(
        not(target_family = "wasm"),
        not(target_os = "android"),
        not(target_os = "ios")
    ))]
    pub(crate) fn requested_area(&self) -> Area<Physical> {
        self.requested_size
            .unwrap_or_default()
            .min(self.max_size.unwrap_or(Area::physical((1920, 1080))))
            .max(self.min_size.unwrap_or(Area::physical((1, 1))))
    }
}
