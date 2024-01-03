use crate::circle::{Circle, CircleStyle, Diameter};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::elm::config::ElmConfiguration;
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::icon::{Icon, IconId, IconScale};
use crate::prebuilt::button::{BackgroundColor, Button, ButtonStyle, ForegroundColor};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use crate::scene_bind_enable;
use crate::texture::factors::Progress;
use bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};

#[derive(Bundle)]
pub struct CircleButton {
    tag: Tag<Self>,
    style: ButtonStyle,
    foreground_color: ForegroundColor,
    background_color: BackgroundColor,
}
impl Leaf for CircleButton {
    type SetDescriptor = <Button as Leaf>::SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((resize
            .in_set(<Button as Leaf>::SetDescriptor::Area)
            .before(<Circle as Leaf>::SetDescriptor::Area)
            .before(<Icon as Leaf>::SetDescriptor::Area),));
        scene_bind_enable!(elm, CircleButton);
    }
}
fn resize(
    scenes: Query<
        (&SceneHandle, &Area<InterfaceContext>, &ButtonStyle, &ForegroundColor, &BackgroundColor),
        (Or<(Changed<Area<InterfaceContext>>, Changed<ButtonStyle>)>, With<Tag<CircleButton>>),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut icons: Query<&mut IconScale>,
    mut circles: Query<(&mut Diameter, &mut CircleStyle)>,
    mut colors: Query<&mut Color>,
) {
    tracing::trace!("updating-circle-buttons");
    for (handle, area, style, foreground, background) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let circle =
            coordinator.binding_entity(&handle.access_chain().target(CircleButtonBindings::Circle));
        let icon =
            coordinator.binding_entity(&handle.access_chain().target(CircleButtonBindings::Icon));
        match style {
            ButtonStyle::Ring => {
                *colors.get_mut(icon).unwrap() = foreground.0;
            }
            ButtonStyle::Fill => {
                *colors.get_mut(icon).unwrap() = background.0;
            }
        }
        let cs = match style {
            ButtonStyle::Ring => CircleStyle::ring(),
            ButtonStyle::Fill => CircleStyle::fill(),
        };
        *circles.get_mut(circle).unwrap().1 = cs;
        *icons.get_mut(icon).unwrap() = IconScale::from_dim(area.width * 0.8);
        circles.get_mut(circle).unwrap().0.0 = area.width;
    }
}
pub enum CircleButtonBindings {
    Circle,
    Icon,
}
impl From<CircleButtonBindings> for SceneBinding {
    fn from(value: CircleButtonBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct CircleButtonArgs {
    pub icon_id: IconId,
    pub style: ButtonStyle,
    pub color: Color,
    pub back_color: Color,
}
impl CircleButtonArgs {
    pub fn new<I: Into<IconId>, BS: Into<ButtonStyle>, C: Into<Color>>(
        id: I,
        bs: BS,
        c: C,
        bc: C,
    ) -> Self {
        Self {
            icon_id: id.into(),
            style: bs.into(),
            color: c.into(),
            back_color: bc.into(),
        }
    }
}
impl Scene for CircleButton {
    type Bindings = CircleButtonBindings;
    type Args<'a> = CircleButtonArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let style = match args.style {
            ButtonStyle::Ring => CircleStyle::ring(),
            ButtonStyle::Fill => CircleStyle::fill(),
        };
        binder.bind(
            CircleButtonBindings::Circle,
            (0.near(), 0.near(), 1),
            Circle::new(
                style,
                Diameter::new(anchor.0.section.width()),
                args.color,
                Progress::full(),
            ),
            cmd,
        );
        binder.bind(
            CircleButtonBindings::Icon,
            (0.center(), 0.center(), 0),
            Icon::new(
                args.icon_id,
                IconScale::from_dim(anchor.0.section.width() * 0.8),
                match args.style {
                    ButtonStyle::Ring => {args.color}
                    ButtonStyle::Fill => {args.back_color}
                },
            ),
            cmd,
        );
        Self { tag: Tag::new(), foreground_color: ForegroundColor(args.color), background_color: BackgroundColor(args.back_color), style: args.style }
    }
}