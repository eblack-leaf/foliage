use crate::Event;

#[derive(Event, Copy, Clone)]
pub struct Disable {}

impl Disable {
    pub fn new() -> Disable {
        Disable {}
    }
}
