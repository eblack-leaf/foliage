use crate::coordinate::{Context, Coordinates};
use crate::CoordinateUnit;
use winit::dpi::{LogicalSize, PhysicalSize, Size};
#[derive(Copy, Clone)]
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
    pub fn width(&self) -> CoordinateUnit {
        self.coordinates.0[0]
    }
    pub fn height(&self) -> CoordinateUnit {
        self.coordinates.0[1]
    }
}
impl From<Area> for Size {
    fn from(value: Area) -> Self {
        match value.context {
            Context::Device => Size::Physical(PhysicalSize::new(
                value.width() as u32,
                value.height() as u32,
            )),
            Context::Logical => Size::Logical(LogicalSize::new(
                value.width() as f64,
                value.height() as f64,
            )),
            Context::Numerical => Size::Logical(LogicalSize::new(
                value.width() as f64,
                value.height() as f64,
            )),
        }
    }
}
