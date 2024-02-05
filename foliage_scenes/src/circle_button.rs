use crate::button::{BackgroundColor, BaseStyle, Button, ButtonStyle, ForegroundColor};
use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, Or, With, Without};
use foliage_proper::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage_proper::circle::{Circle, CircleStyle, Diameter};
use foliage_proper::color::Color;
use foliage_proper::coordinate::area::Area;
use foliage_proper::coordinate::InterfaceContext;
use foliage_proper::differential::Despawn;
use foliage_proper::elm::config::ElmConfiguration;
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::icon::{Icon, IconId, IconScale};
use foliage_proper::interaction::{InteractionListener, InteractionShape};
use foliage_proper::scene::align::SceneAligner;
use foliage_proper::scene::{Anchor, Scene, SceneBinder, SceneCoordinator, SceneHandle};
use foliage_proper::texture::factors::Progress;

#[derive(Bundle)]
pub struct CircleButtonComponents {
    tag: Tag<Self>,
    style: ButtonStyle,
    base: BaseStyle,
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
#[derive(InnerSceneBinding)]
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