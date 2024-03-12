use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs, Or};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Query, Res, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::layer::Layer;
use foliage_proper::coordinate::position::Position;
use foliage_proper::coordinate::InterfaceContext;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::interaction::InteractionListener;
use foliage_proper::rectangle::Rectangle;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::font::MonospacedFont;
use foliage_proper::text::{
    CharacterDimension, GlyphColorChanges, MaxCharacters, Text, TextKey, TextValue,
};
use foliage_proper::texture::factors::Progress;
use foliage_proper::window::ScaleFactor;
use std::ops::RangeInclusive;

use crate::r_scenes::{BackgroundColor, Colors, ForegroundColor};

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
#[derive(InnerSceneBinding)]
pub enum InteractiveTextBindings {
    Text,
}
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Selection {
    pub start: Option<i32>,
    pub span: Option<i32>,
}
impl Selection {
    pub fn new(start: Option<i32>, span: Option<i32>) -> Self {
        Self { start, span }
    }
    pub fn range(&self) -> Option<RangeInclusive<i32>> {
        if let Some(start) = self.start {
            if let Some(span) = self.span {
                if span == 0 {
                    return None;
                }
                return if span.is_positive() {
                    Some(start..=(start + span))
                } else {
                    Some((start + span)..=start)
                };
            }
        }
        None
    }
    pub fn contains(&self, i: i32) -> bool {
        if let Some(start) = self.start {
            if i == start {
                return true;
            }
            if let Some(_span) = self.span {
                if let Some(r) = self.range() {
                    for x in r {
                        if x == i {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}
#[cfg(test)]
#[test]
fn test_selection() {
    let selection = Selection::new(Some(0), Some(4));
    assert_eq!(selection.contains(0), true);
    assert_eq!(selection.contains(1), true);
    assert_eq!(selection.contains(2), true);
    assert_eq!(selection.contains(3), true);
    assert_eq!(selection.contains(4), true);
    let selection = Selection::new(Some(4), Some(-4));
    assert_eq!(selection.contains(0), true);
    assert_eq!(selection.contains(1), true);
    assert_eq!(selection.contains(2), true);
    assert_eq!(selection.contains(3), true);
    assert_eq!(selection.contains(4), true);
}
#[derive(Bundle)]
pub struct InteractiveTextComponents {
    pub selection: Selection,
    pub text: TextValue,
    pub max_chars: MaxCharacters,
    pub colors: Colors,
    pub dims: CharacterDimension,
}
fn update_selection(
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
    mut query: Query<(
        &MaxCharacters,
        &mut Selection,
        &TextValue,
        &Bindings,
        &mut CharacterDimension,
    )>,
    listeners: Query<
        (
            &InteractionListener,
            &Position<InterfaceContext>,
            &Area<InterfaceContext>,
            &Layer,
        ),
        Or<(
            Changed<InteractionListener>,
            Changed<Position<InterfaceContext>>,
            Changed<Area<InterfaceContext>>,
        )>,
    >,
    mut rectangles: Query<
        (
            &mut Position<InterfaceContext>,
            &mut Area<InterfaceContext>,
            &mut Layer,
        ),
        Without<InteractionListener>,
    >,
) {
    for (mc, mut sel, tv, bindings, mut d) in query.iter_mut() {
        if let Ok((listener, pos, area, layer)) =
            listeners.get(bindings.get(InteractiveTextBindings::Text))
        {
            let (_fs, _fa, dims) = font.best_fit(*mc, *area, &scale_factor);
            *d = dims;
            for letter in 1..mc.0 + 1 {
                *rectangles.get_mut(bindings.get(letter as i32)).unwrap().0 = *pos
                    + Position::<InterfaceContext>::new(
                        (letter as f32 - 1f32) * dims.dimensions().width,
                        0.0,
                    );
                *rectangles.get_mut(bindings.get(letter as i32)).unwrap().1 = dims.dimensions();
                *rectangles.get_mut(bindings.get(letter as i32)).unwrap().2 = *layer + 1.into();
            }
            let text_key = ((listener.interaction.current.x - pos.x).max(0.0)
                / dims.dimensions().width)
                .floor()
                .min(tv.0.len() as f32)
                .min(mc.0.checked_sub(1).unwrap_or_default() as f32);
            if listener.engaged_start() {
                sel.start.replace(text_key as i32);
            }
            if listener.engaged() {
                let i = text_key as i32 - sel.start.unwrap();
                sel.span.replace(i);
            }
            if listener.engaged_end() {
                // store selection string
                // finish span
            }
            if listener.lost_focus() {
                sel.start.take();
                sel.span.take();
            }
        }
    }
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
                &'static Selection,
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
        Query<
            'static,
            'static,
            (
                &'static mut GlyphColorChanges,
                &'static mut TextValue,
                &'static InteractionListener,
            ),
            Without<Tag<InteractiveText>>,
        >,
    );
    type Filter = Or<(
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<Selection>,
        Changed<TextValue>,
    )>;
    type Components = InteractiveTextComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let text = bindings.get(0);
        if let Ok((_fc, bc, mc, tv, sel)) = ext.0.get(entity) {
            let mut color_changes = GlyphColorChanges::new();
            for letter in 1..mc.0 + 1 {
                if sel.contains((letter - 1) as i32) {
                    *ext.1
                        .get_mut(bindings.get(letter as i32))
                        .unwrap()
                        .3
                        .alpha_mut() = 1.0;
                    color_changes.0.insert((letter - 1) as TextKey, bc.0);
                } else {
                    *ext.1
                        .get_mut(bindings.get(letter as i32))
                        .unwrap()
                        .3
                        .alpha_mut() = 0.0;
                }
                // if let Some(_c) = tv.0.get((letter - 1) as usize..letter as usize) {
                //
                // }
            }
            *ext.4.get_mut(text).unwrap().1 = tv.clone();
            *ext.4.get_mut(text).unwrap().0 = color_changes;
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let text = binder.bind(
            InteractiveTextBindings::Text,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Text::new(
                self.max_chars,
                self.text_value.clone(),
                self.colors.foreground.0,
            ),
        );
        binder.extend(text, InteractionListener::default());
        for letter in 0..self.max_chars.0 {
            binder.bind(
                letter as i32 + 1,
                MicroGridAlignment::unaligned(),
                Rectangle::new(self.colors.foreground.0.with_alpha(0.0), Progress::full()),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(self.max_chars.mono_aspect()),
            InteractiveTextComponents {
                selection: Selection::default(),
                text: self.text_value.clone(),
                max_chars: self.max_chars,
                colors: self.colors,
                dims: CharacterDimension::new(Area::default()),
            },
        ))
    }
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for InteractiveText {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(((
            update_selection,
            foliage_proper::scene::config::<InteractiveText>,
        )
            .chain()
            .in_set(SetDescriptor::Update)
            .before(<Text as Leaf>::SetDescriptor::Update),));
    }
}
