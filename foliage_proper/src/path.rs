use crate::coordinate::Coordinates;

pub struct Path {
    pub segments: Vec<PathSegment>,
}
pub struct PathSegment {
    start: PathPoint,
    end: PathPoint,
}
pub struct PathPoint {
    coords: Coordinates,
}
