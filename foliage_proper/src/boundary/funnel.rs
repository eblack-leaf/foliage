use crate::boundary::bloom::{Bloom, Emissions};
use crate::boundary::leaf::{Grown, Leaf};
use crate::text_input::TextChanged;
use crate::text_input::action::InputAction;
use crate::{
    Disengaged, Dragged, Engaged, Focused, InputSequence, Layout, OnClick, OnRetrieval, Parent,
    PhysicalInputSequence, Short, Trigger, Unfocused,
};
use bevy_ecs::entity::Entity;
use bevy_ecs::system::{Query, Res, ResMut};

/// The `Leaf` an emission should be reported against.
///
/// An element foliage grew on its own behalf -- a text input's caret, a polyline's segments
/// -- has no `Leaf`, so the walk climbs to the nearest ancestor that does. Without it a click
/// landing on a widget's inner panel would simply vanish, which is never the answer anyone
/// wants; attributing it to the element they actually grew is.
fn attribute(entity: Entity, grown: &Query<&Grown>, parents: &Query<&Parent>) -> Option<Leaf> {
    let mut current = entity;
    loop {
        if grown.contains(current) {
            return Some(Leaf(current));
        }
        current = parents.get(current).ok().and_then(|p| p.id)?;
    }
}

/// One observer per gesture, all the same shape: attribute the target, push the emission.
macro_rules! gesture {
    ($name:ident, $event:ty, $bloom:ident) => {
        fn $name(
            trigger: Trigger<$event>,
            grown: Query<&Grown>,
            parents: Query<&Parent>,
            mut emissions: ResMut<Emissions>,
        ) {
            if let Some(leaf) = attribute(trigger.event_target(), &grown, &parents) {
                emissions.push(Bloom::$bloom(leaf));
            }
        }
    };
}
gesture!(clicked, OnClick, Clicked);
gesture!(engaged, Engaged, Engaged);
gesture!(dragged, Dragged, Dragged);
gesture!(disengaged, Disengaged, Disengaged);
gesture!(focused, Focused, Focused);
gesture!(unfocused, Unfocused, Unfocused);

fn key(trigger: Trigger<InputSequence>, mut emissions: ResMut<Emissions>) {
    let event = trigger.event();
    emissions.push(Bloom::Key {
        key: event.key.clone(),
        mods: event.mods,
    });
}

fn physical_key(trigger: Trigger<PhysicalInputSequence>, mut emissions: ResMut<Emissions>) {
    let event = trigger.event();
    emissions.push(Bloom::PhysicalKey {
        key: event.code,
        mods: event.mods,
    });
}

fn text_changed(
    trigger: Trigger<TextChanged>,
    grown: Query<&Grown>,
    parents: Query<&Parent>,
    values: Query<&crate::TextValue>,
    mut emissions: ResMut<Emissions>,
) {
    let entity = trigger.event_target();
    let Some(leaf) = attribute(entity, &grown, &parents) else {
        return;
    };
    let value = values
        .get(entity)
        .map(|value| value.0.clone())
        .unwrap_or_default();
    emissions.push(Bloom::TextChanged { leaf, value });
}

fn text_action(
    trigger: Trigger<InputAction>,
    grown: Query<&Grown>,
    parents: Query<&Parent>,
    mut emissions: ResMut<Emissions>,
) {
    if let Some(leaf) = attribute(trigger.event_target(), &grown, &parents) {
        emissions.push(Bloom::TextAction {
            leaf,
            action: trigger.event().action,
        });
    }
}

/// `OnEnd` means two different things -- a countdown ran out, or a sequence's last animation
/// finished -- and an app answers them differently, so the two are told apart here by which
/// component the entity carries rather than left for the app to work out.
///
/// Only ones the app asked for are reported, which is what `Grown` is doing here. The engine
/// runs countdowns and sequences of its own -- a widget's internal timer, the anonymous
/// sequence an unjoined animation is counted against -- and those carry no `Leaf` anyone can
/// act on. Reported, they would be emissions naming an id the app never saw, arriving for no
/// reason it could work out. Unlike the gesture funnels above there is no climb to an
/// ancestor: a timer is not part of the display tree, so there is nothing above it to
/// attribute to.
///
/// The `Leaf` is read before the despawn that follows immediately after, which is the only
/// reason it is still nameable at this point.
fn ended(
    trigger: Trigger<crate::OnEnd>,
    grown: Query<&Grown>,
    timers: Query<&crate::Timer>,
    mut emissions: ResMut<Emissions>,
) {
    let entity = trigger.event_target();
    if !grown.contains(entity) {
        return;
    }
    let leaf = Leaf(entity);
    emissions.push(if timers.contains(entity) {
        Bloom::TimerFinished(leaf)
    } else {
        Bloom::SequenceFinished(leaf)
    });
}

fn asset_loaded(trigger: Trigger<OnRetrieval>, mut emissions: ResMut<Emissions>) {
    emissions.push(Bloom::AssetLoaded {
        key: trigger.event().key,
    });
}

/// The viewport moved. Reported with the breakpoint alongside, since a resize that does not
/// cross a breakpoint and one that does call for different responses and an app should not
/// have to ask separately.
fn resized(
    _trigger: Trigger<crate::Resolved<Layout>>,
    viewport: Res<crate::ginkgo::viewport::ViewportHandle>,
    layout: Res<Layout>,
    short: Res<Short>,
    mut emissions: ResMut<Emissions>,
) {
    emissions.push(Bloom::Resized {
        viewport: viewport.section().area,
        layout: *layout,
        short: *short == Short::Yes,
    });
}

/// Every source of emissions, registered in one place. Adding a `Bloom` variant without a
/// line here is the one failure this seam can have that nothing catches at compile time.
pub(crate) struct Funnel;

impl crate::Attachment for Funnel {
    fn attach(foliage: &mut crate::Foliage) {
        foliage.define(clicked);
        foliage.define(engaged);
        foliage.define(dragged);
        foliage.define(disengaged);
        foliage.define(focused);
        foliage.define(unfocused);
        foliage.define(key);
        foliage.define(physical_key);
        foliage.define(text_changed);
        foliage.define(text_action);
        foliage.define(ended);
        foliage.define(asset_loaded);
        foliage.define(resized);
    }
}
