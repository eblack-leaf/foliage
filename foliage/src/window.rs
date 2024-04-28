use crate::Area;
use winit::window::Window;

#[derive(Default)]
pub(crate) struct WindowHandle {
    pub(crate) handle: Option<Window>,
    pub(crate) min_size: Option<Area>,
    pub(crate) title: Option<String>,
    pub(crate) max_size: Option<Area>,
    pub(crate) resizable: Option<bool>,
}
