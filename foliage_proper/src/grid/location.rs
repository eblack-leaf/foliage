use crate::anim::interpolation::Interpolations;
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
#[require(Diff, CreateDiff, LastResolution)]
pub struct Location {
    pub(crate) xs: Option<LocationDescriptor>,
    pub(crate) sm: Option<LocationDescriptor>,
    pub(crate) md: Option<LocationDescriptor>,
    pub(crate) lg: Option<LocationDescriptor>,
    pub(crate) xl: Option<LocationDescriptor>,
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
        tracing::trace!("update_from_visibility for {:?}", trigger.entity());
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
            let (grid, view, context) = if let Some(id) = stem.id {
                let val = grids.get(id).unwrap();
                let context = sections.get(id).unwrap();
                (val.0.config(*layout), *val.1, *context)
            } else {
                (
                    Grid::default().config(*layout),
                    View::default(),
                    viewport.section(),
                )
            };
            let mut stack = None;
            if let Ok(stack) = stacks.get(this) {
                if let Some(id) = stack.id {
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
        );
        if a.is_none() {
            return None;
        }
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
) -> Option<CoordinateUnit> {
    match desc.value {
        LocationValue::Percent(pct) => Some(
            pct * match desc.designator {
                Designator::Left
                | Designator::Right
                | Designator::CenterX
                | Designator::X
                | Designator::Width => context.width(),
                _ => context.height(),
            },
        ),
        LocationValue::Px(px) => Some(px),
        LocationValue::Column(c) => {
            let inclusive = false;
            let val = (c as f32 - 1f32 * f32::from(!inclusive));
            Some(val)
        }
        LocationValue::Row(r) => {
            let inclusive = false;
            let val = (r as f32 - 1f32 * f32::from(!inclusive));
            Some(val)
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
            | Designator::Width => Some(letter_dims.a() * l as f32),
            _ => Some(letter_dims.b() * l as f32),
        },
    }
}
#[derive(Copy, Clone)]
pub struct ValueDescriptor {
    pub(crate) designator: Designator,
    pub(crate) value: LocationValue,
    pub(crate) padding: Padding,
}
impl ValueDescriptor {
    pub fn new(designator: Designator, value: LocationValue) -> Self {
        Self {
            designator,
            value,
            padding: Default::default(),
        }
    }
    pub fn pad<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn with(mut self, b: ValueDescriptor) -> ConfigurationDescriptor {
        ConfigurationDescriptor::new(self, b)
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
pub struct Padding {
    pub coordinates: Coordinates,
}
impl Default for Padding {
    fn default() -> Self {
        Self {
            coordinates: (0, 0).into(),
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Designator {
    X,
    Y,
    Left,
    Top,
    Width,
    Height,
    Right,
    Bottom,
    CenterX,
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
