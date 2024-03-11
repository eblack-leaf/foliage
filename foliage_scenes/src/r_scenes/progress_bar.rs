use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, Or, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::color::Color;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::Elm;
use foliage_proper::rectangle::Rectangle;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::texture::factors::Progress;

use crate::r_scenes::{BackgroundColor, ForegroundColor};

pub struct ProgressBar {
    foreground_color: Color,
    background_color: Color,
    percent: f32,
}
impl ProgressBar {
    #[allow(unused)]
    pub fn new<C: Into<Color>>(p: f32, fc: C, bc: C) -> Self {
        Self {
            foreground_color: fc.into(),
            background_color: bc.into(),
            percent: p,
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum ProgressBarBindings {
    Fill,
    Back,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
#[derive(Component, Copy, Clone, Default)]
pub struct ProgressPercent(pub f32);
impl ProgressPercent {
    pub fn new(v: f32) -> Self {
        Self(v.min(1.0).max(0.0))
    }
}
#[derive(Bundle)]
pub struct ProgressBarComponents {
    pub foreground_color: ForegroundColor,
    pub background_color: BackgroundColor,
    pub percent: ProgressPercent,
}
impl ProgressBarComponents {
    pub fn new<C: Into<Color>>(fc: C, bc: C, p: f32) -> Self {
        Self {
            foreground_color: ForegroundColor(fc.into()),
            background_color: BackgroundColor(bc.into()),
            percent: ProgressPercent::new(p),
        }
    }
}
impl Scene for ProgressBar {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static ProgressPercent,
            ),
        >,
        Query<
            'static,
            'static,
            (&'static mut Color, &'static mut Progress),
            Without<Tag<ProgressBar>>,
        >,
    );
    type Filter = Or<(
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<ProgressPercent>,
    )>;
    type Components = ProgressBarComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let fill = bindings.get(ProgressBarBindings::Fill);
        let back = bindings.get(ProgressBarBindings::Back);
        if let Ok((fc, bc, pp)) = ext.0.get(entity) {
            *ext.1.get_mut(fill).unwrap().0 = fc.0;
            *ext.1.get_mut(back).unwrap().0 = bc.0;
            *ext.1.get_mut(fill).unwrap().1.end_mut() = pp.0;
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            ProgressBarBindings::Fill,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Rectangle::new(self.foreground_color, Progress::new(0.0, self.percent)),
        );
        binder.bind(
            ProgressBarBindings::Back,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            )
            .offset_layer(1),
            Rectangle::new(self.background_color, Progress::full()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            ProgressBarComponents::new(self.foreground_color, self.background_color, self.percent),
        ))
    }
}
impl Leaf for ProgressBar {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            foliage_proper::scene::config::<ProgressBar>.in_set(SetDescriptor::Update),
        );
    }
}