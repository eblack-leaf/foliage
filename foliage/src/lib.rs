mod coordinate;
mod interop;

pub use coordinate::{Area, CoordinateUnit, Coordinates};
pub use interop::AndroidConnection;
pub struct Foliage {}
impl Foliage {
    pub fn new() -> Self {
        Self {}
    }
    pub fn set_window_size<A: Into<Area>>(&mut self, a: A) {}
    pub fn set_worker_path<S: AsRef<str>>(&mut self, s: S) {}
    pub fn set_window_title<S: AsRef<str>>(&mut self, s: S) {}
    pub fn set_android_connection(&mut self, ac: AndroidConnection) {}
}