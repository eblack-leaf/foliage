use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::animate::trigger::{Trigger, TriggerState};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Changed, Component, IntoSystemConfigs, Or};
use foliage_proper::bevy_ecs::query::{With, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::color::Color;

use crate::{BackgroundColor, Colors, ForegroundColor};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, Elm, Style};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{
    Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle, ScenePtr,
};

use crate::icon_text::{IconColor, IconText, TextColor};

#[derive(Clone)]
pub struct Button {
    pub icon_text: IconText,
    pub element_style: Style,
    pub colors: Colors,
}
impl Button {
    pub fn new(icon_text: IconText, element_style: Style, colors: Colors) -> Self {
        Self {
            icon_text,
            element_style,
            colors,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct CurrentStyle(pub Style);
#[derive(Bundle)]
pub struct ButtonComponents {
    pub element_style: Style,
    pub colors: Colors,
    current_style: CurrentStyle,
    trigger: Trigger,
}
impl ButtonComponents {
    pub fn new(element_style: Style, colors: Colors) -> Self {
        Self {
            element_style,
            colors,
            current_style: CurrentStyle(element_style),
            trigger: Trigger::default(),
        }
    }
}
// only on configure side, in Scene input not a component
pub enum ButtonBacking {
    Rounded,
    Square,
}
#[derive(InnerSceneBinding)]
pub enum ButtonBindings {
    Panel,
    IconText,
    Interaction,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Scene for Button {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static Style,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static CurrentStyle,
            ),
            With<Tag<Button>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<Button>>>,
        Query<'static, 'static, &'static mut Style, Without<Tag<Button>>>,
        Query<
            'static,
            'static,
            (&'static mut IconColor, &'static mut TextColor),
            Without<Tag<Button>>,
        >,
    );
    type Filter = Or<(
        Changed<Style>,
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<CurrentStyle>,
    )>;
    type Components = ButtonComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let icon_text = bindings.get(ButtonBindings::IconText);
        let panel = bindings.get(ButtonBindings::Panel);
        if let Ok((_est, fc, bc, cs)) = ext.0.get(entity) {
            if _est.is_fill() {
                if cs.0.is_fill() {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    ext.3.get_mut(icon_text).unwrap().0 .0 = fc.0;
                    ext.3.get_mut(icon_text).unwrap().1 .0 = fc.0;
                } else {
                    ext.3.get_mut(icon_text).unwrap().0 .0 = bc.0;
                    ext.3.get_mut(icon_text).unwrap().1 .0 = bc.0;
                    *ext.1.get_mut(panel).unwrap() = fc.0;
                }
            } else {
                *ext.2.get_mut(panel).unwrap() = cs.0;
                if cs.0.is_fill() {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    ext.3.get_mut(icon_text).unwrap().0 .0 = fc.0;
                    ext.3.get_mut(icon_text).unwrap().1 .0 = fc.0;
                } else {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    ext.3.get_mut(icon_text).unwrap().0 .0 = bc.0;
                    ext.3.get_mut(icon_text).unwrap().1 .0 = bc.0;
                }
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let aspect = self.icon_text.max_chars.mono_aspect().value() * 1.25;
        binder.extend(binder.root(), Tag::<ButtonInteractionHook>::new());
        binder.bind(
            ButtonBindings::Panel,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Left),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Panel::new(self.element_style, self.colors.foreground.0),
        );
        binder.bind_scene(
            ButtonBindings::IconText,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.0.percent_from(RelativeMarker::Center),
                0.9.percent_of(AnchorDim::Width),
                0.8.percent_of(AnchorDim::Height),
            ),
            self.icon_text,
        );
        binder.bind(
            ButtonBindings::Interaction,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Left),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            BlankNode::default()
                .extend(InteractionListener::default())
                .extend(Tag::<ButtonInteractionHook>::new()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().min_height(24.0).min_width(30.0 * aspect),
            ButtonComponents::new(self.element_style, self.colors),
        ))
    }
}
#[derive(Component, Copy, Clone)]
pub(crate) struct ButtonInteractionHook();
fn interaction(
    mut buttons: Query<(&mut Trigger, &Style, &mut CurrentStyle), With<Tag<ButtonInteractionHook>>>,
    interaction_pane: Query<
        (&InteractionListener, &ScenePtr),
        (
            With<Tag<ButtonInteractionHook>>,
            Changed<InteractionListener>,
        ),
    >,
) {
    for (listener, ptr) in interaction_pane.iter() {
        if let Ok((mut trigger, est, mut cs)) = buttons.get_mut(ptr.value()) {
            if listener.engaged_start() {
                if est.is_fill() {
                    *cs = CurrentStyle(Style::ring());
                } else {
                    *cs = CurrentStyle(Style::fill());
                }
            }
            if listener.engaged_end() {
                *cs = CurrentStyle(*est);
            }
            if listener.active() {
                trigger.set(TriggerState::Active);
            }
        }
    }
}
impl Leaf for Button {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.enable_conditional_scene::<Button>();
        elm.main().add_systems((
            interaction.in_set(ExternalSet::InteractionTriggers),
            foliage_proper::scene::config::<Button>
                .in_set(Self::SetDescriptor::Update)
                .before(<IconText as Leaf>::SetDescriptor::Update),
        ));
    }
}