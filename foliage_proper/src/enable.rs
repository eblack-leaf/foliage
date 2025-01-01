use crate::Event;

#[derive(Event, Copy, Clone)]
pub struct Enable {}
impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
}
#[derive(Event, Copy, Clone)]
pub(crate) struct AutoEnable {}
impl AutoEnable {
    pub fn new() -> AutoEnable {
        AutoEnable {}
    }
}
