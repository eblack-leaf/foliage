use crate::compositor::layout::AspectRatio;
use crate::coordinate::layer::Layer;
use crate::coordinate::section::Section;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use bevy_ecs::component::Component;
use std::collections::HashMap;

#[derive(Component, Clone)]
pub struct MicroGrid {
    aspect: Option<AspectRatio>,
    sub_sections: HashMap<MicroGridSectionId, MicroGridSection>,
}
impl MicroGrid {
    pub fn new() -> Self {
        Self {
            aspect: None,
            sub_sections: Default::default(),
        }
    }
    pub fn section<ID: Into<MicroGridSectionId>, MGS: Into<MicroGridSection>>(
        mut self,
        id: ID,
        mgs: MGS,
    ) -> Self {
        self.sub_sections.insert(id.into(), mgs.into());
        self
    }
    // could be used to set the outer anchor box to enable justify in segment
    pub fn aspect<AR: Into<AspectRatio>>(mut self, ar: AR) -> Self {
        self.aspect.replace(ar.into());
        self
    }
    pub fn determine(
        &self,
        anchor: Coordinate<InterfaceContext>,
        alignment: &Alignment,
    ) -> Coordinate<InterfaceContext> {
        todo!()
    }
}
#[derive(Clone)]
pub struct MicroGridSection {
    pub section: Section<InterfaceContext>,
    // center is derivable, and other anchor points
}

impl MicroGridSection {
    pub fn new<S: Into<Section<InterfaceContext>>>(s: S) -> Self {
        Self { section: s.into() }
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone, Default)]
pub struct MicroGridSectionId(pub i32);
impl From<i32> for MicroGridSectionId {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
#[derive(Component, Clone)]
pub struct Alignment {
    x: AlignmentUnit,
    y: AlignmentUnit,
    w: AlignmentUnit,
    h: AlignmentUnit,
    layer_offset: Layer,
}
impl Alignment {
    pub fn new<L: Into<Layer>>(
        x: AlignmentUnit,
        y: AlignmentUnit,
        w: AlignmentUnit,
        h: AlignmentUnit,
        l: L,
    ) -> Self {
        Self {
            x,
            y,
            w,
            h,
            layer_offset: l.into(),
        }
    }
    pub fn aspect() {}
}
pub enum AlignmentOp {
    PercentOf,
    FixedOffset,
}
#[derive(Clone)]
pub struct AlignmentUnitDescriptor {}
#[derive(Copy, Clone)]
pub enum ConditionalAlignment {
    Min,
    Max,
}
#[derive(Clone)]
pub struct AlignmentUnit {
    base: AlignmentUnitDescriptor,
    conditional: HashMap<ConditionalAlignment, AlignmentUnitDescriptor>,
    max: Option<CoordinateUnit>,
    min: Option<CoordinateUnit>,
}
impl AlignmentUnit {
    pub fn maximum() {}
    pub fn minimum() {}
    pub fn or_min(mut self, min: AlignmentUnitDescriptor) -> Self {
        todo!()
    }
    pub fn or_max(mut self, max: AlignmentUnitDescriptor) -> Self {
        todo!()
    }
}
pub trait AlignmentDesc {
    fn percent_of<MGS: Into<MicroGridSectionId>>(self, mgs: MGS) -> AlignmentUnitDescriptor;
    fn centered_on<MGS: Into<MicroGridSectionId>>(self, mgs: MGS) -> AlignmentUnitDescriptor;
    fn derived_from<OP: Into<AlignmentOp>>(
        self,
        op: OP,
        base: AlignmentUnitDescriptor,
    ) -> AlignmentUnitDescriptor;
}
#[test]
fn example() {
    let grid = MicroGrid::new();
    let coordinate = Coordinate::default();
    // let alignment = Alignment::new();
    // let determined = grid.determine(coordinate, alignment);
    // assert_eq!(determined, ...);
}