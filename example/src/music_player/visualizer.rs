use crate::music_player::track_progress::TrackPlayer;
use foliage::bevy_ecs;
use foliage::bevy_ecs::bundle::Bundle;
use foliage::bevy_ecs::component::Component;
use foliage::bevy_ecs::prelude::Commands;
use foliage::bevy_ecs::query::With;
use foliage::bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::elm::leaf::Tag;
use foliage::rectangle::Rectangle;
use foliage::scene::align::SceneAligner;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::texture::factors::Progress;
#[derive(Bundle)]
pub struct Visualizer {
    tag: Tag<Self>,
    count: BarCount,
}
#[derive(Component, Copy, Clone)]
pub struct BarCount(pub i32);
impl Visualizer {
    const SPACING: f32 = 4.0;
    const BAR_WIDTH: f32 = 4.0;
    const MAX_BAR_HEIGHT: f32 = 36.0;
}
pub enum VisualizerBindings {
    Line,
}
impl From<VisualizerBindings> for SceneBinding {
    fn from(value: VisualizerBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct VisualizerArgs {
    pub color: Color,
}
fn divide_count(width: f32) -> i32 {
    let mut r_val = 0;
    let mut current =
        (r_val + 1) as f32 * Visualizer::SPACING + r_val as f32 * Visualizer::BAR_WIDTH;
    while current < width {
        r_val += 1;
        current = (r_val + 1) as f32 * Visualizer::SPACING + r_val as f32 * Visualizer::BAR_WIDTH;
    }
    if current > width {
        r_val -= 1;
    }
    r_val.max(0)
}
fn visualize(
    scenes: Query<(&SceneHandle, &BarCount), With<Tag<Visualizer>>>,
    mut rectangles: Query<&mut Area<InterfaceContext>>,
    mut coordinator: ResMut<SceneCoordinator>,
    player: Res<TrackPlayer>,
) {
    for (handle, count) in scenes.iter() {
        if player.playing {
            // time-threshold impulses
            // if song reaches that point animate from current to level fast
            // slow decay after impulses min @ 4.0
            // pause animations if paused
            // dont decay if paused
        }
    }
}
impl Scene for Visualizer {
    type Bindings = VisualizerBindings;
    type Args<'a> = VisualizerArgs;
    type ExternalArgs = ();

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        _external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        binder.bind(
            VisualizerBindings::Line,
            (0.near(), 0.center(), 0),
            Rectangle::new(anchor.0.section.area, args.color, Progress::full()),
            cmd,
        );
        let count = divide_count(anchor.0.section.width());
        for i in 1..=count {
            let offset =
                (i as f32 * Visualizer::SPACING + (i - 1) as f32 * Visualizer::BAR_WIDTH).near();
            binder.bind(
                i,
                (offset, (-4f32 - Visualizer::MAX_BAR_HEIGHT).center(), 0),
                Rectangle::new(
                    (Visualizer::BAR_WIDTH, 4.0).into(),
                    args.color,
                    Progress::full(),
                ),
                cmd,
            );
            binder.bind(
                i + count + 1,
                (offset, 4.center(), 0),
                Rectangle::new(
                    (Visualizer::BAR_WIDTH, 4.0).into(),
                    args.color,
                    Progress::full(),
                ),
                cmd,
            );
        }
        Self {
            tag: Tag::new(),
            count: BarCount(count),
        }
    }
}
