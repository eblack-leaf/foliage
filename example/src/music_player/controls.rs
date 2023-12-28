use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage::bevy_ecs::query::{Changed, With, Without};
use foliage::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage::button::ButtonStyle;
use foliage::circle::{Circle, CircleStyle, Diameter};
use foliage::circle_button::{CircleButton, CircleButtonArgs};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::{CoordinateUnit, InterfaceContext};
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon::{
    Bell, BellOff, CloudDrizzle, Pause, Play, SkipBack, SkipForward,
};
use foliage::icon::{Icon, IconId};
use foliage::rectangle::Rectangle;
use foliage::scene::align::{SceneAligner, SceneAlignment};
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::texture::factors::Progress;
use foliage::{bevy_ecs, scene_bind_enable, set_descriptor};

#[derive(Bundle)]
pub struct Controls {
    tag: Tag<Self>,
}
pub enum ControlBindings {
    Left,
    Play,
    Right,
}
impl From<ControlBindings> for SceneBinding {
    fn from(value: ControlBindings) -> Self {
        SceneBinding(value as i32)
    }
}
fn control_positions(area: Area<InterfaceContext>) -> CoordinateUnit {
    let middle = area.width / 2f32;
    middle - 24f32 - 4f32 - 12f32 - 12f32 - 12f32
}
set_descriptor!(
    pub enum ControlsSet {
        Area,
    }
);
impl Leaf for Controls {
    type SetDescriptor = ControlsSet;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, ControlsSet::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            (resize
                .in_set(ExternalSet::Configure)
                .in_set(ControlsSet::Area)
                .before(<Circle as Leaf>::SetDescriptor::Area)
                .before(<Icon as Leaf>::SetDescriptor::Area)),
        );
        scene_bind_enable!(elm, Controls);
    }
}
fn resize(
    controls: Query<
        (&SceneHandle, &Area<InterfaceContext>),
        (Changed<Area<InterfaceContext>>, With<Tag<Controls>>),
    >,
    mut rectangle_area: Query<&mut Area<InterfaceContext>, Without<Tag<Controls>>>,
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, area) in controls.iter() {
        let offset = control_positions(*area);
        coordinator
            .get_alignment_mut(&handle.access_chain().target(ControlBindings::Left))
            .pos
            .horizontal = offset.near();
        coordinator
            .get_alignment_mut(&handle.access_chain().target(ControlBindings::Right))
            .pos
            .horizontal = offset.far();
        let left_rectangle = coordinator.binding_entity(&handle.access_chain().target(3));
        rectangle_area.get_mut(left_rectangle).unwrap().width = offset - 24f32;
        coordinator.update_anchor_area(*handle, *area);
    }
}
impl Scene for Controls {
    type Bindings = ControlBindings;
    type Args<'a> = ();
    type ExternalArgs = ();
    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        _args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let offset = control_positions(anchor.0.section.area);
        binder.bind(
            3,
            (12.near(), 0.center(), 0),
            Rectangle::new(
                (offset - 24f32, 3f32).into(),
                Color::GREEN_MEDIUM.into(),
                Progress::full(),
            ),
            cmd,
        );
        binder.bind_scene::<CircleButton>(
            ControlBindings::Left.into(),
            SceneAlignment::from((offset.near(), 0.center(), 0)).into(),
            (24, 24).into(),
            &CircleButtonArgs::new(
                IconId::new(SkipBack),
                ButtonStyle::Ring,
                Color::GREEN_MEDIUM,
                Color::GREEN_MEDIUM,
            ),
            &(),
            cmd,
        );
        binder.bind_scene::<CircleButton>(
            ControlBindings::Play.into(),
            SceneAlignment::from((0.center(), 0.center(), 0)).into(),
            (48, 48).into(),
            &CircleButtonArgs::new(
                IconId::new(CloudDrizzle),
                ButtonStyle::Fill,
                Color::OFF_BLACK,
                Color::GREEN_MEDIUM,
            ),
            &(),
            cmd,
        );
        binder.bind_scene::<CircleButton>(
            ControlBindings::Right.into(),
            SceneAlignment::from((offset.far(), 0.center(), 0)).into(),
            (24, 24).into(),
            &CircleButtonArgs::new(
                IconId::new(SkipForward),
                ButtonStyle::Ring,
                Color::GREEN_MEDIUM,
                Color::GREEN_MEDIUM,
            ),
            &(),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}
