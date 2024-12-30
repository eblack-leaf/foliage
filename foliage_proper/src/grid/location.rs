use crate::grid::{AlignedUnit, ScalarUnit};
use crate::{CoordinateUnit, Coordinates};

pub struct Location {
    pub sm: Option<LocationConfiguration>,
    pub md: Option<LocationConfiguration>,
    pub lg: Option<LocationConfiguration>,
    pub xl: Option<LocationConfiguration>,
}
pub struct LocationConfiguration {
    pub horizontal: LocationAxisDescriptor,
    pub vertical: LocationAxisDescriptor,
}
pub struct LocationAxisDescriptor {
    pub a: LocationAxisUnit,
    pub b: LocationAxisUnit,
    pub ty: LocationAxisType,
    pub padding: Coordinates,
    pub justify: Justify,
    pub max: Option<CoordinateUnit>,
}
pub enum Justify {
    Left,
    Right,
    Center,
}
pub enum LocationAxisType {
    Point,
    Span,
    To,
}
pub enum LocationAxisUnit {
    Scalar(ScalarUnit),
    Aligned(AlignedUnit),
    Stack,
    Auto,
}