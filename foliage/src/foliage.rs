use tracing::info;
use web_time::Instant;

use crate::ash::Ash;
use crate::coordinate::{Area, Position};
use crate::ginkgo::Ginkgo;
use crate::grove::Grove;
use crate::interaction::Claim;
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
    /// Where the pointer was last reported. The platform tells us where a press happened only by
    /// having told us where the pointer went, and a wheel notch names no position at all.
    pub(crate) cursor: Position,
    /// Whether the pointer is down. A move with nothing held means nothing to an engine with no
    /// hover, so it is not queued and does not owe a frame.
    pub(crate) held: bool,
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
            cursor: Position::default(),
            held: false,
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

    /// Sets one of the engine's tuning values.
    ///
    /// These are the numbers behind how the engine feels rather than what it does -- how far a
    /// gesture travels before it is a drag, and later how a coast decays and what a key is bound
    /// to. Each is a value for the whole app, because feel that varies from element to element is
    /// what makes an app feel unpredictable, and each is set here rather than per element for that
    /// reason.
    ///
    /// ```no_run
    /// # use foliage::{Claim, Foliage};
    /// # let mut foliage = Foliage::new();
    /// foliage.tune(Claim {
    ///     horizontal: 18.0,
    ///     vertical: 8.0,
    /// });
    /// ```
    ///
    /// Sealed: the set of tuning values is closed.
    #[allow(private_bounds)]
    pub fn tune(&mut self, tuning: impl Tuning) -> &mut Self {
        tuning.tune(&mut self.grove);
        self
    }
}

/// One of the engine's tuning values, and where it lands.
pub(crate) trait Tuning {
    fn tune(self, grove: &mut Grove);
}

impl Tuning for Claim {
    fn tune(self, grove: &mut Grove) {
        grove.claim = self;
    }
}

impl Default for Foliage {
    fn default() -> Self {
        Self::new()
    }
}
