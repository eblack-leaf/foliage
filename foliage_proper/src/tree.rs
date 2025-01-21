use crate::anim::runner::AnimationRunner;
use crate::anim::sequence::{AnimationTime, Sequence};
use crate::disable::Disable;
use crate::enable::Enable;
use crate::leaf::Leaf;
use crate::ops::Name;
use crate::remove::Remove;
use crate::time::OnEnd;
use crate::{Animate, Animation, OnClick};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::prelude::{Commands, World};
use bevy_ecs::system::IntoObserverSystem;

pub type Tree<'w, 's> = Commands<'w, 's>;

pub trait EcsExtension {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity;
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static);
    fn send<E: Event>(&mut self, e: E);
    fn queue<E: Event>(&mut self, e: E);
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B);
    fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static);
    fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static);
    fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static);
    fn sequence(&mut self) -> Entity;
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity;
    fn sequence_end<END: IntoObserverSystem<OnEnd, B, M>, B: Bundle, M>(
        &mut self,
        seq: Entity,
        end: END,
    );
    fn subscribe<SUB: IntoObserverSystem<S, B, M>, S: Event + 'static, B: Bundle, M>(
        &mut self,
        e: Entity,
        sub: SUB,
    );
    fn on_click<ONC: IntoObserverSystem<OnClick, B, M>, B: Bundle, M>(&mut self, e: Entity, o: ONC);
    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S);
}
impl EcsExtension for Tree<'_, '_> {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        let entity = self.spawn((Leaf::new(), b)).id();
        entity
    }
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.trigger_targets(e, targets);
    }
    fn send<E: Event>(&mut self, e: E) {
        self.trigger(e);
    }
    fn queue<E: Event>(&mut self, e: E) {
        self.send_event(e);
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.entity(entity).insert(b);
    }
    fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.send_to(Remove::new(), targets);
    }
    fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.send_to(Enable::new(), targets);
    }
    fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.send_to(Disable::new(), targets);
    }
    fn sequence(&mut self) -> Entity {
        self.spawn(Sequence::default()).id()
    }
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity {
        let runner = AnimationRunner::new(
            anim.anim_target.unwrap(),
            anim.a,
            anim.ease,
            anim.seq,
            AnimationTime::from(anim.sequence_time_range),
        );
        self.spawn(runner).id()
    }
    fn sequence_end<END: IntoObserverSystem<OnEnd, B, M>, B: Bundle, M>(
        &mut self,
        seq: Entity,
        end: END,
    ) {
        self.entity(seq).observe(end);
    }

    fn subscribe<SUB: IntoObserverSystem<S, B, M>, S: Event + 'static, B: Bundle, M>(
        &mut self,
        e: Entity,
        sub: SUB,
    ) {
        self.entity(e).observe(sub);
    }
    fn on_click<ONC: IntoObserverSystem<OnClick, B, M>, B: Bundle, M>(
        &mut self,
        e: Entity,
        o: ONC,
    ) {
        self.entity(e).observe(o);
    }

    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.send(Name(s.as_ref().to_string(), e));
    }
}

impl EcsExtension for World {
    fn leaf<B: Bundle>(&mut self, b: B) -> Entity {
        self.commands().leaf(b)
    }
    fn send_to<E: Event>(&mut self, e: E, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().send_to(e, targets);
    }
    fn send<E: Event>(&mut self, e: E) {
        self.commands().send(e);
    }
    fn queue<E: Event>(&mut self, e: E) {
        EcsExtension::queue(&mut self.commands(), e);
    }
    fn write_to<B: Bundle>(&mut self, entity: Entity, b: B) {
        self.commands().write_to(entity, b);
    }
    fn remove(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().remove(targets);
    }
    fn enable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().enable(targets);
    }
    fn disable(&mut self, targets: impl TriggerTargets + Send + Sync + 'static) {
        self.commands().disable(targets);
    }
    fn sequence(&mut self) -> Entity {
        self.commands().sequence()
    }
    fn animate<A: Animate>(&mut self, anim: Animation<A>) -> Entity {
        self.commands().animate(anim)
    }
    fn sequence_end<END: IntoObserverSystem<OnEnd, B, M>, B: Bundle, M>(
        &mut self,
        seq: Entity,
        end: END,
    ) {
        self.commands().sequence_end(seq, end);
    }

    fn subscribe<SUB: IntoObserverSystem<S, B, M>, S: Event + 'static, B: Bundle, M>(
        &mut self,
        e: Entity,
        sub: SUB,
    ) {
        self.commands().subscribe(e, sub);
    }

    fn on_click<ONC: IntoObserverSystem<OnClick, B, M>, B: Bundle, M>(
        &mut self,
        e: Entity,
        o: ONC,
    ) {
        self.commands().on_click(e, o);
    }

    fn name<S: AsRef<str>>(&mut self, e: Entity, s: S) {
        self.commands().name(e, s);
    }
}
