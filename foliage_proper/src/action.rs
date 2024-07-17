use std::any::TypeId;
use std::collections::HashSet;

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Bundle, Changed, Commands, DetectChanges, Entity, Query, World};
use bevy_ecs::system::{Command, Res};

use crate::differential::{Remove, RenderLink};
use crate::element::{ActionHandle, Dependents, Element, IdTable, Root, TargetHandle};
use crate::elm::{ActionLimiter, FilterAttrLimiter};
use crate::grid::{Grid, GridPlacement, Layout, LayoutFilter};

pub struct ElmHandle<'a> {
    pub(crate) world_handle: Option<&'a mut World>,
}
pub struct ElementHandle<'a> {
    pub(crate) world_handle: Option<&'a mut World>,
    pub(crate) handle: TargetHandle,
    pub(crate) entity: Entity,
}
pub struct FilteredAttributeConfig<A: Bundle + Send + Sync + 'static + Clone> {
    pub filter: LayoutFilter,
    pub a: A,
}
impl<A: Bundle + Send + Sync + 'static + Clone> FilteredAttributeConfig<A> {
    pub fn new(layout: Layout, a: A) -> Self {
        Self {
            filter: layout.into(),
            a,
        }
    }
}
#[derive(Component)]
pub struct FilteredAttribute<A: Bundle + Send + Sync + 'static + Clone> {
    filtered: Vec<FilteredAttributeConfig<A>>,
}
impl<A: Bundle + Send + Sync + 'static + Clone> FilteredAttribute<A> {
    pub fn new() -> Self {
        Self { filtered: vec![] }
    }
    pub fn with(mut self, layout: Layout, a: A) -> Self {
        self.filtered.push(FilteredAttributeConfig::new(layout, a));
        self
    }
}
pub trait HasRenderLink {
    fn has_link() -> bool {
        false
    }
}
pub(crate) fn filter_attr_layout_change<A: Bundle + Send + Sync + 'static + Clone>(
    filtered: Query<(Entity, &FilteredAttribute<A>, Option<&RenderLink>)>,
    layout: Res<Layout>,
    mut cmd: Commands,
) {
    if layout.is_changed() {
        for (entity, filter_attr, opt_link) in filtered.iter() {
            todo!()
            // if we have match then give else remove<A>
            // if removing + <A as HasRenderLink>::has_link() => send render-queue remove
        }
    }
}
pub(crate) fn filter_attr_changed<A: Bundle + Send + Sync + 'static + Clone>(
    filtered: Query<
        (Entity, &FilteredAttribute<A>, Option<&RenderLink>),
        Changed<FilteredAttribute<A>>,
    >,
    layout: Res<Layout>,
    mut cmd: Commands,
) {
    for (entity, filtered_attr, opt_link) in filtered.iter() {
        todo!()
        // if we have match then give else remove<A>
        // if removing + <A as HasRenderLink>::has_link() => send render-queue remove
    }
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
    pub fn with_filtered_attr<A: Bundle + Send + Sync + 'static + Clone>(
        mut self,
        filtered_attribute: FilteredAttribute<A>,
    ) -> Self {
        if !self.world_handle.as_ref().unwrap().contains_resource::<FilterAttrLimiter<A>>() {
            panic!("enable filtering for this attribute type")
        }
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(self.entity)
            .insert(filtered_attribute);
        self
    }
    pub fn dependent_of<RTH: Into<TargetHandle>>(mut self, rth: RTH) -> Self {
        // lookup root
        let rth = rth.into();
        let root = self.lookup_target_entity(rth.clone()).unwrap();
        // give to that dependents
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<Dependents>(root)
            .unwrap()
            .0
            .insert(self.handle.clone());
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<Root>(self.entity)
            .unwrap()
            .0
            .replace(rth);
        self
    }
    fn lookup_target_entity<TH: Into<TargetHandle>>(&self, th: TH) -> Option<Entity> {
        self.world_handle
            .as_ref()
            .unwrap()
            .get_resource::<IdTable>()
            .unwrap()
            .lookup_target(th.into())
    }
}
impl<'a> ElmHandle<'a> {
    pub fn add_element<
        TH: Into<TargetHandle>,
        EFN: FnOnce(ElementHandle<'a>) -> ElementHandle<'a>,
    >(
        &mut self,
        th: TH,
        grid_placement: GridPlacement,
        grid: Option<Grid>,
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
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(entity)
            .insert(grid_placement);
        if let Some(g) = grid {
            self.world_handle
                .as_mut()
                .unwrap()
                .entity_mut(entity)
                .insert(g);
        }
        self.update_element(target, e_fn);
    }
    pub fn remove_element<TH: Into<TargetHandle>>(&mut self, th: TH) {
        // queue remove of all dependents
        let handle = th.into();
        let start = self
            .lookup_target_entity(handle.clone())
            .expect("attempting to remove non-existent element");
        self.world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<IdTable>()
            .unwrap()
            .targets
            .remove(&handle);
        self.world_handle
            .as_mut()
            .unwrap()
            .entity_mut(start)
            .insert(Remove::remove());
        if let Some(root) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Root>(start)
            .unwrap()
            .0
            .clone()
        {
            let entity = self.lookup_target_entity(root).unwrap();
            self.world_handle
                .as_mut()
                .unwrap()
                .get_mut::<Dependents>(entity)
                .unwrap()
                .0
                .remove(&handle);
        }
        let dependents = self.recursive_remove_element(start);
        for (t, e) in dependents {
            self.world_handle
                .as_mut()
                .unwrap()
                .entity_mut(e)
                .insert(Remove::remove());
            self.world_handle
                .as_mut()
                .unwrap()
                .get_resource_mut::<IdTable>()
                .unwrap()
                .targets
                .remove(&t);
        }
    }
    fn recursive_remove_element(&self, current: Entity) -> HashSet<(TargetHandle, Entity)> {
        let mut removed_set = HashSet::new();
        if let Some(deps) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Dependents>(current)
        {
            let dependents = deps.0.clone();
            for d in dependents.iter() {
                let e = self.lookup_target_entity(d.clone()).unwrap();
                removed_set.insert((d.clone(), e));
                removed_set.extend(self.recursive_remove_element(e));
            }
        }
        removed_set
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
        let entity = self.lookup_target_entity(th.clone()).unwrap();
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
        let rth = new_root.into();
        let th = th.into();
        let new_root_entity = self.lookup_target_entity(rth).unwrap();
        let this = self.lookup_target_entity(th.clone()).unwrap();
        if let Some(old) = self
            .world_handle
            .as_ref()
            .unwrap()
            .get::<Root>(this)
            .unwrap()
            .0
            .as_ref()
        {
            let old_entity = self.lookup_target_entity(old.clone());
            if let Some(oe) = old_entity {
                self.world_handle
                    .as_mut()
                    .unwrap()
                    .get_mut::<Dependents>(oe)
                    .unwrap()
                    .0
                    .remove(&th);
            }
        }
        self.world_handle
            .as_mut()
            .unwrap()
            .get_mut::<Dependents>(new_root_entity)
            .unwrap()
            .0
            .insert(th.clone());
    }
    pub fn run_action<A: Actionable>(&mut self, a: A) {
        let action = Action { data: a };
        action.apply(self.world_handle.as_mut().unwrap());
    }
    pub fn create_signaled_action<A: Actionable, AH: Into<ActionHandle>>(&mut self, ah: AH, a: A) {
        if !self.world_handle.as_ref().unwrap().contains_resource::<ActionLimiter<A>>() {
            panic!("please enable_signaled_action for this action type")
        }
        let signaler = self
            .world_handle
            .as_mut()
            .unwrap()
            .spawn(Signaler::new(a))
            .id();
        self.world_handle
            .as_mut()
            .unwrap()
            .get_resource_mut::<IdTable>()
            .unwrap()
            .add_action(ah, signaler);
    }
    fn lookup_target_entity<TH: Into<TargetHandle>>(&self, th: TH) -> Option<Entity> {
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
#[derive(Bundle)]
pub struct Signaler<A: Actionable> {
    action: SignaledAction<A>,
    signal: Signal,
}
impl<A: Actionable> Signaler<A> {
    pub fn new(a: A) -> Self {
        Self {
            action: SignaledAction {
                a: Action { data: a },
            },
            signal: Signal(false),
        }
    }
}
