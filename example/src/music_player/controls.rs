use crate::music_player::track_progress::{TrackEvent, TrackLength};
use foliage::bevy_ecs::event::EventWriter;
use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage::bevy_ecs::query::{Changed, With, Without};
use foliage::bevy_ecs::system::{Query, ResMut, SystemParamItem};
use foliage::circle::Circle;
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::{CoordinateUnit, InterfaceContext};
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::{BundleExtend, Elm};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::{Icon, IconId, IconScale};
use foliage::interaction::InteractionListener;
use foliage::rectangle::Rectangle;
use foliage::scene::align::{SceneAligner, SceneAlignment};
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::texture::factors::Progress;
use foliage::{bevy_ecs, scene_bind_enable, set_descriptor};
use std::time::Duration;

#[derive(Bundle)]
pub struct Controls {
    tag: Tag<Self>,
}
pub enum ControlBindings {
    LeftBar,
    Left,
    Play,
    Right,
    RightBar,
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
        elm.main().add_systems((
            resize
                .in_set(ExternalSet::Configure)
                .in_set(ControlsSet::Area)
                .before(<Circle as Leaf>::SetDescriptor::Area)
                .before(<Icon as Leaf>::SetDescriptor::Area),
            with_play_hook.in_set(ExternalSet::Process),
        ));
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
        let left_rectangle =
            coordinator.binding_entity(&handle.access_chain().target(ControlBindings::LeftBar));
        rectangle_area.get_mut(left_rectangle).unwrap().width = offset - 48f32;
        let right_rectangle =
            coordinator.binding_entity(&handle.access_chain().target(ControlBindings::RightBar));
        rectangle_area.get_mut(right_rectangle).unwrap().width = offset - 48f32;
        coordinator.update_anchor_area(*handle, *area);
    }
}
pub struct PlayHook();
fn with_play_hook(
    button: Query<&InteractionListener, With<Tag<PlayHook>>>,
    mut events: EventWriter<TrackEvent>,
) {
    for listener in button.iter() {
        if listener.active() {
            events.send(TrackEvent {
                length: TrackLength(Duration::from_secs(18)),
            });
        }
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
            ControlBindings::LeftBar,
            (24.near(), 0.center(), 0),
            Rectangle::new(
                (offset - 48f32, 3f32).into(),
                Color::GREY_MEDIUM.into(),
                Progress::full(),
            ),
            cmd,
        );
        binder.bind(
            ControlBindings::Left,
            SceneAlignment::from((offset.near(), 0.center(), 0)),
            Icon::new(
                IconId::new(BundledIcon::SkipBack),
                IconScale::from_dim(24f32),
                Color::GREEN_MEDIUM.into(),
            ),
            cmd,
        );
        binder.bind(
            ControlBindings::Play,
            SceneAlignment::from((0.center(), 0.center(), 0)),
            Icon::new(
                IconId::new(BundledIcon::Play),
                IconScale::from_dim(48f32),
                Color::GREEN_MEDIUM.into(),
            )
            .extend(InteractionListener::default())
            .extend(Tag::<PlayHook>::new()),
            cmd,
        );
        binder.bind(
            ControlBindings::Right,
            SceneAlignment::from((offset.far(), 0.center(), 0)),
            Icon::new(
                IconId::new(BundledIcon::SkipForward),
                IconScale::from_dim(24f32),
                Color::GREEN_MEDIUM.into(),
            ),
            cmd,
        );
        binder.bind(
            ControlBindings::RightBar,
            (24.far(), 0.center(), 0),
            Rectangle::new(
                (offset - 48f32, 3f32).into(),
                Color::GREY_MEDIUM.into(),
                Progress::full(),
            ),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}
