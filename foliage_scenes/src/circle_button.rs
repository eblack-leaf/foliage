use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{IntoSystemConfigs, Query, With, Without};
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::circle::Circle;
use foliage_proper::color::Color;

use crate::{BackgroundColor, Colors, ForegroundColor};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, Elm, Style};
use foliage_proper::icon::{Icon, IconId};
use foliage_proper::interaction::{InteractionListener, InteractionShape};
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};
use foliage_proper::texture::factors::Progress;

use crate::button::{Button, ButtonComponents, ButtonInteractionHook, CurrentStyle};

#[derive(Clone)]
pub struct CircleButton {
    icon_id: IconId,
    element_style: Style,
    pub colors: Colors,
}
impl CircleButton {
    pub fn new<ID: Into<IconId>>(id: ID, element_style: Style, colors: Colors) -> Self {
        Self {
            icon_id: id.into(),
            element_style,
            colors,
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum CircleButtonBindings {
    Circle,
    Icon,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Scene for CircleButton {
    type Params = (
        Query<
            'static,
            'static,
            (
                &'static Style,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static CurrentStyle,
            ),
            With<Tag<CircleButton>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<CircleButton>>>,
        Query<'static, 'static, &'static mut Style, Without<Tag<CircleButton>>>,
    );
    type Filter = <Button as Scene>::Filter;
    type Components = <Button as Scene>::Components;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let circle = bindings.get(CircleButtonBindings::Circle);
        let icon = bindings.get(CircleButtonBindings::Icon);
        if let Ok((_est, fc, bc, cs)) = ext.0.get(entity) {
            if _est.is_fill() {
                if cs.0.is_fill() {
                    *ext.1.get_mut(circle).unwrap() = bc.0;
                    *ext.1.get_mut(icon).unwrap() = fc.0;
                } else {
                    *ext.1.get_mut(icon).unwrap() = bc.0;
                    *ext.1.get_mut(circle).unwrap() = fc.0;
                }
            } else {
                *ext.2.get_mut(circle).unwrap() = cs.0;
                if cs.0.is_fill() {
                    *ext.1.get_mut(circle).unwrap() = bc.0;
                    *ext.1.get_mut(icon).unwrap() = fc.0;
                } else {
                    *ext.1.get_mut(circle).unwrap() = bc.0;
                    *ext.1.get_mut(icon).unwrap() = bc.0;
                }
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.extend(binder.root(), Tag::<ButtonInteractionHook>::new());
        binder.bind(
            CircleButtonBindings::Circle,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Circle::new(
                self.element_style,
                self.colors.foreground.0,
                Progress::full(),
            ),
        );
        binder.bind(
            CircleButtonBindings::Icon,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                0.5.percent_of(AnchorDim::Width),
                0.5.percent_of(AnchorDim::Width),
            ),
            Icon::new(self.icon_id, self.colors.background.0),
        );
        binder.bind(
            2,
            MicroGridAlignment::new(
                0.fixed_from(RelativeMarker::Left),
                0.fixed_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            BlankNode::default()
                .extend(
                    InteractionListener::default().with_shape(InteractionShape::InteractiveCircle),
                )
                .extend(Tag::<ButtonInteractionHook>::new()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new()
                .aspect_ratio(1.0)
                .min_width(44.0)
                .min_height(44.0),
            ButtonComponents::new(self.element_style, self.colors),
        ))
    }
}
impl Leaf for CircleButton {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            foliage_proper::scene::config::<CircleButton>
                .in_set(SetDescriptor::Update)
                .before(<Icon as Leaf>::SetDescriptor::Update)
                .before(<Circle as Leaf>::SetDescriptor::Update),
        );
    }
}