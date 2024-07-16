use crate::element::{ActionHandle, Element, IdTable, TargetHandle};
use crate::grid::Layout;
use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, Changed, Commands, Entity, Query, World};
use bevy_ecs::system::Command;

pub struct ElmHandle<'a> {
    pub(crate) world_handle: Option<&'a mut World>,
}
pub struct ElementHandle<'a> {
    pub(crate) world_handle: Option<&'a mut World>,
    pub(crate) handle: TargetHandle,
    pub(crate) entity: Entity,
}
impl<'a> ElementHandle<'a> {
    pub fn with_attr<A: Bundle>(mut self, a: A) -> Self {
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(self.entity)
            .insert(a);
        self
    }
    pub fn with_filtered_attr<A: Bundle>(mut self, layout: Layout, a: A) -> Self {
        // each filtered uses target (like anim) to give attr if layout-filter.accepts
        // signal all on-layout change + let filters resolve
        // so give comp to handle that here
        // would need enable in main so maybe not
        todo!()
    }
    pub fn dependent_of<RTH: Into<TargetHandle>>(mut self, rth: RTH) -> Self {
        // lookup root
        // give to that dependents
        self
    }
}
impl<'a> ElmHandle<'a> {
    pub fn add_element<
        TH: Into<TargetHandle>,
        EFN: FnOnce(ElementHandle<'a>) -> ElementHandle<'a>,
    >(
        &mut self,
        th: TH,
        e_fn: EFN,
    ) {
        let entity = self
            .world_handle
            .as_mut()
            .unwrap()
            .spawn(Element::default())
            .id();
        let target = th.into();
        self.world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<IdTable>()
            .unwrap()
            .add_target(target.clone(), entity);
        self.update_element(target, e_fn);
    }
    pub fn remove_element<TH: Into<TargetHandle>>(&mut self, th: TH) {
        // queue remove of all dependents
        // update id-table as necessary
        // remove from roots dependents
    }
    pub fn update_element<
        TH: Into<TargetHandle>,
        EFN: FnOnce(ElementHandle<'a>) -> ElementHandle<'a>,
    >(
        &mut self,
        th: TH,
        e_fn: EFN,
    ) {
        let th = th.into();
        let entity = self.lookup_target_entity(th.clone());
        let mut element_handle = ElementHandle {
            world_handle: self.world_handle.take(),
            entity,
            handle: th,
        };
        element_handle = e_fn(element_handle);
        self.world_handle = element_handle.world_handle.take();
    }
    pub fn change_element_root<TH: Into<TargetHandle>, RTH: Into<TargetHandle>>(
        &mut self,
        th: TH,
        new_root: RTH,
    ) {
        // get current-root (if any)
        // + remove from that dependents
        // add to new dependents (of new root)
    }
    pub fn run_action<A: Actionable>(&mut self, a: A) {
        let action = Action { data: a };
        action.apply(self.world_handle.as_mut().unwrap());
    }
    pub fn create_signaled_action<A: Actionable, AH: Into<ActionHandle>>(&mut self, ah: AH, a: A) {
        todo!()
    }
    fn lookup_target_entity<TH: Into<TargetHandle>>(&mut self, th: TH) -> Entity {
        self.world_handle
            .as_ref()
            .unwrap()
            .get_resource::<IdTable>()
            .unwrap()
            .lookup_target(th.into())
    }
}
pub trait Actionable
where
    Self: Clone + Send + Sync + 'static,
{
    fn apply(self, handle: ElmHandle);
}

#[derive(Clone)]
pub struct Action<A: Actionable> {
    data: A,
}

impl<A: Actionable> Command for Action<A> {
    fn apply(self, world: &mut World) {
        let connection = ElmHandle {
            world_handle: Some(world),
        };
        self.data.apply(connection);
    }
}

#[derive(Component)]
pub struct Signal(pub bool);
impl Signal {
    pub fn active() -> Self {
        Self(true)
    }
    pub fn inactive() -> Self {
        Self(false)
    }
}
#[derive(Component)]
pub struct SignaledAction<A: Actionable> {
    a: Action<A>,
}

pub(crate) fn signal_action<A: Actionable>(
    mut signals: Query<(&Signal, &SignaledAction<A>)>,
    mut cmd: Commands,
) {
    for (signal, signaled_action) in signals.iter() {
        if signal.0 {
            let action = signaled_action.a.clone();
            cmd.add(action);
        }
    }
}

pub(crate) fn clear_signal(mut signals: Query<(&mut Signal), Changed<Signal>>) {
    for mut signal in signals.iter_mut() {
        signal.0 = false;
    }
}
