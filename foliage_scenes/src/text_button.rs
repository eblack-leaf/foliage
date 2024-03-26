use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{IntoSystemConfigs, Or, Query, With, Without};
use foliage_proper::bevy_ecs::query::Changed;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::color::Color;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, Elm, Style};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{Text, TextLineStructure, TextValue};

use crate::button::{Button, ButtonInteractionHook, CurrentStyle};
use crate::{BackgroundColor, Colors, ForegroundColor};

#[derive(Clone)]
pub struct TextButton {
    element_style: Style,
    text_value: TextValue,
    colors: Colors,
    lines: TextLineStructure,
}
impl TextButton {
    pub fn new<TLS: Into<TextLineStructure>>(
        text_value: TextValue,
        lines: TLS,
        element_style: Style,
        colors: Colors,
    ) -> Self {
        Self {
            element_style,
            text_value,
            colors,
            lines: lines.into(),
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum TextButtonBindings {
    Panel,
    Text,
    Interaction,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Scene for TextButton {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static Style,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static CurrentStyle,
                &'static TextValue,
            ),
            With<Tag<TextButton>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<TextButton>>>,
        Query<'static, 'static, &'static mut Style, Without<Tag<TextButton>>>,
        Query<'static, 'static, &'static mut TextValue, Without<Tag<TextButton>>>,
    );
    type Filter = Or<(
        <Button as Scene>::Filter,
        Changed<TextValue>,
        Changed<TextLineStructure>,
    )>;
    type Components = (<Button as Scene>::Components, TextValue, TextLineStructure);

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let panel = bindings.get(TextButtonBindings::Panel);
        let text = bindings.get(TextButtonBindings::Text);
        if let Ok((_est, fc, bc, cs, tv)) = ext.0.get(entity) {
            ext.3.get_mut(text).unwrap().0 = tv.0.clone();
            if _est.is_fill() {
                if cs.0.is_fill() {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    *ext.1.get_mut(text).unwrap() = fc.0;
                } else {
                    *ext.1.get_mut(text).unwrap() = bc.0;
                    *ext.1.get_mut(panel).unwrap() = fc.0;
                }
            } else {
                *ext.2.get_mut(panel).unwrap() = cs.0;
                if cs.0.is_fill() {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    *ext.1.get_mut(text).unwrap() = fc.0;
                } else {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    *ext.1.get_mut(text).unwrap() = bc.0;
                }
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let aspect = self.lines.max_chars().mono_aspect();
        binder.extend(binder.root(), Tag::<ButtonInteractionHook>::new());
        binder.bind(
            TextButtonBindings::Panel,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Panel::new(self.element_style, self.colors.foreground.0),
        );
        let adjusted_lines = self.lines.with_lines(1);
        binder.bind(
            TextButtonBindings::Text,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                0.7.percent_of(AnchorDim::Width),
                0.8.percent_of(AnchorDim::Height),
            ),
            Text::new(
                self.text_value.clone(),
                adjusted_lines,
                self.colors.background.0,
            ),
        );
        binder.bind(
            TextButtonBindings::Interaction,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            BlankNode::default()
                .extend(InteractionListener::default())
                .extend(Tag::<ButtonInteractionHook>::new()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new()
                .min_height(24.0)
                .min_width(30.0 * aspect.value()),
            (
                <Button as Scene>::Components::new(self.element_style, self.colors),
                self.text_value,
                adjusted_lines,
            ),
        ))
    }
}
impl Leaf for TextButton {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.enable_conditional_scene::<TextButton>();
        elm.main().add_systems(
            foliage_proper::scene::config::<TextButton>
                .in_set(SetDescriptor::Update)
                .before(<Text as Leaf>::SetDescriptor::Update)
                .before(<Panel as Leaf>::SetDescriptor::Update),
        );
    }
}
