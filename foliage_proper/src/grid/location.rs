use crate::coordinate::points::Points;
use crate::disable::AutoDisable;
use crate::enable::AutoEnable;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::{AspectRatio, GridUnit, ScalarUnit, View};
use crate::visibility::AutoVisibility;
use crate::{
    Component, Coordinates, Grid, Layout, Logical, ResolvedVisibility, Section, Stem, Tree, Update,
    Visibility, Write,
};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Res, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
use std::cmp::PartialEq;
use std::collections::HashSet;

#[derive(Component)]
#[component(on_insert = Location::on_insert)]
pub struct Location {
    pub xs: Option<LocationConfiguration>,
    pub sm: Option<LocationConfiguration>,
    pub md: Option<LocationConfiguration>,
    pub lg: Option<LocationConfiguration>,
    pub xl: Option<LocationConfiguration>,
}
#[derive(Copy, Clone)]
pub enum ResolvedLocation {
    Points(Points<Logical>),
    Section(Section<Logical>),
}

impl Default for Location {
    fn default() -> Self {
        Self::new()
    }
}

impl Location {
    pub fn new() -> Self {
        Self {
            xs: None,
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
    }
    pub fn xs<HAD: Into<LocationAxisDescriptor>, VAD: Into<LocationAxisDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.xs.replace((had.into(), vad.into()).into());
        self
    }
    pub fn sm<HAD: Into<LocationAxisDescriptor>, VAD: Into<LocationAxisDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.sm.replace((had.into(), vad.into()).into());
        self
    }
    pub fn md<HAD: Into<LocationAxisDescriptor>, VAD: Into<LocationAxisDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.md.replace((had.into(), vad.into()).into());
        self
    }
    pub fn lg<HAD: Into<LocationAxisDescriptor>, VAD: Into<LocationAxisDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.lg.replace((had.into(), vad.into()).into());
        self
    }
    pub fn xl<HAD: Into<LocationAxisDescriptor>, VAD: Into<LocationAxisDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.xl.replace((had.into(), vad.into()).into());
        self
    }
    fn at_least_xs(&self) -> Option<LocationConfiguration> {
        if self.xs.is_none() {
            None
        } else {
            Some(self.xs.unwrap())
        }
    }
    fn at_least_sm(&self) -> Option<LocationConfiguration> {
        if let Some(sm) = &self.sm {
            Some(*sm)
        } else {
            self.at_least_xs()
        }
    }
    fn at_least_md(&self) -> Option<LocationConfiguration> {
        if let Some(md) = &self.md {
            Some(*md)
        } else {
            self.at_least_sm()
        }
    }
    fn at_least_lg(&self) -> Option<LocationConfiguration> {
        if let Some(lg) = &self.lg {
            Some(*lg)
        } else {
            self.at_least_md()
        }
    }
    pub fn config(&self, layout: Layout) -> Option<LocationConfiguration> {
        match layout {
            Layout::Xs => self.at_least_xs(),
            Layout::Sm => self.at_least_sm(),
            Layout::Md => self.at_least_md(),
            Layout::Lg => self.at_least_lg(),
            Layout::Xl => {
                if let Some(xl) = &self.xl {
                    Some(*xl)
                } else {
                    self.at_least_lg()
                }
            }
        }
    }
    pub fn resolve(
        &self,
        layout: Layout,
        stem: Section<Logical>,
        stack: Option<Section<Logical>>,
        grid: Grid,
        aspect: Option<AspectRatio>,
        view: View,
        current: Section<Logical>,
    ) -> Option<ResolvedLocation> {
        let mut resolved_points = Option::<Points<Logical>>::None;
        if let Some(config) = self.config(layout) {
            let mut ax = match config.horizontal.a {
                GridUnit::Aligned(a) => grid.column(layout, stem, a, false),
                GridUnit::Scalar(s) => s.horizontal(stem),
                GridUnit::Stack => {
                    if let Some(stack) = stack {
                        stack.right()
                    } else {
                        return None;
                    }
                }
                GridUnit::Auto => {
                    panic!("Auto not supported in horizontal-begin.");
                }
            } + config.horizontal.padding.coordinates.a();
            let mut bx = match config.horizontal.b {
                GridUnit::Aligned(a) => grid.column(layout, stem, a, true),
                GridUnit::Scalar(s) => s.horizontal(stem),
                GridUnit::Stack => {
                    panic!("Stack not supported in horizontal-end")
                }
                GridUnit::Auto => {
                    0.0 // Zeroed on purpose
                }
            } - config.horizontal.padding.coordinates.b();
            match config.horizontal.ty {
                LocationAxisType::Point => {
                    if let GridUnit::Aligned(_) = config.horizontal.a {
                        ax += 0.5 * grid.column_metrics(layout, stem).0;
                    }
                    if let GridUnit::Aligned(_) = config.horizontal.b {
                        bx -= 0.5 * grid.row_metrics(layout, stem).0;
                    }
                    resolved_points.replace(Points::new().a((ax, bx)));
                }
                LocationAxisType::Span => {
                    // already in x / w context
                }
                LocationAxisType::To => {
                    bx -= ax; // convert to x / w context
                }
            }
            let mut ay = match config.vertical.a {
                GridUnit::Aligned(a) => grid.row(layout, stem, a, false),
                GridUnit::Scalar(s) => s.vertical(stem),
                GridUnit::Stack => {
                    if let Some(stack) = stack {
                        stack.bottom()
                    } else {
                        return None;
                    }
                }
                GridUnit::Auto => {
                    panic!("Auto not supported in vertical-begin");
                }
            } + config.vertical.padding.coordinates.a();
            let mut by = match config.vertical.b {
                GridUnit::Aligned(a) => grid.row(layout, stem, a, true),
                GridUnit::Scalar(s) => s.vertical(stem),
                GridUnit::Stack => {
                    panic!("Stack not supported in vertical-end");
                }
                GridUnit::Auto => {
                    0.0 // Zeroed on purpose
                }
            } - config.vertical.padding.coordinates.b();
            match config.vertical.ty {
                LocationAxisType::Point => {
                    if let GridUnit::Aligned(_) = config.vertical.a {
                        ay += 0.5 * grid.column_metrics(layout, stem).0;
                    }
                    if let GridUnit::Aligned(_) = config.vertical.b {
                        by -= 0.5 * grid.row_metrics(layout, stem).0;
                    }
                    resolved_points.as_mut().unwrap().set_b((ay, by));
                }
                LocationAxisType::Span => {
                    // already in y / h context
                }
                LocationAxisType::To => {
                    by -= ay; // convert to y / h context
                }
            }
            if let Some(mut pts) = resolved_points {
                for pt in pts.data.iter_mut() {
                    *pt += view.offset;
                }
                return Some(ResolvedLocation::Points(pts));
            }
            if config.horizontal.b == GridUnit::Auto {
                bx = current.width();
            }
            if config.vertical.b == GridUnit::Auto {
                by = current.height();
            }
            if let Some(max_width) = config.horizontal.max {
                let mx = max_width.horizontal(stem);
                if bx > mx {
                    let diff = bx - mx;
                    match config.horizontal.justify {
                        Justify::Near => {
                            // do nothing to the ax
                        }
                        Justify::Far => {
                            ax += diff;
                        }
                        Justify::Center => {
                            ax += diff / 2.0;
                        }
                    }
                    bx = mx;
                }
            }
            if let Some(max_height) = config.vertical.max {
                let my = max_height.vertical(stem);
                if by > my {
                    let diff = by - my;
                    match config.vertical.justify {
                        Justify::Near => {
                            // do nothing to ay
                        }
                        Justify::Far => {
                            ay += diff;
                        }
                        Justify::Center => {
                            ay += diff / 2.0;
                        }
                    }
                    by = my;
                }
            }
            if let Some(min_width) = config.horizontal.min {
                let mx = min_width.horizontal(stem);
                if bx < mx {
                    bx = mx;
                }
            }
            if let Some(min_height) = config.vertical.min {
                let my = min_height.vertical(stem);
                if by < my {
                    by = my;
                }
            }
            let mut resolved = Section::new((ax, ay), (bx, by));
            if let Some(_aspect) = aspect {
                // involve auto for which dimension to be dependent
                todo!("constrain by aspect")
            }
            if config.horizontal.a != GridUnit::Stack {
                resolved.position -= (view.offset.left(), 0).into();
            }
            if config.vertical.a != GridUnit::Stack {
                resolved.position -= (0, view.offset.top()).into();
            }
            Some(ResolvedLocation::Section(resolved))
        } else {
            None
        }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Location>::new(), this);
    }
    pub(crate) fn update_from_visibility(trigger: Trigger<Write<Visibility>>, mut tree: Tree) {
        tracing::trace!("update_from_visibility for {:?}", trigger.entity());
        tree.trigger_targets(Update::<Location>::new(), trigger.entity());
    }
    pub(crate) fn update_location(
        trigger: Trigger<Update<Location>>,
        mut tree: Tree,
        layout: Res<Layout>,
        locations: Query<&Location>,
        sections: Query<&Section<Logical>>,
        grids: Query<&Grid>,
        stems: Query<&Stem>,
        stacks: Query<&Stack>,
        visibilities: Query<(&ResolvedVisibility, &AutoVisibility)>,
        aspect_ratios: Query<&AspectRatio>,
        views: Query<&View>,
        viewport: Res<ViewportHandle>,
    ) {
        let this = trigger.entity();
        if let Ok(location) = locations.get(this) {
            if let Ok((_, auto_vis)) = visibilities.get(this) {
                let stem = stems.get(this).unwrap();
                let stem_section = stem
                    .id
                    .map(|id| *sections.get(id).unwrap())
                    .unwrap_or(viewport.section());
                let stack = if let Ok(stack) = stacks.get(this) {
                    if let Some(s) = stack.id {
                        if let Ok(sec) = sections.get(s) {
                            if visibilities.get(s).unwrap().0.visible() {
                                Some(*sec)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                let grid = stem
                    .id
                    .map(|id| *grids.get(id).unwrap())
                    .unwrap_or_default();
                let aspect = aspect_ratios.get(this).copied().ok();
                let view = stem
                    .id
                    .map(|id| *views.get(id).unwrap())
                    .unwrap_or_default();
                let current = *sections.get(this).unwrap();
                if let Some(resolved) =
                    location.resolve(*layout, stem_section, stack, grid, aspect, view, current)
                {
                    if !auto_vis.visible {
                        tracing::trace!("auto-enabling for {:?}", this);
                        tree.entity(this).insert(AutoVisibility::new(true));
                        tree.trigger_targets(AutoEnable::new(), this);
                    }
                    match resolved {
                        ResolvedLocation::Points(pts) => {
                            tree.entity(this).insert(pts);
                        }
                        ResolvedLocation::Section(section) => {
                            tree.entity(this).insert(section);
                        }
                    }
                } else if auto_vis.visible {
                    tracing::trace!("auto-disable for {:?}", this);
                    tree.entity(this).insert(AutoVisibility::new(false));
                    tree.trigger_targets(AutoDisable::new(), this);
                }
            }
        }
    }
}
#[derive(Clone, Component, Default)]
pub struct StackDeps {
    pub ids: HashSet<Entity>,
}
#[derive(Component, Copy, Clone)]
#[component(on_insert = Stack::on_insert)]
#[component(on_replace = Stack::on_replace)]
#[derive(Default)]
pub struct Stack {
    pub id: Option<Entity>,
}
impl Stack {
    pub fn new(entity: Entity) -> Self {
        Self { id: Some(entity) }
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let stack = world.get::<Stack>(this).unwrap();
        if let Some(id) = stack.id {
            if let Some(mut deps) = world.get_mut::<StackDeps>(id) {
                deps.ids.insert(this);
            } else {
                let mut stack_deps = StackDeps::default();
                stack_deps.ids.insert(this);
                world.commands().entity(id).insert(stack_deps);
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, id: Entity, _c: ComponentId) {
        let stack = world.get::<Stack>(id).unwrap();
        if let Some(id) = stack.id {
            if let Some(mut deps) = world.get_mut::<StackDeps>(id) {
                deps.ids.remove(&id);
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct LocationConfiguration {
    pub horizontal: LocationAxisDescriptor,
    pub vertical: LocationAxisDescriptor,
}
impl From<(LocationAxisDescriptor, LocationAxisDescriptor)> for LocationConfiguration {
    fn from(value: (LocationAxisDescriptor, LocationAxisDescriptor)) -> Self {
        Self {
            horizontal: value.0,
            vertical: value.1,
        }
    }
}
#[derive(Copy, Clone)]
pub struct Padding {
    pub coordinates: Coordinates,
}
impl Default for Padding {
    fn default() -> Self {
        Self {
            coordinates: (8, 8).into(),
        }
    }
}
impl From<i32> for Padding {
    fn from(value: i32) -> Self {
        Self {
            coordinates: Coordinates::from((value, value)),
        }
    }
}
impl From<(i32, i32)> for Padding {
    fn from(value: (i32, i32)) -> Self {
        Self {
            coordinates: Coordinates::from((value.0, value.1)),
        }
    }
}
#[derive(Copy, Clone)]
pub struct LocationAxisDescriptor {
    pub a: GridUnit,
    pub b: GridUnit,
    pub ty: LocationAxisType,
    pub padding: Padding,
    pub justify: Justify,
    pub max: Option<ScalarUnit>,
    pub min: Option<ScalarUnit>,
}
impl LocationAxisDescriptor {
    pub fn justify(mut self, justify: Justify) -> Self {
        debug_assert_ne!(self.ty, LocationAxisType::Point);
        self.justify = justify;
        self
    }
    pub fn pad<P: Into<Padding>>(mut self, pad: P) -> Self {
        self.padding = pad.into();
        self
    }
    pub fn max<S: Into<ScalarUnit>>(mut self, max: S) -> Self {
        let max = max.into();
        debug_assert!(if let Some(mn) = self.min {
            max >= mn
        } else {
            true
        });
        debug_assert_ne!(self.ty, LocationAxisType::Point);
        self.max.replace(max);
        self
    }
    pub fn min<S: Into<ScalarUnit>>(mut self, min: S) -> Self {
        let min = min.into();
        debug_assert!(if let Some(mx) = self.max {
            mx >= min
        } else {
            true
        });
        debug_assert_ne!(self.ty, LocationAxisType::Point);
        self.min.replace(min);
        self
    }
}
pub fn stack() -> GridUnit {
    GridUnit::Stack
}
pub fn auto() -> GridUnit {
    GridUnit::Auto
}
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Justify {
    Near,
    Far,
    #[default]
    Center,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocationAxisType {
    Point,
    Span,
    To,
}
