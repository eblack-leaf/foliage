use crate::Event;

#[derive(Event, Copy, Clone)]
pub struct Enable {}

impl Enable {
    pub fn new() -> Enable {
        Enable {}
    }
}
