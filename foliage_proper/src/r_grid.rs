use std::collections::HashMap;
use crate::coordinate::{CoordinateUnit, LogicalContext};
use crate::layout::Layout;
use crate::leaf::LeafHandle;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::Component;
use std::ops::{Add, Sub};
use crate::coordinate::placement::Placement;

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
    fn x(self) -> GridToken;
    fn y(self) -> GridToken;
    fn height(self) -> GridToken;
    fn width(self) -> GridToken;
    fn right(self) -> GridToken;
}
impl<LH: Into<LeafHandle>> GridContextDesc for LH {
    fn x(self) -> GridToken {
        context(self).x()
    }
    fn y(self) -> GridToken {
        todo!()
    }
    fn height(self) -> GridToken {
        todo!()
    }
    fn width(self) -> GridToken {
        todo!()
    }
    fn right(self) -> GridToken {
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
pub(crate) fn place_on_grid() {}
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
#[derive(Clone)]
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
    pub fn between<PH: Into<PointHandle>, PHH: Into<PointHandle>>(
        self,
        ph: PH,
        phh: PHH,
    ) -> GridPoint {
        todo!()
    }
}
pub(crate) fn animate_grid_location() {}
pub(crate) fn recursive_placement() {}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GridTokenDesc {
    X,
    Y,
    Center,
    Top,
    Right,
    Bottom,
    Left,
    Horizontal,
    Vertical,
    Width,
    Height
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GridTokenDescException {
    layout: Layout,
    desc: GridTokenDesc,
}
#[derive(Component, Clone)]
pub struct GridLocation {
    token_descriptions: HashMap<GridTokenDesc, GridToken>,
    exceptions: HashMap<GridTokenDescException, GridToken>,
}
impl GridLocation {
    pub fn new() -> GridLocation {
        Self {
            token_descriptions: HashMap::new(),
            exceptions: Default::default(),
        }
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
#[derive(Clone, Copy)]
pub struct GridTemplate {
    columns: u32,
    rows: u32,
}
impl GridTemplate {
    pub fn new(columns: u32, rows: u32) -> GridTemplate {
        Self { columns, rows }
    }
}
#[derive(Component)]
pub struct Grid {
    // actual grid mechanisms
    template: Option<GridTemplate>,
    placement: Placement<LogicalContext>,
}
impl Grid {
    pub fn new() -> Grid {
        Self { template: None }
    }
    pub fn template(c: u32, r: u32) -> Grid {
        Self { template: Some(GridTemplate::new(c, r)) }
    }
}
#[derive(Clone)]
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
#[derive(Clone)]
pub struct GridToken {
    // desc of location on grid
    // context
    partitions: Vec<GridTokenPartition>
}
#[derive(Clone)]
pub struct GridTokenPartition {
    pub op: GridTokenOp,
    pub context: GridContext,
    pub value: GridTokenValue,
}
#[derive(Clone)]
pub enum GridTokenValue {
    Unit(GridUnit),
    Desc(RelativeDesc),
}
#[derive(Clone)]
pub struct RelativeDesc {
    // right + x token part
    // path-point?
}
#[derive(Clone)]
pub enum GridTokenOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl Sub for GridUnit {
    type Output = GridToken;

    fn sub(self, rhs: Self) -> Self::Output {
        todo!()
    }
}

impl Add for GridUnit {
    type Output = GridToken;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
impl From<GridToken> for GridPointToken {
    fn from(value: GridToken) -> Self {
        todo!()
    }
}
#[cfg(test)]
#[test]
fn path_behavior() {
    // try to make useful for data-points => viewbox + normalize + give data array?
    // + line-graph => connect-points w/ lines + line-joins (circle) at intersections
    let path = GridPath::new()
        .add("a", (50.percent(), 100.percent() - 16.px()))
        .add("b", (50.percent(), 0.percent() + 16.px()))
        .add("c", 50.percent().between("a", "b"))
        .add("d", 35.degrees().from("b").at_x(100.percent() - 16.px()))
        .add("e", 35.degrees().inverse().from("b").distance(16.px()));
}
pub trait GridPathDesc {
    fn degrees(self) -> GridPathAngle;
}
impl GridPathDesc for i32 {
    fn degrees(self) -> GridPathAngle {
        todo!()
    }
}
pub struct GridPathAngle {}
impl GridPathAngle {
    pub fn from<PH: Into<PointHandle>>(self, ph: PH) -> GridPathAngleFrom {
        todo!()
    }
    pub fn inverse(self) -> Self {
        todo!()
    }
}
pub struct GridPathAngleFrom {}
impl GridPathAngleFrom {
    pub fn at_x<GT: Into<GridToken>>(self, gt: GT) -> GridPoint {
        todo!()
    }
    pub fn distance(self, gu: GridUnit) -> GridPoint {
        todo!()
    }
}
pub struct PointHandle(pub(crate) String);
impl<S: AsRef<str>> From<S> for PointHandle {
    fn from(value: S) -> Self {
        todo!()
    }
}
pub struct GridPath {}
impl GridPath {
    pub fn new() -> GridPath {
        Self {}
    }
    pub fn add<PH: Into<PointHandle>, GP: Into<GridPoint>>(mut self, ph: PH, gp: GP) -> Self {
        self
    }
}
pub struct GridPoint {
    pub x: GridPointToken,
    pub y: GridPointToken,
}
impl GridPoint {
    pub fn new<GT: Into<GridPointToken>, GTT: Into<GridPointToken>>(gt: GT, gtt: GTT) -> Self {
        todo!()
    }
}
impl<GT: Into<GridPointToken>, GTT: Into<GridPointToken>> From<(GT, GTT)> for GridPoint {
    fn from(value: (GT, GTT)) -> Self {
        todo!()
    }
}
impl From<GridUnit> for GridPointToken {
    fn from(value: GridUnit) -> Self {
        todo!()
    }
}
pub struct GridPointToken {}
