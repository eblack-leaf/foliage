use crate::coordinate::placement::Placement;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::layout::Layout;
use crate::leaf::{IdTable, LeafHandle};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Query};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::Res;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Sub};

impl Sub<GridUnit> for GridToken {
    type Output = GridToken;

    fn sub(self, rhs: GridUnit) -> Self::Output {
        let token: GridToken = rhs.into();
        self - token
    }
}
impl Add<GridToken> for GridUnit {
    type Output = GridToken;
    fn add(self, rhs: GridToken) -> Self::Output {
        let token: GridToken = self.into();
        token + rhs
    }
}
impl Sub<GridToken> for GridUnit {
    type Output = GridToken;
    fn sub(self, rhs: GridToken) -> Self::Output {
        let token: GridToken = self.into();
        token - rhs
    }
}
impl Sub<GridToken> for GridToken {
    type Output = GridToken;
    fn sub(mut self, mut rhs: GridToken) -> Self::Output {
        if let Some(first) = rhs.partitions.get_mut(0) {
            first.op = GridTokenOp::Sub;
        }
        self.partitions.extend(rhs.partitions);
        self
    }
}
impl From<GridUnit> for GridToken {
    fn from(value: GridUnit) -> Self {
        GridToken::new(GridContext::This, GridTokenValue::Unit(value))
    }
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
    fn add(mut self, rhs: GridToken) -> Self::Output {
        self.partitions.extend(rhs.partitions);
        self
    }
}
pub fn screen() -> GridContext {
    GridContext::Screen
}
pub fn context<LH: Into<LeafHandle>>(lh: LH) -> GridContext {
    GridContext::Named(lh.into())
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
        context(self).y()
    }
    fn height(self) -> GridToken {
        context(self).height()
    }
    fn width(self) -> GridToken {
        context(self).width()
    }
    fn right(self) -> GridToken {
        context(self).right()
    }
}
#[cfg(test)]
#[test]
fn behavior() {
    let location = GridLocation::new()
        .bottom(10.px() + screen().x() - 16.px())
        .top("header".y() + 10.percent().of("header"))
        .width(50.percent().of(screen()))
        .left("button".right() + 10.px())
        .right_at(Layout::LANDSCAPE_MOBILE, screen().x() + "footer".width());
}
pub trait GridUnitDesc {
    fn px(self) -> GridUnit;
    fn percent(self) -> GridUnit;
    fn column(self) -> GridUnit;
    fn row(self) -> GridUnit;
}
impl GridUnitDesc for i32 {
    fn px(self) -> GridUnit {
        GridUnit::Px(self as CoordinateUnit)
    }
    fn percent(self) -> GridUnit {
        GridUnit::Percent(self as f32 / 100.0)
    }
    fn column(self) -> GridUnit {
        GridUnit::Column(self)
    }
    fn row(self) -> GridUnit {
        GridUnit::Row(self)
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
        GridToken::new(context.into(), GridTokenValue::Unit(self))
    }
}
pub(crate) fn animate_grid_location() {}

pub(crate) fn resolve_grid_locations(
    check: Query<Entity, Or<(Changed<GridLocation>, Changed<Grid>)>>,
    read: Query<(&LeafHandle, &GridLocation, Option<&Grid>)>,
    id_table: Res<IdTable>,
) {
    if check.is_empty() {
        return;
    }
    let mut referential_context = vec![];
    for (handle, location, grid) in read.iter() {
        let refs = location.references();
        let is_screen = if refs.is_screen() { // is-screen => 4.col().of(screen()) ... only reference to screen
             // is root to start with
        };
        referential_context.push((handle.clone(), refs, grid.clone(), is_screen));
    }
    referential_context.sort_by(|a, b| {
        // roots (is_screen) first => by referential-dependency
    });
    let mut resolved = HashMap::new();
    // placements.insert(screen, viewport-handle:section, Grid-None);
    for (handle, location, grid) in referential_context.iter() {
        let (placement, points) = Grid::resolve(location, &resolved);
    }
}
pub(crate) fn placement_recursion() {}
#[derive(Clone, Component)]
pub(crate) struct GridReferentialContext {
    references: HashSet<LeafHandle>,
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GridTokenDesc {
    X,
    Y,
    CenterX,
    CenterY,
    Top,
    Right,
    Bottom,
    Left,
    HorizontalBegin,
    HorizontalEnd,
    VerticalBegin,
    VerticalEnd,
    Width,
    Height,
    LineBegin,
    LineEnd,
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GridTokenDescException {
    layout: Layout,
    desc: GridTokenDesc,
}
impl GridTokenDescException {
    pub fn new(layout: Layout, desc: GridTokenDesc) -> Self {
        Self { layout, desc }
    }
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
        self.token_descriptions
            .insert(GridTokenDesc::Bottom, gt.into());
        self
    }
    pub fn top<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        self.token_descriptions
            .insert(GridTokenDesc::Top, gt.into());
        self
    }
    pub fn width<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        self.token_descriptions
            .insert(GridTokenDesc::Width, gt.into());
        self
    }
    pub fn left<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        self.token_descriptions
            .insert(GridTokenDesc::Left, gt.into());
        self
    }
    pub fn right_at<GT: Into<GridToken>>(mut self, layout: Layout, gt: GT) -> Self {
        self.exceptions.insert(
            GridTokenDescException::new(layout, GridTokenDesc::Right),
            gt.into(),
        );
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
    gap: Coordinates,
}
impl GridTemplate {
    pub fn new(columns: u32, rows: u32) -> GridTemplate {
        Self {
            columns,
            rows,
            gap: Coordinates::new(8.0, 8.0),
        }
    }
    pub fn columns(&self) -> f32 {
        self.columns as f32
    }
    pub fn rows(&self) -> f32 {
        self.rows as f32
    }
    pub fn gap<C: Into<Coordinates>>(mut self, g: C) -> Self {
        self.gap = g.into();
        self
    }
}
impl Default for GridTemplate {
    fn default() -> Self {
        Self::new(1, 1)
    }
}
#[derive(Component, Default)]
pub struct Grid {
    template: Option<GridTemplate>,
    placement: Placement<LogicalContext>,
}
impl Grid {
    pub fn new() -> Grid {
        Self {
            template: None,
            placement: Default::default(),
        }
    }
    pub fn template(c: u32, r: u32) -> Grid {
        Self {
            template: Some(GridTemplate::new(c, r)),
            placement: Default::default(),
        }
    }
    pub fn with_gap<C: Into<Coordinates>>(mut self, g: C) -> Self {
        if let Some(t) = self.template.as_mut() {
            t.gap = g.into();
        }
        self
    }
    pub fn column_size(&self) -> CoordinateUnit {
        if let Some(template) = &self.template {
            self.placement.section.width() / template.columns()
                - template.gap.horizontal() * (template.columns() + 1.0)
        } else {
            0.0
        }
    }
    pub fn size_to(&mut self, placement: Placement<LogicalContext>) {
        self.placement = placement;
    }
    pub fn sized(mut self, placement: Placement<LogicalContext>) -> Self {
        self.placement = placement;
        self
    }
    pub fn row_size(&self) -> CoordinateUnit {
        if let Some(template) = &self.template {
            self.placement.section.height() / template.rows()
                - template.gap.vertical() * (template.rows() + 1.0)
        } else {
            0.0
        }
    }
}
#[derive(Clone)]
pub enum GridContext {
    Screen,
    Named(LeafHandle),
    Path(LeafHandle),
    This,
}
impl GridContext {
    pub fn x(self) -> GridToken {
        GridToken::new(self, GridTokenValue::Desc(GridTokenDesc::X))
    }
    pub fn y(self) -> GridToken {
        GridToken::new(self, GridTokenValue::Desc(GridTokenDesc::Y))
    }
    pub fn height(self) -> GridToken {
        GridToken::new(self, GridTokenValue::Desc(GridTokenDesc::Height))
    }
    pub fn width(self) -> GridToken {
        GridToken::new(self, GridTokenValue::Desc(GridTokenDesc::Width))
    }
    pub fn right(self) -> GridToken {
        GridToken::new(self, GridTokenValue::Desc(GridTokenDesc::Right))
    }
}
impl<LH: Into<LeafHandle>> From<LH> for GridContext {
    fn from(lh: LH) -> GridContext {
        context(lh)
    }
}
#[derive(Clone)]
pub struct GridToken {
    partitions: Vec<GridTokenPartition>,
}
impl GridToken {
    pub fn new(context: GridContext, value: GridTokenValue) -> GridToken {
        Self {
            partitions: vec![GridTokenPartition::new(GridTokenOp::Add, context, value)],
        }
    }
}
#[derive(Clone)]
pub struct GridTokenPartition {
    pub op: GridTokenOp,
    pub context: GridContext,
    pub value: GridTokenValue,
}
impl GridTokenPartition {
    pub fn new(op: GridTokenOp, context: GridContext, value: GridTokenValue) -> Self {
        Self { op, context, value }
    }
}
#[derive(Clone)]
pub enum GridTokenValue {
    Unit(GridUnit),
    Desc(GridTokenDesc),
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
        let token: GridToken = self.into();
        let rhs: GridToken = rhs.into();
        token - rhs
    }
}
impl Add for GridUnit {
    type Output = GridToken;

    fn add(self, rhs: Self) -> Self::Output {
        let token: GridToken = self.into();
        let rhs: GridToken = rhs.into();
        token + rhs
    }
}
