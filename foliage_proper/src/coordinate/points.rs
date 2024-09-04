use crate::coordinate::position::Position;
use crate::coordinate::CoordinateContext;

pub struct Points<Context: CoordinateContext> {
    data: Vec<Position<Context>>,
}
impl<Context: CoordinateContext> Points<Context> {
    pub fn new(data: Vec<Position<Context>>) -> Self {
        Self { data }
    }
}
