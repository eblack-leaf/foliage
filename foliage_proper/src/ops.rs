use bevy_ecs::prelude::Event;

#[derive(Event, Copy, Clone)]
pub struct Write<W> {
    _phantom: std::marker::PhantomData<W>,
}
impl<W> Default for Write<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W> Write<W> {
    pub fn new() -> Write<W> {
        Write {
            _phantom: std::marker::PhantomData,
        }
    }
}
#[derive(Event, Copy, Clone)]
pub struct Update<U> {
    _phantom: std::marker::PhantomData<U>,
}
impl<U> Default for Update<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U> Update<U> {
    pub fn new() -> Update<U> {
        Update {
            _phantom: std::marker::PhantomData,
        }
    }
}
