use crate::coordinate::{Context, Coordinates};

pub struct Area {
    context: Context,
    coordinates: Coordinates<2>,
}

impl Area {
    pub fn device<C: Into<Coordinates<2>>>(c: C) -> Self {
        Self {
            context: Context::Device,
            coordinates: c.into(),
        }
    }
}