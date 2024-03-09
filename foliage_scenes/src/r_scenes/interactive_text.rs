use crate::r_scenes::{BackgroundColor, Colors, ForegroundColor};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, Or};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Query, Res, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::layer::Layer;
use foliage_proper::coordinate::position::Position;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::leaf::Tag;
use foliage_proper::rectangle::Rectangle;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::font::MonospacedFont;
use foliage_proper::text::{MaxCharacters, Text, TextValue};
use foliage_proper::texture::factors::Progress;
use foliage_proper::window::ScaleFactor;

pub struct InteractiveText {
    pub max_chars: MaxCharacters,
    pub text_value: TextValue,
    pub colors: Colors,
}
impl InteractiveText {
    pub fn new(max_characters: MaxCharacters, text_value: TextValue, colors: Colors) -> Self {
        Self {
            max_chars: max_characters,
            text_value,
            colors,
        }
    }
}
#[derive(Component)]
pub struct Selection {
    pub value: String,
    pub start: Option<u32>,
    pub span: Option<i32>,
}
impl Selection {
    pub fn new(value: String, start: Option<u32>, span: Option<i32>) -> Self {
        Self { value, start, span }
    }
}
#[derive(Bundle)]
pub struct InteractiveTextComponents {
    pub selection: Selection,
    pub text: TextValue,
    pub max_chars: MaxCharacters,
    pub colors: Colors,
}
impl Scene for InteractiveText {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static MaxCharacters,
                &'static TextValue,
            ),
            With<Tag<InteractiveText>>,
        >,
        Query<
            'static,
            'static,
            (
                &'static mut Position<InterfaceContext>,
                &'static mut Area<InterfaceContext>,
                &'static mut Layer,
                &'static mut Color,
            ),
            Without<Tag<InteractiveText>>,
        >,
        Res<'static, MonospacedFont>,
        Res<'static, ScaleFactor>,
    );
    type Filter = Or<(
        Changed<Position<InterfaceContext>>,
        Changed<Area<InterfaceContext>>,
        Changed<Layer>,
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
    )>;
    type Components = InteractiveTextComponents;

    fn config(
        entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        let text = bindings.get(0);
        if let Ok((fc, bc, mc, tv)) = ext.0.get(entity) {
            let (fs, fa, dims) = ext.2.best_fit(*mc, _coordinate.section.area, &ext.3);
            for letter in 0..mc.0 {}
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            0,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Text::new(self.max_chars, self.text_value, self.colors.foreground.0),
        );
        for letter in 0..self.max_chars.0 {
            binder.bind_conditional(
                letter as i32 + 1,
                MicroGridAlignment::unaligned(),
                Rectangle::new(self.colors.foreground.0, Progress::full()),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            InteractiveTextComponents {
                selection: Selection::new(String::default(), None, None),
                text: self.text_value.clone(),
                max_chars: self.max_chars,
                colors: self.colors,
            },
        ))
    }
}