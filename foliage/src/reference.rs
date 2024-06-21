use crate::elm::Elm;
use crate::grid::LayoutFilter;
use crate::signal::{ActionHandle, ActionSignal};
use crate::signal::{
    FilteredTriggeredAttribute, Signal, Signaler, TargetComponents, TriggerTarget,
    TriggeredAttribute,
};
use crate::view::{
    CurrentViewStage, SignalHandle, Stage, StagedSignal, View, ViewActive, ViewHandle, ViewStage,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;

pub struct ViewConfig<'a> {
    pub(crate) root: Entity,
    pub(crate) reference: &'a mut Elm,
}

impl<'a> ViewConfig<'a> {
    pub fn handle(self) -> ViewHandle {
        ViewHandle(self.root)
    }
}

pub struct ViewReference<'a> {
    pub(crate) root: Entity,
    pub(crate) reference: &'a mut Elm,
}

pub struct TargetReference<'a> {
    root: Entity,
    this: Entity,
    reference: &'a mut Elm,
}

pub struct StageReference<'a> {
    root: Entity,
    reference: &'a mut Elm,
    stage: Stage,
}

pub struct SignalReference<'a> {
    root: Entity,
    this: Entity,
    reference: &'a mut Elm,
    stage: Stage,
}

impl<'a> StageReference<'a> {
    pub fn add_signal_targeting(mut self, target: TriggerTarget) -> SignalReference<'a> {
        let signal = self.reference.ecs.world.spawn(Signaler::new(target)).id();
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("invalid-stage")
            .signals
            .insert(
                signal,
                StagedSignal {
                    handle: SignalHandle(signal),
                    state_on_stage_start: Signal::spawn(),
                },
            );
        SignalReference {
            root: self.root,
            this: signal,
            reference: self.reference,
            stage: self.stage,
        }
    }
    pub fn signal_action(mut self, action_handle: ActionHandle) -> Self {
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("no-stage")
            .signals
            .insert(
                action_handle.value(),
                StagedSignal {
                    handle: SignalHandle(action_handle.value()),
                    state_on_stage_start: Signal::spawn(),
                },
            );
        self
    }
    pub fn on_end(mut self, action_handle: ActionHandle) -> Self {
        // action to hook to when the stage is confirmed done
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("no-stage")
            .on_end
            .replace(action_handle);
        self
    }
}

impl<'a> SignalReference<'a> {
    pub fn with_filtered_attribute<
        A: Bundle + 'static + Clone + Send + Sync,
        F: Into<LayoutFilter>,
    >(
        mut self,
        a: A,
        filter: F,
    ) -> Self {
        let exceptional_layout_config = filter.into();
        self.reference.checked_add_filtered_signal_fns::<A>();
        self.reference
            .ecs
            .world
            .entity_mut(self.this)
            .insert(FilteredTriggeredAttribute(
                a,
                exceptional_layout_config.into(),
            ));
        self
    }
    pub fn with_attribute<A: Bundle + 'static + Clone + Send + Sync>(mut self, a: A) -> Self {
        self.reference.checked_add_signal_fns::<A>();
        self.reference
            .ecs
            .world
            .entity_mut(self.this)
            .insert(TriggeredAttribute(a));
        self
    }
    pub fn clean(mut self) {
        // set Signal::clean() when stage fires instead of Signal::spawn()
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("no-stage")
            .signals
            .get_mut(&self.this)
            .expect("no-signal")
            .state_on_stage_start = Signal::clean();
    }
    pub fn with_transition(mut self) -> Self {
        // TODO self.reference.checked_add_transition_fns::<T>();
        self
    }
    pub fn filter_signal(mut self, layout_filter: LayoutFilter) -> Self {
        self.reference
            .ecs
            .world
            .entity_mut(self.this)
            .insert(layout_filter);
        self
    }
}

impl<'a> TargetReference<'a> {
    pub fn handle(self) -> TriggerTarget {
        TriggerTarget(self.this)
    }
}

impl<'a> ViewReference<'a> {
    pub fn add_target(mut self) -> TargetReference<'a> {
        let target = self
            .reference
            .ecs
            .world
            .spawn(TargetComponents::new(ViewHandle(self.root)))
            .id();
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .targets
            .insert(TriggerTarget(target));
        TargetReference {
            root: self.root,
            this: target,
            reference: self.reference,
        }
    }
    pub fn set_initial_stage(mut self, stage: Stage) {
        self.reference
            .ecs
            .world
            .get_mut::<CurrentViewStage>(self.root)
            .expect("no-current")
            .stage = stage;
    }
    pub fn activate(mut self) {
        self.reference
            .ecs
            .world
            .get_mut::<ViewActive>(self.root)
            .expect("no-active")
            .0 = true;
    }
    pub fn create_stage(&mut self) -> Stage {
        let stage = self
            .reference
            .ecs
            .world
            .entity(self.root)
            .get::<View>()
            .expect("no-view")
            .stages
            .len();
        self.reference
            .ecs
            .world
            .entity_mut(self.root)
            .get_mut::<View>()
            .expect("no-view")
            .stages
            .push(ViewStage::default());
        Stage(stage as i32)
    }
    pub fn stage(&mut self, stage: Stage) -> StageReference {
        StageReference {
            root: self.root,
            stage,
            reference: self.reference,
        }
    }
}
