use crate::circle::{Circle, CircleStyle, Diameter};
use crate::color::Color;
use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;
use crate::differential::Despawn;
use crate::elm::config::ElmConfiguration;
use crate::elm::leaf::{Leaf, Tag};
use crate::elm::Elm;
use crate::icon::{Icon, IconId, IconScale};
use crate::interaction::{InteractionListener, InteractionShape};
use crate::prebuilt::button::{
    BackgroundColor, BaseStyle, ButtonComponents, ButtonStyle, ForegroundColor,
};
use crate::scene::align::SceneAligner;
use crate::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use crate::texture::factors::Progress;
use bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use bevy_ecs::query::{Changed, Or, With, Without};
use bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_macros::SceneBinding;

#[derive(Bundle)]
pub struct CircleButtonComponents {
    tag: Tag<Self>,
    style: ButtonStyle,
    base: BaseStyle,
    foreground_color: ForegroundColor,
    background_color: BackgroundColor,
}
impl Leaf for CircleButtonComponents {
    type SetDescriptor = <ButtonComponents as Leaf>::SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {}

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((resize
            .in_set(<ButtonComponents as Leaf>::SetDescriptor::Area)
            .before(<Circle as Leaf>::SetDescriptor::Area)
            .before(<Icon as Leaf>::SetDescriptor::Area),));
    }
}
fn resize(
    scenes: Query<
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &ButtonStyle,
            &ForegroundColor,
            &BackgroundColor,
            &Despawn,
        ),
        (
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<ButtonStyle>,
                Changed<ForegroundColor>,
                Changed<BackgroundColor>,
            )>,
            With<Tag<CircleButtonComponents>>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut areas: Query<&mut Area<InterfaceContext>, Without<Tag<CircleButtonComponents>>>,
    mut circles: Query<&mut CircleStyle, Without<Tag<CircleButtonComponents>>>,
    mut colors: Query<&mut Color>,
) {
    for (handle, area, style, foreground, background, despawn) in scenes.iter() {
        if despawn.should_despawn() {
            continue;
        }
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
        *circles.get_mut(circle).unwrap() = cs;
        areas.get_mut(icon).unwrap().width = area.width * 0.7;
        *areas.get_mut(circle).unwrap() = *area;
    }
}
#[derive(SceneBinding)]
pub enum CircleButtonBindings {
    Circle,
    Icon,
}
#[derive(Clone)]
pub struct CircleButton {
    pub icon_id: IconId,
    pub style: ButtonStyle,
    pub color: Color,
    pub back_color: Color,
}
impl CircleButton {
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
    type Components = CircleButtonComponents;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: Self,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self::Components {
        let style = match args.style {
            ButtonStyle::Ring => CircleStyle::ring(),
            ButtonStyle::Fill => CircleStyle::fill(),
        };
        cmd.entity(binder.this())
            .insert(InteractionListener::default().with_shape(InteractionShape::Circle));
        binder.bind(
            CircleButtonBindings::Circle,
            (0.from_left(), 0.from_left(), 1),
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
                    ButtonStyle::Ring => args.color,
                    ButtonStyle::Fill => args.back_color,
                },
            ),
            cmd,
        );
        Self::Components {
            tag: Tag::new(),
            foreground_color: ForegroundColor(args.color),
            background_color: BackgroundColor(args.back_color),
            style: args.style,
            base: BaseStyle(args.style),
        }
    }
}
