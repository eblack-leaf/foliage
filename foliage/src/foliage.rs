use tracing::info;
use web_time::Instant;

use crate::ash::Ash;
use crate::coordinate::Area;
use crate::ginkgo::Ginkgo;
use crate::grove::Grove;
use crate::root::{Registered, Root, Rooted};
use crate::willow::Willow;

/// The engine.
///
/// Built once at boot, told what to grow, and then run with
/// [`photosynthesize`](Foliage::photosynthesize). Everything an app does once it is running, it
/// does through the [`Grove`] it is handed.
pub struct Foliage {
    pub(crate) grove: Grove,
    pub(crate) root: Option<Box<dyn Rooted>>,
    pub(crate) willow: Willow,
    /// Absent until the platform has resumed and a device has been acquired.
    pub(crate) ginkgo: Option<Ginkgo>,
    pub(crate) ash: Option<Ash>,
    /// When the last frame was sampled, which is what the clock is advanced by.
    pub(crate) sampled: Option<Instant>,
    /// The device on its way over, on the one platform that cannot wait for one.
    #[cfg(target_family = "wasm")]
    pub(crate) acquiring: Option<std::sync::mpsc::Receiver<Ginkgo>>,
}

impl Foliage {
    pub fn new() -> Self {
        info!("boot");
        Self {
            grove: Grove::new(Area::default()),
            root: None,
            willow: Willow::default(),
            ginkgo: None,
            ash: None,
            sampled: None,
            #[cfg(target_family = "wasm")]
            acquiring: None,
        }
    }

    /// Registers the app. [`Root::take_root`] runs inside the first frame.
    pub fn root<R: Root>(&mut self) -> &mut Self {
        self.root = Some(Box::new(Registered::<R>::new()));
        self
    }

    /// What the window is titled.
    pub fn title(&mut self, title: impl Into<String>) -> &mut Self {
        self.willow.title(title.into());
        self
    }

    /// How the desktop identifies this application, as against how it labels this window.
    ///
    /// Not a title. A title is prose shown to a person and free to change while the app runs; this
    /// is a stable identifier a shell matches against the entry that launched the program, and it
    /// should stay fixed for the life of it. Conventionally the binary's name, or a reverse-DNS id.
    ///
    /// Unset, nothing is published, and the consequence is not a missing name: a desktop with no
    /// way to recognise the window as the application that was launched draws a second, generic
    /// icon beside the one that was clicked.
    ///
    /// Ignored where the platform has no such notion.
    pub fn app_id(&mut self, app_id: impl Into<String>) -> &mut Self {
        self.willow.app_id(app_id.into());
        self
    }

    /// The size to open the window at.
    ///
    /// Desktop only. Web and Android take their size from the surface they are given, and ignore
    /// this.
    pub fn desktop_size(&mut self, size: Area) -> &mut Self {
        self.willow.desktop_size(size);
        self
    }
}

impl Default for Foliage {
    fn default() -> Self {
        Self::new()
    }
}
