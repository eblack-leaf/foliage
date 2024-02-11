#[cfg(feature = "derive")]
pub use foliage_macros::{assets, set_descriptor, SceneBinding};
pub use foliage_proper::*;
#[cfg(feature = "scenes")]
pub use foliage_scenes::*;
