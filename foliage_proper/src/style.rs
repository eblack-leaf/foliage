use crate::color::Color;
use crate::elm::{Elm, ScheduleMarkers};
use crate::interaction::ClickInteractionListener;
use crate::signal::TriggerTarget;
use crate::Leaf;
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Commands, Query};

pub(crate) struct Style;
impl Leaf for Style {
    fn attach(elm: &mut Elm) {
        elm.scheduler.main.add_systems(
            alternate_color_on_engage
                .before(ScheduleMarkers::Config)
                .after(ScheduleMarkers::Interaction),
        );
    }
}
#[derive(Component, Clone)]
pub struct InteractiveColor {
    pub base: Color,
    pub alternate_color: Color,
    pub linked: Vec<TriggerTarget>,
}
impl InteractiveColor {
    pub fn new<B: Into<Color>, A: Into<Color>>(b: B, a: A) -> Self {
        Self {
            base: b.into(),
            alternate_color: a.into(),
            linked: vec![],
        }
    }
    pub fn with_linked(mut self, linked: Vec<TriggerTarget>) -> Self {
        self.linked = linked;
        self
    }
}
pub(crate) fn alternate_color_on_engage(
    mut alts: Query<
        (&mut Color, &InteractiveColor, &ClickInteractionListener),
        Changed<ClickInteractionListener>,
    >,
    mut cmd: Commands,
) {
    for (mut color, alt, listener) in alts.iter_mut() {
        if listener.engaged_start {
            for linked in alt.linked.iter() {
                cmd.entity(linked.value()).insert(alt.base);
            }
            *color = alt.alternate_color;
        } else if listener.engaged_end || !listener.engaged {
            for linked in alt.linked.iter() {
                cmd.entity(linked.value()).insert(alt.alternate_color);
            }
            *color = alt.base;
        }
    }
}