use crate::anim::{Animate, Interpolations};
use crate::coordinate::area::Area;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;
use crate::layout::{Layout, LayoutGrid};
use crate::leaf::{IdTable, LeafHandle};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::query::{Changed, Or};
use bevy_ecs::system::{ParamSet, Query, Res, ResMut};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[cfg(test)]
#[test]
fn behavior() {
    let location = GridLocation::new();
}
#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd)]
pub enum GridContext {
    Screen,
    Named(LeafHandle),
    Absolute,
}
impl GridContext {
    pub fn top(self) -> LocationAspectTokenValue {
        LocationAspectTokenValue::ContextAspect(GridAspect::Top)
    }
    // ...
}
pub fn screen() -> GridContext {
    GridContext::Screen
}
#[derive(Clone, Copy)]
pub(crate) enum LocationAspectTokenOp {
    Add,
    Minus,
    // ...
}
#[derive(Clone, Copy)]
pub enum RelativeUnit {
    Column(u32),
    Row(u32),
    Percent(f32),
}
#[derive(Clone, Copy)]
pub enum LocationAspectTokenValue {
    ContextAspect(GridAspect),
    Relative(RelativeUnit),
    Absolute(CoordinateUnit),
}
#[derive(Clone)]
pub(crate) struct LocationAspectToken {
    op: LocationAspectTokenOp,
    context: GridContext,
    value: LocationAspectTokenValue,
}
#[derive(Clone)]
pub struct SpecifiedDescriptorValue {
    tokens: Vec<LocationAspectToken>,
}
impl SpecifiedDescriptorValue {
    pub(crate) fn dependencies(&self) -> ReferentialDependencies {
        let mut set = HashSet::new();
        for token in &self.tokens {
            set.insert(token.context.clone());
        }
        ReferentialDependencies::new(set)
    }
}
#[derive(Default, Clone)]
pub(crate) enum LocationAspectDescriptorValue {
    #[default]
    Existing,
    Specified(SpecifiedDescriptorValue),
}
#[derive(Default, Clone)]
pub(crate) struct LocationAspectDescriptor {
    aspect: GridAspect,
    value: LocationAspectDescriptorValue,
}
impl LocationAspectDescriptor {
    pub(crate) fn new(aspect: GridAspect, value: LocationAspectDescriptorValue) -> Self {
        Self { aspect, value }
    }
}
#[derive(Default, Clone)]
pub struct LocationAspect {
    independent_or_x: LocationAspectDescriptor,
    other_or_y: LocationAspectDescriptor,
}
impl LocationAspect {
    pub fn new() -> LocationAspect {
        LocationAspect {
            independent_or_x: Default::default(),
            other_or_y: Default::default(),
        }
    }
    pub(crate) fn config(&self) -> AspectConfiguration {
        match self.independent_or_x.aspect {
            GridAspect::Top | GridAspect::Height | GridAspect::Bottom | GridAspect::CenterY => {
                AspectConfiguration::Vertical
            }
            GridAspect::Left | GridAspect::Width | GridAspect::Right | GridAspect::CenterX => {
                AspectConfiguration::Horizontal
            }
            GridAspect::PointAX | GridAspect::PointAY => AspectConfiguration::PointA,
            GridAspect::PointBX | GridAspect::PointBY => AspectConfiguration::PointB,
            GridAspect::PointCX | GridAspect::PointCY => AspectConfiguration::PointC,
            GridAspect::PointDX | GridAspect::PointDY => AspectConfiguration::PointD,
        }
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.independent_or_x = LocationAspectDescriptor::new(
            GridAspect::Top,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub fn existing_top(mut self) -> Self {
        self.independent_or_x =
            LocationAspectDescriptor::new(GridAspect::Top, LocationAspectDescriptorValue::Existing);
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.other_or_y = LocationAspectDescriptor::new(
            GridAspect::Bottom,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    // ...
}
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub(crate) enum AspectConfiguration {
    Horizontal,
    Vertical,
    PointA,
    PointB,
    PointC,
    PointD,
}
#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct GridLocationException {
    layout: Layout,
    config: AspectConfiguration,
}
impl GridLocationException {
    fn new(layout: Layout, config: AspectConfiguration) -> GridLocationException {
        Self { layout, config }
    }
}
#[derive(Clone, Default)]
pub(crate) struct AnimationHookContext {
    pub(crate) hook_percent: f32,
    pub(crate) last: Section<LogicalContext>,
    diff: Section<LogicalContext>,
    offset: Section<LogicalContext>,
    pub(crate) create_diff: bool,
    pub(crate) hook_changed: bool,
}
#[derive(Clone, Default)]
pub(crate) struct PointDrivenAnimationHook {
    pub(crate) point_a: AnimationHookContext,
    pub(crate) point_b: AnimationHookContext,
    pub(crate) point_c: AnimationHookContext,
    pub(crate) point_d: AnimationHookContext,
}
#[derive(Clone)]
pub(crate) enum GridLocationAnimationHook {
    SectionDriven(AnimationHookContext),
    PointDriven(PointDrivenAnimationHook),
}
impl Default for GridLocationAnimationHook {
    fn default() -> Self {
        Self::SectionDriven(AnimationHookContext::default())
    }
}
#[derive(Clone, Component)]
pub struct GridLocation {
    configurations: HashMap<AspectConfiguration, LocationAspect>,
    exceptions: HashMap<GridLocationException, LocationAspect>,
    pub(crate) animation_hook: GridLocationAnimationHook,
}
impl Animate for GridLocation {
    fn interpolations(start: &Self, _end: &Self) -> Interpolations {
        match &start.animation_hook {
            GridLocationAnimationHook::SectionDriven(_) => Interpolations::new().with(1.0, 0.0),
            GridLocationAnimationHook::PointDriven(_) => Interpolations::new()
                .with(1.0, 0.0)
                .with(1.0, 0.0)
                .with(1.0, 0.0)
                .with(1.0, 0.0),
        }
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        match &mut self.animation_hook {
            GridLocationAnimationHook::SectionDriven(hook) => {
                // hook changed
                if let Some(p) = interpolations.read(0) {
                    hook.hook_percent = p;
                    hook.hook_changed = true;
                }
            }
            GridLocationAnimationHook::PointDriven(hook) => {
                if let Some(p) = interpolations.read(0) {
                    hook.point_a.hook_percent = p;
                    hook.point_a.hook_changed = true;
                }
                if let Some(p) = interpolations.read(1) {
                    hook.point_b.hook_percent = p;
                    hook.point_b.hook_changed = true;
                }
                if let Some(p) = interpolations.read(2) {
                    hook.point_c.hook_percent = p;
                    hook.point_c.hook_changed = true;
                }
                if let Some(p) = interpolations.read(3) {
                    hook.point_d.hook_percent = p;
                    hook.point_d.hook_changed = true;
                }
            }
        }
    }
}
impl GridLocation {
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
            exceptions: Default::default(),
            animation_hook: Default::default(),
        }
    }
    pub(crate) fn deps(&self) -> ReferentialDependencies {
        let mut set = HashSet::new();
        for (_config, aspect) in self.configurations.iter() {
            if let LocationAspectDescriptorValue::Specified(s) = &aspect.independent_or_x.value {
                set.extend(s.dependencies().deps);
            }
            if let LocationAspectDescriptorValue::Specified(s) = &aspect.other_or_y.value {
                set.extend(s.dependencies().deps);
            }
        }
        ReferentialDependencies::new(set)
    }
    pub(crate) fn resolve(
        &self,
        context: &HashMap<GridContext, ReferentialData>,
        layout: Layout,
    ) -> Option<ResolvedLocation> {
        todo!()
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.independent_or_x.aspect = GridAspect::Top;
            aspect.independent_or_x.value = LocationAspectDescriptorValue::Specified(d.into());
        } else {
            self.configurations
                .insert(AspectConfiguration::Vertical, LocationAspect::new().top(d));
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self.configurations.get_mut(&AspectConfiguration::Vertical) {
            // sanitize that other is compatible
            aspect.other_or_y.aspect = GridAspect::Bottom;
            aspect.other_or_y.value = LocationAspectDescriptorValue::Specified(d.into());
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().bottom(d),
            );
        }
        self
    }
    // TODO when fn for points => set hook to point-driven
    pub fn except_at<LA: Into<LocationAspect>>(mut self, layout: Layout, la: LA) -> Self {
        let aspect = la.into();
        let ac = aspect.config();
        self.exceptions
            .insert(GridLocationException::new(layout, ac), aspect);
        self
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum GridAspect {
    #[default]
    Top,
    Left,
    Width,
    Height,
    PointAX,
    PointAY,
    PointBX,
    PointBY,
    PointCX,
    PointCY,
    PointDX,
    PointDY,
    CenterX, // Dependent => Right | Width | Left
    CenterY, // Dependent => Top | Height | Bottom
    Right,   // Dependent => Width | Left | CenterX
    Bottom,  // Dependent => Height | Top | CenterY
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
#[derive(Clone, Default, Component)]
pub(crate) struct ReferentialDependencies {
    deps: HashSet<GridContext>,
}
impl ReferentialDependencies {
    fn new(deps: HashSet<GridContext>) -> ReferentialDependencies {
        Self { deps }
    }
}
pub(crate) struct ReferentialOrderDeterminant<'a> {
    deps: &'a ReferentialDependencies,
    lh: &'a LeafHandle,
    location: &'a GridLocation,
    grid: Grid,
}
pub(crate) fn distill_location_deps(
    mut query: Query<(&GridLocation, &mut ReferentialDependencies), Changed<GridLocation>>,
) {
    for (location, mut dep) in query.iter_mut() {
        *dep = location.deps();
    }
}
pub(crate) fn resolve_grid_locations(
    check: Query<Entity, Or<(Changed<GridLocation>, Changed<Grid>)>>,
    mut read_and_update: ParamSet<(
        Query<
            (&LeafHandle, &GridLocation, &ReferentialDependencies, &Grid),
            Or<(Changed<GridLocation>, Changed<Grid>)>,
        >,
        Query<(
            &mut Position<LogicalContext>,
            &mut Area<LogicalContext>,
            &mut Points<LogicalContext>,
            &mut GridLocation,
        )>,
    )>,
    id_table: Res<IdTable>,
    viewport_handle: ResMut<ViewportHandle>,
    layout_grid: Res<LayoutGrid>,
    layout: Res<Layout>,
) {
    if check.is_empty() {
        return;
    }
    let mut ref_context = ReferentialContext::new(viewport_handle.section(), layout_grid.grid);
    let binding = read_and_update.p0();
    for (handle, location, deps, grid) in binding.iter() {
        ref_context.queue_leaf(handle, location, deps, *grid);
    }
    ref_context.resolve(*layout);
    let updates = ref_context.updates();
    drop(ref_context);
    drop(binding);
    for (handle, resolved) in updates {
        let e = id_table.lookup_leaf(handle).unwrap();
        *read_and_update.p1().get_mut(e).unwrap().0 = resolved.section.position;
        *read_and_update.p1().get_mut(e).unwrap().1 = resolved.section.area;
        if let Some(p) = resolved.points {
            *read_and_update.p1().get_mut(e).unwrap().2 = p;
        }
        if let Some(hook) = resolved.hook_update {
            read_and_update.p1().get_mut(e).unwrap().3.animation_hook = hook;
        }
    }
}
pub(crate) struct ReferentialContext<'a> {
    context: HashMap<GridContext, ReferentialData>,
    order: Vec<ReferentialOrderDeterminant<'a>>,
}
impl<'a> ReferentialContext<'a> {
    pub(crate) fn new(screen_section: Section<LogicalContext>, layout_grid: Grid) -> Self {
        Self {
            context: {
                let mut context = HashMap::new();
                context.insert(
                    GridContext::Screen,
                    ReferentialData::new(ResolvedLocation::new(screen_section), layout_grid),
                );
                context
            },
            order: vec![],
        }
    }
    pub(crate) fn queue_leaf(
        &mut self,
        lh: &'a LeafHandle,
        location: &'a GridLocation,
        deps: &'a ReferentialDependencies,
        grid: Grid,
    ) {
        self.order.push(ReferentialOrderDeterminant {
            lh,
            location,
            deps,
            grid,
        });
    }
    pub(crate) fn resolve(&mut self, layout: Layout) {
        self.order.sort_by(|a, b| {
            let b_depends_a = b.deps.deps.contains(&GridContext::Named(a.lh.clone()));
            let a_depends_b = a.deps.deps.contains(&GridContext::Named(b.lh.clone()));
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
        let order = self
            .order
            .drain(..)
            .collect::<Vec<ReferentialOrderDeterminant>>();
        for determinant in order {
            let resolved = determinant.location.resolve(&self.context, layout);
            if let Some(resolved) = resolved {
                self.context.insert(
                    GridContext::Named(determinant.lh.clone()),
                    ReferentialData::new(resolved, determinant.grid),
                );
            } else {
                panic!("invalid grid-location")
            }
        }
    }
    pub(crate) fn updates(&mut self) -> Vec<(LeafHandle, ResolvedLocation)> {
        let mut updates = vec![];
        for (k, v) in self.context.drain() {
            match k {
                GridContext::Screen => {
                    continue;
                }
                GridContext::Named(lh) => {
                    updates.push((lh, v.resolved));
                }
                GridContext::Absolute => {
                    continue;
                }
            }
        }
        updates
    }
}
pub(crate) struct ResolvedLocation {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) points: Option<Points<LogicalContext>>,
    pub(crate) hook_update: Option<GridLocationAnimationHook>,
}

impl ResolvedLocation {
    pub(crate) fn new(section: Section<LogicalContext>) -> Self {
        Self {
            section,
            points: None,
            hook_update: None,
        }
    }
    pub(crate) fn with_points(mut self, points: Points<LogicalContext>) -> Self {
        self.points = Some(points);
        self
    }
}

pub(crate) struct ReferentialData {
    pub(crate) resolved: ResolvedLocation,
    pub(crate) grid: Grid,
}

impl ReferentialData {
    pub(crate) fn new(resolved: ResolvedLocation, grid: Grid) -> Self {
        Self { resolved, grid }
    }
}
