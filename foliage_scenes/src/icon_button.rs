use crate::button::{BackgroundColor, BaseStyle, Button, ButtonStyle, ForegroundColor};
use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage::bevy_ecs::query::{Changed, Or, With, Without};
use foliage::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::differential::Despawn;
use foliage::elm::config::ElmConfiguration;
use foliage::elm::leaf::{EmptySetDescriptor, Leaf, Tag};
use foliage::elm::Elm;
use foliage::icon::{Icon, IconId, IconScale};
use foliage::interaction::InteractionListener;
use foliage::panel::{Panel, PanelStyle};
use foliage::scene::align::SceneAligner;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage::SceneBinding;

#[derive(Bundle)]
pub struct IconButtonComponents {
    tag: Tag<Self>,
    style: ButtonStyle,
    base: BaseStyle,
    foreground_color: ForegroundColor,
    background_color: BackgroundColor,
}
#[derive(SceneBinding)]
pub enum IconButtonBindings {
    Panel,
    Icon,
}
#[derive(Clone)]
pub struct IconButton {
    style: ButtonStyle,
    foreground_color: Color,
    background_color: Color,
    id: IconId,
}
impl IconButton {
    pub fn new<C: Into<Color>, ID: Into<IconId>>(id: ID, style: ButtonStyle, fc: C, bc: C) -> Self {
        Self {
            style,
            foreground_color: fc.into(),
            background_color: bc.into(),
            id: id.into(),
        }
    }
}
fn resize(
    scenes: Query<
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &ForegroundColor,
            &BackgroundColor,
            &ButtonStyle,
            &Despawn,
        ),
        (
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<ForegroundColor>,
                Changed<BackgroundColor>,
                Changed<ButtonStyle>,
            )>,
            With<Tag<IconButton>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut panels: Query<&mut PanelStyle, Without<Tag<IconButton>>>,
    mut areas: Query<&mut Area<InterfaceContext>, Without<Tag<IconButton>>>,
    mut colors: Query<&mut Color>,
) {
    for (handle, area, foreground, background, style, despawn) in scenes.iter() {
        if despawn.should_despawn() {
            continue;
        }
        coordinator.update_anchor_area(*handle, *area);
        let iac = handle.access_chain().target(IconButtonBindings::Icon);
        let pac = handle.access_chain().target(IconButtonBindings::Panel);
        let panel = coordinator.binding_entity(&pac);
        *areas.get_mut(panel).unwrap() = *area;
        *panels.get_mut(panel).unwrap() = match style {
            ButtonStyle::Ring => PanelStyle::ring(),
            ButtonStyle::Fill => PanelStyle::fill(),
        };
        let icon = coordinator.binding_entity(&iac);
        areas.get_mut(icon).unwrap().width = area.height * 0.9;
        match style {
            ButtonStyle::Ring => {
                *colors.get_mut(icon).unwrap() = foreground.0;
            }
            ButtonStyle::Fill => {
                *colors.get_mut(icon).unwrap() = background.0;
            }
        }
    }
}
impl Leaf for IconButton {
    type SetDescriptor = EmptySetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((resize
            .in_set(<Button as Leaf>::SetDescriptor::Area)
            .before(<Panel as Leaf>::SetDescriptor::Area),));
    }
}
impl Scene for IconButton {
    type Bindings = IconButtonBindings;
    type Components = IconButtonComponents;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        cmd.entity(binder.this())
            .insert(InteractionListener::default());
        let entity = binder.bind(
            Self::Bindings::Panel,
            (0.from_left(), 0.from_left(), 1),
            Panel::new(
                match args.style {
                    ButtonStyle::Ring => PanelStyle::ring(),
                    ButtonStyle::Fill => PanelStyle::fill(),
                },
                anchor.0.section.area,
                args.foreground_color,
            ),
            cmd,
        );
        tracing::trace!("binding-icon-button-panel: {:?}", entity);
        let entity = binder.bind(
            Self::Bindings::Icon,
            (0.center(), 0.center(), 0),
            Icon::new(
                args.id,
                IconScale::from_dim(anchor.0.section.area.height * 0.9),
                match args.style {
                    ButtonStyle::Ring => args.foreground_color,
                    ButtonStyle::Fill => args.background_color,
                },
            ),
            cmd,
        );
        tracing::trace!("binding-icon-button-icon: {:?}", entity);
        Self::Components {
            tag: Tag::new(),
            style: args.style,
            base: BaseStyle(args.style),
            foreground_color: ForegroundColor(args.foreground_color),
            background_color: BackgroundColor(args.background_color),
        }
    }
}