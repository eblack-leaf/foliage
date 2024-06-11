use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Bundle, Changed, Commands, Component, Query, ResMut, Resource};
use bitflags::bitflags;

use crate::differential::{RenderLink, RenderRemoveQueue};
use crate::view::SignalConfirmation;

#[derive(Component, Default, Copy, Clone)]
pub struct Signal {
    message: i32,
}
impl Signal {
    pub fn is_neutral(&self) -> bool {
        self.message == 0
    }
    pub fn should_clean(&self) -> bool {
        self.message == 1
    }
    pub fn should_spawn(&self) -> bool {
        self.message == 2
    }
    pub fn neutral() -> Self {
        Self { message: 0 }
    }
    pub fn clean() -> Self {
        Self { message: 1 }
    }
    pub fn spawn() -> Self {
        Self { message: 2 }
    }
}
#[derive(Bundle)]
pub(crate) struct Signaler {
    signal: Signal,
    target: TriggerTarget,
}
impl Signaler {
    pub(crate) fn new(target: TriggerTarget) -> Self {
        Self {
            signal: Default::default(),
            target,
        }
    }
}
#[derive(Component, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TriggerTarget(pub(crate) Entity);
#[derive(Component)]
pub(crate) struct TriggeredAttribute<B: Bundle + 'static + Send + Sync + Clone>(
    pub(crate) B,
    pub(crate) Option<LayoutFilter>,
);
pub(crate) fn signaled_clean(
    mut to_clean: Query<(&mut Signal, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
) {
    for (mut signal, target) in to_clean.iter_mut() {
        if signal.should_clean() {
            cmd.entity(target.0).insert(Clean::should_clean());
        }
    }
}
pub(crate) fn clear_signal(mut signals: Query<&mut Signal, Changed<Signal>>) {
    for mut signal in signals.iter_mut() {
        *signal = Signal::default();
    }
}
pub(crate) fn signaled_spawn<B: Bundle + 'static + Send + Sync + Clone>(
    to_spawn: Query<(&Signal, &TriggeredAttribute<B>, &TriggerTarget), Changed<Signal>>,
    mut cmd: Commands,
    layout_config: Res<LayoutConfig>,
) {
    for (signal, attribute, target) in to_spawn.iter() {
        if signal.should_spawn() {
            let mut should_spawn = false;
            if let Some(filter) = attribute.1 {
                if filter.accepts(*layout_config) {
                    should_spawn = true;
                }
            } else {
                should_spawn = true;
            }
            if should_spawn {
                cmd.entity(target.0).insert(attribute.0.clone());
            }
        }
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct Clean {
    should_clean: bool,
}
impl Clean {
    pub fn clean_entity(&mut self) {
        self.should_clean = true;
    }
    pub fn should_clean() -> Self {
        Self { should_clean: true }
    }
}
pub(crate) fn clean(
    mut to_clean: Query<(Entity, &mut Clean, Option<&RenderLink>)>,
    mut remove_queue: ResMut<RenderRemoveQueue>,
    mut cmd: Commands,
) {
    for (entity, mut clean, opt_link) in to_clean.iter_mut() {
        if clean.should_clean {
            if let Some(link) = opt_link {
                remove_queue
                    .queue
                    .get_mut(&link)
                    .expect("invalid render link")
                    .insert(entity);
            }
            clean.should_clean = false;
            cmd.entity(entity).retain::<TargetComponents>();
        }
    }
}
#[derive(Bundle, Default)]
pub(crate) struct TargetComponents {
    clean: Clean,
    confirm: SignalConfirmation,
}
#[derive(Resource, Copy, Clone)]
pub struct LayoutConfig(u16);

bitflags! {
    impl LayoutConfig: u16 {
        const BASE_MOBILE = 1;
        const PORTRAIT_MOBILE = 1 << 1;
        const LANDSCAPE_MOBILE = 1 << 2;
        const PORTRAIT_TABLET = 1 << 3;
        const LANDSCAPE_TABLET = 1 << 4;
        const PORTRAIT_DESKTOP = 1 << 5;
        const LANDSCAPE_DESKTOP = 1 << 6;
        const BASE_TABLET = 1 << 7;
        const BASE_DESKTOP = 1 << 8;
    }
}

// set of layouts this will (not) signal at
#[derive(Component, Copy, Clone)]
pub struct LayoutFilter {
    mode: FilterMode,
    config: LayoutConfig,
}

impl LayoutFilter {
    pub fn new(mode: FilterMode, config: LayoutConfig) -> Self {
        Self { mode, config }
    }
    pub fn accepts(&self, current: LayoutConfig) -> bool {
        match self.mode {
            FilterMode::None => (current & self.config).is_empty(),
            FilterMode::Any => !(current & self.config).is_empty(),
        }
    }
}

pub(crate) fn filter_signal(
    mut signals: Query<(&mut Signal, &LayoutFilter), Changed<Signal>>,
    layout_config: Res<LayoutConfig>,
) {
    for (mut signal, filter) in signals.iter_mut() {
        if filter.accepts(*layout_config) {
            *signal = Signal::default();
        }
    }
}
#[derive(Copy, Clone)]
pub enum FilterMode {
    None,
    Any,
}
