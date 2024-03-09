use foliage_macros::inner_set_descriptor;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs, Or};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Query, SystemParamItem};
use foliage_proper::circle::Circle;
use foliage_proper::color::Color;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::texture::factors::Progress;

use crate::r_scenes::Direction;

pub struct Ellipsis {
    pub amount: u32,
    pub direction: Direction,
    pub color: Color,
    pub selected: Option<u32>,
}
impl Ellipsis {
    pub fn new(amount: u32, direction: Direction, color: Color, selected: Option<u32>) -> Self {
        Self {
            amount,
            direction,
            color,
            selected,
        }
    }
}
#[derive(Component, Copy, Clone)]
pub struct Selected(pub Option<u32>);
#[derive(Component, Copy, Clone)]
pub struct Total(pub u32);
#[derive(Bundle)]
pub struct EllipsisComponents {
    pub color: Color,
    pub selected: Selected,
    pub total: Total,
}
impl Scene for Ellipsis {
    type Params = (
        Query<
            'static,
            'static,
            (&'static Color, &'static Selected, &'static Total),
            With<Tag<Ellipsis>>,
        >,
        Query<'static, 'static, &'static mut Style, Without<Tag<Ellipsis>>>,
        Query<'static, 'static, &'static mut Color, Without<Tag<Ellipsis>>>,
    );
    type Filter = Or<(Changed<Selected>, Changed<Color>)>;
    type Components = EllipsisComponents;

    fn config(
        _entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        if let Ok((fc, select, total)) = ext.0.get(_entity) {
            for b in bindings.nodes().values() {
                *ext.2.get_mut(b.entity()).unwrap() = *fc;
            }
            if let Some(s) = select.0 {
                for b in bindings.nodes().values() {
                    *ext.1.get_mut(b.entity()).unwrap() = Style::ring();
                }
                if s < total.0 {
                    if total.0 > 7 {
                        if s < total.0 - 3 && s > 3 {
                            *ext.1.get_mut(bindings.get(3i32)).unwrap() = Style::fill();
                        } else if s == total.0 - 3 {
                            *ext.1.get_mut(bindings.get(4)).unwrap() = Style::fill();
                        } else if s == total.0 - 2 {
                            *ext.1.get_mut(bindings.get(5)).unwrap() = Style::fill();
                        } else if s == total.0 - 1 {
                            *ext.1.get_mut(bindings.get(6)).unwrap() = Style::fill();
                        } else {
                            *ext.1.get_mut(bindings.get(s as i32)).unwrap() = Style::fill();
                        }
                    } else {
                        *ext.1.get_mut(bindings.get(s as i32)).unwrap() = Style::fill();
                    }
                }
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        let amount = self.amount.min(7);
        let factor = amount as f32 * 2f32;
        let aspect = match self.direction {
            Direction::Horizontal => factor,
            Direction::Vertical => 1f32 / factor,
        };
        let clean_interval = 1f32 / factor;
        let interval = 1f32 / (factor + 2f32);
        let alignment = |i: f32| match self.direction {
            Direction::Horizontal => MicroGridAlignment::new(
                (i * clean_interval * 2f32).percent_from(RelativeMarker::Left),
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

        for i in 0..amount {
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
                Circle::new(style, self.color, Progress::full()),
            );
        }
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new().aspect_ratio(aspect),
            EllipsisComponents {
                color: self.color,
                selected: Selected(self.selected),
                total: Total(self.amount),
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
