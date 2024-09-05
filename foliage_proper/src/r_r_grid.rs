use crate::coordinate::{CoordinateUnit, Coordinates};
use crate::layout::Layout;
use crate::leaf::LeafHandle;
use bevy_ecs::prelude::Component;
use std::collections::HashMap;

#[cfg(test)]
#[test]
fn behavior() {
    let location = GridLocation::new();
}

pub enum GridContext {
    Screen,
    Named(LeafHandle),
    Absolute,
}
impl GridContext {
    pub fn top(self) -> LocationAspectTokenValue {
        LocationAspectTokenValue::ContextAspect(GridAspect::Top)
    }
}
pub fn screen() -> GridContext {
    GridContext::Screen
}
pub enum LocationAspectTokenOp {
    Add,
    Minus,
    // ...
}
pub enum RelativeUnit {
    Column(u32),
    Row(u32),
    Percent(f32),
}
pub enum LocationAspectTokenValue {
    ContextAspect(GridAspect),
    Relative(RelativeUnit),
    Absolute(CoordinateUnit),
}
pub struct LocationAspectToken {
    op: LocationAspectTokenOp,
    context: GridContext,
    value: LocationAspectTokenValue,
}
#[derive(Default)]
pub enum LocationAspectDescriptor {
    #[default]
    Existing(GridAspect),
    Specified(Vec<LocationAspectToken>),
}
#[derive(Default)]
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
    pub fn top<LAD: Into<LocationAspectDescriptor>>(mut self, t: LAD) -> Self {
        self.independent_or_x = t.into();
        self
    }
    pub fn using_top(mut self) -> Self {
        self.independent_or_x = LocationAspectDescriptor::Existing(GridAspect::Top);
        self
    }
    pub fn bottom<LAD: Into<LocationAspectDescriptor>>(mut self, t: LAD) -> Self {
        self.other_or_y = t.into();
        self
    }
    // ...
}
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum AspectConfiguration {
    Horizontal,
    Vertical,
    PointA,
    PointB,
    PointC,
    PointD,
}
pub struct GridLocationException {
    layout: Layout,
    config: AspectConfiguration,
}

impl GridLocationException {
    fn new(layout: Layout, config: AspectConfiguration) -> GridLocationException {
        Self { layout, config }
    }
}

pub struct GridLocation {
    configurations: HashMap<AspectConfiguration, LocationAspect>,
    exceptions: HashMap<GridLocationException, LocationAspect>,
}
impl GridLocation {
    pub fn new() -> Self {
        Self {
            configurations: Default::default(),
            exceptions: Default::default(),
        }
    }
    pub fn top<LAD: Into<LocationAspectDescriptor>>(mut self, d: LAD) -> Self {
        if self
            .configurations
            .contains_key(&AspectConfiguration::Vertical)
        {
            // sanitize that other is compatible
            // add
        } else {
            self.configurations
                .insert(AspectConfiguration::Vertical, LocationAspect::new().top(d));
        }
        self
    }
    pub fn bottom<LAD: Into<LocationAspectDescriptor>>(mut self, d: LAD) -> Self {
        if self
            .configurations
            .contains_key(&AspectConfiguration::Vertical)
        {
            // sanitize that other is compatible
            // add
        } else {
            self.configurations.insert(
                AspectConfiguration::Vertical,
                LocationAspect::new().bottom(d),
            );
        }
        self
    }
    pub fn except_at<LA: Into<LocationAspect>>(
        mut self,
        layout: Layout,
        ac: AspectConfiguration,
        la: LA,
    ) -> Self {
        self.exceptions
            .insert(GridLocationException::new(layout, ac), la.into());
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
    PointA,
    PointB,
    PointC,
    PointD,
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
