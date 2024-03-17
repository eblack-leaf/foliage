use crate::Colors;
use compact_str::{CompactString, ToCompactString};
use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Commands, Query, SystemParamItem};
use foliage_proper::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::panel::Panel;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{Binder, Bindings, Scene, SceneComponents, SceneHandle};
use foliage_proper::text::{MaxCharacters, Text, TextValue};
#[derive(Clone, Component)]
pub struct Notification(pub CompactString);
impl Notification {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_compact_string())
    }
}
pub struct Notifications {
    pub colors: Colors,
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for Notifications {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((
            foliage_proper::scene::config::<Notifications>.in_set(SetDescriptor::Update),
            engage_notification_bar.in_set(CoreSet::ProcessEvent),
        ));
    }
}
#[derive(Component, Copy, Clone)]
pub enum NotificationState {
    Showing,
    Hidden,
}
#[derive(InnerSceneBinding)]
pub enum SnackBarBindings {
    Background,
    LineOne,
    LineTwo,
}
#[derive(Bundle)]
pub struct NotificationsComponents {
    pub state: NotificationState,
    pub notification: Notification,
}
fn engage_notification_bar(
    query: Query<(Entity, &Notification), Without<Tag<Notifications>>>,
    notification_listener: Query<
        (&mut Notification, &mut NotificationState, &Bindings),
        With<Tag<Notifications>>,
    >,
    mut cmd: Commands,
) {
    for (entity, notification) in query.iter() {
        // enable notification bar conditionals,
        // if !already_open,
        // -- anim from offscreen position_adjust
        // -- anim includes on-end 30s timer w/ on-end (timer) anim to hidden
        // -- change state to Showing
        // signal text-change by replacing notes with new selection
        cmd.entity(entity).despawn();
    }
}
impl Scene for Notifications {
    type Params = (Query<'static, 'static, &'static Notification>,);
    type Filter = Changed<Notification>;
    type Components = NotificationsComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let one = bindings.get(SnackBarBindings::LineOne);
        let two = bindings.get(SnackBarBindings::LineTwo);
        if let Ok(notification) = ext.0.get(entity) {
            if !notification.0.is_empty() {
                // replace text split if too big
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            SnackBarBindings::Background,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            ),
            Panel::new(Style::fill(), self.colors.background.0),
        );
        binder.bind(
            SnackBarBindings::LineOne,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Top),
                0.9.percent_of(AnchorDim::Width),
                0.5.percent_of(AnchorDim::Height),
            ),
            Text::new(
                MaxCharacters(30),
                TextValue::new(""),
                self.colors.foreground.0,
            ),
        );
        binder.bind(
            SnackBarBindings::LineTwo,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Left),
                0.5.percent_from(RelativeMarker::Top),
                0.9.percent_of(AnchorDim::Width),
                0.5.percent_of(AnchorDim::Height),
            ),
            Text::new(
                MaxCharacters(30),
                TextValue::new(""),
                self.colors.foreground.0,
            ),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            NotificationsComponents {
                state: NotificationState::Hidden,
                notification: Notification::new(""),
            },
        ))
    }
}