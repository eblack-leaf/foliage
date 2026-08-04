use crate::AsTree;
use crate::Trigger;
use crate::anim::interpolation::Interpolations;
use crate::disable::AutoDisable;
use crate::enable::AutoEnable;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::{Gap, GridAxisDescriptor, GridConfiguration, Short};
use crate::node::SpawnedAt;
use crate::text::monospaced::FontContext;
use crate::visibility::AutoVisibility;
use crate::{
    Animate, AspectRatio, Attachment, Component, CoordinateUnit, Coordinates, Foliage, Grid,
    Layout, LayoutSection, Line, Logical, Parent, Points, Position, Resolve, Resolved,
    ResolvedVisibility, Section, Tree, View, Visibility,
};
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::prelude::{ParamSet, Query};
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
/// Where an entity sits and how big it is, expressed per breakpoint and resolved against
/// its parent.
///
/// Each breakpoint takes two [`ConfigurationDescriptor`]s -- horizontal, then vertical --
/// and each of those pairs two [`ValueDescriptor`]s joined by
/// [`.with()`](ValueDescriptor::with). Two values per axis fully pin it: a left and a
/// width, a left and a right, a center and a width.
///
/// ```ignore
/// Location::new().xs(
///     20.pct().as_left().with(60.pct().as_width()),
///     1.row().as_top().with(1.row().as_bottom()),
/// )
/// ```
///
/// Values come from [`GridExt`] (`pct`, `px`, `col`, `row`, `letters`),
/// [`anchor()`](anchor) for another entity's geometry, or
/// [`text_content()`](text_content) for a text run's own measured size. Only `xs` is
/// required; larger breakpoints fall back to the nearest smaller one set.
///
/// Resolves to a [`Section`], or to [`Points`] when built from `as_x`/`as_y` -- see
/// `Designator` (engine-internal). Animating a `Location` tweens between the current
/// resolved box and the new one.
#[derive(Component, Copy, Clone, Default)]
#[component(on_insert = Location::on_insert)]
#[require(Diff, CreateDiff, Resolution)]
pub struct Location {
    xs: Option<LocationDescriptor>,
    sm: Option<LocationDescriptor>,
    md: Option<LocationDescriptor>,
    lg: Option<LocationDescriptor>,
    xl: Option<LocationDescriptor>,
    short: Option<LocationDescriptor>,
    pub(crate) animation_percent: CoordinateUnit,
}
impl Location {
    /// An unconfigured `Location`. Set at least [`xs`](Self::xs) -- an entity with none is
    /// treated as non-positional and skipped by layout rather than failing to resolve.
    pub fn new() -> Self {
        Self {
            xs: None,
            sm: None,
            md: None,
            lg: None,
            xl: None,
            short: None,
            animation_percent: 0.0,
        }
    }
    /// Overrides every width breakpoint while the viewport is vertically cramped (see
    /// [`Short`]). Takes horizontal then vertical.
    ///
    /// The escape hatch for the case width alone gets wrong: a phone held landscape is
    /// `Md`-wide with almost no height, so a `Location` that spends vertical room at `md`
    /// runs off the bottom. Set this and that one value switches; leave it off and the
    /// entity resolves purely by width exactly as before.
    ///
    /// Off by default and never inferred -- the width answer is right far more often than
    /// not (a tagline moving *beside* its subject at `md` is the correct landscape layout
    /// precisely because it stops spending height), so this is opt-in per value rather
    /// than a blanket demotion of the whole breakpoint.
    pub fn short<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.short.replace((had.into(), vad.into()).into());
        self
    }
    /// The base configuration, used at every breakpoint without its own. Takes horizontal then vertical.
    pub fn xs<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.xs.replace((had.into(), vad.into()).into());
        self
    }
    /// Overrides the configuration from the `sm` breakpoint up. Takes horizontal then vertical.
    pub fn sm<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.sm.replace((had.into(), vad.into()).into());
        self
    }
    /// Overrides the configuration from the `md` breakpoint up. Takes horizontal then vertical.
    pub fn md<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.md.replace((had.into(), vad.into()).into());
        self
    }
    /// Overrides the configuration from the `lg` breakpoint up. Takes horizontal then vertical.
    pub fn lg<HAD: Into<ConfigurationDescriptor>, VAD: Into<ConfigurationDescriptor>>(
        mut self,
        had: HAD,
        vad: VAD,
    ) -> Self {
        self.lg.replace((had.into(), vad.into()).into());
        self
    }
    /// Overrides the configuration at the `xl` breakpoint. Takes horizontal then vertical.
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
            && self.short.is_none()
    }
    fn config(&self, layout: Layout, short: Short) -> Option<LocationDescriptor> {
        // One extra link on the front of the same chain, not a second dimension: `short`
        // wins when it is both relevant and set, and otherwise this is the width lookup
        // untouched.
        if short == Short::Yes
            && let Some(s) = &self.short
        {
            return Some(*s);
        }
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
    pub(crate) fn depends_on_own_font_size(&self, layout: Layout, short: Short) -> bool {
        let Some(config) = self.config(layout, short) else {
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
        world.tree().send_to(Resolve::<Location>::new(), this);
    }
    fn stem_insert(trigger: Trigger<Insert, Parent>, mut tree: Tree) {
        tree.send_to(Resolve::<Location>::new(), trigger.event_target());
    }
    fn update_from_visibility(trigger: Trigger<Resolved<Visibility>>, mut tree: Tree) {
        tree.send_to(Resolve::<Location>::new(), trigger.event_target());
    }
    fn update(
        trigger: Trigger<Resolve<Location>>,
        mut tree: Tree,
        layout: Res<Layout>,
        locations: Query<(&Location, Option<&SpawnedAt>)>,
        sections: Query<&Section<Logical>>,
        layout_sections: Query<&LayoutSection>,
        mut grids: ParamSet<(Query<(&Grid, &View)>, Query<&mut View>)>,
        stems: Query<&Parent>,
        stacks: Query<&Anchor>,
        visibilities: Query<(&ResolvedVisibility, &AutoVisibility)>,
        aspect_ratios: Query<&AspectRatio>,
        lines: Query<&Line>,
        viewport: Res<ViewportHandle>,
        create_diff_and_last: Query<(&CreateDiff, &Resolution)>,
        diffs: Query<&Diff>,
        fonts: FontContext,
    ) {
        let this = trigger.event_target();
        // Ahead of everything else, and deliberately not inside the `unset` guard below: an
        // entity that never resolves a `Location` of its own still sits in the middle of the
        // view chain, and its children read their accumulated offset off it. Skipping it
        // here would hand a whole subtree a zero offset -- placing it as though nothing
        // above it were scrolled at all. A non-panicking lookup, since an entity with no
        // layout of its own is under no obligation to have a `Grid`-carrying parent.
        let inherited = stems
            .get(this)
            .ok()
            .and_then(|s| s.id)
            .and_then(|p| grids.p0().get(p).ok().map(|(_, v)| v.accumulated_offset))
            .unwrap_or_default();
        if let Ok(mut view) = grids.p1().get_mut(this) {
            // `snapped_offset`, matching `propagate_offsets` -- the two paths write the same
            // entity's `Section`, so a resolve landing mid-scroll has to agree with the walk
            // on where the subtree sits, down to the pixel.
            view.accumulated_offset = inherited + view.snapped_offset;
        }
        if let Ok((location, spawned_at)) = locations.get(this) {
            if location.unset() {
                // never configured -- not a positional element (a coordinator root, say);
                // nothing to resolve, so leave AutoVisibility at its default true instead of
                // treating "no config" the same as "config present but failed to resolve".
                return;
            }
            let (_, auto_vis) = visibilities.get(this).unwrap();
            tracing::trace!(entity = ?this, visible = auto_vis.visible, "location: resolve start");
            let stem = stems.get(this).unwrap();
            // `accumulated` is what this entity subtracts to go from layout space to screen
            // space: every scroll offset between it and the root, which is exactly what its
            // parent's `View` already carries for its children.
            let (grid, accumulated, context, stem_letters) =
                if let Some(id) = stem.id {
                    let val = grids.p0().get(id).map(|(g, v)| (*g, *v)).unwrap_or_else(|_| {
                    // The entity ids alone are unusable from an app -- point at the
                    // `branch`/`leaf` call that spawned the child instead, since that call
                    // names the very parent that needs the `Grid`.
                    let at = spawned_at
                        .map(|s| format!("\n  spawned at {}", s.0))
                        .unwrap_or_default();
                    panic!(
                        "a `Location` resolves relative to its parent, but that parent has no \
                        `Grid` component -- ANY child with a `Location` needs its parent to \
                        carry one, regardless of what values that `Location` actually uses.\
                        {at}\n  fix: give the parent `.with(Grid::new(1.col().gap(0), \
                        1.row().gap(0)))` for a plain single-cell grid, or a real column/row \
                        split if it actually lays out multiple children.\n  (child {this:?}, \
                        parent {id:?})"
                    )
                });
                    // layout space on both sides: a child is placed relative to where the layout
                    // put its parent, not to where a scroll currently shows it
                    let context = layout_sections.get(id).unwrap().0;
                    // the stem's own font/size -- `.letters()` measures against the cell the
                    // parent lays out in
                    let stem_letter_dims = fonts.character_block(id, *layout).unwrap_or_default();
                    (
                        val.0.config(*layout),
                        val.1.accumulated_offset,
                        context,
                        stem_letter_dims,
                    )
                } else {
                    (
                        Grid::default().config(*layout),
                        Position::default(),
                        viewport.section(),
                        Coordinates::default(),
                    )
                };
            let aspect_ratio = aspect_ratios.get(this).ok().copied();
            let mut stack = None;
            if let Ok(s) = stacks.get(this) {
                if let Some(id) = s.id {
                    if visibilities.get(id).unwrap().0.visible() {
                        // An anchor target is read in screen space, since it can sit
                        // anywhere in the tree -- under a different view, or under none.
                        // Adding back what this entity subtracts states the target in *this*
                        // entity's own layout space, which is the only space the resolve
                        // below works in. Two entities in the same view cancel out exactly;
                        // across a scroll boundary this is the real distance between them.
                        let mut target = *sections.get(id).unwrap();
                        target.position += accumulated;
                        stack.replace(target);
                    } else {
                        tracing::trace!(entity = ?this, target = ?id, "location: anchor target not visible, ignoring");
                    }
                } else {
                    tracing::trace!(entity = ?this, "location: has Anchor with no target");
                }
            };
            let current = layout_sections.get(this).unwrap().0;
            let letter_dims = fonts.character_block(this, *layout).unwrap_or_default();
            if let Some(mut resolution) = resolve(
                *layout,
                // the same `Short` `FontContext` already carries, rather than a second copy
                // of it as its own param -- an observer's parameter list is a bounded
                // resource, and this one is close to spending it
                *fonts.short,
                location,
                grid,
                context,
                stack,
                current,
                letter_dims,
                aspect_ratio,
                stem_letters,
            ) {
                if !auto_vis.visible {
                    tracing::trace!(entity = ?this, "location: resolved, re-enabling");
                    tree.write_to(this, AutoVisibility::new(true));
                    tree.send_to(AutoEnable::new(), this);
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
                        tree.write_to(this, (CreateDiff(false), diff));
                        val
                    } else {
                        diffs.get(this).unwrap().0.section
                    };
                    let anim_diff = diff * location.animation_percent;
                    resolution.section += anim_diff;
                    let mut screen = resolution.section;
                    screen.position -= accumulated;
                    tracing::trace!(
                        entity = ?this,
                        accumulated = ?accumulated,
                        context = ?context,
                        resolved_section = ?resolution.section,
                        screen_section = ?screen,
                        "location: resolved section"
                    );
                    // `LayoutSection` last: its insert is what re-resolves this entity's
                    // children and everything anchored to it, and both read values that the
                    // other two inserts carry -- a child reads this `LayoutSection` as its
                    // context, an anchored entity reads this `Section`. Cascading before
                    // either has landed resolves them against the previous box.
                    tree.write_to(this, resolution);
                    tree.write_to(this, screen);
                    tree.write_to(this, LayoutSection(resolution.section));
                } else {
                    // points
                    let diff = if cd.0 {
                        let val = last.points - resolution.points;
                        let diff = Diff({
                            let mut res = Resolution::default();
                            res.points = val;
                            res
                        });
                        tree.write_to(this, (CreateDiff(false), diff));
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
                    let mut screen = resolution.section;
                    screen.position -= accumulated;
                    let mut screen_points = resolution.points;
                    for pt in screen_points.data.iter_mut() {
                        *pt -= accumulated;
                    }
                    tree.write_to(
                        this,
                        (
                            resolution,
                            screen_points,
                            screen,
                            LayoutSection(resolution.section),
                        ),
                    );
                }
            } else if auto_vis.visible {
                tracing::trace!(entity = ?this, "location: resolve failed, auto-disabling");
                tree.write_to(this, AutoVisibility::new(false));
                tree.send_to(AutoDisable::new(), this);
            }
        }
    }
}
/// Resolves a `Location` entirely in layout space -- no scroll offset appears anywhere in
/// here. What an ancestor's scroll does to the result is a translation applied afterwards,
/// by the one caller that knows the accumulated total.
fn resolve(
    layout: Layout,
    short: Short,
    location: &Location,
    grid: GridConfiguration,
    context: Section<Logical>,
    stack: Option<Section<Logical>>,
    current: Section<Logical>,
    letter_dims: Coordinates,
    aspect_ratio: Option<AspectRatio>,
    stem_letters: Coordinates,
) -> Option<Resolution> {
    if let Some(config) = location.config(layout, short) {
        let mut resolution = Resolution::default();
        let a = calc(
            config.horizontal.a,
            grid,
            context,
            stack,
            current,
            letter_dims,
            stem_letters,
        )?;
        let b = calc(
            config.horizontal.b,
            grid,
            context,
            stack,
            current,
            letter_dims,
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
                resolution.points.set_a((data.0, data.2));
                resolution.from_points = true;
            }
            (Designator::Left, Designator::Width) => {
                resolution.section.position.set_left(data.0);
                resolution.section.area.set_width(data.2);
            }
            (Designator::Left, Designator::Right) => {
                resolution.section.position.set_left(data.0);
                resolution.section.area.set_width(data.2 - data.0);
            }
            (Designator::Left, Designator::CenterX) => {
                resolution.section.position.set_left(data.0);
                resolution.section.area.set_width((data.2 - data.0) * 2.0);
            }
            (Designator::Width, Designator::Right) => {
                resolution.section.set_left(data.2 - data.0);
                resolution.section.set_width(data.0);
            }
            (Designator::Width, Designator::CenterX) => {
                resolution.section.set_left(data.2 - data.0 / 2.0);
                resolution.section.set_width(data.0);
            }
            (Designator::Right, Designator::CenterX) => {
                let diff = data.0 - data.2;
                resolution.section.set_left(data.2 - diff);
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
            stem_letters,
        )?;
        let d = calc(
            config.vertical.b,
            grid,
            context,
            stack,
            current,
            letter_dims,
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
                resolution.points.set_b((data.0, data.2));
                resolution.from_points = true;
            }
            (Designator::Top, Designator::Height) => {
                resolution.section.position.set_top(data.0);
                resolution.section.area.set_height(data.2);
            }
            (Designator::Top, Designator::Bottom) => {
                resolution.section.position.set_top(data.0);
                resolution.section.area.set_height(data.2 - data.0);
            }
            (Designator::Top, Designator::CenterY) => {
                resolution.section.position.set_top(data.0);
                resolution.section.area.set_height((data.2 - data.0) * 2.0);
            }
            (Designator::Height, Designator::Bottom) => {
                resolution.section.set_top(data.2 - data.0);
                resolution.section.set_height(data.0);
            }
            (Designator::Height, Designator::CenterY) => {
                resolution.section.set_top(data.2 - data.0 / 2.0);
                resolution.section.set_height(data.0);
            }
            (Designator::Bottom, Designator::CenterY) => {
                let diff = data.0 - data.2;
                resolution.section.set_top(data.2 - diff);
                resolution.section.set_height(diff * 2.0);
            }
            _ => panic!("unsupported combination"),
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
/// One resolved edge or extent: a [`LocationValue`] plus the `Designator` (engine-internal)
/// saying which part of the box it describes, and an optional pixel [`Adjust`].
#[derive(Copy, Clone)]
pub struct ValueDescriptor {
    designator: Designator,
    value: LocationValue,
    adjust: Adjust,
}
impl ValueDescriptor {
    /// Usually written as `20.pct().as_left()` rather than called directly.
    pub fn new(designator: Designator, value: LocationValue) -> Self {
        Self {
            designator,
            value,
            adjust: Default::default(),
        }
    }
    /// Pairs this value with the other one pinning the same axis -- a left with a width,
    /// a top with a bottom.
    pub fn with(self, b: ValueDescriptor) -> ConfigurationDescriptor {
        ConfigurationDescriptor::new(self, b)
    }
    /// Shifts this value by a fixed number of logical pixels after it resolves -- for
    /// nudging off a grid line without abandoning the grid.
    pub fn adjust<P: Into<Adjust>>(mut self, adjust: P) -> Self {
        self.adjust = adjust.into();
        self
    }
}
/// One axis of a [`Location`]: the two [`ValueDescriptor`]s that pin it, plus optional
/// size limits and a [`Justify`] for how to use any slack.
#[derive(Copy, Clone)]
pub struct ConfigurationDescriptor {
    pub(crate) a: ValueDescriptor,
    pub(crate) b: ValueDescriptor,
    pub(crate) min: Option<CoordinateUnit>,
    pub(crate) max: Option<CoordinateUnit>,
    pub(crate) justify: Justify,
}
impl ConfigurationDescriptor {
    /// Usually produced by [`ValueDescriptor::with`] rather than called directly.
    pub fn new(a: ValueDescriptor, b: ValueDescriptor) -> Self {
        Self {
            a,
            b,
            min: None,
            max: None,
            justify: Default::default(),
        }
    }
    /// Floor on this axis's resolved extent, in logical pixels.
    pub fn min(mut self, min: CoordinateUnit) -> Self {
        self.min.replace(min);
        self
    }
    /// Ceiling on this axis's resolved extent, in logical pixels. Where the box lands
    /// within the leftover room is [`justify`](Self::justify)'s decision.
    pub fn max(mut self, max: CoordinateUnit) -> Self {
        self.max.replace(max);
        self
    }
    /// Where the box sits within space a [`max`](Self::max) left over. Centered by
    /// default.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }
}
/// A fixed logical-pixel offset applied to a value after it resolves.
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
/// The numeric literal vocabulary for layout: `20.pct()`, `8.px()`, `2.col()`.
/// Implemented for every integer and float type, so a bare literal works.
pub trait GridExt {
    /// A percentage of the parent's own box on this axis. `100.pct()` is the full extent.
    fn pct(self) -> LocationValue;
    /// A fixed number of logical pixels.
    fn px(self) -> LocationValue;
    /// A 1-based column line in the parent's [`Grid`]. `1.col().as_left()` is the left
    /// edge of column one; as a right edge the same index is inclusive, so
    /// `1.col().as_left().with(1.col().as_right())` is exactly one column wide.
    fn col(self) -> LocationValue;
    /// A 1-based row line in the parent's [`Grid`], inclusive as a bottom edge just as
    /// [`col`](Self::col) is as a right edge.
    fn row(self) -> LocationValue;
    /// A multiple of one character's advance at the entity's own
    /// [`FontSize`](crate::FontSize) -- for sizing a box to the text it will hold.
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
/// A single unresolved number in a [`Location`], before it is told which edge it
/// describes. Built through [`GridExt`], [`anchor()`](anchor) or
/// [`text_content()`](text_content), then given a `Designator` (engine-internal) by
/// `as_left()` and friends.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocationValue {
    /// Fraction of the parent's own extent on this axis.
    Percent(f32),
    /// Fixed logical pixels.
    Px(CoordinateUnit),
    /// 1-based column line in the parent's grid.
    Column(i32),
    /// 1-based row line in the parent's grid.
    Row(i32),
    /// The `f32` is a scale factor, identity at `1.0` -- an anchor's resolved value isn't
    /// known until it's actually looked up against the target's live section, so `Mul`
    /// can't multiply a number that doesn't exist yet the way `Percent`/`Px` do; instead it
    /// multiplies this factor in place (same pattern, different field), and `calc()`
    /// applies it once the anchor's real value is finally in hand.
    Anchor(Designator, f32),
    /// The entity's own measured text extent -- see [`text_content()`](text_content).
    TextContent,
    /// Multiple of one character's advance at this entity's own font size.
    Letters(i32),
}
impl LocationValue {
    fn is_stack(self) -> bool {
        match self {
            LocationValue::Anchor(_, _) => true,
            _ => false,
        }
    }
    /// Uses this value as the box's left edge.
    pub fn as_left(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Left, self)
    }
    /// Uses this value as the box's right edge. Grid indices are inclusive here.
    pub fn as_right(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Right, self)
    }
    /// Uses this value as the box's top edge.
    pub fn as_top(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Top, self)
    }
    /// Uses this value as the box's bottom edge. Grid indices are inclusive here.
    pub fn as_bottom(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Bottom, self)
    }
    /// Uses this value as the box's width.
    pub fn as_width(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Width, self)
    }
    /// Uses this value as the box's height.
    pub fn as_height(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Height, self)
    }
    /// Centers the box horizontally on this value.
    pub fn as_center_x(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::CenterX, self)
    }
    /// Centers the box vertically on this value.
    pub fn as_center_y(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::CenterY, self)
    }
    /// Uses this value as a point's X, putting the entity in point mode: it resolves to
    /// [`Points`] rather than a box, and its `Section` becomes their bounding rectangle.
    /// For [`Line`] and [`Polygon`](crate::Polygon).
    pub fn as_x(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::X, self)
    }
    /// Uses this value as a point's Y -- see [`as_x`](Self::as_x).
    pub fn as_y(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Y, self)
    }
    /// Turns this value into a [`Grid`] axis with `g` logical pixels between tracks.
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
/// Which part of a box a [`ValueDescriptor`] describes. Set by the `as_*` methods rather
/// than named directly.
///
/// `X`/`Y` are the odd pair: they put the entity in point mode, resolving to [`Points`]
/// instead of a box.
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
/// Picks which part of an [`Anchor`] target's geometry to read. Produced by
/// [`anchor()`](anchor); each method yields a [`LocationValue`] that then takes its own
/// `as_*`, so the source and destination edges need not match --
/// `anchor().bottom().as_top()` places this box directly below its target.
pub struct AnchorDescriptor {}
impl AnchorDescriptor {
    /// The target's left edge.
    pub fn left(self) -> LocationValue {
        LocationValue::Anchor(Designator::Left, 1.0)
    }
    /// The target's top edge.
    pub fn top(self) -> LocationValue {
        LocationValue::Anchor(Designator::Top, 1.0)
    }
    /// The target's width.
    pub fn width(self) -> LocationValue {
        LocationValue::Anchor(Designator::Width, 1.0)
    }
    /// The target's height.
    pub fn height(self) -> LocationValue {
        LocationValue::Anchor(Designator::Height, 1.0)
    }
    /// The target's horizontal center.
    pub fn center_x(self) -> LocationValue {
        LocationValue::Anchor(Designator::CenterX, 1.0)
    }
    /// The target's vertical center.
    pub fn center_y(self) -> LocationValue {
        LocationValue::Anchor(Designator::CenterY, 1.0)
    }
    /// The target's right edge.
    pub fn right(self) -> LocationValue {
        LocationValue::Anchor(Designator::Right, 1.0)
    }
    /// The target's bottom edge.
    pub fn bottom(self) -> LocationValue {
        LocationValue::Anchor(Designator::Bottom, 1.0)
    }
}
/// Reads geometry from the entity named by this one's [`Anchor`] component, letting a
/// box position itself against another that is not its parent.
///
/// The anchor target must be visible and resolved; if it is not, this entity's own
/// resolve fails and it is auto-hidden until the target comes back.
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
/// Where a box sits inside space left over by a [`ConfigurationDescriptor::max`].
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Justify {
    /// Against the leading edge -- left or top.
    Near,
    /// Against the trailing edge -- right or bottom.
    Far,
    /// Centered in the leftover space.
    #[default]
    Center,
}
/// The entities anchored *to* this one, maintained by the engine so a change to this
/// entity's own `Section` can re-resolve everything positioned against it.
#[derive(Clone, Component, Default)]
pub struct AnchorDeps {
    pub ids: HashSet<Entity>,
}
#[derive(Component, Copy, Clone)]
#[component(on_insert = Anchor::on_insert)]
#[component(on_discard = Anchor::on_replace)]
/// Names the entity this one's [`anchor()`](anchor) values resolve against.
///
/// Independent of parenting: an anchor target can be anywhere in the tree, and does not
/// affect who owns or clips this entity. A box with `anchor()` values in its `Location`
/// and no `Anchor` cannot resolve.
#[derive(Default)]
pub struct Anchor {
    pub id: Option<Entity>,
}
impl Anchor {
    /// Anchors to `entity`.
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
                world.tree().write_to(id, anchor_deps);
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
