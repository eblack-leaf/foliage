use crate::anim::interpolation::Interpolations;
use crate::disable::AutoDisable;
use crate::enable::AutoEnable;
use crate::ginkgo::viewport::ViewportHandle;
use crate::grid::{Gap, GridAxisDescriptor, GridConfiguration};
use crate::text::monospaced::MonospacedFont;
use crate::visibility::AutoVisibility;
use crate::{
    Animate, AspectRatio, Attachment, Component, CoordinateUnit, Coordinates, Foliage, FontSize,
    Grid, Layout, Line, Logical, Points, ResolvedVisibility, Section, Stem, Tree, Update, View,
    Visibility, Write,
};
use bevy_ecs::change_detection::Res;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::Trigger;
use bevy_ecs::prelude::{OnInsert, Query};
use bevy_ecs::world::DeferredWorld;
use std::collections::HashSet;

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
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Location>::new(), this);
    }
    fn stem_insert(trigger: Trigger<OnInsert, Stem>, mut tree: Tree) {
        tree.trigger_targets(Update::<Location>::new(), trigger.entity());
    }
    fn update_from_visibility(trigger: Trigger<Write<Visibility>>, mut tree: Tree) {
        tree.trigger_targets(Update::<Location>::new(), trigger.entity());
    }
    fn update(
        trigger: Trigger<Update<Location>>,
        mut tree: Tree,
        layout: Res<Layout>,
        locations: Query<&Location>,
        sections: Query<&Section<Logical>>,
        grids: Query<(&Grid, &View)>,
        stems: Query<&Stem>,
        stacks: Query<&Stack>,
        visibilities: Query<(&ResolvedVisibility, &AutoVisibility)>,
        aspect_ratios: Query<&AspectRatio>,
        lines: Query<&Line>,
        viewport: Res<ViewportHandle>,
        create_diff_and_last: Query<(&CreateDiff, &Resolution)>,
        diffs: Query<&Diff>,
        font: Res<MonospacedFont>,
        font_sizes: Query<&FontSize>,
    ) {
        let this = trigger.entity();
        if let Ok(location) = locations.get(this) {
            let (_, auto_vis) = visibilities.get(this).unwrap();
            let stem = stems.get(this).unwrap();
            let (grid, view, context, stem_letters) = if let Some(id) = stem.id {
                let val = grids.get(id).unwrap();
                let context = sections.get(id).unwrap();
                let stem_letter_dims = if let Ok(fs) = font_sizes.get(id) {
                    font.character_block(fs.resolve(*layout).value)
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
                    }
                }
            };
            let current = *sections.get(this).unwrap();
            let letter_dims = if let Ok(fs) = font_sizes.get(this) {
                let f = fs.resolve(*layout);
                font.character_block(f.value)
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
            if config.horizontal.a.value == LocationValue::Auto
                && config.horizontal.a.designator == Designator::Width
                || config.horizontal.b.value == LocationValue::Auto
                    && config.horizontal.b.designator == Designator::Width
            {
                resolution
                    .section
                    .set_width(resolution.section.height() * ratio);
            } else if config.vertical.b.value == LocationValue::Auto
                && config.vertical.b.designator == Designator::Height
                || config.vertical.a.value == LocationValue::Auto
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
                (context.width() - grid.columns.gap.amount * (n + 1) as f32) / (n as f32)
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
                + c as f32 * grid.columns.gap.amount;
            Some(val + offset + context.left() * f32::from(desc.designator != Designator::Width))
        }
        LocationValue::Row(r) => {
            let inclusive = match desc.designator {
                Designator::Bottom | Designator::Height => true,
                _ => false,
            };
            let row = if let LocationValue::Row(n) = grid.rows.value {
                (context.height() - grid.rows.gap.amount * (n + 1) as f32) / (n as f32)
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
            let val =
                (r as f32 - 1f32 * f32::from(!inclusive)) * row + r as f32 * grid.rows.gap.amount;
            Some(val + offset + context.top() * f32::from(desc.designator != Designator::Height))
        }
        LocationValue::Stack(s) => {
            if let Some(stack) = stack {
                Some(match s {
                    Designator::X => stack.left(),
                    Designator::Y => stack.top(),
                    Designator::Left => stack.left(),
                    Designator::Top => stack.top(),
                    Designator::Width => stack.width(),
                    Designator::Height => stack.height(),
                    Designator::Right => stack.right(),
                    Designator::Bottom => stack.bottom(),
                    Designator::CenterX => stack.center().left(),
                    Designator::CenterY => stack.center().top(),
                })
            } else {
                None
            }
        }
        LocationValue::Auto => match desc.designator {
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
    Stack(Designator),
    Auto,
    Letters(i32),
}
impl LocationValue {
    fn is_stack(self) -> bool {
        match self {
            LocationValue::Stack(_) => true,
            _ => false,
        }
    }
    pub fn left(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Left, self)
    }
    pub fn right(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Right, self)
    }
    pub fn top(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Top, self)
    }
    pub fn bottom(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Bottom, self)
    }
    pub fn width(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Width, self)
    }
    pub fn height(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Height, self)
    }
    pub fn center_x(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::CenterX, self)
    }
    pub fn center_y(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::CenterY, self)
    }
    pub fn x(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::X, self)
    }
    pub fn y(self) -> ValueDescriptor {
        ValueDescriptor::new(Designator::Y, self)
    }
    pub fn gap<G: Into<Gap>>(self, g: G) -> GridAxisDescriptor {
        debug_assert!(match self {
            LocationValue::Px(_)
            | LocationValue::Percent(_)
            | LocationValue::Column(_)
            | LocationValue::Row(_) => true,
            _ => false,
        });
        GridAxisDescriptor {
            value: self,
            gap: g.into(),
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
pub struct StackDescriptor {}
impl StackDescriptor {
    pub fn left(self) -> LocationValue {
        LocationValue::Stack(Designator::Left)
    }
    pub fn top(self) -> LocationValue {
        LocationValue::Stack(Designator::Top)
    }
    pub fn width(self) -> LocationValue {
        LocationValue::Stack(Designator::Width)
    }
    pub fn height(self) -> LocationValue {
        LocationValue::Stack(Designator::Height)
    }
    pub fn center_x(self) -> LocationValue {
        LocationValue::Stack(Designator::CenterX)
    }
    pub fn center_y(self) -> LocationValue {
        LocationValue::Stack(Designator::CenterY)
    }
    pub fn right(self) -> LocationValue {
        LocationValue::Stack(Designator::Right)
    }
    pub fn bottom(self) -> LocationValue {
        LocationValue::Stack(Designator::Bottom)
    }
}
pub fn stack() -> StackDescriptor {
    StackDescriptor {}
}
pub fn auto() -> LocationValue {
    LocationValue::Auto
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
