use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::elm::config::ElmConfiguration;
use crate::elm::leaf::{EmptySetDescriptor, Leaf, Tag};
use crate::elm::Elm;
use crate::icon::{Icon, IconId, IconScale};
use crate::panel::{Panel, PanelStyle};
use crate::prebuilt::button::{BackgroundColor, Button, ButtonStyle, ForegroundColor};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};

#[derive(Bundle)]
pub struct IconButton {
    tag: Tag<Self>,
    style: ButtonStyle,
    foreground_color: ForegroundColor,
    background_color: BackgroundColor,
}
pub enum IconButtonBindings {
    Panel,
    Icon,
}
impl From<IconButtonBindings> for SceneBinding {
    fn from(value: IconButtonBindings) -> Self {
        Self(value as i32)
    }
}
pub struct IconButtonArgs {
    style: ButtonStyle,
    foreground_color: Color,
    background_color: Color,
    id: IconId,
}
impl IconButtonArgs {
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
    mut panels: Query<&mut Area<InterfaceContext>, Without<Tag<IconButton>>>,
    mut icons: Query<&mut IconScale>,
    mut colors: Query<&mut Color>,
) {
    for (handle, area, foreground, background, style) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let iac = handle.access_chain().target(IconButtonBindings::Icon);
        let pac = handle.access_chain().target(IconButtonBindings::Panel);
        let panel = coordinator.binding_entity(&pac);
        *panels.get_mut(panel).unwrap() = *area;
        let icon = coordinator.binding_entity(&iac);
        *icons.get_mut(icon).unwrap() = IconScale::from_dim(area.height * 0.9);
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
        elm.main()
            .add_systems((resize.in_set(<Button as Leaf>::SetDescriptor::Area),));
    }
}
impl Scene for IconButton {
    type Bindings = IconButtonBindings;
    type Args<'a> = IconButtonArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            Self::Bindings::Panel,
            (0.near(), 0.near(), 1),
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
        binder.bind(
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
        Self {
            tag: Tag::new(),
            style: args.style,
            foreground_color: ForegroundColor(args.foreground_color),
            background_color: BackgroundColor(args.background_color),
        }
    }
}
