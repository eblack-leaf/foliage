use crate::EcsExtension;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::disable::AutoDisable;
use crate::enable::AutoEnable;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::{Gap, GridAxisDescriptor, GridConfiguration};
use crate::text::monospaced::MonospacedFont;
use crate::visibility::AutoVisibility;
use crate::{
    Animate, AspectRatio, Attachment, Component, CoordinateUnit, Coordinates, Foliage, FontSize,
    Grid, Layout, Line, Logical, Points, ResolvedVisibility, Section, Stem, Tree, Resolve, View,
    Visibility, Resolved,
};
use bevy_ecs::change_detection::Res;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::Query;
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;
use std::ops::Mul;

impl Attachment for Location {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Location::update);
        foliage.define(Location::update_from_visibility);
        foliage.define(Location::stem_insert);
        foliage.enable_animation::<Location>();
    }
}
impl Animate for Location {
    fn interpolations(_start: &Self, _end: &Self) -> Interpolations {
        Interpolations::new().with(1.0, 0.0)
    }

    fn apply(&mut self, interpolations: &mut Interpolations) {
        if let Some(pct) = interpolations.read(0) {
            self.animation_percent = pct;
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
#[component(on_insert = Location::on_insert)]
#[require(Diff, CreateDiff, Resolution)]
pub struct Location {
    xs: Option<LocationDescriptor>,
    sm: Option<LocationDescriptor>,
    md: Option<LocationDescriptor>,
    lg: Option<LocationDescriptor>,
    xl: Option<LocationDescriptor>,
    pub(crate) animation_percent: CoordinateUnit,
}
impl Location {
    pub fn new() -> Self {
        Self {
            xs: None,
            sm: None,
            md: None,
            lg: None,
            xl: None,
            animation_percent: 0.0,
        }
    }
    pub fn xs<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.xs.replace((had.into(), vad.into()).into());
        self
    }
    pub fn sm<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.sm.replace((had.into(), vad.into()).into());
        self
    }
    pub fn md<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.md.replace((had.into(), vad.into()).into());
        self
    }
    pub fn lg<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.lg.replace((had.into(), vad.into()).into());
        self
    }
    pub fn xl<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.xl.replace((had.into(), vad.into()).into());
        self
    }
    fn at_least_xs(&self) -> Option<LocationDescriptor> {
        if self.xs.is_none() {
            None
        } else {
            Some(self.xs.unwrap())
        }
    }
    fn at_least_sm(&self) -> Option<LocationDescriptor> {
        if let Some(sm) = &self.sm {
            Some(*sm)
        } else {
            self.at_least_xs()
        }
    }
    fn at_least_md(&self) -> Option<LocationDescriptor> {
        if let Some(md) = &self.md {
            Some(*md)
        } else {
            self.at_least_sm()
        }
    }
    fn at_least_lg(&self) -> Option<LocationDescriptor> {
        if let Some(lg) = &self.lg {
            Some(*lg)
        } else {
            self.at_least_md()
        }
    }
    fn unset(&self) -> bool {
        self.xs.is_none()
            && self.sm.is_none()
            && self.md.is_none()
            && self.lg.is_none()
            && self.xl.is_none()
    }
    fn config(&self, layout: Layout) -> Option<LocationDescriptor> {
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
    /// Whether resolving this `Location` under `layout` actually reads the entity's own
    /// `FontSize` -- i.e. whether any of its four values is a `Letters`, the only variant
    /// `calc` answers out of `letter_dims` (which `update` sources from this entity's own
    /// `FontSize`). Everything else -- `Px`, `Percent`, `Column`/`Row`, `Anchor`,
    /// `TextContent` -- resolves without it. `ResolvedFontSize::on_insert` gates its
    /// re-resolve on this so a `FontSize` write only re-resolves the handful of entities
    /// whose geometry genuinely depends on it, rather than every entity in the tree on
    /// every layout change.
    pub(crate) fn depends_on_own_font_size(&self, layout: Layout) -> bool {
        let Some(config) = self.config(layout) else {
            return false;
        };
        [
            config.horizontal.a,
            config.horizontal.b,
            config.vertical.a,
            config.vertical.b,
        ]
        .iter()
        .any(|d| matches!(d.value, LocationValue::Letters(_)))
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world.trigger_targets(Resolve::<Location>::new(), this);
    }
    fn stem_insert(trigger: Trigger<Insert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Resolve::<Location>::new(), trigger.event_target());
    }
    fn update_from_visibility(trigger: Trigger<Resolved<Visibility>>, mut tree: Tree) {
        tree.trigger_targets(Resolve::<Location>::new(), trigger.event_target());
    }
    fn update(
        trigger: Trigger<Resolve<Location>>,
        mut tree: Tree,
        layout: Res<Layout>,
        locations: Query<&Location>,
        sections: Query<&Section<Logical>>,
        grids: Query<(&Grid, &View)>,
        stems: Query<&Stem>,
        stacks: Query<&Anchor>,
        visibilities: Query<(&ResolvedVisibility, &AutoVisibility)>,
        aspect_ratios: Query<&AspectRatio>,
        lines: Query<&Line>,
        viewport: Res<ViewportHandle>,
        create_diff_and_last: Query<(&CreateDiff, &Resolution)>,
        diffs: Query<&Diff>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
    ) {
        let this = trigger.event_target();
        if let Ok(location) = locations.get(this) {
            if location.unset() {
                // never configured -- not a positional element (a coordinator root, say);
                // nothing to resolve, so leave AutoVisibility at its default true instead of
                // treating "no config" the same as "config present but failed to resolve".
                return;
            }
            let (_, auto_vis) = visibilities.get(this).unwrap();
            tracing::trace!(entity = ?this, visible = auto_vis.visible, "location: resolve start");
            let stem = stems.get(this).unwrap();
            let (grid, view, context, stem_letters) = if let Some(id) = stem.id {
                let val = grids.get(id).unwrap_or_else(|_| {
                    panic!(
                        "{this:?}'s `Location` resolves relative to its parent {id:?}, but \
                        {id:?} has no `Grid` component -- ANY child with a `Location` needs its \
                        parent to carry one, regardless of what values that `Location` actually \
                        uses (e.g. spawn the parent `.with(Grid::new(1.col().gap(0), 1.row().gap(0)))` \
                        for a plain single-cell grid, or a real column/row split if the parent \
                        actually lays out multiple children)"
                    )
                });
                let context = sections.get(id).unwrap();
                let stem_letter_dims = if let Ok(fs) = font_sizes.get(id) {
                    font.character_block(fs.resolve(*layout))
                } else {
                    Coordinates::default()
                };
                (val.0.config(*layout), *val.1, *context, stem_letter_dims)
            } else {
                (
                    Grid::default().config(*layout),
                    View::default(),
                    viewport.section(),
                    Coordinates::default(),
                )
            };
            let aspect_ratio = aspect_ratios.get(this).ok().copied();
            let mut stack = None;
            if let Ok(s) = stacks.get(this) {
                if let Some(id) = s.id {
                    if visibilities.get(id).unwrap().0.visible() {
                        stack.replace(*sections.get(id).unwrap());
                    } else {
                        tracing::trace!(entity = ?this, target = ?id, "location: anchor target not visible, ignoring");
                    }
                } else {
                    tracing::trace!(entity = ?this, "location: has Anchor with no target");
                }
            };
            let current = *sections.get(this).unwrap();
            let letter_dims = if let Ok(fs) = font_sizes.get(this) {
                font.character_block(fs.resolve(*layout))
            } else {
                Coordinates::default()
            };
            if let Some(mut resolution) = resolve(
                *layout,
                location,
                grid,
                view,
                context,
                stack,
                current,
                letter_dims,
                aspect_ratio,
                stem_letters,
            ) {
                if !auto_vis.visible {
                    tracing::trace!(entity = ?this, "location: resolved, re-enabling");
                    tree.entity(this).insert(AutoVisibility::new(true));
                    tree.trigger_targets(AutoEnable::new(), this);
                }
                let (cd, last) = create_diff_and_last.get(this).unwrap();
                if !resolution.from_points {
                    // section
                    let diff = if cd.0 {
                        let val = last.section - resolution.section;
                        let diff = Diff({
                            let mut res = Resolution::default();
                            res.section = val;
                            res
                        });
                        tree.entity(this).insert(CreateDiff(false)).insert(diff);
                        val
                    } else {
                        diffs.get(this).unwrap().0.section
                    };
                    let anim_diff = diff * location.animation_percent;
                    resolution.section += anim_diff;
                    tracing::trace!(
                        entity = ?this,
                        view_offset = ?view.offset,
                        context = ?context,
                        resolved_section = ?resolution.section,
                        "location: resolved section"
                    );
                    tree.entity(this).insert(resolution);
                    tree.entity(this).insert(resolution.section);
                } else {
                    // points
                    let diff = if cd.0 {
                        let val = last.points - resolution.points;
                        let diff = Diff({
                            let mut res = Resolution::default();
                            res.points = val;
                            res
                        });
                        tree.entity(this).insert(CreateDiff(false)).insert(diff);
                        val
                    } else {
                        diffs.get(this).unwrap().0.points
                    };
                    resolution.points += diff * location.animation_percent;
                    let mut bbox = resolution.points.bbox();
                    if let Ok(line) = lines.get(this) {
                        let w = bbox
                            .width()
                            .max(line.weight as CoordinateUnit + 2f32 * grid.columns.gap.amount);
                        let h = bbox
                            .height()
                            .max(line.weight as CoordinateUnit + 2f32 * grid.rows.gap.amount);
                        bbox.set_width(w);
                        bbox.set_height(h);
                    }
                    resolution.section = bbox;
                    tree.entity(this)
                        .insert(resolution)
                        .insert(resolution.points)
                        .insert(resolution.section);
                }
            } else if auto_vis.visible {
                tracing::trace!(entity = ?this, "location: resolve failed, auto-disabling");
                tree.entity(this).insert(AutoVisibility::new(false));
                tree.trigger_targets(AutoDisable::new(), this);
            }
        }
    }
}
fn resolve(
    layout: Layout,
    location: &Location,
    grid: GridConfiguration,
    view: View,
    context: Section<Logical>,
    stack: Option<Section<Logical>>,
    current: Section<Logical>,
    letter_dims: Coordinates,
    aspect_ratio: Option<AspectRatio>,
    stem_letters: Coordinates,
) -> Option<Resolution> {
    if let Some(config) = location.config(layout) {
        let mut resolution = Resolution::default();
        let a = calc(
            config.horizontal.a,
            grid,
            context,
            stack,
            current,
            letter_dims,
            view,
            stem_letters,
        )?;
        let b = calc(
            config.horizontal.b,
            grid,
            context,
            stack,
            current,
            letter_dims,
            view,
            stem_letters,
        )?;
        let (pair, data) = if config.horizontal.a.designator > config.horizontal.b.designator {
            (
                (
                    config.horizontal.b.designator,
                    config.horizontal.a.designator,
                ),
                (
                    b,
                    config.horizontal.b.value.is_stack(),
                    a,
                    config.horizontal.a.value.is_stack(),
                ),
            )
        } else {
            (
                (
                    config.horizontal.a.designator,
                    config.horizontal.b.designator,
                ),
                (
                    a,
                    config.horizontal.a.value.is_stack(),
                    b,
                    config.horizontal.b.value.is_stack(),
                ),
            )
        };
        match pair {
            (Designator::X, Designator::Y) => {
                resolution.points.set_a((
                    data.0 + view.offset.left() * f32::from(data.1),
                    data.2 + view.offset.top() * f32::from(data.3),
                ));
                resolution.from_points = true;
            }
            (Designator::Left, Designator::Width) => {
                resolution
                    .section
                    .position
                    .set_left(data.0 + view.offset.left() * f32::from(data.1));
                resolution.section.area.set_width(data.2);
            }
            (Designator::Left, Designator::Right) => {
                resolution
                    .section
                    .position
                    .set_left(data.0 + view.offset.left() * f32::from(data.1));
                resolution.section.area.set_width(data.2 - data.0);
            }
            (Designator::Left, Designator::CenterX) => {
                resolution
                    .section
                    .position
                    .set_left(data.0 + view.offset.left() * f32::from(data.1));
                resolution.section.area.set_width((data.2 - data.0) * 2.0);
            }
            (Designator::Width, Designator::Right) => {
                resolution
                    .section
                    .set_left(data.2 - data.0 + view.offset.left() * f32::from(data.3));
                resolution.section.set_width(data.0);
            }
            (Designator::Width, Designator::CenterX) => {
                let left = data.2 - data.0 / 2.0;
                resolution
                    .section
                    .set_left(left + view.offset.left() * f32::from(data.3));
                resolution.section.set_width(data.0);
            }
            (Designator::Right, Designator::CenterX) => {
                let diff = data.0 - data.2;
                resolution
                    .section
                    .set_left(data.2 - diff + view.offset.left() * f32::from(data.1));
                resolution.section.set_width(diff * 2.0);
            }
            _ => panic!("unsupported combination"),
        }
        let c = calc(
            config.vertical.a,
            grid,
            context,
            stack,
            current,
            letter_dims,
            view,
            stem_letters,
        )?;
        let d = calc(
            config.vertical.b,
            grid,
            context,
            stack,
            current,
            letter_dims,
            view,
            stem_letters,
        )?;
        let (pair, data) = if config.vertical.a.designator > config.vertical.b.designator {
            (
                (config.vertical.b.designator, config.vertical.a.designator),
                (
                    d,
                    config.vertical.b.value.is_stack(),
                    c,
                    config.vertical.a.value.is_stack(),
                ),
            )
        } else {
            (
                (config.vertical.a.designator, config.vertical.b.designator),
                (
                    c,
                    config.vertical.a.value.is_stack(),
                    d,
                    config.vertical.b.value.is_stack(),
                ),
            )
        };
        match pair {
            (Designator::X, Designator::Y) => {
                resolution.points.set_b((
                    data.0 + view.offset.left() * f32::from(data.1),
                    data.2 + view.offset.top() * f32::from(data.3),
                ));
                resolution.from_points = true;
            }
            (Designator::Top, Designator::Height) => {
                resolution
                    .section
                    .position
                    .set_top(data.0 + view.offset.top() * f32::from(data.1));
                resolution.section.area.set_height(data.2);
            }
            (Designator::Top, Designator::Bottom) => {
                resolution
                    .section
                    .position
                    .set_top(data.0 + view.offset.top() * f32::from(data.1));
                resolution.section.area.set_height(data.2 - data.0);
            }
            (Designator::Top, Designator::CenterY) => {
                resolution
                    .section
                    .position
                    .set_top(data.0 + view.offset.top() * f32::from(data.1));
                resolution.section.area.set_height((data.2 - data.0) * 2.0);
            }
            (Designator::Height, Designator::Bottom) => {
                resolution
                    .section
                    .set_top(data.2 - data.0 + view.offset.top() * f32::from(data.3));
                resolution.section.set_height(data.0);
            }
            (Designator::Height, Designator::CenterY) => {
                resolution
                    .section
                    .set_top(data.2 - data.0 / 2.0 + view.offset.top() * f32::from(data.3));
                resolution.section.set_height(data.0);
            }
            (Designator::Bottom, Designator::CenterY) => {
                let diff = data.0 - data.2;
                resolution
                    .section
                    .set_top(data.2 - diff + view.offset.top() * f32::from(data.1));
                resolution.section.set_height(diff * 2.0);
            }
            _ => panic!("unsupported combination"),
        }
        resolution.section.position -= view.offset;
        for pt in resolution.points.data.iter_mut() {
            *pt -= view.offset;
        }
        let unconstrained = resolution.section;
        if let Some(a) = aspect_ratio {
            let ratio = if let Some(r) = a.config(layout) {
                r
            } else {
                1.0
            };
            if config.horizontal.a.value == LocationValue::TextContent
                && config.horizontal.a.designator == Designator::Width
                || config.horizontal.b.value == LocationValue::TextContent
                    && config.horizontal.b.designator == Designator::Width
            {
                resolution
                    .section
                    .set_width(resolution.section.height() * ratio);
            } else if config.vertical.b.value == LocationValue::TextContent
                && config.vertical.b.designator == Designator::Height
                || config.vertical.a.value == LocationValue::TextContent
                    && config.vertical.a.designator == Designator::Height
            {
                resolution
                    .section
                    .set_height(resolution.section.width() * 1f32 / ratio);
            } else {
                if let Some(constrained) = a.constrain(resolution.section, layout) {
                    resolution.section = constrained;
                }
            }
        }
        if let Some(max_w) = config.horizontal.max {
            let val = resolution.section.width().min(max_w);
            if val < unconstrained.width() {
                let diff = unconstrained.width() - val;
                match config.horizontal.justify {
                    Justify::Near => {
                        // Do nothing
                    }
                    Justify::Far => {
                        resolution
                            .section
                            .position
                            .set_left(resolution.section.position.left() + diff);
                    }
                    Justify::Center => {
                        resolution
                            .section
                            .position
                            .set_left(resolution.section.position.left() + diff / 2f32);
                    }
                }
            }
            resolution.section.set_width(val);
        }
        if let Some(min_w) = config.horizontal.min {
            resolution
                .section
                .set_width(resolution.section.width().max(min_w));
        }
        if let Some(max_h) = config.vertical.max {
            let val = resolution.section.height().min(max_h);
            if val < unconstrained.height() {
                let diff = unconstrained.height() - val;
                match config.horizontal.justify {
                    Justify::Near => {
                        // Do nothing
                    }
                    Justify::Far => {
                        resolution
                            .section
                            .position
                            .set_top(resolution.section.position.top() + diff);
                    }
                    Justify::Center => {
                        resolution
                            .section
                            .position
                            .set_top(resolution.section.position.top() + diff / 2f32);
                    }
                }
            }
            resolution.section.set_height(val);
        }
        if let Some(min_h) = config.vertical.min {
            resolution
                .section
                .set_width(resolution.section.height().max(min_h));
        }
        resolution.section.area = resolution.section.area.max((0, 0));
        Some(resolution)
    } else {
        None
    }
}
fn calc(
    desc: ValueDescriptor,
    grid: GridConfiguration,
    context: Section<Logical>,
    stack: Option<Section<Logical>>,
    current: Section<Logical>,
    letter_dims: Coordinates,
    view: View,
    stem_letters: Coordinates,
) -> Option<CoordinateUnit> {
    let calculated = match desc.value {
        LocationValue::Percent(pct) => {
            let pct_value = match desc.designator {
                Designator::Left
                | Designator::Right
                | Designator::CenterX
                | Designator::X
                | Designator::Width => {
                    pct * context.width()
                        + context.left() * f32::from(desc.designator != Designator::Width)
                }
                _ => {
                    pct * context.height()
                        + context.top() * f32::from(desc.designator != Designator::Height)
                }
            };
            Some(pct_value)
        }
        LocationValue::Px(px) => Some(match desc.designator {
            Designator::Left | Designator::X | Designator::CenterX | Designator::Right => {
                px + context.left()
            }
            Designator::Top | Designator::Y | Designator::CenterY | Designator::Bottom => {
                px + context.top()
            }
            _ => px,
        }),
        LocationValue::Column(c) => {
            let inclusive = match desc.designator {
                Designator::Right | Designator::Width => true,
                _ => false,
            };
            let column = if let LocationValue::Column(n) = grid.columns.value {
                (context.width() - grid.columns.gap.amount * (n - 1) as f32) / (n as f32)
            } else if let LocationValue::Px(px) = grid.columns.value {
                px
            } else if let LocationValue::Letters(l) = grid.columns.value {
                l as f32 * stem_letters.a()
            } else {
                return None;
            };
            let offset = match desc.designator {
                Designator::X | Designator::CenterX => 0.5 * column,
                _ => 0.0,
            };
            let val = (c as f32 - 1f32 * f32::from(!inclusive)) * column
                + (c as f32 - 1.0) * grid.columns.gap.amount;
            Some(val + offset + context.left() * f32::from(desc.designator != Designator::Width))
        }
        LocationValue::Row(r) => {
            let inclusive = match desc.designator {
                Designator::Bottom | Designator::Height => true,
                _ => false,
            };
            let row = if let LocationValue::Row(n) = grid.rows.value {
                (context.height() - grid.rows.gap.amount * (n - 1) as f32) / (n as f32)
            } else if let LocationValue::Px(px) = grid.rows.value {
                px
            } else if let LocationValue::Letters(l) = grid.rows.value {
                l as f32 * stem_letters.b()
            } else {
                return None;
            };
            let offset = match desc.designator {
                Designator::Y | Designator::CenterY => 0.5 * row,
                _ => 0.0,
            };
            let val = (r as f32 - 1f32 * f32::from(!inclusive)) * row
                + (r as f32 - 1.0) * grid.rows.gap.amount;
            Some(val + offset + context.top() * f32::from(desc.designator != Designator::Height))
        }
        LocationValue::Anchor(s, scale) => {
            if let Some(anchor) = stack {
                Some(
                    match s {
                        Designator::X => anchor.left(),
                        Designator::Y => anchor.top(),
                        Designator::Left => anchor.left(),
                        Designator::Top => anchor.top(),
                        Designator::Width => anchor.width(),
                        Designator::Height => anchor.height(),
                        Designator::Right => anchor.right(),
                        Designator::Bottom => anchor.bottom(),
                        Designator::CenterX => anchor.center().left(),
                        Designator::CenterY => anchor.center().top(),
                    } * scale,
                )
            } else {
                None
            }
        }
        LocationValue::TextContent => match desc.designator {
            Designator::Height => Some(current.height()),
            Designator::Width => Some(current.width()),
            _ => None,
        },
        LocationValue::Letters(l) => match desc.designator {
            Designator::Left
            | Designator::Right
            | Designator::CenterX
            | Designator::X
            | Designator::Width => Some(
                letter_dims.a() * l as f32
                    + context.left() * f32::from(desc.designator != Designator::Width),
            ),
            _ => Some(
                letter_dims.b() * l as f32
                    + context.top() * f32::from(desc.designator != Designator::Height),
            ),
        },
    };
    calculated.and_then(|c| Some(c + desc.adjust.amount))
}
#[derive(Copy, Clone)]
pub struct ValueDescriptor {
    designator: Designator,
    value: LocationValue,
    adjust: Adjust,
}
impl ValueDescriptor {
    pub fn new(designator: Designator, value: LocationValue) -> Self {
        Self {
            designator,
            value,
            adjust: Default::default(),
        }
    }
    pub fn with(mut self, b: ValueDescriptor) -> ConfigurationDescriptor {
        ConfigurationDescriptor::new(self, b)
    }
    pub fn adjust<P: Into<Adjust>>(mut self, adjust: P) -> Self {
        self.adjust = adjust.into();
        self
    }
}
#[derive(Copy, Clone)]
pub struct ConfigurationDescriptor {
    pub(crate) a: ValueDescriptor,
    pub(crate) b: ValueDescriptor,
    pub(crate) min: Option<CoordinateUnit>,
    pub(crate) max: Option<CoordinateUnit>,
    pub(crate) justify: Justify,
}
impl ConfigurationDescriptor {
    pub fn new(a: ValueDescriptor, b: ValueDescriptor) -> Self {
        Self {
            a,
            b,
            min: None,
            max: None,
            justify: Default::default(),
        }
    }
    pub fn min(mut self, min: CoordinateUnit) -> Self {
        self.min.replace(min);
        self
    }
    pub fn max(mut self, max: CoordinateUnit) -> Self {
        self.max.replace(max);
        self
    }
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }
}
#[derive(Copy, Clone)]
pub struct Adjust {
    pub amount: CoordinateUnit,
}
impl Default for Adjust {
    fn default() -> Self {
        Self { amount: 0.0 }
    }
}
impl From<i32> for Adjust {
    fn from(value: i32) -> Self {
        Self {
            amount: value as f32,
        }
    }
}
pub trait GridExt {
    fn pct(self) -> LocationValue;
    fn px(self) -> LocationValue;
    fn col(self) -> LocationValue;
    fn row(self) -> LocationValue;
    fn letters(self) -> LocationValue;
}
macro_rules! impl_grid_ext {
    ($i:ty) => {
        impl GridExt for $i {
            fn pct(self) -> LocationValue {
                LocationValue::Percent(self as f32 / 100.0)
            }
            fn px(self) -> LocationValue {
                LocationValue::Px(self as f32)
            }
            fn col(self) -> LocationValue {
                LocationValue::Column(self as i32)
            }
            fn row(self) -> LocationValue {
                LocationValue::Row(self as i32)
            }
            fn letters(self) -> LocationValue {
                LocationValue::Letters(self as i32)
            }
        }
    };
}
impl_grid_ext!(i32);
impl_grid_ext!(f32);
impl_grid_ext!(u32);
impl_grid_ext!(usize);
impl_grid_ext!(isize);
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocationValue {
    Percent(f32),
    Px(CoordinateUnit),
    Column(i32),
    Row(i32),
    /// The `f32` is a scale factor, identity at `1.0` -- an anchor's resolved value isn't
    /// known until it's actually looked up against the target's live section, so `Mul`
    /// can't multiply a number that doesn't exist yet the way `Percent`/`Px` do; instead it
    /// multiplies this factor in place (same pattern, different field), and `calc()`
    /// applies it once the anchor's real value is finally in hand.
    Anchor(Designator, f32),
    TextContent,
    Letters(i32),
}
impl LocationValue {
    fn is_stack(self) -> bool {
        match self {
            LocationValue::Anchor(_, _) => true,
            _ => false,
        }
    }
    pub fn as_left(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Left, self)
    }
    pub fn as_right(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Right, self)
    }
    pub fn as_top(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Top, self)
    }
    pub fn as_bottom(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Bottom, self)
    }
    pub fn as_width(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Width, self)
    }
    pub fn as_height(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Height, self)
    }
    pub fn as_center_x(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::CenterX, self)
    }
    pub fn as_center_y(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::CenterY, self)
    }
    pub fn as_x(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::X, self)
    }
    pub fn as_y(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Y, self)
    }
    pub fn gap<G: Into<Gap>>(self, g: G) -> GridAxisDescriptor {
        debug_assert!(match self {
            LocationValue::Px(_)
            | LocationValue::Percent(_)
            | LocationValue::Column(_)
            | LocationValue::Row(_)
            | LocationValue::Letters(_) => true,
            _ => false,
        });
        GridAxisDescriptor {
            value: self,
            gap: g.into(),
        }
    }
}
/// `Percent`/`Px`/`Anchor` each scale by multiplying whatever numeric field they already
/// carry in place -- same pattern used everywhere else a `Mul<f32>` is implemented in this
/// crate (`Section`/`Area`/`Position`/...), just applied to whichever field a given variant
/// actually has. The other variants (`Column`/`Row`/`Letters`/`TextContent`) have no
/// standalone numeric value that scaling could mean anything for -- rejected loudly in
/// debug builds (same convention as `gap`'s own `debug_assert!` above) rather than
/// silently doing nothing, so `Column(3) * 2.0` can't quietly look like it worked when it
/// didn't.
impl Mul<f32> for LocationValue {
    type Output = LocationValue;
    fn mul(self, rhs: f32) -> LocationValue {
        match self {
            LocationValue::Percent(p) => LocationValue::Percent(p * rhs),
            LocationValue::Px(p) => LocationValue::Px(p * rhs),
            LocationValue::Anchor(d, scale) => LocationValue::Anchor(d, scale * rhs),
            other => {
                debug_assert!(
                    false,
                    "LocationValue::{other:?} has no scalable value -- Mul<f32> isn't meaningful for it"
                );
                other
            }
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Ord, PartialOrd, Eq, Hash)]
pub enum Designator {
    X,
    Left,
    Width,
    Right,
    CenterX,
    Y,
    Top,
    Height,
    Bottom,
    CenterY,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AnchorDescriptor {}
impl AnchorDescriptor {
    pub fn left(self) -> LocationValue {
        LocationValue::Anchor(Designator::Left, 1.0)
    }
    pub fn top(self) -> LocationValue {
        LocationValue::Anchor(Designator::Top, 1.0)
    }
    pub fn width(self) -> LocationValue {
        LocationValue::Anchor(Designator::Width, 1.0)
    }
    pub fn height(self) -> LocationValue {
        LocationValue::Anchor(Designator::Height, 1.0)
    }
    pub fn center_x(self) -> LocationValue {
        LocationValue::Anchor(Designator::CenterX, 1.0)
    }
    pub fn center_y(self) -> LocationValue {
        LocationValue::Anchor(Designator::CenterY, 1.0)
    }
    pub fn right(self) -> LocationValue {
        LocationValue::Anchor(Designator::Right, 1.0)
    }
    pub fn bottom(self) -> LocationValue {
        LocationValue::Anchor(Designator::Bottom, 1.0)
    }
}
pub fn anchor() -> AnchorDescriptor {
    AnchorDescriptor {}
}
/// Sizes this dimension to the entity's own text content, measured via font shaping
/// (`fontdue`) rather than resolved against the parent -- `Text`/`TextInput`'s own
/// mechanism specifically, not a general "size to fit children" primitive (no such thing
/// exists in this framework: layout is single-pass and top-down, with no path for a
/// child's computed size to flow back into a parent's).
pub fn text_content() -> LocationValue {
    LocationValue::TextContent
}
#[derive(Copy, Clone)]
pub(crate) struct LocationDescriptor {
    pub(crate) horizontal: ConfigurationDescriptor,
    pub(crate) vertical: ConfigurationDescriptor,
}
impl From<(ConfigurationDescriptor, ConfigurationDescriptor)> for LocationDescriptor {
    fn from((horizontal, vertical): (ConfigurationDescriptor, ConfigurationDescriptor)) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct CreateDiff(pub(crate) bool);
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct Diff(pub(crate) Resolution);
#[derive(Component, Copy, Clone, Default)]
pub(crate) struct Resolution {
    pub(crate) section: Section<Logical>,
    pub(crate) points: Points<Logical>,
    pub(crate) from_points: bool,
}
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Justify {
    Near,
    Far,
    #[default]
    Center,
}
#[derive(Clone, Component, Default)]
pub struct AnchorDeps {
    pub ids: HashSet<Entity>,
}
#[derive(Component, Copy, Clone)]
#[component(on_insert = Anchor::on_insert)]
#[component(on_discard = Anchor::on_replace)]
#[derive(Default)]
pub struct Anchor {
    pub id: Option<Entity>,
}
impl Anchor {
    pub fn new(entity: Entity) -> Self {
        Self { id: Some(entity) }
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let anchor = world.get::<Anchor>(this).unwrap();
        if let Some(id) = anchor.id {
            if let Some(mut deps) = world.get_mut::<AnchorDeps>(id) {
                deps.ids.insert(this);
            } else {
                let mut anchor_deps = AnchorDeps::default();
                anchor_deps.ids.insert(this);
                world.commands().entity(id).insert(anchor_deps);
            }
        }
    }
    fn on_replace(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let anchor = world.get::<Anchor>(this).unwrap();
        if let Some(id) = anchor.id {
            if let Some(mut deps) = world.get_mut::<AnchorDeps>(id) {
                deps.ids.remove(&this);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EcsExtension, Elevation, Foliage, Leaf, Location, Logical, Section, Sprout};

    /// `Mul<f32>` on an `Anchor`-sourced value scales the *factor* it carries (identity
    /// `1.0`), applied once the anchor's real value is resolved against its live target --
    /// not something baked in at construction time, unlike `Percent`/`Px` which can just
    /// multiply their own stored number immediately. This is the actual resolution path
    /// (`anchor().width() * 0.5`, fed through real `Location` resolution against a real
    /// target), not just a check that the enum's stored factor changed.
    #[test]
    fn anchor_value_scales_by_the_target_s_own_resolved_size() {
        let mut foliage = Foliage::new();
        let target = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(60.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        let dependent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with((anchor().width() * 0.5).as_width()),
                    0.px().as_top().with((anchor().height() * 0.5).as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Anchor::new(target)),
        );
        foliage.world.flush();

        let section = *foliage.world.get::<Section<Logical>>(dependent).unwrap();
        assert_eq!(
            section.width(),
            50.0,
            "anchor().width() * 0.5 should resolve to half the target's real resolved width"
        );
        assert_eq!(
            section.height(),
            30.0,
            "anchor().height() * 0.5 should resolve to half the target's real resolved height"
        );
    }

    /// Replicates `application/src/navigator.rs`'s `shadow_box` exactly: a `(Width,
    /// Right)` pair where `Right` is `anchor().center_x()` (an *unscaled* anchor value)
    /// and `Width` is `anchor().width() * scale` (a *scaled* one), mixed in the same pair
    /// -- plus the analogous `(Top, Height)` pair for the vertical axis. Checking this
    /// directly against known numbers, rather than reasoning about it, since the app-level
    /// symptom (shadows reading as offset almost purely vertically, not diagonally) implies
    /// this exact combination doesn't resolve the way it was designed to.
    #[test]
    fn scaled_width_paired_with_unscaled_anchor_center_resolves_to_expected_offset_box() {
        let mut foliage = Foliage::new();
        let target = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    100.px().as_left().with(80.px().as_width()),
                    50.px().as_top().with(80.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        // target: X in [100, 180] (center_x = 140), Y in [50, 130] (center_y = 90)
        let dependent = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    (anchor().width() * 0.5)
                        .as_width()
                        .with(anchor().center_x().as_right()),
                    anchor()
                        .center_y()
                        .as_top()
                        .with((anchor().height() * 0.5).as_height()),
                ))
                .elevate(Elevation::up(1))
                .with(Anchor::new(target)),
        );
        foliage.world.flush();

        let section = *foliage.world.get::<Section<Logical>>(dependent).unwrap();
        assert_eq!(
            section.width(),
            40.0,
            "width should be 0.5 * target's 80px width"
        );
        assert_eq!(
            section.height(),
            40.0,
            "height should be 0.5 * target's 80px height"
        );
        assert_eq!(
            section.left(),
            100.0,
            "left should be target's center_x (140) minus this box's own width (40)"
        );
        assert_eq!(
            section.top(),
            90.0,
            "top should be exactly target's center_y (90) -- extends downward from there"
        );
    }

    /// Replicates `navigator.rs`'s `back_line`: a point-mode `Line` anchored to a target
    /// on X, spawned as a zero-length dot at the target's edge, then immediately animated
    /// (via `tree.animate`) to a longer segment at the *same* fixed row. The reported bug:
    /// the draw-in visually starts at some other row entirely while the target is still
    /// mid-animation elsewhere. Drives the real `animate::<Location>` system (not just
    /// `flush()`) across several ticks to see what the line's Y actually does.
    #[test]
    fn a_point_mode_line_animating_its_far_endpoint_keeps_its_anchored_row_throughout() {
        use crate::anim::Animation;
        use crate::anim::ease::Ease;
        use crate::{Anchor, EcsExtension, Line};
        use std::thread::sleep;
        use std::time::Duration;

        let mut foliage = Foliage::new();
        let target = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    0.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        let row = 25.0; // a fixed row, never anchored -- matches `cy` in `build_lines`
        let fwd_left = anchor().left().as_x();
        let line = foliage.world.leaf(
            Line::new(2)
                .at(Location::new().xs(
                    fwd_left.with(row.px().as_y()),
                    fwd_left.with(row.px().as_y()),
                ))
                .elevate(Elevation::up(10))
                .with(Anchor::new(target)),
        );
        foliage.world.flush();

        let initial = *foliage.world.get::<Points<Logical>>(line).unwrap();
        assert_eq!(
            initial.data[0].top(),
            row,
            "spawn point should sit at the fixed row"
        );
        assert_eq!(
            initial.data[1].top(),
            row,
            "spawn point should sit at the fixed row"
        );

        let seq = foliage.world.sequence();
        foliage.world.animate(
            Animation::new(Location::new().xs(
                fwd_left.with(row.px().as_y()),
                200.0.px().as_x().with(row.px().as_y()),
            ))
            .targeting(line)
            .during(seq)
            .start(0)
            .finish(1200)
            .eased(Ease::DECELERATE),
        );
        foliage.world.flush();

        for _ in 0..5 {
            sleep(Duration::from_millis(16));
            foliage.main.run(&mut foliage.world);
            foliage.world.flush();
            let pts = *foliage.world.get::<Points<Logical>>(line).unwrap();
            assert_eq!(
                pts.data[0].top(),
                row,
                "point a's row must stay put throughout the draw-in"
            );
            assert_eq!(
                pts.data[1].top(),
                row,
                "point b's row must stay put throughout the draw-in, not jump to some other row"
            );
        }
    }

    /// Isolates whether a *stationary* target is enough to make a line's own Y-changing
    /// animation snap immediately to its end value (ruling in/out "target also animating"
    /// as a necessary ingredient for the jump seen in the next test).
    #[test]
    fn a_line_alone_changing_its_own_row_does_not_snap_to_the_end_row_immediately() {
        use crate::anim::Animation;
        use crate::anim::ease::Ease;
        use crate::{Anchor, EcsExtension, Line};
        use std::thread::sleep;
        use std::time::Duration;

        let mut foliage = Foliage::new();
        let start_row = 30.0;
        let end_row = 300.0;
        let target = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    start_row.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        let fwd_left = anchor().left().as_x();
        let line = foliage.world.leaf(
            Line::new(2)
                .at(Location::new().xs(
                    fwd_left.with(start_row.px().as_y()),
                    200.0.px().as_x().with(start_row.px().as_y()),
                ))
                .elevate(Elevation::up(10))
                .with(Anchor::new(target)),
        );
        foliage.world.flush();

        let seq = foliage.world.sequence();
        foliage.world.animate(
            Animation::new(Location::new().xs(
                fwd_left.with(end_row.px().as_y()),
                200.0.px().as_x().with(end_row.px().as_y()),
            ))
            .targeting(line)
            .during(seq)
            .start(0)
            .finish(900)
            .eased(Ease::DECELERATE),
        );
        foliage.world.flush();

        sleep(Duration::from_millis(16));
        foliage.main.run(&mut foliage.world);
        foliage.world.flush();
        let pts = *foliage.world.get::<Points<Logical>>(line).unwrap();
        println!(
            "tick 0 (stationary target): line row = {}",
            pts.data[0].top()
        );
        assert!(
            pts.data[0].top() < 200.0,
            "line row {} should still be close to start_row (30), not already near end_row (300)",
            pts.data[0].top()
        );
    }

    /// Replicates `build_down_move`: the target (`forward`) and an already-settled line
    /// both animate their row from `start_row` to `end_row` *at the same time*, via two
    /// separate `Animation<Location>` runs (not one shared value) -- the line's target row
    /// is the same hardcoded `end_row` constant the target itself animates toward, not
    /// something read live off the target. Checks whether the line's resolved row ever
    /// jumps ahead of (or lags badly behind) the target's own live row while both are still
    /// mid-flight, which is what "line shows the resting row while the polygon is still
    /// visibly up top" would actually look like.
    #[test]
    fn a_line_and_its_target_animating_the_same_row_change_together_stay_in_sync() {
        use crate::anim::Animation;
        use crate::anim::ease::Ease;
        use crate::{Anchor, EcsExtension, Line};
        use std::thread::sleep;
        use std::time::Duration;

        let mut foliage = Foliage::new();
        let start_row = 30.0;
        let end_row = 300.0;
        let target = foliage.world.leaf(
            Leaf::sprout()
                .at(Location::new().xs(
                    0.px().as_left().with(100.px().as_width()),
                    start_row.px().as_top().with(50.px().as_height()),
                ))
                .elevate(Elevation::up(1)),
        );
        let fwd_left = anchor().left().as_x();
        let line = foliage.world.leaf(
            Line::new(2)
                .at(Location::new().xs(
                    fwd_left.with(start_row.px().as_y()),
                    200.0.px().as_x().with(start_row.px().as_y()),
                ))
                .elevate(Elevation::up(10))
                .with(Anchor::new(target)),
        );
        foliage.world.flush();

        let seq = foliage.world.sequence();
        // target rides down, row only (mirrors `box_at(END_CENTER_X, REST_CENTER_Y)`)
        foliage.world.animate(
            Animation::new(Location::new().xs(
                0.px().as_left().with(100.px().as_width()),
                end_row.px().as_top().with(50.px().as_height()),
            ))
            .targeting(target)
            .during(seq)
            .start(0)
            .finish(900)
            .eased(Ease::DECELERATE),
        );
        // line rides down too, its own separate animation to the same hardcoded end_row
        foliage.world.animate(
            Animation::new(Location::new().xs(
                fwd_left.with(end_row.px().as_y()),
                200.0.px().as_x().with(end_row.px().as_y()),
            ))
            .targeting(line)
            .during(seq)
            .start(0)
            .finish(900)
            .eased(Ease::DECELERATE),
        );
        foliage.world.flush();

        for i in 0..10 {
            sleep(Duration::from_millis(16));
            foliage.main.run(&mut foliage.world);
            foliage.world.flush();
            let target_row = foliage.world.get::<Section<Logical>>(target).unwrap().top();
            let pts = *foliage.world.get::<Points<Logical>>(line).unwrap();
            let diff = (pts.data[0].top() - target_row).abs();
            assert!(
                diff < 1.0,
                "tick {i}: line row {} should track target row {} closely, diverged by {diff}",
                pts.data[0].top(),
                target_row
            );
        }
    }
}
