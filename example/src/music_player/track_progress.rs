use crate::music_player::controls::{ControlBindings, Controls, CurrentTrack};
use foliage::bevy_ecs::component::Component;
use foliage::bevy_ecs::event::{Event, EventReader};
use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs, With, Without};
use foliage::bevy_ecs::query::{Changed, Or};
use foliage::bevy_ecs::system::{Query, Res, ResMut, Resource, SystemParamItem};
use foliage::circle::{Circle, CircleStyle, Diameter};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::{Elm, EventStage};
use foliage::icon::bundled_cov::BundledIcon;
use foliage::icon::IconId;
use foliage::prebuilt::progress_bar::{ProgressBar, ProgressBarArgs};
use foliage::scene::align::SceneAligner;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::text::font::MonospacedFont;
use foliage::text::{FontSize, MaxCharacters, Text, TextValue};
use foliage::texture::factors::Progress;
use foliage::window::ScaleFactor;
use foliage::{bevy_ecs, scene_bind_enable, set_descriptor};
use std::time::{Duration, Instant};

#[derive(Bundle)]
pub struct TrackProgress {
    tag: Tag<Self>,
}
#[derive(Resource, Default)]
pub struct TrackPlayer {
    pub playing: bool,
    pub current: Duration,
    pub last: Option<Instant>,
    pub ratio: f32,
    pub length: Duration,
    pub(crate) done: bool,
}
set_descriptor!(
    pub enum TrackProgressSet {
        Area,
    }
);
impl Leaf for TrackProgress {
    type SetDescriptor = TrackProgressSet;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, TrackProgressSet::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.startup().add_systems((setup,));
        elm.main().add_systems(
            (
                read_track_event
                    .in_set(ExternalSet::Configure)
                    .in_set(TrackProgressSet::Area),
                config_track_progress
                    .in_set(ExternalSet::Configure)
                    .in_set(TrackProgressSet::Area)
                    .before(<ProgressBar as Leaf>::SetDescriptor::Area)
                    .before(<Text as Leaf>::SetDescriptor::Area),
                config_track_time
                    .in_set(ExternalSet::Configure)
                    .in_set(TrackProgressSet::Area)
                    .before(<ProgressBar as Leaf>::SetDescriptor::Area)
                    .before(<Text as Leaf>::SetDescriptor::Area),
            )
                .chain(),
        );
        scene_bind_enable!(elm, TrackProgress);
        elm.add_event::<TrackEvent>(EventStage::Process);
        elm.add_event::<TrackPlayEvent>(EventStage::Process);
    }
}
fn setup(mut cmd: Commands) {
    cmd.insert_resource(TrackPlayer::default());
    cmd.insert_resource(CurrentTrack::default());
}
pub enum TrackProgressBindings {
    Progress,
    TrackTime,
}
impl From<TrackProgressBindings> for SceneBinding {
    fn from(value: TrackProgressBindings) -> Self {
        SceneBinding(value as i32)
    }
}
fn config_track_progress(
    scenes: Query<
        (&SceneHandle, &Area<InterfaceContext>),
        (With<Tag<TrackProgress>>, Changed<Area<InterfaceContext>>),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut prog_areas: Query<&mut Area<InterfaceContext>, Without<Tag<TrackProgress>>>,
    player: Res<TrackPlayer>,
) {
    for (handle, area) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let prog_entity = coordinator.binding_entity(
            &handle
                .access_chain()
                .target(TrackProgressBindings::Progress),
        );
        prog_areas.get_mut(prog_entity).unwrap().width = area.width;
        let tt_chain = handle
            .access_chain()
            .binding(TrackProgressBindings::TrackTime);
        coordinator.get_alignment_mut(&tt_chain).pos.horizontal =
            ((area.width * player.ratio).round() - 24f32).near();
    }
}
pub struct TrackProgressArgs {
    pub fill_color: Color,
    pub back_color: Color,
}
#[derive(Event, Copy, Clone)]
pub struct TrackPlayEvent(pub(crate) bool);
impl TrackPlayEvent {
    pub fn play() -> Self {
        Self(false)
    }
    pub fn pause() -> Self {
        Self(true)
    }
}
fn read_track_event(
    mut pause_events: EventReader<TrackPlayEvent>,
    mut events: EventReader<TrackEvent>,
    mut scenes: Query<(&SceneHandle, &Area<InterfaceContext>), With<Tag<TrackProgress>>>,
    control: Query<&SceneHandle, With<Tag<Controls>>>,
    mut player: ResMut<TrackPlayer>,
    mut coordinator: ResMut<SceneCoordinator>,
    mut text_vals: Query<&mut TextValue>,
    mut progresses: Query<&mut Progress>,
    mut icons: Query<&mut IconId>,
    mut current_track: ResMut<CurrentTrack>,
) {
    for event in pause_events.read() {
        if player.done {
            if let Some(track) = current_track.0.as_ref() {
                player.current = Duration::default();
                player.last.replace(Instant::now());
                player.length = track.length;
                player.ratio = 0.0;
                player.done = false;
                for (handle, area) in scenes.iter() {
                    coordinator
                        .get_alignment_mut(
                            &handle
                                .access_chain()
                                .target(TrackProgressBindings::TrackTime),
                        )
                        .pos
                        .horizontal = ((area.width * player.ratio).round() - 24f32).near();
                    let prog_ac = handle
                        .access_chain()
                        .target(TrackProgressBindings::Progress);
                    let progress = Progress::new(0.0, player.ratio);
                    let prog = coordinator.binding_entity(&prog_ac);
                    *progresses.get_mut(prog).unwrap() = progress;
                }
            }
        }
        player.playing = !event.0;
    }
    for (handle, area) in scenes.iter_mut() {
        if player.playing && !player.done {
            let diff = Instant::now() - player.last.unwrap();
            player.last.replace(Instant::now());
            player.current += diff;
            if player.current >= player.length {
                player.playing = false;
                player.current = player.length;
                player.done = true;
                for h in control.iter() {
                    let entity =
                        coordinator.binding_entity(&h.access_chain().target(ControlBindings::Play));
                    *icons.get_mut(entity).unwrap() = IconId::new(BundledIcon::Play);
                }
            }
            player.ratio = player.current.as_nanos() as f32 / player.length.as_nanos() as f32;
            let time_text = coordinator.binding_entity(
                &handle
                    .access_chain()
                    .binding(TrackProgressBindings::TrackTime)
                    .target(TrackTimeBindings::TimeText),
            );
            let t_val = format!(
                "{:02}:{:02}",
                (player.current.as_secs_f32() / 60f32).floor(),
                (player.current.as_secs_f32() % 60f32).floor()
            );
            *text_vals.get_mut(time_text).unwrap() = TextValue::new(t_val);
            let prog_ac = handle
                .access_chain()
                .target(TrackProgressBindings::Progress);
            let prog = coordinator.binding_entity(&prog_ac);
            let progress = Progress::new(0.0, player.ratio);
            *progresses.get_mut(prog).unwrap() = progress;
            coordinator
                .get_alignment_mut(
                    &handle
                        .access_chain()
                        .target(TrackProgressBindings::TrackTime),
                )
                .pos
                .horizontal = ((area.width * player.ratio).round() - 24f32).near();
        } else {
            // forward last to keep in sync
            player.last.replace(Instant::now());
            // timer will forward for me as i call .elapsed()
        }
    }
    for event in events.read() {
        current_track.0.replace(event.clone());
        player.current = Duration::default();
        player.last.replace(Instant::now());
        player.length = event.length;
        player.ratio = 0.0;
        player.done = false;
        for (handle, area) in scenes.iter() {
            coordinator
                .get_alignment_mut(
                    &handle
                        .access_chain()
                        .target(TrackProgressBindings::TrackTime),
                )
                .pos
                .horizontal = ((area.width * player.ratio).round() - 24f32).near();
            let prog_ac = handle
                .access_chain()
                .target(TrackProgressBindings::Progress);
            let progress = Progress::new(0.0, player.ratio);
            let prog = coordinator.binding_entity(&prog_ac);
            *progresses.get_mut(prog).unwrap() = progress;
        }
    }
}
#[derive(Event, Clone)]
pub struct TrackEvent {
    pub length: Duration,
}
impl TrackProgressArgs {
    pub fn new<C: Into<Color>>(fill: C, back: C) -> Self {
        Self {
            fill_color: fill.into(),
            back_color: back.into(),
        }
    }
}
impl Scene for TrackProgress {
    type Bindings = TrackProgressBindings;
    type Args<'a> = TrackProgressArgs;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind_scene::<ProgressBar>(
            TrackProgressBindings::Progress.into(),
            (0.near(), 0.center(), 1).into(),
            (anchor.0.section.width(), 4f32).into(),
            &ProgressBarArgs::new(Progress::empty(), args.fill_color, args.back_color),
            &(),
            cmd,
        );
        binder.bind_scene::<TrackTime>(
            TrackProgressBindings::TrackTime.into(),
            ((-24).near(), 12.center(), 0).into(),
            (48, 48).into(),
            &TrackTimeArgs {
                color: args.fill_color,
            },
            external_args,
            cmd,
        );
        Self { tag: Tag::new() }
    }
}
fn config_track_time(
    scenes: Query<
        (&SceneHandle, &Area<InterfaceContext>),
        (With<Tag<TrackTime>>, Changed<Area<InterfaceContext>>),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut rectangles: Query<
        &mut Area<InterfaceContext>,
        (Without<Tag<TrackTime>>, Without<FontSize>),
    >,
    mut text: Query<(&mut Area<InterfaceContext>, &mut FontSize), Without<Tag<TrackTime>>>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
) {
    for (handle, area) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        coordinator
            .get_alignment_mut(&handle.access_chain().target(TrackTimeBindings::Marker))
            .pos
            .vertical = (area.height / 4f32 - 6f32).near();
        let marker =
            coordinator.binding_entity(&handle.access_chain().target(TrackTimeBindings::Marker));
        let time_text =
            coordinator.binding_entity(&handle.access_chain().target(TrackTimeBindings::TimeText));
        rectangles.get_mut(marker).unwrap().height = area.height / 2f32;
        let (fs, a) = font.best_fit(MaxCharacters(5), *area / (1, 2).into(), &scale_factor);
        *text.get_mut(time_text).unwrap().1 = fs;
        *text.get_mut(time_text).unwrap().0 = a;
    }
}
#[derive(Bundle)]
pub struct TrackTime {
    tag: Tag<Self>,
}
pub enum TrackTimeBindings {
    Marker,
    TimeText,
}
impl From<TrackTimeBindings> for SceneBinding {
    fn from(value: TrackTimeBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct TrackTimeArgs {
    pub color: Color,
}
impl Scene for TrackTime {
    type Bindings = TrackTimeBindings;
    type Args<'a> = TrackTimeArgs;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            TrackTimeBindings::Marker,
            (
                0.center(),
                (anchor.0.section.height() / 4f32 - 6f32).near(),
                0,
            ),
            Circle::new(
                CircleStyle::fill(),
                Diameter::new(12f32),
                args.color,
                Progress::full(),
            ),
            // Rectangle::new(
            //     (4f32, anchor.0.section.height() / 2f32).into(),
            //     args.color,
            //     Progress::full(),
            // ),
            cmd,
        );
        let (fs, _area) = external_args.0.best_fit(
            MaxCharacters(5),
            anchor.0.section.area / (1, 2).into(),
            &external_args.1,
        );
        binder.bind(
            TrackTimeBindings::TimeText,
            (0.center(), 0.far(), 0),
            Text::new(
                MaxCharacters(5),
                fs,
                TextValue::new("00:00"),
                Color::GREY_MEDIUM.into(),
            ),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}
