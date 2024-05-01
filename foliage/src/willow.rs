use crate::Area;
use winit::window::Window;

#[derive(Default)]
pub(crate) struct Willow {
    pub(crate) handle: Option<Window>,
    pub(crate) min_size: Option<Area>,
    pub(crate) title: Option<String>,
    pub(crate) max_size: Option<Area>,
    pub(crate) resizable: Option<bool>,
}
impl Willow {
    pub(crate) fn window(&self) -> &Window {
        self.handle.as_ref().unwrap()
    }
}
