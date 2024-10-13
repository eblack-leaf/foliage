use crate::anim::{Animate, Interpolations};
use crate::coordinate::points::Points;
use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::aspect::{GridAspect, LocationAspect, SectionAspectConfiguration};
use crate::grid::responsive_points::{
    PointsDiff, ResolvedPoints, ResponsivePoints, ResponsivePointsException,
};
use crate::grid::token::{LocationAspectDescriptorValue, SpecifiedDescriptorValue};
use crate::grid::Grid;
use crate::layout::{Layout, LayoutGrid};
use crate::leaf::{Dependents, Stem};
use crate::tree::Tree;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{Component, ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::Trigger;
use bevy_ecs::system::{Query, Res};
use bevy_ecs::world::DeferredWorld;
use smallvec::SmallVec;
use crate::twig::Configure;

#[derive(Bundle, Default)]
pub struct ResponsiveSectionBundle {
    pub(crate) resolved_configuration: ResolvedConfiguration,
    pub(crate) exceptions: ResponsiveSectionException,
    pub(crate) responsive_section: ResponsiveSection,
    pub(crate) layout_check: ConfigureFromLayoutAndException,
}
#[derive(Bundle, Default)]
pub struct ResponsiveSectionAnimationHelpers {
    pub(crate) last: SectionLast,
    pub(crate) diff: SectionDiff,
}

impl ResponsiveSectionBundle {
    pub(crate) fn new() -> Self {
        ResponsiveSectionBundle {
            resolved_configuration: ResolvedConfiguration::default(),
            exceptions: ResponsiveSectionException::default(),
            responsive_section: ResponsiveSection::default(),
            layout_check: ConfigureFromLayoutAndException {},
        }
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::Top,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::Bottom,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::Height,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::CenterY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect.1.set(
                GridAspect::Left,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect.1.set(
                GridAspect::Right,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect.1.set(
                GridAspect::Width,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .responsive_section
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect.1.set(
                GridAspect::CenterX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn except_at<LA: Into<ResponsiveSection>>(mut self, layout: Layout, la: LA) -> Self {
        let config = la.into();
        for c in config.configurations {
            self.exceptions
                .exceptions
                .push((SectionException::new(layout, c.0), c.1));
        }
        self
    }
}

#[derive(Component, Clone, Default)]
pub struct ResolvedConfiguration {
    configurations: [(SectionAspectConfiguration, LocationAspect); 2],
}

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct ReferentialData {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) grid: Grid,
    pub(crate) points: Points<LogicalContext>,
}

impl ResolvedConfiguration {
    pub(crate) fn evaluate(
        &self,
        stem: ReferentialData,
        screen: ReferentialData,
    ) -> Option<Section<LogicalContext>> {
        let mut resolution = Section::default();
        for (aspect_config, aspect_value) in self.configurations.iter() {
            let pair_config = (
                aspect_value.aspects[0].aspect,
                aspect_value.aspects[1].aspect,
            );
            let data = (
                aspect_value.aspects[0].value.resolve(stem, screen),
                aspect_value.aspects[1].value.resolve(stem, screen),
            );
            match aspect_config {
                SectionAspectConfiguration::Horizontal => {
                    if pair_config == (GridAspect::Left, GridAspect::Right) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width(data.1 - data.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::CenterX) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Left, GridAspect::Width) {
                        resolution.position.set_x(data.0);
                        resolution.area.set_width(data.1);
                    } else if pair_config == (GridAspect::Width, GridAspect::CenterX) {
                        resolution.position.set_x(data.1 - data.0 / 2.0);
                        resolution.area.set_width(data.0);
                    } else if pair_config == (GridAspect::Width, GridAspect::Right) {
                        resolution.position.set_x(data.1 - data.0);
                        resolution.area.set_width(data.0);
                    } else if pair_config == (GridAspect::CenterX, GridAspect::Right) {
                        let diff = data.1 - data.0;
                        resolution.position.set_x(data.0 - diff);
                        resolution.area.set_width(diff * 2.0);
                    }
                }
                SectionAspectConfiguration::Vertical => {
                    if pair_config == (GridAspect::Top, GridAspect::Bottom) {
                        resolution.position.set_y(data.0);
                        resolution.area.set_height(data.1 - data.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::CenterY) {
                        resolution.position.set_y(data.0);
                        resolution.area.set_height((data.1 - data.0) * 2.0);
                    } else if pair_config == (GridAspect::Top, GridAspect::Height) {
                        resolution.position.set_y(data.0);
                        resolution.area.set_height(data.1);
                    } else if pair_config == (GridAspect::Height, GridAspect::CenterY) {
                        resolution.position.set_y(data.1 - data.0 / 2.0);
                        resolution.area.set_height(data.0);
                    } else if pair_config == (GridAspect::Height, GridAspect::Bottom) {
                        resolution.position.set_y(data.1 - data.0);
                        resolution.area.set_height(data.0);
                    } else if pair_config == (GridAspect::CenterY, GridAspect::Bottom) {
                        let diff = data.1 - data.0;
                        resolution.position.set_y(data.0 - diff);
                        resolution.area.set_height(diff * 2.0);
                    }
                }
            }
        }
        Some(resolution)
    }
}

#[derive(Copy, Clone, Event)]
pub struct EvaluateLocation {
    skip_deps: bool,
}
impl EvaluateLocation {
    pub fn no_deps() -> Self {
        Self { skip_deps: true }
    }
    pub fn full() -> Self {
        Self { skip_deps: false }
    }
}

pub(crate) fn evaluate_location(
    trigger: Trigger<EvaluateLocation>,
    stems: Query<&Stem>,
    dependents: Query<&Dependents>,
    viewport_handle: Res<ViewportHandle>,
    layout_grid: Res<LayoutGrid>,
    grids: Query<&Grid>,
    responsive: Query<&ResolvedConfiguration>,
    diff: Query<&SectionDiff>,
    mut eval: Query<&mut Section<LogicalContext>>,
    responsive_points: Query<&ResolvedPoints>,
    point_diff: Query<&PointsDiff>,
    mut point_eval: Query<&mut Points<LogicalContext>>,
    mut tree: Tree,
) {
    let screen = ReferentialData {
        section: viewport_handle.section(),
        grid: layout_grid.grid,
        points: Points::default(),
    };
    if let Ok(stem) = stems.get(trigger.entity()) {
        let stem = if let Some(s) = stem.0 {
            ReferentialData {
                section: eval.get(s).copied().unwrap_or_default(),
                grid: grids.get(s).copied().unwrap_or_default(),
                points: point_eval.get(s).copied().unwrap_or_default(),
            }
        } else {
            screen
        };
        if let Ok(res) = responsive.get(trigger.entity()) {
            let resolved = res.evaluate(stem, screen);
            if let Some(s) = resolved {
                *eval.get_mut(trigger.entity()).unwrap() = s + diff
                    .get(trigger.entity())
                    .copied()
                    .unwrap_or_default()
                    .value();
                tree.trigger_targets(Configure{}, trigger.entity());
            }
        }
        if let Ok(res) = responsive_points.get(trigger.entity()) {
            if let Some(resolved) = res.evaluate(stem, screen) {
                let solved = resolved
                    + point_diff
                        .get(trigger.entity())
                        .copied()
                        .unwrap_or_default()
                        .value();
                *point_eval.get_mut(trigger.entity()).unwrap() = solved;
                *eval.get_mut(trigger.entity()).unwrap() = solved.bbox();
                tree.trigger_targets(Configure{}, trigger.entity());
            }
        }
        if trigger.event().skip_deps {
            return;
        }
        if let Ok(mut deps) = dependents.get(trigger.entity()) {
            if deps.0.is_empty() {
                return;
            }
            tree.trigger_targets(
                EvaluateLocation::full(),
                deps.0
                    .iter()
                    .copied()
                    .filter(|d| responsive.contains(*d) || responsive_points.contains(*d))
                    .collect::<Vec<Entity>>(),
            )
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct ConfigureFromLayoutAndException {}

impl ConfigureFromLayoutAndException {
    fn on_insert(mut world: DeferredWorld, entity: Entity, _c: ComponentId) {
        let layout = world.get_resource::<Layout>().unwrap().clone();
        if let Some(base) = world.entity(entity).get::<ResponsiveSection>().cloned() {
            let mut to_use = base.configurations.clone();
            if let Some(exceptions) = world.entity(entity).get::<ResponsiveSectionException>() {
                for (a, b) in exceptions.exceptions.iter() {
                    if a.layout.contains(layout) {
                        let mut aspect = b.clone();
                        match aspect.aspects[0].value {
                            LocationAspectDescriptorValue::Existing => {
                                let config = a.config.value();
                                aspect.aspects[0].value =
                                    if base.configurations[config].1.aspects[0].aspect
                                        == aspect.aspects[0].aspect
                                    {
                                        base.configurations[config].1.aspects[0].value.clone()
                                    } else {
                                        debug_assert_eq!(
                                            base.configurations[config].1.aspects[1].aspect,
                                            aspect.aspects[0].aspect
                                        );
                                        base.configurations[config].1.aspects[1].value.clone()
                                    };
                            }
                            LocationAspectDescriptorValue::Specified(_) => {}
                        }
                        match aspect.aspects[1].value {
                            LocationAspectDescriptorValue::Existing => {
                                let config = a.config.value();
                                aspect.aspects[1].value =
                                    if base.configurations[config].1.aspects[0].aspect
                                        == aspect.aspects[1].aspect
                                    {
                                        base.configurations[config].1.aspects[0].value.clone()
                                    } else {
                                        debug_assert_eq!(
                                            base.configurations[config].1.aspects[1].aspect,
                                            aspect.aspects[1].aspect
                                        );
                                        base.configurations[config].1.aspects[1].value.clone()
                                    };
                            }
                            LocationAspectDescriptorValue::Specified(_) => {}
                        }
                        to_use[a.config.value()].1 = aspect;
                    }
                }
            }
            if let Some(mut resolved) = world.get_mut::<ResolvedConfiguration>(entity) {
                resolved.configurations = to_use;
            }
        }
        if let Some(base) = world.entity(entity).get::<ResponsivePoints>().cloned() {
            let mut to_use = base.configurations.clone();
            if let Some(exceptions) = world.entity(entity).get::<ResponsivePointsException>() {
                for (a, b) in exceptions.exceptions.iter() {
                    if a.layout.contains(layout) {
                        let config = a.pac.value();
                        let mut aspect = b.clone();
                        if aspect.count == 0 {
                            continue;
                        }
                        for i in 0..2 {
                            match aspect.aspects[i].value {
                                LocationAspectDescriptorValue::Existing => {
                                    aspect.aspects[i].value =
                                        if base.configurations[config].1.aspects[0].aspect
                                            == aspect.aspects[i].aspect
                                        {
                                            base.configurations[config].1.aspects[0].value.clone()
                                        } else {
                                            debug_assert_eq!(
                                                base.configurations[config].1.aspects[1].aspect,
                                                aspect.aspects[i].aspect
                                            );
                                            base.configurations[config].1.aspects[1].value.clone()
                                        }
                                }
                                LocationAspectDescriptorValue::Specified(_) => {}
                            }
                        }
                        to_use[config].1 = aspect;
                    }
                }
            }
            if let Some(mut resolved) = world.get_mut::<ResolvedPoints>(entity) {
                resolved.configurations = to_use;
            }
        }
    }
}

impl Component for ConfigureFromLayoutAndException {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(ConfigureFromLayoutAndException::on_insert);
    }
}

#[derive(Component, Default, Clone)]
pub struct ResponsiveSectionException {
    pub exceptions: SmallVec<[(SectionException, LocationAspect); 2]>,
}

#[derive(Component, Default, Copy, Clone)]
pub struct SectionLast {
    pub section: Section<LogicalContext>,
}

#[derive(Component, Default, Copy, Clone)]
pub struct SectionDiff {
    pub section: Section<LogicalContext>,
    pub percent: f32,
}
impl SectionDiff {
    pub(crate) fn value(&self) -> Section<LogicalContext> {
        self.section * self.percent
    }
}
#[derive(Event)]
pub(crate) struct ResponsiveSectionAnimationCalc {}
pub(crate) fn anim_calc(
    trigger: Trigger<ResponsiveSectionAnimationCalc>,
    actual: Query<&Section<LogicalContext>>,
    pts: Query<&Points<LogicalContext>>,
    mut tree: Tree,
) {
    let last = actual.get(trigger.entity()).copied().unwrap();
    let last_pts = pts.get(trigger.entity()).copied().unwrap();
    tree.entity(trigger.entity())
        .insert(ConfigureFromLayoutAndException {});
    tree.trigger_targets(EvaluateLocation::no_deps(), trigger.entity());
    tree.trigger_targets(CalcDiff { last, last_pts }, trigger.entity());
}
#[derive(Event, Copy, Clone)]
pub(crate) struct CalcDiff {
    last: Section<LogicalContext>,
    last_pts: Points<LogicalContext>,
}
pub(crate) fn calc_diff(
    trigger: Trigger<CalcDiff>,
    mut diffs: Query<&mut SectionDiff>,
    mut pt_diffs: Query<&mut PointsDiff>,
    calculated: Query<&Section<LogicalContext>>,
    calc_pts: Query<&Points<LogicalContext>>,
) {
    if let Ok(mut diff) = diffs.get_mut(trigger.entity()) {
        let last = trigger.event().last;
        let new = calculated.get(trigger.entity()).copied().unwrap();
        diff.section = new - last;
    }
    if let Ok(mut diff) = pt_diffs.get_mut(trigger.entity()) {
        let last = trigger.event().last_pts;
        let new = calc_pts.get(trigger.entity()).copied().unwrap();
        diff.points = new - last;
    }
}
#[derive(Component, Clone, Default)]
pub struct ResponsiveSectionAnimPackage {
    pub res: ResponsiveSection,
    pub exc: ResponsiveSectionException,
}
impl Animate for ResponsiveSectionAnimPackage {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        todo!()
    }
}
impl Animate for SectionDiff {
    fn interpolations(start: &Self, end: &Self) -> Interpolations {
        todo!()
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(s) = interpolations.read(0) {
            self.percent = s;
        }
    }
}
#[derive(Clone, Component, Default)]
pub struct ResponsiveSection {
    configurations: [(SectionAspectConfiguration, LocationAspect); 2],
}

impl ResponsiveSection {
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
        }
    }
    pub fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::Top,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_top(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect
                .1
                .set(GridAspect::Top, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::Bottom,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_bottom(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect
                .1
                .set(GridAspect::Bottom, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::Height,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_height(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect
                .1
                .set(GridAspect::Height, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect.1.set(
                GridAspect::CenterY,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_center_y(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Vertical.value())
        {
            aspect.0 = SectionAspectConfiguration::Vertical;
            aspect
                .1
                .set(GridAspect::CenterY, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            // sanitize that other is compatible
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect.1.set(
                GridAspect::Left,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_left(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect
                .1
                .set(GridAspect::Left, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Right,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_right(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect
                .1
                .set(GridAspect::Right, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::Width,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_width(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect
                .1
                .set(GridAspect::Width, LocationAspectDescriptorValue::Existing);
        }
        self
    }
    pub fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, d: LAD) -> Self {
        if let Some(mut aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            // sanitize that other is compatible
            aspect.1.set(
                GridAspect::CenterX,
                LocationAspectDescriptorValue::Specified(d.into()),
            );
        }
        self
    }
    pub fn existing_center_x(mut self) -> Self {
        if let Some(aspect) = self
            .configurations
            .get_mut(SectionAspectConfiguration::Horizontal.value())
        {
            aspect.0 = SectionAspectConfiguration::Horizontal;
            aspect
                .1
                .set(GridAspect::CenterX, LocationAspectDescriptorValue::Existing);
        }
        self
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct SectionException {
    layout: Layout,
    config: SectionAspectConfiguration,
}

impl SectionException {
    fn new(layout: Layout, config: SectionAspectConfiguration) -> SectionException {
        Self { layout, config }
    }
}
