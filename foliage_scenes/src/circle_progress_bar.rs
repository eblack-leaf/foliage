use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Changed, IntoSystemConfigs, Or, Query};
use foliage_proper::bevy_ecs::query::{With, Without};
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::circle::Circle;
use foliage_proper::color::Color;

use crate::{BackgroundColor, Colors, ForegroundColor};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::texture::factors::Progress;

use crate::progress_bar::ProgressPercent;

pub struct CircleProgressBar {
    pub percent: f32,
    pub colors: Colors,
}
impl CircleProgressBar {
    pub fn new(percent: f32, colors: Colors) -> Self {
        Self { percent, colors }
    }
}
#[derive(InnerSceneBinding)]
pub enum CircleProgressBarBindings {
    Fill,
    Back,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
#[derive(Bundle, Clone)]
pub struct CircleProgressBarComponents {
    pub colors: Colors,
    pub percent: ProgressPercent,
}
impl Scene for CircleProgressBar {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static ProgressPercent,
            ),
            With<Tag<CircleProgressBar>>,
        >,
        Query<
            'static,
            'static,
            (&'static mut Color, &'static mut Progress),
            Without<Tag<CircleProgressBar>>,
        >,
    );
    type Filter = Or<(
        Changed<ForegroundColor>,
        Changed<BackgroundColor>,
        Changed<ProgressPercent>,
    )>;
    type Components = CircleProgressBarComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let fill = bindings.get(CircleProgressBarBindings::Fill);
        let back = bindings.get(CircleProgressBarBindings::Back);
        if let Ok((fc, bc, pp)) = ext.0.get(entity) {
            *ext.1.get_mut(fill).unwrap().0 = fc.0;
            *ext.1.get_mut(back).unwrap().0 = bc.0;
            *ext.1.get_mut(fill).unwrap().1.end_mut() = pp.0;
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            CircleProgressBarBindings::Fill,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Circle::new(
                Style::ring(),
                self.colors.foreground.0,
                Progress::new(0.0, self.percent),
            ),
        );
        binder.bind(
            CircleProgressBarBindings::Back,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            )
            .offset_layer(1),
            Circle::new(Style::ring(), self.colors.background.0, Progress::full()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(1.0),
            CircleProgressBarComponents {
                colors: self.colors,
                percent: ProgressPercent::new(self.percent),
            },
        ))
    }
}
impl Leaf for CircleProgressBar {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            foliage_proper::scene::config::<CircleProgressBar>
                .in_set(SetDescriptor::Update)
                .before(<Circle as Leaf>::SetDescriptor::Update),
        );
    }
}