use tracing::info;

use crate::coordinate::Area;
use crate::grove::Grove;
use crate::root::{Registered, Root, Rooted};

/// The engine.
///
/// Built once at boot, told what to grow, and then driven by the platform. Everything an app does
/// once it is running, it does through the [`Grove`] it is handed.
pub struct Foliage {
    grove: Grove,
    root: Option<Box<dyn Rooted>>,
    desktop_size: Option<Area>,
}

impl Foliage {
    pub fn new() -> Self {
        info!("boot");
        Self {
            grove: Grove::new(Area::default()),
            root: None,
            desktop_size: None,
        }
    }

    /// Registers the app. [`Root::take_root`] runs inside the first frame.
    pub fn root<R: Root>(&mut self) -> &mut Self {
        self.root = Some(Box::new(Registered::<R>::new()));
        self
    }

    /// The size to open the window at.
    ///
    /// Desktop only. Web and Android take their size from the surface they are given, and ignore
    /// this.
    pub fn desktop_size(&mut self, size: Area) -> &mut Self {
        self.desktop_size = Some(size);
        self
    }
}

impl Default for Foliage {
    fn default() -> Self {
        Self::new()
    }
}
