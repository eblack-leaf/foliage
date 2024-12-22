use bevy_ecs::prelude::Resource;

use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, DeviceContext, LogicalContext, NumericalContext};
use crate::ginkgo::{GraphicContext, Uniform};
use crate::willow::NearFarDescriptor;

type ViewportRepresentation = [[CoordinateUnit; 4]; 4];

pub(crate) struct Viewport {
    translation: Position<NumericalContext>,
    area: Area<NumericalContext>,
    pub(crate) near_far: NearFarDescriptor,
    matrix: ViewportRepresentation,
    pub(crate) uniform: Uniform<ViewportRepresentation>,
}

impl Viewport {
    pub(crate) fn section(&self) -> Section<DeviceContext> {
        Section::new(self.translation.coordinates, self.area.coordinates)
    }
    pub(crate) fn set_position(
        &mut self,
        position: Position<DeviceContext>,
        context: &GraphicContext,
    ) {
        self.translation = position.to_numerical();
        self.matrix = self.remake();
        self.uniform.write(context, self.matrix);
    }
    pub(crate) fn set_size(&mut self, area: Area<NumericalContext>, context: &GraphicContext) {
        self.area = area;
        self.matrix = self.remake();
        self.uniform.write(context, self.matrix);
    }

    fn remake(&mut self) -> ViewportRepresentation {
        Self::generate(
            Section::new(self.translation.coordinates, self.area.coordinates),
            self.near_far,
        )
    }
    pub(crate) fn new(
        context: &GraphicContext,
        section: Section<NumericalContext>,
        near_far: NearFarDescriptor,
    ) -> Self {
        let matrix = Self::generate(section, near_far);
        Self {
            translation: section.position.to_numerical(),
            area: section.area,
            near_far,
            matrix,
            uniform: Uniform::new(context, matrix),
        }
    }
    fn generate(
        section: Section<NumericalContext>,
        near_far: NearFarDescriptor,
    ) -> ViewportRepresentation {
        let right_left = 2f32 / (section.right() - section.left());
        let top_bottom = 2f32 / (section.top() - section.bottom());
        let nf = 1f32 / (near_far.far.0 - near_far.near.0);

        [
            [right_left, 0f32, 0f32, 0f32],
            [0f32, top_bottom, 0f32, 0f32],
            [0f32, 0f32, nf, 0f32],
            [
                right_left * -section.left() - 1f32,
                top_bottom * -section.top() + 1f32,
                nf * near_far.near.0,
                1f32,
            ],
        ]
    }
}

#[derive(Default, Resource)]
pub struct ViewportHandle {
    translation: Position<LogicalContext>,
    area: Area<LogicalContext>,
    changes: bool,
    updated: bool,
}

impl ViewportHandle {
    pub(crate) fn new(area: Area<LogicalContext>) -> Self {
        Self {
            translation: Position::default(),
            area,
            changes: false,
            updated: false,
        }
    }
    pub fn translate(&mut self, position: Position<LogicalContext>) {
        self.translation += position;
        self.changes = true;
    }
    pub(crate) fn changes(&mut self) -> Option<Position<LogicalContext>> {
        if self.changes {
            self.changes = false;
            return Some(self.translation);
        }
        None
    }
    pub(crate) fn resize(&mut self, area: Area<LogicalContext>) {
        self.updated = true;
        self.area = area;
    }
    pub(crate) fn updated(&mut self) -> bool {
        let mut val = false;
        if self.updated {
            val = true;
            self.updated = false;
        }
        val
    }
    pub fn section(&self) -> Section<LogicalContext> {
        Section::new(self.translation.coordinates, self.area.coordinates)
    }
}
