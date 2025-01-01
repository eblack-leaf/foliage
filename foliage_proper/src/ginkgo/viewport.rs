use bevy_ecs::prelude::Resource;

use crate::coordinate::area::Area;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, Logical, Numerical, Physical};
use crate::ginkgo::{GraphicContext, Uniform};
use crate::willow::NearFarDescriptor;

type ViewportRepresentation = [[CoordinateUnit; 4]; 4];

pub(crate) struct Viewport {
    translation: Position<Numerical>,
    area: Area<Numerical>,
    pub(crate) near_far: NearFarDescriptor,
    matrix: ViewportRepresentation,
    pub(crate) uniform: Uniform<ViewportRepresentation>,
}

impl Viewport {
    pub(crate) fn section(&self) -> Section<Physical> {
        Section::new(self.translation.coordinates, self.area.coordinates)
    }
    pub(crate) fn set_position(&mut self, position: Position<Physical>, context: &GraphicContext) {
        self.translation = position.to_numerical();
        self.matrix = self.remake();
        self.uniform.write(context, self.matrix);
    }
    pub(crate) fn set_size(&mut self, area: Area<Numerical>, context: &GraphicContext) {
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
        section: Section<Numerical>,
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
        section: Section<Numerical>,
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
    translation: Position<Logical>,
    area: Area<Logical>,
    user_translated: bool,
    window_forced_resize: bool,
}

impl ViewportHandle {
    pub(crate) fn new(area: Area<Logical>) -> Self {
        Self {
            translation: Position::default(),
            area,
            user_translated: false,
            window_forced_resize: false,
        }
    }
    pub fn translate(&mut self, position: Position<Logical>) {
        self.translation += position;
        self.user_translated = true;
    }
    pub(crate) fn user_translations(&mut self) -> Option<Position<Logical>> {
        if self.user_translated {
            self.user_translated = false;
            return Some(self.translation);
        }
        None
    }
    pub(crate) fn resize(&mut self, area: Area<Logical>) {
        self.window_forced_resize = true;
        self.area = area;
    }
    pub(crate) fn window_forced_resize(&mut self) -> bool {
        let mut val = false;
        if self.window_forced_resize {
            val = true;
            self.window_forced_resize = false;
        }
        val
    }
    pub fn section(&self) -> Section<Logical> {
        Section::new(self.translation.coordinates, self.area.coordinates)
    }
}
