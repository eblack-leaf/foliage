use compact_str::{CompactString, ToCompactString};
use std::ops::RangeInclusive;

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
    CharacterDimension, Text, TextColorExceptions, TextLineLocation, TextLineStructure, TextValue,
};
use foliage_proper::texture::factors::Progress;
use foliage_proper::window::ScaleFactor;

use crate::{BackgroundColor, Colors, ForegroundColor};

#[derive(Clone)]
pub struct InteractiveText {
    pub line_structure: TextLineStructure,
    pub text_value: TextValue,
    pub colors: Colors,
}
impl InteractiveText {
    pub fn new<TLS: Into<TextLineStructure>, TV: Into<TextValue>>(
        tls: TLS,
        text_value: TV,
        colors: Colors,
    ) -> Self {
        Self {
            line_structure: tls.into(),
            text_value: text_value.into(),
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
    pub start: Option<TextLineLocation>,
    pub end: Option<TextLineLocation>,
    pub span: Option<(usize, usize)>,
}
impl Selection {
    pub fn new(start: Option<TextLineLocation>, end: Option<TextLineLocation>) -> Self {
        Self {
            start,
            end,
            span: None,
        }
    }
    pub fn derive_span(&mut self, text_line_structure: TextLineStructure) {
        if let Some(start) = self.start {
            if let Some(end) = self.end {
                if start == end || start < end {
                    self.span.replace((
                        text_line_structure.letter(start),
                        text_line_structure.letter(end),
                    ));
                } else {
                    self.span.replace((
                        text_line_structure.letter(end),
                        text_line_structure.letter(start),
                    ));
                }
            }
        }
    }
    pub fn range(&self) -> Option<RangeInclusive<usize>> {
        if let Some(span) = self.span {
            return Some(span.0..=span.1);
        }
        None
    }
    pub fn substring(&self, tv: &CompactString) -> CompactString {
        let mut accumulator = CompactString::default();
        if let Some(r) = self.range() {
            for i in r {
                accumulator += tv.get(i..i + 1).unwrap_or_default();
            }
        }
        accumulator
    }
    pub fn clear(&self, tv: &mut CompactString) {}
    // pub fn range(&self) -> Option<RangeInclusive<i32>> {
    //     if let Some(start) = self.start {
    //         if let Some(span) = self.end {
    //             if span == 0 {
    //                 return None;
    //             }
    //             return if span.is_positive() {
    //                 Some(start..=(start + span))
    //             } else {
    //                 Some((start + span)..=start)
    //             };
    //         }
    //     }
    //     None
    // }
    // pub fn contains(&self, i: i32) -> bool {
    //     if let Some(start) = self.start {
    //         if i == start {
    //             return true;
    //         }
    //         if let Some(_span) = self.end {
    //             if let Some(r) = self.range() {
    //                 for x in r {
    //                     if x == i {
    //                         return true;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }
}
#[cfg(test)]
#[test]
fn test_selection() {
    // let selection = Selection::new(Some(0), Some(4));
    // assert_eq!(selection.contains(0), true);
    // assert_eq!(selection.contains(1), true);
    // assert_eq!(selection.contains(2), true);
    // assert_eq!(selection.contains(3), true);
    // assert_eq!(selection.contains(4), true);
    // let selection = Selection::new(Some(4), Some(-4));
    // assert_eq!(selection.contains(0), true);
    // assert_eq!(selection.contains(1), true);
    // assert_eq!(selection.contains(2), true);
    // assert_eq!(selection.contains(3), true);
    // assert_eq!(selection.contains(4), true);
}
#[derive(Bundle)]
pub struct InteractiveTextComponents {
    pub selection: Selection,
    pub text: TextValue,
    pub line_structure: TextLineStructure,
    pub colors: Colors,
    pub dims: CharacterDimension,
}
fn update_selection(
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
    mut query: Query<(
        &TextLineStructure,
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
            Changed<Layer>,
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
    for (line_structure, mut sel, tv, bindings, mut d) in query.iter_mut() {
        if let Ok((listener, pos, area, layer)) =
            listeners.get(bindings.get(InteractiveTextBindings::Text))
        {
            let metrics = font.line_metrics(line_structure, *area, &scale_factor);
            *d = metrics.character_dimensions;
            // resize bindings
            for y in 0..line_structure.lines {
                for x in 0..line_structure.per_line {
                    let letter_binding = x + line_structure.per_line * y + 1;
                    let entity = bindings.get(letter_binding as i32);
                    *rectangles.get_mut(entity).unwrap().0 = *pos
                        + Position::new(
                            x as f32 * d.dimensions().width,
                            y as f32 * d.dimensions().height,
                        );
                    *rectangles.get_mut(entity).unwrap().1 = d.dimensions();
                    *rectangles.get_mut(entity).unwrap().2 = *layer + 1.into();
                }
            }
            // get current position
            if listener.lost_focus() {
                sel.start.take();
                sel.end.take();
            }
            let current = TextLineLocation::new(listener.interaction.current, d.dimensions());
            if listener.engaged_start() {
                sel.start.replace(current);
            }
            if listener.engaged() {
                sel.end.replace(current);
                sel.derive_span(*line_structure);
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
                &'static TextLineStructure,
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
                &'static mut TextColorExceptions,
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
        if let Ok((_fc, bc, ls, tv, sel)) = ext.0.get(entity) {
            let mut color_changes = TextColorExceptions::blank();
            for i in 1..=ls.max_chars().0 {
                *ext.1.get_mut(bindings.get(i as i32)).unwrap().3.alpha_mut() = 0.0;
            }
            if let Some(r) = sel.range() {
                for i in r {
                    *ext.1
                        .get_mut(bindings.get(i as i32 + 1))
                        .unwrap()
                        .3
                        .alpha_mut() = 1.0;
                    color_changes.exceptions.insert(i, bc.0);
                }
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
                self.text_value.clone(),
                self.line_structure,
                self.colors.foreground.0,
            ),
        );
        binder.extend(text, InteractionListener::default());
        for letter in 0..self.line_structure.max_chars().0 {
            binder.bind(
                letter as i32 + 1,
                MicroGridAlignment::unaligned(),
                Rectangle::new(self.colors.foreground.0.with_alpha(0.0), Progress::full()),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(self.line_structure.max_chars().mono_aspect()),
            InteractiveTextComponents {
                selection: Selection::default(),
                text: self.text_value.clone(),
                line_structure: self.line_structure,
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
        elm.enable_conditional_scene::<InteractiveText>();
        elm.main().add_systems(((
            update_selection,
            foliage_proper::scene::config::<InteractiveText>,
        )
            .chain()
            .in_set(SetDescriptor::Update)
            .before(<Text as Leaf>::SetDescriptor::Update),));
    }
}
