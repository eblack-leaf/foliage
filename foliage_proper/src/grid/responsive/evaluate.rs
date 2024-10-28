use crate::ash::ClippingContext;
use crate::coordinate::points::Points;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinates, LogicalContext};
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::aspect::GridAspect;
use crate::grid::responsive::anim::{
    CalcDiff, ResponsiveAnimationHook, ResponsivePointsAnimationHook,
};
use crate::grid::responsive::resolve::ResolvedConfiguration;
use crate::grid::responsive::resolve::ResolvedPoints;
use crate::grid::Grid;
use crate::layout::LayoutGrid;
use crate::leaf::{Dependents, Stem};
use crate::text::{FontSize, GlyphPlacer, MonospacedFont, TextAlignment, TextValue};
use crate::twig::Configure;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::Component;
use bevy_ecs::world::DeferredWorld;

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct ReferentialData {
    pub(crate) section: Section<LogicalContext>,
    pub(crate) grid: Grid,
    pub(crate) points: Points<LogicalContext>,
    pub(crate) view: ScrollView,
}
#[derive(Bundle, Copy, Clone)]
pub struct ScrollContext {
    clipping_context: ClippingContext,
    extent_checker: ExtentChecker,
}
impl ScrollContext {
    pub fn new(root: Entity) -> Self {
        Self {
            clipping_context: ClippingContext::Entity(root),
            extent_checker: ExtentChecker(root),
        }
    }
}
#[derive(Copy, Clone, Component)]
pub(crate) struct ExtentChecker(pub(crate) Entity);
#[derive(Copy, Clone, Default, Debug, Component)]
pub struct ScrollView {
    pub position: Position<LogicalContext>,
}
#[derive(Component, Copy, Clone, Debug, Default)]
pub struct ScrollExtent {
    pub horizontal_extent: Coordinates,
    pub vertical_extent: Coordinates,
}
impl ScrollExtent {
    pub fn new(horizontal_extent: Coordinates, vertical_extent: Coordinates) -> Self {
        Self {
            horizontal_extent,
            vertical_extent,
        }
    }
}
impl ScrollView {
    pub fn new(pos: Position<LogicalContext>) -> Self {
        Self { position: pos }
    }
}

#[derive(Copy, Clone)]
pub struct EvaluateLocation {
    pub(crate) skip_deps: bool,
}
impl EvaluateLocation {
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let screen = ReferentialData {
            section: world.get_resource::<ViewportHandle>().unwrap().section(),
            grid: world.get_resource::<LayoutGrid>().unwrap().grid,
            points: Default::default(),
            view: Default::default(),
        };
        if let Some(stem) = world.get::<Stem>(entity).copied() {
            let stem = if let Some(s) = stem.0 {
                ReferentialData {
                    section: world
                        .get::<Section<LogicalContext>>(s)
                        .copied()
                        .unwrap_or_default(),
                    grid: world.get::<Grid>(s).copied().unwrap_or_default(),
                    points: world
                        .get::<Points<LogicalContext>>(s)
                        .copied()
                        .unwrap_or_default(),
                    view: world.get::<ScrollView>(s).copied().unwrap_or_default(),
                }
            } else {
                screen
            };
            let mut resolved = None;
            if let Some(res) = world.get::<ResolvedConfiguration>(entity) {
                if let Some((r, aw, ah)) = res.evaluate(stem, screen) {
                    // if res.aspect_ratio.is_some() => r = post_process(r, res.aspect_ratio)
                    // min/max ?
                    let r = if ah.0 {
                        // derive logical section bounds based on placer.layout.height()...
                        if let Some(tv) = world.get::<TextValue>(entity) {
                            if let Some(fs) = world.get::<FontSize>(entity) {
                                if let Some(ta) = world.get::<TextAlignment>(entity) {
                                    let mut placer = GlyphPlacer::default();
                                    let font = world.get_resource::<MonospacedFont>().unwrap();
                                    placer.layout.reset(&fontdue::layout::LayoutSettings {
                                        max_width: Some(r.width()),
                                        max_height: None,
                                        horizontal_align: ta.horizontal.into(),
                                        vertical_align: ta.vertical.into(),
                                        ..fontdue::layout::LayoutSettings::default()
                                    });
                                    placer.layout.append(
                                        &[&font.0],
                                        &fontdue::layout::TextStyle::new(tv.0.as_str(), fs.0, 0),
                                    );
                                    let derived_height = placer.layout.height();
                                    let mut fitted = Section::from(r);
                                    if ah.1 == 1 {
                                        fitted.set_height(derived_height);
                                    } else {
                                        match ah.2 {
                                            GridAspect::CenterY => {
                                                fitted.set_y(fitted.top() - derived_height / 2.0);
                                            }
                                            GridAspect::Bottom => {
                                                fitted.set_y(fitted.top() - derived_height);
                                            }
                                            _ => {}
                                        }
                                        fitted.set_height(derived_height);
                                    }
                                    fitted
                                } else {
                                    r
                                }
                            } else {
                                r
                            }
                        } else {
                            r
                        }
                    } else {
                        r
                    };
                    let old_diff = world
                        .get::<ResponsiveAnimationHook>(entity)
                        .copied()
                        .unwrap_or_default();
                    let d = if world.get::<CalcDiff>(entity).is_some() {
                        let last = world
                            .get::<Section<LogicalContext>>(entity)
                            .copied()
                            .unwrap();
                        let diff_value = last - r;
                        let value = diff_value * old_diff.percent;
                        world
                            .commands()
                            .entity(entity)
                            .insert(ResponsiveAnimationHook {
                                section: diff_value,
                                percent: old_diff.percent,
                            });
                        world.commands().entity(entity).remove::<CalcDiff>();
                        value
                    } else {
                        old_diff.value()
                    };
                    resolved = Some(r + d);
                }
            }
            if let Some(r) = resolved {
                world.commands().entity(entity).insert(r);
                if let Some(ec) = world.get::<ExtentChecker>(entity).copied() {
                    if let Some(context) = world.get::<ScrollView>(ec.0).copied() {
                        let extent = world.get::<ScrollExtent>(ec.0).copied().unwrap_or_default();
                        let mut new_horizontal = extent.horizontal_extent;
                        let mut new_vertical = extent.vertical_extent;
                        if r.right() - stem.section.left() - context.position.x()
                            > extent.horizontal_extent.vertical()
                        {
                            new_horizontal.set_vertical(
                                r.right() - stem.section.left() - context.position.x(),
                            );
                        }
                        if r.left() - stem.section.left() - context.position.x()
                            < extent.horizontal_extent.horizontal()
                        {
                            new_horizontal.set_horizontal(
                                r.left() - stem.section.left() - context.position.x(),
                            );
                        }
                        if r.top() - stem.section.top() - context.position.y()
                            < extent.vertical_extent.horizontal()
                        {
                            new_vertical.set_horizontal(
                                r.top() - stem.section.top() - context.position.y(),
                            );
                        }
                        if r.bottom() - stem.section.top() - context.position.y()
                            > extent.vertical_extent.vertical()
                        {
                            new_vertical.set_vertical(
                                r.bottom() - stem.section.top() - context.position.y(),
                            );
                        }
                        world
                            .commands()
                            .entity(ec.0)
                            .insert(ScrollExtent::new(new_horizontal, new_vertical));
                    }
                }
                world
                    .commands()
                    .entity(entity)
                    .insert(ScrollExtent::default());
                world.trigger_targets(Configure {}, entity);
            }
            let mut resolved = None;
            if let Some(res) = world.get::<ResolvedPoints>(entity) {
                if let Some(r) = res.evaluate(stem, screen) {
                    let old_diff = world
                        .get::<ResponsivePointsAnimationHook>(entity)
                        .copied()
                        .unwrap_or_default();
                    let d = if world.get::<CalcDiff>(entity).is_some() {
                        let last = world
                            .get::<Points<LogicalContext>>(entity)
                            .copied()
                            .unwrap_or_default();
                        let diff_value = last - r;
                        let value = diff_value * old_diff.percent;
                        world
                            .commands()
                            .entity(entity)
                            .insert(ResponsivePointsAnimationHook {
                                points: diff_value,
                                percent: old_diff.percent,
                            });
                        world.commands().entity(entity).remove::<CalcDiff>();
                        value
                    } else {
                        old_diff.value()
                    };
                    resolved.replace(r + d);
                }
            }
            if let Some(r) = resolved {
                world.commands().entity(entity).insert(r).insert(r.bbox());
                world.trigger_targets(Configure {}, entity);
            }
        }
        if world.get::<EvaluateLocation>(entity).unwrap().skip_deps {
            return;
        }
        if let Some(deps) = world.get::<Dependents>(entity).cloned() {
            for dep in deps.0 {
                world
                    .commands()
                    .entity(dep)
                    .insert(EvaluateLocation::recursive());
            }
        }
    }
}
impl Component for EvaluateLocation {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(EvaluateLocation::on_insert);
    }
}
impl EvaluateLocation {
    pub fn no_deps() -> Self {
        Self { skip_deps: true }
    }
    pub fn recursive() -> Self {
        Self { skip_deps: false }
    }
}
