use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage::bevy_ecs::query::{Changed, With};
use foliage::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage::button::ButtonStyle;
use foliage::circle::Circle;
use foliage::circle_button::{CircleButton, CircleButtonArgs};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::{CoordinateUnit, InterfaceContext};
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon::{Play, SkipBack, SkipForward};
use foliage::icon::{Icon, IconId};
use foliage::scene::align::{SceneAligner, SceneAlignment};
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
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
fn control_positions(area: Area<InterfaceContext>) -> (CoordinateUnit, CoordinateUnit) {
    let half_biased_left = area.width / 4f32 - 24f32;
    let half_biased_right = half_biased_left * 3f32 - 24f32;
    (half_biased_left, half_biased_right)
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
    mut coordinator: ResMut<SceneCoordinator>,
) {
    for (handle, area) in controls.iter() {
        let (left, right) = control_positions(*area);
        coordinator
            .get_alignment_mut(&handle.access_chain().target(ControlBindings::Left))
            .pos
            .horizontal = left.near();
        coordinator
            .get_alignment_mut(&handle.access_chain().target(ControlBindings::Right))
            .pos
            .horizontal = left.far();
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
        let (left, right) = control_positions(anchor.0.section.area);
        binder.bind_scene::<CircleButton>(
            ControlBindings::Left.into(),
            SceneAlignment::from((left.near(), 0.center(), 0)).into(),
            (28, 28).into(),
            &CircleButtonArgs::new(
                IconId::new(SkipBack),
                ButtonStyle::Ring,
                Color::GREEN,
                Color::GREEN,
            ),
            &(),
            cmd,
        );
        binder.bind_scene::<CircleButton>(
            ControlBindings::Play.into(),
            SceneAlignment::from((0.center(), 0.center(), 0)).into(),
            (48, 48).into(),
            &CircleButtonArgs::new(
                IconId::new(Play),
                ButtonStyle::Ring,
                Color::GREEN,
                Color::GREEN,
            ),
            &(),
            cmd,
        );
        binder.bind_scene::<CircleButton>(
            ControlBindings::Right.into(),
            SceneAlignment::from((right.far(), 0.center(), 0)).into(),
            (28, 28).into(),
            &CircleButtonArgs::new(
                IconId::new(SkipForward),
                ButtonStyle::Ring,
                Color::GREEN,
                Color::GREEN,
            ),
            &(),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}