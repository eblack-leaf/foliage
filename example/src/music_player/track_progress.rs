use foliage::bevy_ecs::component::Component;
use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs, With, Without};
use foliage::bevy_ecs::query::{Changed, Or};
use foliage::bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::Elm;
use foliage::progress_bar::{ProgressBar, ProgressBarArgs, ProgressBarBindings};
use foliage::rectangle::Rectangle;
use foliage::scene::align::SceneAligner;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::text::font::MonospacedFont;
use foliage::text::{FontSize, MaxCharacters, Text, TextValue};
use foliage::texture::factors::Progress;
use foliage::window::ScaleFactor;
use foliage::{bevy_ecs, scene_bind_enable, set_descriptor};
use std::time::Duration;

#[derive(Bundle)]
pub struct TrackProgress {
    pub length: TrackLength,
    pub current: TrackCurrentTime,
    tag: Tag<Self>,
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
        elm.main().add_systems((
            config_track_progress
                .in_set(ExternalSet::Configure)
                .in_set(TrackProgressSet::Area)
                .before(config_track_time)
                .before(<ProgressBar as Leaf>::SetDescriptor::Area)
                .before(<Text as Leaf>::SetDescriptor::Area),
            config_track_time
                .in_set(ExternalSet::Configure)
                .in_set(TrackProgressSet::Area)
                .before(<ProgressBar as Leaf>::SetDescriptor::Area)
                .before(<Text as Leaf>::SetDescriptor::Area),
        ));
        scene_bind_enable!(elm, TrackProgress);
    }
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
        (
            &SceneHandle,
            &Area<InterfaceContext>,
            &TrackLength,
            &TrackCurrentTime,
        ),
        (
            With<Tag<TrackProgress>>,
            Or<(
                Changed<Area<InterfaceContext>>,
                Changed<TrackCurrentTime>,
                Changed<TrackLength>,
            )>,
        ),
    >,
    mut coordinator: ResMut<SceneCoordinator>,
    mut progresses: Query<&mut Progress>,
    mut text_vals: Query<&mut TextValue>,
    mut prog_areas: Query<&mut Area<InterfaceContext>, Without<Tag<TrackProgress>>>,
) {
    for (handle, area, length, current) in scenes.iter() {
        coordinator.update_anchor_area(*handle, *area);
        let prog_entity = coordinator.binding_entity(
            &handle
                .access_chain()
                .target(TrackProgressBindings::Progress),
        );
        prog_areas.get_mut(prog_entity).unwrap().width = area.width;
        let ratio = current.0.as_millis() as f32 / length.0.as_millis() as f32;
        let progress = Progress::new(0.0, ratio);
        *progresses.get_mut(prog_entity).unwrap() = progress;
        let tt_chain = handle
            .access_chain()
            .binding(TrackProgressBindings::TrackTime);
        let time_text = coordinator.binding_entity(
            &handle
                .access_chain()
                .binding(TrackProgressBindings::TrackTime)
                .target(TrackTimeBindings::TimeText),
        );
        let t_val = format!(
            "{:02}:{:02}",
            (current.0.as_secs_f32() / 60f32).floor(),
            current.0.as_secs_f32() % 60f32
        );
        *text_vals.get_mut(time_text).unwrap() = TextValue::new(t_val);
        coordinator.get_alignment_mut(&tt_chain).pos.horizontal =
            (area.width * ratio - 24f32).near();
    }
}
#[derive(Component, Copy, Clone)]
pub struct TrackLength(pub Duration);
#[derive(Component, Copy, Clone)]
pub struct TrackCurrentTime(pub Duration);
pub struct TrackProgressArgs {
    pub length: TrackLength,
    pub fill_color: Color,
    pub back_color: Color,
}
impl TrackProgressArgs {
    pub fn new<C: Into<Color>>(length: Duration, fill: C, back: C) -> Self {
        Self {
            length: TrackLength(length),
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
            &external_args,
            cmd,
        );
        Self {
            length: args.length,
            current: TrackCurrentTime(Duration::from_secs_f32(160f32)),
            tag: Tag::new(),
        }
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
            (0.center(), 0.near(), 0),
            Rectangle::new(
                (4f32, anchor.0.section.height() / 2f32).into(),
                args.color,
                Progress::full(),
            ),
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
            Text::new(MaxCharacters(5), fs, TextValue::new("00:00"), args.color),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}