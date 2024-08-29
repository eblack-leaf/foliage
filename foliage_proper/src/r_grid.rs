use crate::coordinate::CoordinateUnit;
use crate::grid::Layout;
use crate::leaf::LeafHandle;
use std::ops::{Add, Sub};

impl Sub<GridUnit> for GridToken {
    type Output = GridToken;

    fn sub(self, rhs: GridUnit) -> Self::Output {
        let token: GridToken = rhs.into();
        self - token
    }
}
impl Sub<GridToken> for GridToken {
    type Output = GridToken;
    fn sub(self, rhs: GridToken) -> Self::Output {
        todo!()
    }
}
impl From<GridUnit> for GridToken {
    fn from(value: GridUnit) -> Self {
        todo!()
    }
}
pub fn screen() -> GridContext {
    GridContext::Screen
}
pub fn context<LH: Into<LeafHandle>>(lh: LH) -> GridContext {
    GridContext::Named(lh.into())
}
pub fn path<LH: Into<LeafHandle>>(path: LH) -> GridContext {
    GridContext::Path(path.into())
}
impl Add<GridUnit> for GridToken {
    type Output = GridToken;

    fn add(self, rhs: GridUnit) -> Self::Output {
        let token: GridToken = rhs.into();
        self + token
    }
}
impl Add<GridToken> for GridToken {
    type Output = GridToken;
    fn add(self, rhs: GridToken) -> Self::Output {
        todo!()
    }
}
pub trait GridContextDesc {
    fn x(&self) -> GridToken;
    fn y(&self) -> GridToken;
    fn height(&self) -> GridToken;
    fn width(&self) -> GridToken;
    fn right(&self) -> GridToken;
}
impl<LH: Into<LeafHandle>> GridContextDesc for LH {
    fn x(&self) -> GridToken {
        context(self.into()).x()
    }
    fn y(&self) -> GridToken {
        todo!()
    }
    fn height(&self) -> GridToken {
        todo!()
    }
    fn width(&self) -> GridToken {
        todo!()
    }
    fn right(&self) -> GridToken {
        todo!()
    }
}
#[cfg(test)]
#[test]
fn behavior() {
    // depends on "header" + "button" respectively
    let location = GridLocation::new()
        .bottom(screen().x() - 16.px())
        .top("header".y() + 10.percent().of("header"))
        .width(50.percent().of(screen()))
        .left("button".right() + 10.px())
        .right_at(Layout::LANDSCAPE_MOBILE, screen().x() + "footer".width());
}
pub trait GridUnitDesc {
    fn px(self) -> GridUnit;
    fn percent(self) -> GridUnit;
}
impl GridUnitDesc for i32 {
    fn px(self) -> GridUnit {
        GridUnit::Px(self as CoordinateUnit)
    }
    fn percent(self) -> GridUnit {
        GridUnit::Percent(self as f32 / 100.0)
    }
}
pub enum GridUnit {
    Px(CoordinateUnit),
    Percent(f32),
    Column(i32),
    Row(i32),
}
impl GridUnit {
    pub fn of<GC: Into<GridContext>>(self, context: GC) -> GridToken {
        todo!()
    }
}
pub struct GridLocation {
    // configuration of tokens
    // bot | top | left | right
    // ...
}
impl GridLocation {
    pub fn new() -> GridLocation {
        Self {}
    }
    pub fn bottom<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        // save token to bottom-slot
        self
    }
    pub fn top<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        self
    }
    pub fn width<GT: Into<GridToken>>(self, gt: GT) -> Self {
        self
    }
    pub fn left<GT: Into<GridToken>>(self, gt: GT) -> Self {
        self
    }
    pub fn right_at<GT: Into<GridToken>>(self, layout: Layout, gt: GT) -> Self {
        // exception @ layout overrides normal context
        self
    }
    // top-left
    // center
    // area
    // horizontal
    // vertical
}
pub struct Grid {
    // actual grid mechanisms
}
impl Grid {}
pub enum GridContext {
    Screen,
    Named(LeafHandle),
    Path(LeafHandle),
    None,
}
impl GridContext {
    pub fn x(&self) -> GridToken {
        todo!()
    }
    pub fn y(&self) -> GridToken {
        todo!()
    }
    pub fn height(&self) -> GridToken {
        todo!()
    }
    pub fn width(&self) -> GridToken {
        todo!()
    }
    pub fn right(&self) -> GridToken {
        todo!()
    }
}
impl<LH: Into<LeafHandle>> From<LH> for GridContext {
    fn from(lh: LH) -> GridContext {
        context(lh)
    }
}
pub struct GridToken {
    // desc of location on grid
    // context
}
pub struct GridTokenPartition {
    pub op: GridTokenOp,
    pub context: GridContext,
    pub value: GridTokenValue,
}
pub enum GridTokenValue {
    Unit(GridUnit),
    Desc(RelativeDesc),
}
pub struct RelativeDesc {
    // right + x token part
    // path-point?
}
pub enum GridTokenOp {
    Add,
    Sub,
    Mul,
    Div,
}
