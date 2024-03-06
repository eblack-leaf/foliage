use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{IntoSystemConfigs, Query, With, Without};
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::color::Color;
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, Elm, Style};
use foliage_proper::icon::{Icon, IconId};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    Alignment, AlignmentDesc, AnchorDim, MicroGrid, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneHandle};

use crate::r_scenes::button::{Button, ButtonInteractionHook, CurrentStyle};
use crate::r_scenes::{BackgroundColor, Colors, ForegroundColor};

pub struct IconButton {
    element_style: Style,
    icon_id: IconId,
    ui_color: Colors,
}
impl IconButton {
    pub fn new<ID: Into<IconId>, C: Into<Color>>(
        id: ID,
        element_style: Style,
        fg: C,
        bg: C,
    ) -> Self {
        Self {
            element_style,
            icon_id: id.into(),
            ui_color: Colors::new(fg, bg),
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum IconButtonBindings {
    Panel,
    Icon,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Scene for IconButton {
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
            With<Tag<IconButton>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<IconButton>>>,
        Query<'static, 'static, &'static mut Style, Without<Tag<IconButton>>>,
    );
    type Filter = <Button as Scene>::Filter;
    type Components = <Button as Scene>::Components;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let panel = bindings.get(IconButtonBindings::Panel);
        let icon = bindings.get(IconButtonBindings::Icon);
        if let Ok((_est, fc, bc, cs)) = ext.0.get(entity) {
            if _est.is_fill() {
                if cs.0.is_fill() {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    *ext.1.get_mut(icon).unwrap() = fc.0;
                } else {
                    *ext.1.get_mut(icon).unwrap() = bc.0;
                    *ext.1.get_mut(panel).unwrap() = fc.0;
                }
            } else {
                *ext.2.get_mut(panel).unwrap() = cs.0;
                if cs.0.is_fill() {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    *ext.1.get_mut(icon).unwrap() = fc.0;
                } else {
                    *ext.1.get_mut(panel).unwrap() = bc.0;
                    *ext.1.get_mut(icon).unwrap() = bc.0;
                }
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.extend(binder.root(), Tag::<ButtonInteractionHook>::new());
        binder.bind(
            IconButtonBindings::Panel,
            Alignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Panel::new(self.element_style, self.ui_color.foreground.0),
        );
        binder.bind(
            IconButtonBindings::Icon,
            Alignment::new(
                0.fixed_from(RelativeMarker::Center),
                0.fixed_from(RelativeMarker::Center),
                0.7.percent_of(AnchorDim::Width),
                0.7.percent_of(AnchorDim::Width),
            ),
            Icon::new(self.icon_id, self.ui_color.background.0),
        );
        binder.bind(
            2,
            Alignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            BlankNode::default()
                .extend(InteractionListener::default())
                .extend(Tag::<ButtonInteractionHook>::new()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new()
                .aspect_ratio(1.0)
                .min_height(34.0)
                .min_width(34.0),
            <Button as Scene>::Components::new(
                self.element_style,
                self.ui_color.foreground.0,
                self.ui_color.background.0,
            ),
        ))
    }
}
impl Leaf for IconButton {
    type SetDescriptor = SetDescriptor;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems(
            foliage_proper::scene::config::<IconButton>
                .in_set(SetDescriptor::Update)
                .before(<Icon as Leaf>::SetDescriptor::Update)
                .before(<Panel as Leaf>::SetDescriptor::Update),
        );
    }
}