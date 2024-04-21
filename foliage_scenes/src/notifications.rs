use compact_str::{CompactString, ToCompactString};

use foliage_macros::{inner_set_descriptor, InnerSceneBinding};
use foliage_proper::animate::trigger::Trigger;
use foliage_proper::animate::Animate;
use foliage_proper::bevy_ecs;
use foliage_proper::bevy_ecs::bundle::Bundle;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::{Component, IntoSystemConfigs, World};
use foliage_proper::bevy_ecs::query::{Changed, With, Without};
use foliage_proper::bevy_ecs::system::{Command, Commands, Query, SystemParamItem};
use foliage_proper::conditional::{ConditionHandle, ConditionalCommand};
use foliage_proper::coordinate::position::Position;
use foliage_proper::coordinate::PositionAdjust;
use foliage_proper::elm::config::{CoreSet, ElmConfiguration, ExternalSet};
use foliage_proper::elm::leaf::{Leaf, Tag};
use foliage_proper::elm::{Elm, Style};
use foliage_proper::panel::Panel;
use foliage_proper::procedure::Procedure;
use foliage_proper::scene::micro_grid::{
    AlignmentDesc, AnchorDim, MicroGrid, MicroGridAlignment, RelativeMarker,
};
use foliage_proper::scene::{
    Binder, Bindings, BlankNode, ExtendTarget, Scene, SceneComponents, SceneHandle,
};
use foliage_proper::segment::{ResponsiveSegment, Segment, SegmentUnitDesc};
use foliage_proper::text::{Text, TextLineStructure, TextValue};
use foliage_proper::time::timer::Timer;
use foliage_proper::time::TimeDelta;
use foliage_proper::view::ViewBuilder;

use crate::Colors;

#[derive(Clone, Component)]
pub struct Notification(pub CompactString);
impl Notification {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Self(s.as_ref().to_compact_string())
    }
}
impl Procedure for NotificationBar {
    fn steps(self, view_builder: &mut ViewBuilder) {
        let e = view_builder.conditional_scene(
            self,
            ResponsiveSegment::base(Segment::new(
                0.15.relative().to(0.7.relative()),
                1.relative().offset(-64.0).to(56.absolute()),
            )),
        );
        view_builder.extend_conditional(
            e,
            ExtendTarget::Binding(NotificationBarBindings::Closer.into()),
            ConditionalCommand(Closer(e.this())),
        );
        view_builder.extend(e.this(), NotificationState::Hidden);
        view_builder.extend(e.this(), NotificationHandle(e));
    }
}
#[derive(Clone)]
pub struct NotificationBar {
    pub colors: Colors,
}
impl NotificationBar {
    pub fn new(colors: Colors) -> Self {
        Self { colors }
    }
}
#[inner_set_descriptor]
pub enum SetDescriptor {
    Update,
}
impl Leaf for NotificationBar {
    type SetDescriptor = SetDescriptor;

    fn config(_elm_configuration: &mut ElmConfiguration) {
        _elm_configuration.configure_hook(ExternalSet::Configure, SetDescriptor::Update);
    }

    fn attach(elm: &mut Elm) {
        elm.enable_conditional_scene::<NotificationBar>();
        elm.enable_conditional_command::<Closer>();
        elm.enable_conditional_command::<Notification>();
        elm.enable_conditional::<BlankNode>();
        elm.main().add_systems((
            foliage_proper::scene::config::<NotificationBar>
                .in_set(SetDescriptor::Update)
                .before(<Text as Leaf>::SetDescriptor::Update)
                .before(<Panel as Leaf>::SetDescriptor::Update),
            engage_notification_bar.in_set(CoreSet::ProcessEvent),
        ));
    }
}
impl Command for Notification {
    fn apply(self, world: &mut World) {
        world.spawn(self);
    }
}
#[derive(Component, Copy, Clone)]
pub enum NotificationState {
    Showing,
    Hidden,
}
#[derive(InnerSceneBinding)]
pub enum NotificationBarBindings {
    Background,
    Info,
    Timer,
    Closer,
}
pub const NOTIFICATION_BAR_ADJUST: PositionAdjust = PositionAdjust(Position::new(0.0, -64.0));
#[derive(Component)]
struct NotificationHandle(ConditionHandle);
#[derive(Bundle)]
pub struct NotificationsComponents {
    pub notification: Notification,
}
fn engage_notification_bar(
    query: Query<(Entity, &Notification), Without<Tag<NotificationBar>>>,
    mut notification_listener: Query<
        (Entity, &mut Notification, &PositionAdjust, &Bindings),
        With<Tag<NotificationBar>>,
    >,
    mut trigger_element: Query<(&mut NotificationState, &mut Trigger, &NotificationHandle)>,
    mut cmd: Commands,
) {
    for (entity, notification) in query.iter() {
        for (mut state, mut trigger, handle) in trigger_element.iter_mut() {
            // engage trigger
            *trigger = Trigger::active();
            // change state to showing
            *state = NotificationState::Showing;
            for (entity, mut notif, adjust, bindings) in notification_listener.iter_mut() {
                notif.0 = notification.0.clone();
                if adjust.0 != NOTIFICATION_BAR_ADJUST.0 {
                    cmd.entity(entity).insert(
                        entity
                            .animate(
                                None,
                                NOTIFICATION_BAR_ADJUST,
                                TimeDelta::from_millis(500),
                                None,
                            )
                            .with_on_end((
                                bindings.get(NotificationBarBindings::Timer),
                                Trigger::active(),
                            )),
                    );
                }
            }
        }
        cmd.entity(entity).despawn();
    }
}
impl Scene for NotificationBar {
    type Params = (
        Query<'static, 'static, &'static Notification>,
        Query<'static, 'static, (&'static mut TextValue), Without<Tag<NotificationBar>>>,
    );
    type Filter = Changed<Notification>;
    type Components = NotificationsComponents;

    fn config(entity: Entity, ext: &mut SystemParamItem<Self::Params>, bindings: &Bindings) {
        let info = bindings.get(NotificationBarBindings::Info);
        if let Ok(notification) = ext.0.get(entity) {
            if !notification.0.is_empty() {
                *ext.1.get_mut(info).unwrap() = TextValue::new(notification.0.as_str());
            }
        }
    }

    fn create(self, mut binder: Binder) -> SceneHandle {
        binder.bind(
            NotificationBarBindings::Background,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Center),
                0.percent_from(RelativeMarker::Center),
                1.percent_of(AnchorDim::Width),
                1.percent_of(AnchorDim::Height),
            )
            .offset_layer(1),
            Panel::new(Style::fill(), self.colors.background.0),
        );
        let info = binder.bind(
            NotificationBarBindings::Info,
            MicroGridAlignment::new(
                0.percent_from(RelativeMarker::Left),
                0.percent_from(RelativeMarker::Top),
                0.9.percent_of(AnchorDim::Width),
                0.5.percent_of(AnchorDim::Height),
            ),
            Text::new(
                TextLineStructure::new(30, 2),
                TextValue::new(""),
                self.colors.foreground.0,
            ),
        );
        let closer = binder.bind_conditional(
            NotificationBarBindings::Closer,
            MicroGridAlignment::unaligned(),
            BlankNode::default(),
        );
        binder.bind_conditional(
            NotificationBarBindings::Timer,
            MicroGridAlignment::unaligned(),
            Timer::new(TimeDelta::from_secs(1)).on_end(closer.this(), Trigger::active()),
        );
        binder.finish::<Self>(SceneComponents::new(
            MicroGrid::new(),
            NotificationsComponents {
                notification: Notification::new(""),
            },
        ))
    }
}
#[derive(Component, Clone)]
pub(crate) struct Closer(Entity);
impl Command for Closer {
    fn apply(self, world: &mut World) {
        *world.get_mut::<Trigger>(self.0).unwrap() = Trigger::inverse();
    }
}