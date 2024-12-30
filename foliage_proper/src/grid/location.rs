use crate::grid::{GridUnit, ScalarUnit};
use crate::{Component, Coordinates, Grid, Layout, LogicalContext, Section, Stem, Tree, Update};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Res, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;
#[derive(Component)]
#[component(on_insert = Location::on_insert)]
pub struct Location {
    pub sm: Option<LocationConfiguration>,
    pub md: Option<LocationConfiguration>,
    pub lg: Option<LocationConfiguration>,
    pub xl: Option<LocationConfiguration>,
}
impl Location {
    pub fn new() -> Self {
        Self {
            sm: None,
            md: None,
            lg: None,
            xl: None,
        }
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
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        world.trigger_targets(Update::<Location>::new(), this);
    }
    pub(crate) fn update_location(
        trigger: Trigger<Update<Location>>,
        mut tree: Tree,
        layout: Res<Layout>,
        locations: Query<&Location>,
        sections: Query<&Section<LogicalContext>>,
        grids: Query<&Grid>,
        stems: Query<&Stem>,
        stacks: Query<&Stack>,
    ) {
        // if trigger.entity() has location
        // resolve location w/ stems + stacks + sections + grids
        // if resolved => insert resolved-section + if invisible => visible
        // else if visible => invisible
    }
}
#[derive(Component, Copy, Clone)]
pub struct Stack {
    pub id: Option<Entity>,
}
impl Default for Stack {
    fn default() -> Self {
        Self { id: None }
    }
}
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
pub struct LocationAxisDescriptor {
    pub a: GridUnit,
    pub b: GridUnit,
    pub ty: LocationAxisType,
    pub padding: Padding,
    pub justify: Justify,
    pub max: Option<ScalarUnit>,
}
impl LocationAxisDescriptor {
    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }
    pub fn pad<P: Into<Padding>>(mut self, pad: P) -> Self {
        self.padding = pad.into();
        self
    }
    pub fn max<S: Into<ScalarUnit>>(mut self, max: S) -> Self {
        self.max.replace(max.into());
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
    Left,
    Right,
    #[default]
    Center,
}
pub enum LocationAxisType {
    Point,
    Span,
    To,
}
