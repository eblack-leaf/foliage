use crate::coordinate::area::Area;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::{Layout, LayoutGrid};
use crate::leaf::{IdTable, LeafHandle};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Query};
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{Commands, Res};
use std::cmp::Ordering;
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
    read: Query<(&LeafHandle, &GridLocation, &Grid)>,
    mut update: Query<(&mut Position<LogicalContext>, &mut Area<LogicalContext>)>,
    id_table: Res<IdTable>,
    viewport_handle: Res<ViewportHandle>,
    layout_grid: Res<LayoutGrid>,
    layout: Res<Layout>,
    mut cmd: Commands,
) {
    if check.is_empty() {
        return;
    }
    let mut referential_context = vec![];
    for (handle, location, _) in read.iter() {
        referential_context.push((handle.clone(), location.references()));
    }
    referential_context.sort_by(|a, b| {
        let b_depends_a = b.1.references.contains(&GridContext::Named(a.0.clone()));
        let a_depends_b = a.1.references.contains(&GridContext::Named(b.0.clone()));
        if a_depends_b && b_depends_a {
            panic!("circular grid reference")
        }
        if a_depends_b {
            Ordering::Greater
        } else if b_depends_a {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    let mut resolved = HashMap::new();
    resolved.insert(
        GridContext::Screen,
        (viewport_handle.section(), None, layout_grid.grid),
    );
    for (handle, _) in referential_context {
        let (_, location, grid) = read
            .get(id_table.lookup_leaf(handle.clone()).unwrap())
            .unwrap();
        let (section, points) = location.resolve(&resolved, *layout);
        resolved.insert(GridContext::Named(handle), (section, points, *grid));
    }
    for (handle, (section, new_points, _)) in resolved {
        match handle {
            GridContext::Screen => {}
            GridContext::Named(name) => {
                let e = id_table.lookup_leaf(name).unwrap();
                if let Ok((mut pos, mut area)) = update.get_mut(e) {
                    *pos = section.position;
                    *area = section.area;
                    if let Some(p) = new_points {
                        cmd.entity(e).insert(p);
                    }
                }
            }
            GridContext::This => {}
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct GridReferentialContext {
    references: HashSet<GridContext>,
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
    animation_hook: Option<Section<LogicalContext>>,
    ref_context: GridReferentialContext,
}
impl GridLocation {
    fn resolve(
        &self,
        resolved: &HashMap<
            GridContext,
            (
                Section<LogicalContext>,
                Option<Points<LogicalContext>>,
                Grid,
            ),
        >,
        layout: Layout,
    ) -> (Section<LogicalContext>, Option<Points<LogicalContext>>) {
        todo!()
    }
    pub fn new() -> GridLocation {
        Self {
            token_descriptions: HashMap::new(),
            exceptions: Default::default(),
            animation_hook: None,
            ref_context: GridReferentialContext::default(),
        }
    }
    pub fn references(&self) -> GridReferentialContext {
        self.ref_context.clone()
    }
    pub fn bottom<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        let token = gt.into();
        self.ref_context.references.extend(token.references());
        self.token_descriptions.insert(GridTokenDesc::Bottom, token);
        self
    }
    pub fn top<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        let token = gt.into();
        self.ref_context.references.extend(token.references());
        self.token_descriptions.insert(GridTokenDesc::Top, token);
        self
    }
    pub fn width<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        let token = gt.into();
        self.ref_context.references.extend(token.references());
        self.token_descriptions.insert(GridTokenDesc::Width, token);
        self
    }
    pub fn left<GT: Into<GridToken>>(mut self, gt: GT) -> Self {
        let token = gt.into();
        self.ref_context.references.extend(token.references());
        self.token_descriptions.insert(GridTokenDesc::Left, token);
        self
    }
    pub fn right_at<GT: Into<GridToken>>(mut self, layout: Layout, gt: GT) -> Self {
        let token = gt.into();
        self.ref_context.references.extend(token.references());
        self.exceptions.insert(
            GridTokenDescException::new(layout, GridTokenDesc::Right),
            token,
        );
        self
    }
    // top-left
    // center
    // area
    // horizontal
    // vertical
}
#[derive(Clone, Copy, Component)]
pub struct Grid {
    columns: u32,
    rows: u32,
    gap: Coordinates,
}
impl Grid {
    pub fn new(columns: u32, rows: u32) -> Grid {
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
impl Default for Grid {
    fn default() -> Self {
        Self::new(1, 1)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum GridContext {
    Screen,
    Named(LeafHandle),
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
    pub(crate) fn references(&self) -> HashSet<GridContext> {
        let mut set = HashSet::new();
        for partition in &self.partitions {
            set.insert(partition.context.clone());
        }
        set
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
