use crate::ash::{ClippingContext, EnableClipping};
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
use crate::interaction::{ClickInteractionListener, Draggable};
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
    extent_checker: ScrollExtentCheck,
}
impl ScrollContext {
    pub fn new(root: Entity) -> Self {
        Self {
            clipping_context: ClippingContext::Entity(root),
            extent_checker: ScrollExtentCheck(root),
        }
    }
}
#[derive(Copy, Clone)]
pub(crate) struct ScrollExtentCheck(pub(crate) Entity);
impl ScrollExtentCheck {
    fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let ec = world.get::<ScrollExtentCheck>(entity).copied().unwrap();
        let current_total = world
            .get::<ScrollRefTotal>(ec.0)
            .copied()
            .unwrap_or_default();
        world.commands().entity(ec.0).insert(ScrollRefTotal {
            total: current_total.total + 1,
        });
    }
    fn on_remove(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let ec = world.get::<ScrollExtentCheck>(entity).copied().unwrap();
        let current_total = world
            .get::<ScrollRefTotal>(ec.0)
            .copied()
            .unwrap_or_default();
        world.commands().entity(ec.0).insert(ScrollRefTotal {
            total: (current_total.total - 1).max(0),
        });
    }
}
impl Component for ScrollExtentCheck {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(Self::on_insert);
    }
}
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
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct ScrollRefs {
    current_pass: i32,
}

impl ScrollRefs {
    fn new(i: i32) -> Self {
        Self { current_pass: i }
    }
}

#[derive(Component, Copy, Clone, Default)]
pub(crate) struct ScrollRefTotal {
    total: i32,
}
#[derive(Component, Copy, Clone, Debug, Default)]
pub(crate) struct ScrollableTag {}
#[derive(Bundle)]
pub struct Scrollable {
    tag: ScrollableTag,
    refs: ScrollRefs,
    view: ScrollView,
    extent: ScrollExtent,
    total: ScrollRefTotal,
    listener: ClickInteractionListener,
    draggable: Draggable,
    enable_clipping: EnableClipping,
}
impl Scrollable {
    pub fn new() -> Self {
        Self {
            tag: Default::default(),
            refs: Default::default(),
            view: Default::default(),
            extent: Default::default(),
            total: Default::default(),
            listener: ClickInteractionListener::new().pass_through().listen_scroll(),
            draggable: Default::default(),
            enable_clipping: Default::default(),
        }
    }
}
#[derive(Copy, Clone)]
pub struct EvaluateLocation {
    pub(crate) skip_deps: bool,
    pub(crate) skip_extent_check: bool,
}
impl EvaluateLocation {
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        let evaluation_criterion = world.get::<EvaluateLocation>(entity).copied().unwrap();
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
            if world.get::<ScrollableTag>(entity).is_some()
                && !evaluation_criterion.skip_extent_check
            {
                world
                    .commands()
                    .entity(entity)
                    .insert(ScrollRefs { current_pass: 0 });
            }
            if !evaluation_criterion.skip_extent_check {
                if let Some(ec) = world.get::<ScrollExtentCheck>(entity).copied() {
                    let current_pass = world
                        .get::<ScrollRefs>(ec.0)
                        .copied()
                        .unwrap_or_default()
                        .current_pass;
                    let total = world
                        .get::<ScrollRefTotal>(entity)
                        .copied()
                        .unwrap_or_default()
                        .total;
                    world
                        .commands()
                        .entity(ec.0)
                        .insert(ScrollRefs::new((current_pass + 1).min(total)));
                    if current_pass + 1 >= total {
                        world.commands().entity(ec.0).insert(EvaluateExtent::new());
                    }
                }
            }
        }
        if evaluation_criterion.skip_deps {
            return;
        }
        if let Some(deps) = world.get::<Dependents>(entity).cloned() {
            for dep in deps.0 {
                world.commands().entity(dep).insert(evaluation_criterion);
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct EvaluateExtent {
    is_root: bool,
}
impl EvaluateExtent {
    pub fn new() -> Self {
        Self { is_root: true }
    }
    fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        if world.get::<EvaluateExtent>(entity).unwrap().is_root {
            world
                .commands()
                .entity(entity)
                .insert(ScrollRefs::new(0))
                .insert(ScrollExtent::default());
        }
        if let Some(ec) = world.get::<ScrollExtentCheck>(entity).copied() {
            let r = world
                .get::<Section<LogicalContext>>(entity)
                .copied()
                .unwrap_or_default();
            let stem = world
                .get::<Section<LogicalContext>>(ec.0)
                .copied()
                .unwrap_or_default();
            if let Some(view) = world.get::<ScrollView>(ec.0).copied() {
                let extent = world.get::<ScrollExtent>(ec.0).copied().unwrap_or_default();
                let mut new_horizontal = extent.horizontal_extent;
                let mut new_vertical = extent.vertical_extent;
                let calc = r.right() - stem.left() + view.position.x();
                if calc > extent.horizontal_extent.vertical() {
                    new_horizontal.set_vertical(calc);
                }
                let calc = r.left() - stem.left() + view.position.x();
                if calc < extent.horizontal_extent.horizontal() {
                    new_horizontal.set_horizontal(calc);
                }
                let calc = r.top() - stem.top() + view.position.y();
                if calc < extent.vertical_extent.horizontal() {
                    new_vertical.set_horizontal(calc);
                }
                let calc = r.bottom() - stem.top() + view.position.y();
                if calc > extent.vertical_extent.vertical() {
                    new_vertical.set_vertical(calc);
                }
                world
                    .commands()
                    .entity(ec.0)
                    .insert(ScrollExtent::new(new_horizontal, new_vertical));
                let current_pass = world
                    .get::<ScrollRefs>(ec.0)
                    .copied()
                    .unwrap_or_default()
                    .current_pass;
                let total = world
                    .get::<ScrollRefTotal>(ec.0)
                    .copied()
                    .unwrap_or_default()
                    .total;
                let ending_ref_count = (current_pass + 1).min(total);
                world
                    .commands()
                    .entity(ec.0)
                    .insert(ScrollRefs::new(ending_ref_count));
                if current_pass + 1 >= total {
                    // overscroll + evaluate-location if adjust w/ skip_extent_check()
                    let mut new_view = Option::<Position<LogicalContext>>::None;
                    if view.position.x() + stem.area.width() > new_horizontal.vertical() {
                        // right overscroll
                        new_view.replace(Position::new((
                            (new_horizontal.vertical() - stem.area.width())
                                .max(new_horizontal.horizontal()),
                            view.position.y(),
                        )));
                    }
                    if view.position.x() < new_horizontal.horizontal() {
                        // left overscroll
                        new_view.replace(Position::new((
                            new_horizontal.horizontal(),
                            view.position.y(),
                        )));
                    }
                    if view.position.y() + stem.area.height() > new_vertical.vertical() {
                        // bottom overscroll
                        let calc_y = (new_vertical.vertical() - stem.area.height())
                            .max(new_vertical.horizontal());
                        let new = if let Some(n) = new_view {
                            Position::new((n.x(), calc_y))
                        } else {
                            Position::new((view.position.x(), calc_y))
                        };
                        new_view.replace(new);
                    }
                    if view.position.y() < new_vertical.horizontal() {
                        // top overscroll
                        let new = if let Some(n) = new_view {
                            Position::new((n.x(), new_vertical.horizontal()))
                        } else {
                            Position::new((view.position.x(), new_vertical.horizontal()))
                        };
                        new_view.replace(new);
                    }
                    if let Some(n) = new_view {
                        world
                            .commands()
                            .entity(ec.0)
                            .insert(ScrollView::new(n))
                            .insert(EvaluateLocation::skip_extent_check());
                    }
                }
            }
        }
        for d in world
            .get::<Dependents>(entity)
            .cloned()
            .unwrap_or_default()
            .0
        {
            world
                .commands()
                .entity(d)
                .insert(EvaluateExtent { is_root: false });
        }
    }
}
impl Component for EvaluateExtent {
    const STORAGE_TYPE: StorageType = SparseSet;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(Self::on_insert);
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
        Self {
            skip_deps: true,
            skip_extent_check: false,
        }
    }
    pub fn recursive() -> Self {
        Self {
            skip_deps: false,
            skip_extent_check: false,
        }
    }
    pub(crate) fn skip_extent_check() -> Self {
        Self {
            skip_deps: false,
            skip_extent_check: true,
        }
    }
}
