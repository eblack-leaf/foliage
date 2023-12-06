use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Changed, Or, Query};
use bevy_ecs::bundle::Bundle;
use crate::coordinate::{Coordinate, CoordinateUnit, InterfaceContext};
use crate::coordinate::area::Area;
use crate::coordinate::layer::Layer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;

#[derive(Bundle, Copy, Clone)]
pub struct AlignmentCoordinate {
    pub ha: HorizontalAlignment,
    pub va: VerticalAlignment,
    pub la: LayerAlignment,
}

impl<HA: Into<HorizontalAlignment>, VA: Into<VerticalAlignment>, LA: Into<LayerAlignment>>
    From<(HA, VA, LA)> for AlignmentCoordinate
{
    fn from(value: (HA, VA, LA)) -> Self {
        Self {
            ha: value.0.into(),
            va: value.1.into(),
            la: value.2.into(),
        }
    }
}

pub trait AlignedNumber {
    fn hcenter(self) -> HorizontalAlignment;
    fn left_align(self) -> HorizontalAlignment;
    fn right_align(self) -> HorizontalAlignment;
    fn vcenter(self) -> VerticalAlignment;
    fn top_align(self) -> VerticalAlignment;
    fn bottom_align(self) -> VerticalAlignment;
    fn layer_align(self) -> LayerAlignment;
}

impl AlignedNumber for f32 {
    fn hcenter(self) -> HorizontalAlignment {
        HorizontalAlignment::Center(self)
    }

    fn left_align(self) -> HorizontalAlignment {
        HorizontalAlignment::Left(self)
    }

    fn right_align(self) -> HorizontalAlignment {
        HorizontalAlignment::Right(self)
    }

    fn vcenter(self) -> VerticalAlignment {
        VerticalAlignment::Center(self)
    }

    fn top_align(self) -> VerticalAlignment {
        VerticalAlignment::Top(self)
    }

    fn bottom_align(self) -> VerticalAlignment {
        VerticalAlignment::Bottom(self)
    }

    fn layer_align(self) -> LayerAlignment {
        LayerAlignment::new(self)
    }
}

impl AlignedNumber for u32 {
    fn hcenter(self) -> HorizontalAlignment {
        HorizontalAlignment::Center(self as CoordinateUnit)
    }

    fn left_align(self) -> HorizontalAlignment {
        HorizontalAlignment::Left(self as CoordinateUnit)
    }

    fn right_align(self) -> HorizontalAlignment {
        HorizontalAlignment::Right(self as CoordinateUnit)
    }

    fn vcenter(self) -> VerticalAlignment {
        VerticalAlignment::Center(self as CoordinateUnit)
    }

    fn top_align(self) -> VerticalAlignment {
        VerticalAlignment::Top(self as CoordinateUnit)
    }

    fn bottom_align(self) -> VerticalAlignment {
        VerticalAlignment::Bottom(self as CoordinateUnit)
    }

    fn layer_align(self) -> LayerAlignment {
        LayerAlignment::new(self)
    }
}

impl AlignedNumber for i32 {
    fn hcenter(self) -> HorizontalAlignment {
        HorizontalAlignment::Center(self as CoordinateUnit)
    }

    fn left_align(self) -> HorizontalAlignment {
        HorizontalAlignment::Left(self as CoordinateUnit)
    }

    fn right_align(self) -> HorizontalAlignment {
        HorizontalAlignment::Right(self as CoordinateUnit)
    }

    fn vcenter(self) -> VerticalAlignment {
        VerticalAlignment::Center(self as CoordinateUnit)
    }

    fn top_align(self) -> VerticalAlignment {
        VerticalAlignment::Top(self as CoordinateUnit)
    }

    fn bottom_align(self) -> VerticalAlignment {
        VerticalAlignment::Bottom(self as CoordinateUnit)
    }

    fn layer_align(self) -> LayerAlignment {
        LayerAlignment::new(self)
    }
}

pub(crate) fn place(
    mut aligned: Query<
        (
            &AlignmentAnchor,
            &HorizontalAlignment,
            &VerticalAlignment,
            &mut Position<InterfaceContext>,
            &Area<InterfaceContext>,
        ),
        Or<(
            Changed<AlignmentAnchor>,
            Changed<HorizontalAlignment>,
            Changed<VerticalAlignment>,
            Changed<Position<InterfaceContext>>,
            Changed<Area<InterfaceContext>>,
        )>,
    >,
) {
    for (anchor, ha, va, mut pos, area) in aligned.iter_mut() {
        let x = ha.calc(anchor.section(), *area);
        let y = va.calc(anchor.section(), *area);
        *pos = (x, y).into();
    }
}

pub(crate) fn place_layer(
    mut aligned: Query<
        (&AlignmentAnchor, &LayerAlignment, &mut Layer),
        Or<(
            Changed<AlignmentAnchor>,
            Changed<LayerAlignment>,
            Changed<Layer>,
        )>,
    >,
) {
    for (anchor, la, mut layer) in aligned.iter_mut() {
        *layer = la.calc(anchor.layer());
    }
}

#[derive(Copy, Clone, Component)]
pub struct AlignmentAnchor(pub Coordinate<InterfaceContext>);

impl AlignmentAnchor {
    pub fn section(&self) -> Section<InterfaceContext> {
        self.0.section
    }
    pub fn layer(&self) -> Layer {
        self.0.layer
    }
}

#[derive(Component, Copy, Clone)]
pub enum HorizontalAlignment {
    Center(CoordinateUnit),
    Left(CoordinateUnit),
    Right(CoordinateUnit),
}

impl HorizontalAlignment {
    pub fn calc(
        &self,
        scene_section: Section<InterfaceContext>,
        target: Area<InterfaceContext>,
    ) -> CoordinateUnit {
        match self {
            HorizontalAlignment::Center(alignment) => {
                scene_section.center().x - target.width / 2f32 + alignment
            }
            HorizontalAlignment::Left(alignment) => scene_section.left() + alignment,
            HorizontalAlignment::Right(alignment) => scene_section.right() - alignment,
        }
    }
}

#[derive(Component, Copy, Clone)]
pub enum VerticalAlignment {
    Center(CoordinateUnit),
    Top(CoordinateUnit),
    Bottom(CoordinateUnit),
}

impl VerticalAlignment {
    pub fn calc(
        &self,
        scene_section: Section<InterfaceContext>,
        target: Area<InterfaceContext>,
    ) -> CoordinateUnit {
        match self {
            VerticalAlignment::Center(alignment) => {
                scene_section.center().y - target.width / 2f32 + alignment
            }
            VerticalAlignment::Top(alignment) => scene_section.top() + alignment,
            VerticalAlignment::Bottom(alignment) => scene_section.bottom() - alignment,
        }
    }
}

#[derive(Component, Copy, Clone)]
pub struct LayerAlignment(pub Layer);

impl LayerAlignment {
    pub fn new<L: Into<Layer>>(l: L) -> Self {
        Self(l.into())
    }
    pub fn calc(&self, scene: Layer) -> Layer {
        self.0 + scene
    }
}