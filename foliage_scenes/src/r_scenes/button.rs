use crate::r_scenes::icon_text::{IconColor, IconText, TextColor};
use crate::r_scenes::{BackgroundColor, ForegroundColor};
use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Changed, Commands, Component, IntoSystemConfigs, Or};
use foliage_proper::bevy_ecs::query::{With, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, ElementStyle, Elm};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    Alignment, AlignmentDesc, AnchorDim, MicroGrid, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, ScenePtr};
#[derive(Clone)]
pub struct Button {
    pub icon_text: IconText,
    pub element_style: ElementStyle,
    pub foreground_color: Color,
    pub background_color: Color,
}
impl Button {
    pub fn new<C: Into<Color>>(
        icon_text: IconText,
        element_style: ElementStyle,
        foreground_color: C,
        background_color: C,
    ) -> Self {
        Self {
            icon_text,
            element_style,
            foreground_color: foreground_color.into(),
            background_color: background_color.into(),
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct CurrentStyle(pub ElementStyle);
#[derive(Bundle)]
pub struct ButtonComponents {
    pub element_style: ElementStyle,
    pub foreground_color: ForegroundColor,
    pub background_color: BackgroundColor,
    current_style: CurrentStyle,
    trigger: Trigger,
}
impl ButtonComponents {
    pub fn new<C: Into<Color>>(
        element_style: ElementStyle,
        foreground_color: C,
        background_color: C,
    ) -> Self {
        Self {
            element_style,
            foreground_color: ForegroundColor(foreground_color.into()),
            background_color: BackgroundColor(background_color.into()),
            current_style: CurrentStyle(element_style),
            trigger: Trigger::default(),
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum ButtonBindings {
    Panel,
    IconText,
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
                &'static ElementStyle,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static CurrentStyle,
            ),
            With<Tag<Button>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<Button>>>,
        Query<'static, 'static, &'static mut ElementStyle, Without<Tag<Button>>>,
        Query<
            'static,
            'static,
            (&'static mut IconColor, &'static mut TextColor),
            Without<Tag<Button>>,
        >,
    );
    type Filter = Or<(
        Changed<ElementStyle>,
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<CurrentStyle>,
    )>;
    type Components = ButtonComponents;

    fn config(
        entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        let icon_text = bindings.get(ButtonBindings::IconText);
        let panel = bindings.get(ButtonBindings::Panel);
        if let Ok((_est, fc, bc, cs)) = ext.0.get(entity) {
            *ext.1.get_mut(panel).unwrap() = fc.0;
            *ext.2.get_mut(panel).unwrap() = cs.0;
            if cs.0.is_fill() {
                ext.3.get_mut(icon_text).unwrap().0 .0 = bc.0;
                ext.3.get_mut(icon_text).unwrap().1 .0 = bc.0;
            } else {
                ext.3.get_mut(icon_text).unwrap().0 .0 = fc.0;
                ext.3.get_mut(icon_text).unwrap().1 .0 = fc.0;
            }
        }
    }

    fn create(self, cmd: &mut Commands) -> Entity {
        let mut binder = Binder::new(cmd);
        let aspect = (self.icon_text.max_chars.0 as f32 + 4f32) / 2f32;
        binder.bind(
            ButtonBindings::Panel,
            Alignment::new(
                0.fixed_from(RelativeMarker::Left),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Panel::new(self.element_style, self.foreground_color),
            cmd,
        );
        binder.bind_scene(
            ButtonBindings::IconText,
            Alignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                0.8.percent_of(AnchorDim::Width),
                0.8.percent_of(AnchorDim::Height),
            ),
            self.icon_text,
            cmd,
        );
        binder.bind(
            2,
            Alignment::new(
                0.fixed_from(RelativeMarker::Left),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            InteractionListener::default()
                .extend(Tag::<ButtonInteractionHook>::new())
                .extend(Coordinate::<InterfaceContext>::default()),
            cmd,
        );
        binder.finish::<Self>(
            SceneComponents::new(
                MicroGrid::new()
                    .min_height(24.0)
                    .min_width(24.0 * aspect)
                    .aspect(aspect),
                ButtonComponents::new(
                    self.element_style,
                    self.foreground_color,
                    self.background_color,
                ),
            ),
            cmd,
        )
    }
}
#[derive(Component, Copy, Clone)]
struct ButtonInteractionHook();
fn interaction(
    mut buttons: Query<(&mut Trigger, &ElementStyle, &mut CurrentStyle), With<Tag<Button>>>,
    interaction_pane: Query<
        (&InteractionListener, &ScenePtr),
        (
            Without<Tag<Button>>,
            With<Tag<ButtonInteractionHook>>,
            Changed<InteractionListener>,
        ),
    >,
) {
    for (listener, ptr) in interaction_pane.iter() {
        if let Ok((mut trigger, est, mut cs)) = buttons.get_mut(ptr.value()) {
            if listener.engaged_start() {
                if est.is_fill() {
                    *cs = CurrentStyle(ElementStyle::ring());
                } else {
                    *cs = CurrentStyle(ElementStyle::fill());
                }
            }
            if listener.engaged_end() {
                *cs = CurrentStyle(*est);
            }
            if listener.active() {
                trigger.set();
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
        elm.main().add_systems((
            interaction.in_set(ExternalSet::InteractionTriggers),
            foliage_proper::scene::config::<Button>
                .in_set(Self::SetDescriptor::Update)
                .before(<IconText as Leaf>::SetDescriptor::Update),
        ));
    }
}
