use std::collections::HashMap;

use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;

use crate::elm::Elm;
use crate::grid::LayoutFilter;
use crate::signal::ActionHandle;
use crate::signal::{
    FilteredTriggeredAttribute, Signal, Signaler, TargetComponents, TriggerTarget,
    TriggeredAttribute,
};
use crate::view::{
    CurrentViewStage, SignalHandle, Stage, StageBinding, StagedSignal, TargetBinding, View,
    ViewHandle, ViewStage,
};
use crate::Foliage;

pub struct ViewConfig<'a> {
    pub(crate) root: Entity,
    pub(crate) reference: &'a mut Elm,
    pub(crate) targets: HashMap<TargetBinding, TriggerTarget>,
    pub(crate) stages: HashMap<StageBinding, Stage>,
}

pub struct StageBuilder<'a> {
    binding: StageBinding,
    func: StageDefinition<'a>,
}
impl<'a> StageBuilder<'a> {
    pub fn build(self, foliage: &mut Foliage) {
        todo!()
    }
}
pub type StageDefinition<'a> = fn(&mut StageReference<'a>);
impl<'a> ViewConfig<'a> {
    pub fn with_target<T: Into<TargetBinding>>(mut self, t: T) -> Self {
        let target = self.add_target();
        self.targets.insert(t, target);
        self
    }
    pub fn handle(&self) -> ViewHandle {
        ViewHandle(self.root)
    }
    pub fn define_stage<SB: Into<StageBinding>>(
        &mut self,
        sb: SB,
        func: StageDefinition,
    ) -> StageBuilder<'a> {
        todo!()
    }
    pub(crate) fn add_target(&mut self) -> TriggerTarget {
        let target = self
            .reference
            .ecs
            .world
            .spawn(TargetComponents::new(ViewHandle(self.root)))
            .id();
        let trigger_target = TriggerTarget(target);
        self.reference
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .targets
            .insert(trigger_target);
        trigger_target
    }
    pub fn set_initial_stage<SB: Into<StageBinding>>(self, b: SB) -> Self {
        let stage = *self.stages.get(&b.into()).expect("no-such-stage");
        self.reference
            .ecs
            .world
            .get_mut::<CurrentViewStage>(self.root)
            .expect("no-current")
            .stage = stage;
        self
    }
    pub fn with_stage<SB: Into<StageBinding>>(mut self, sb: SB) -> Self {
        let index = self
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
        let stage = Stage(index as i32);
        self.stages.insert(sb.into(), stage);
        self
    }
    pub fn target<TB: Into<TargetBinding>>(&self, tb: TB) -> TriggerTarget {
        *self.targets.get(&tb.into()).expect("no-target")
    }
    pub fn stage<SB: Into<StageBinding>>(&self, sb: SB) -> Stage {
        *self.stages.get(&sb.into()).expect("no-such-stage")
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
    reference: Option<&'a mut Elm>,
    stage: Stage,
    targets: HashMap<TargetBinding, TriggerTarget>,
    stages: HashMap<StageBinding, Stage>,
}

pub struct SignalReference<'a> {
    root: Entity,
    this: Entity,
    reference: Option<&'a mut Elm>,
    stage: Stage,
}

impl<'a> StageReference<'a> {
    pub fn target<TB: Into<TargetBinding>>(&self, tb: TB) -> TriggerTarget {
        *self.targets.get(&tb.into()).expect("no-target")
    }
    pub fn add_signal_targeting(
        &mut self,
        target: TriggerTarget,
        a_fn: fn(SignalReference<'a>) -> SignalReference<'a>,
    ) {
        let signal = self
            .reference
            .as_ref()
            .unwrap()
            .ecs
            .world
            .spawn(Signaler::new(target))
            .id();
        self.reference
            .as_ref()
            .unwrap()
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
        let mut sr = SignalReference {
            root: self.root,
            this: signal,
            reference: Some(self.reference.take().unwrap()),
            stage: self.stage,
        };
        sr = (a_fn)(sr);
        self.reference
            .replace(sr.reference.take().expect("signal-reference"));
    }
    pub fn signal_action(&mut self, action_handle: ActionHandle) {
        self.reference
            .as_ref()
            .unwrap()
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
    }
    pub fn on_end(&mut self, action_handle: ActionHandle) {
        self.reference
            .as_ref()
            .unwrap()
            .ecs
            .world
            .get_mut::<View>(self.root)
            .expect("no-view")
            .stages
            .get_mut(self.stage.0 as usize)
            .expect("no-stage")
            .on_end
            .insert(action_handle);
    }
}

impl<'a> SignalReference<'a> {
    pub fn with_filtered_attribute<
        A: Bundle + 'static + Clone + Send + Sync,
        F: Into<LayoutFilter>,
    >(
        self,
        a: A,
        filter: F,
    ) -> Self {
        let exceptional_layout_config = filter.into();
        self.reference.checked_add_filtered_signal_fns::<A>();
        self.reference
            .as_ref()
            .unwrap()
            .ecs
            .world
            .entity_mut(self.this)
            .insert(FilteredTriggeredAttribute(
                a,
                exceptional_layout_config.into(),
            ));
        self
    }
    pub fn with_attribute<A: Bundle + 'static + Clone + Send + Sync>(self, a: A) -> Self {
        self.reference.checked_add_signal_fns::<A>();
        self.reference
            .as_ref()
            .unwrap()
            .ecs
            .world
            .entity_mut(self.this)
            .insert(TriggeredAttribute(a));
        self
    }
    pub fn clean(self) -> Self {
        // set Signal::clean() when stage fires instead of Signal::spawn()
        self.reference
            .as_ref()
            .unwrap()
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
        self
    }
    pub fn with_transition(self) -> Self {
        // TODO self.reference.checked_add_transition_fns::<T>();
        self
    }
    pub fn filter_signal(self, layout_filter: LayoutFilter) -> Self {
        self.reference
            .as_ref()
            .unwrap()
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
