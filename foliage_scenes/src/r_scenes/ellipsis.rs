use crate::r_scenes::{BackgroundColor, Colors, Direction, ForegroundColor};
use foliage_macros::inner_set_descriptor;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::With;
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::circle::Circle;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::texture::factors::Progress;
pub struct Ellipsis {
    pub amount: u32,
    pub direction: Direction,
    pub colors: Colors,
    pub selected: Option<u32>,
}
impl Ellipsis {
    pub fn new(amount: u32, direction: Direction, colors: Colors, selected: Option<u32>) -> Self {
        Self {
            amount,
            direction,
            colors,
            selected,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Selected(pub Option<u32>);
#[derive(Bundle)]
pub struct EllipsisComponents {
    pub colors: Colors,
    pub selected: Selected,
}
impl Scene for Ellipsis {
    type Params = (
        Query<
            'static,
            'static,
            (&'static ForegroundColor, &'static BackgroundColor),
            With<Tag<Ellipsis>>,
        >,
    );
    type Filter = ();
    type Components = EllipsisComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        for b in bindings.nodes().iter() {
            // update colors
            // update selected style?
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let aspect = match self.direction {
            Direction::Horizontal => self.amount as f32,
            Direction::Vertical => 1f32 / self.amount as f32,
        };
        let clean_interval = 1f32 / self.amount as f32;
        let interval = 1f32 / (self.amount as f32 + 2f32);
        let alignment = |i: f32| match self.direction {
            Direction::Horizontal => MicroGridAlignment::new(
                (i * clean_interval).percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Center),
                interval.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Direction::Vertical => MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                (i * interval).percent_from(RelativeMarker::Top),
                1.percent_of(AnchorDim::Width),
                interval.percent_of(AnchorDim::Height),
            ),
        };
        for i in 0..self.amount {
            let style = if let Some(u) = self.selected {
                if i == u {
                    Style::fill()
                } else {
                    Style::ring()
                }
            } else {
                Style::ring()
            };
            binder.bind(
                i as i32,
                alignment(i as f32),
                Circle::new(style, self.colors.foreground.0, Progress::full()),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(aspect),
            EllipsisComponents {
                colors: self.colors,
                selected: Selected(self.selected),
            },
        ))
    }
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for Ellipsis {
    type SetDescriptor = SetDescriptor;
    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }
    fn attach(elm: &mut Elm) {
        elm.main()
            .add_systems(foliage_proper::scene::config::<Ellipsis>.in_set(SetDescriptor::Update));
    }
}
