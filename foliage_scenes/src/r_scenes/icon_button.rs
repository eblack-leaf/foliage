use crate::r_scenes::button::{Button, ButtonInteractionHook, CurrentStyle};
use crate::r_scenes::{BackgroundColor, ForegroundColor};
use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{IntoSystemConfigs, Query, With, Without};
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::color::Color;
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::elm::config::{ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{BundleExtend, ElementStyle, Elm};
use foliage_proper::icon::{Icon, IconId};
use foliage_proper::interaction::InteractionListener;
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{Alignment, MicroGrid};
use foliage_proper::scene::{Binder, Bindings, BlankNode, Scene, SceneComponents, SceneDesc};

pub struct IconButton {
    element_style: ElementStyle,
    icon_id: IconId,
    foreground_color: Color,
    background_color: Color,
}
impl IconButton {
    pub fn new<ID: Into<IconId>, C: Into<Color>>(
        id: ID,
        element_style: ElementStyle,
        fg: C,
        bg: C,
    ) -> Self {
        Self {
            element_style,
            icon_id: id.into(),
            foreground_color: fg.into(),
            background_color: bg.into(),
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
                &'static ElementStyle,
                &'static ForegroundColor,
                &'static BackgroundColor,
                &'static CurrentStyle,
            ),
            With<Tag<IconButton>>,
        >,
        Query<'static, 'static, &'static mut Color, Without<Tag<IconButton>>>,
        Query<'static, 'static, &'static mut ElementStyle, Without<Tag<IconButton>>>,
    );
    type Filter = <Button as Scene>::Filter;
    type Components = <Button as Scene>::Components;

    fn config(
        entity: Entity,
        _coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        let panel = bindings.get(IconButtonBindings::Panel);
        let icon = bindings.get(IconButtonBindings::Icon);
        if let Ok((_est, fc, bc, cs)) = ext.0.get(entity) {
            *ext.1.get_mut(panel).unwrap() = fc.0;
            if cs.0.is_fill() {
                *ext.1.get_mut(icon).unwrap() = bc.0;
            } else {
                *ext.1.get_mut(icon).unwrap() = fc.0;
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneDesc {
        binder.extend(binder.root(), Tag::<ButtonInteractionHook>::new());
        binder.bind(
            2,
            Alignment::new(),
            BlankNode::default()
                .extend(InteractionListener::default())
                .extend(Tag::<ButtonInteractionHook>::new()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            <Button as Scene>::Components::new(
                self.element_style,
                self.foreground_color,
                self.background_color,
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