#[cfg(not(target_os = "android"))]
#[derive(Default, Copy, Clone)]
pub struct AndroidConnection();
#[cfg(target_os = "android")]
pub struct AndroidConnection();