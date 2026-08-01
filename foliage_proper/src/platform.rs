//! Platform handles that must be threaded in from an OS entry point.

/// The Android activity handle, carried into [`Foliage`](crate::Foliage) at startup.
/// Off Android it is an empty placeholder, so construction reads the same everywhere.
#[cfg(not(target_os = "android"))]
#[derive(Default, Copy, Clone)]
pub struct AndroidConnection();

/// The Android activity handle. Only `android_main` can produce one, which is why it is
/// passed in rather than obtained by the engine.
#[cfg(target_os = "android")]
#[derive(Clone)]
pub struct AndroidConnection(pub AndroidApp);

/// Re-exported so an `android_main` need not depend on winit directly.
#[cfg(target_os = "android")]
pub type AndroidApp = winit::platform::android::activity::AndroidApp;
