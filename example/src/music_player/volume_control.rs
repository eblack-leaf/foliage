use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::prelude::{Commands, IntoSystemConfigs};
use foliage::bevy_ecs::query::{Changed, With, Without};
use foliage::bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::Elm;
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::{Icon, IconId, IconScale};
use foliage::prebuilt::interactive_progress_bar::{
    InteractiveProgressBar, InteractiveProgressBarArgs, ProgressPercent,
};
use foliage::scene::align::{SceneAligner, SceneAlignment};
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::{bevy_ecs, scene_bind_enable, set_descriptor};

#[derive(Bundle)]
pub struct VolumeControl {
    tag: Tag<Self>,
}
impl VolumeControl {
    const OFFSET: f32 = 28.0;
}
pub enum VolumeControlBindings {
    Icon,
    Slider,
}
impl From<VolumeControlBindings> for SceneBinding {
    fn from(value: VolumeControlBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct VolumeControlArgs {
    pub percent: f32,
    pub color: Color,
    pub back_color: Color,
}
impl VolumeControlArgs {
    pub fn new<C: Into<Color>>(percent: f32, c: C, bc: C) -> Self {
        Self {
            percent,
            color: c.into(),
            back_color: bc.into(),
        }
    }
}
set_descriptor!(
    pub enum VolumeControlSets {
        Area,
    }
);
impl Leaf for VolumeControl {
    type SetDescriptor = VolumeControlSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(((resize, change_icon)
            .chain()
            .in_set(Self::SetDescriptor::Area)
                                    .before(<InteractiveProgressBar as Leaf>::SetDescriptor::Area)
                                    .before(<Icon as Leaf>::SetDescriptor::Area),));
        scene_bind_enable!(elm, VolumeControl);
    }
}
fn change_icon(
    volume: Query<&SceneHandle, With<Tag<VolumeControl>>>,
    interactive_prog: Query<
        (&SceneHandle, &ProgressPercent),
        (Changed<ProgressPercent>, Without<Tag<VolumeControl>>),
    >,
    mut icons: Query<&mut IconId>,
    coordinator: Res<SceneCoordinator>,
) {
    for v in volume.iter() {
        let entity =
            coordinator.binding_entity(&v.access_chain().target(VolumeControlBindings::Slider));
        if let Ok((_handle, percent)) = interactive_prog.get(entity) {
            let id = if percent.0 < 0.05 {
                IconId::new(BundledIcon::VolumeX)
            } else if percent.0 < 0.33 {
                IconId::new(BundledIcon::Volume)
            } else if percent.0 < 0.67 {
                IconId::new(BundledIcon::VolumeOne)
            } else {
                IconId::new(BundledIcon::VolumeTwo)
            };
            let icon =
                coordinator.binding_entity(&v.access_chain().target(VolumeControlBindings::Icon));
            *icons.get_mut(icon).unwrap() = id;
        }
    }
}
fn resize(
    scenes: Query<
        (&SceneHandle, &Area<InterfaceContext>),
        (Changed<Area<InterfaceContext>>, With<Tag<VolumeControl>>),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut rectangles: Query<&mut Area<InterfaceContext>, Without<Tag<VolumeControl>>>,
) {
    for (handle, area) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let rect = coordinator
            .binding_entity(&handle.access_chain().target(VolumeControlBindings::Slider));
        rectangles.get_mut(rect).unwrap().width = area.width - VolumeControl::OFFSET;
    }
}
impl Scene for VolumeControl {
    type Bindings = VolumeControlBindings;
    type Args<'a> = VolumeControlArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            VolumeControlBindings::Icon,
            (0.near(), 0.center(), 0),
            Icon::new(
                IconId::new(BundledIcon::Volume),
                IconScale::from_dim(16f32),
                args.color,
            ),
            cmd,
        );
        binder.bind_scene::<InteractiveProgressBar>(
            VolumeControlBindings::Slider.into(),
            SceneAlignment::from((VolumeControl::OFFSET.near(), 0.center(), 0)),
            anchor.0.section.area - (VolumeControl::OFFSET, 0f32).into(),
            &InteractiveProgressBarArgs::new(args.percent, args.color, args.back_color),
            &(),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}