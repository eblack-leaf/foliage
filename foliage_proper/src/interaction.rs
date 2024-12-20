use std::collections::HashMap;

use crate::ash::ClippingContext;
use crate::coordinate::elevation::RenderLayer;
use crate::coordinate::position::Position;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, LogicalContext};
use crate::elm::{Elm, InternalStage};
use crate::ginkgo::ScaleFactor;
use crate::grid::responsive::evaluate::{EvaluateLocation, ScrollExtent, ScrollView};
use crate::tree::Tree;
use crate::Root;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, ResMut, Resource};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton, Touch, TouchPhase};
use winit::keyboard::{Key, ModifiersState};

#[derive(Resource, Default)]
pub(crate) struct TouchAdapter {
    primary: Option<u64>,
}
impl TouchAdapter {
    pub(crate) fn parse(
        &mut self,
        touch: Touch,
        viewport_position: Position<LogicalContext>,
        scale_factor: ScaleFactor,
    ) -> Option<ClickInteraction> {
        let position = Position::device((touch.location.x, touch.location.y))
            .to_logical(scale_factor.value())
            + viewport_position;
        if self.primary.is_none() {
            if touch.phase == TouchPhase::Started {
                self.primary.replace(touch.id);
                return Some(ClickInteraction::new(ClickPhase::Start, position, false));
            }
        } else if self.primary.unwrap() == touch.id {
            match touch.phase {
                TouchPhase::Started => {}
                TouchPhase::Moved => {
                    return Some(ClickInteraction::new(ClickPhase::Moved, position, false));
                }
                TouchPhase::Ended => {
                    self.primary.take();
                    return Some(ClickInteraction::new(ClickPhase::End, position, false));
                }
                TouchPhase::Cancelled => {
                    self.primary.take();
                    return Some(ClickInteraction::new(ClickPhase::Cancel, position, false));
                }
            }
        }
        None
    }
}
#[derive(Resource, Default)]
pub(crate) struct MouseAdapter {
    started: bool,
    pub(crate) cursor: Position<LogicalContext>,
}
impl MouseAdapter {
    pub(crate) fn parse(
        &mut self,
        mouse_button: MouseButton,
        state: ElementState,
    ) -> Option<ClickInteraction> {
        if mouse_button != MouseButton::Left {
            return None;
        }
        if self.started && !state.is_pressed() {
            self.started = false;
            return Some(ClickInteraction::new(ClickPhase::End, self.cursor, false));
        }
        if !self.started && state.is_pressed() {
            self.started = true;
            return Some(ClickInteraction::new(ClickPhase::Start, self.cursor, false));
        }
        None
    }
    pub(crate) fn set_cursor(
        &mut self,
        position: PhysicalPosition<f64>,
        viewport_position: Position<LogicalContext>,
        scale_factor: ScaleFactor,
    ) -> Option<ClickInteraction> {
        let adjusted_position =
            Position::device((position.x, position.y)).to_logical(scale_factor.value());
        self.cursor = adjusted_position;
        if self.started {
            return Some(ClickInteraction::new(
                ClickPhase::Moved,
                adjusted_position + viewport_position,
                false,
            ));
        }
        None
    }
}
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ClickPhase {
    Start,
    Moved,
    End,
    Cancel,
}
#[derive(Event, Debug, Copy, Clone)]
pub struct ClickInteraction {
    click_phase: ClickPhase,
    position: Position<LogicalContext>,
    from_scroll: bool,
}
impl ClickInteraction {
    pub fn new(
        click_phase: ClickPhase,
        position: Position<LogicalContext>,
        from_scroll: bool,
    ) -> Self {
        Self {
            click_phase,
            position,
            from_scroll,
        }
    }
}
#[derive(Default, Copy, Clone, Debug)]
pub struct Click {
    pub start: Position<LogicalContext>,
    pub current: Position<LogicalContext>,
    pub end: Option<Position<LogicalContext>>,
}
impl Click {
    pub fn new(start: Position<LogicalContext>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
#[derive(Default, Copy, Clone, Component)]
pub struct ClickInteractionListener {
    click: Click,
    focused: bool,
    engaged_start: bool,
    engaged: bool,
    engaged_end: bool,
    active: bool,
    shape: ClickInteractionShape,
    disabled: bool,
    moved: bool,
    pass_through: bool,
    listen_scroll: bool,
}
impl ClickInteractionListener {
    pub(crate) const DRAG_THRESHOLD: CoordinateUnit = 20.0;
    pub fn new() -> Self {
        Self {
            click: Default::default(),
            focused: false,
            engaged_start: false,
            engaged: false,
            engaged_end: false,
            active: false,
            shape: Default::default(),
            disabled: false,
            moved: false,
            pass_through: false,
            listen_scroll: false,
        }
    }
    pub fn as_circle(mut self) -> Self {
        self.shape = ClickInteractionShape::Circle;
        self
    }
    pub fn pass_through(mut self) -> Self {
        self.pass_through = true;
        self
    }
    pub fn listen_scroll(mut self) -> Self {
        self.listen_scroll = true;
        self
    }
    pub fn click(&self) -> Click {
        self.click
    }
    pub fn active(&self) -> bool {
        self.active
    }
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
    pub fn disable(&mut self) {
        self.disabled = true;
    }
    pub fn enable(&mut self) {
        self.disabled = false;
    }
    pub fn engaged(&self) -> bool {
        self.engaged
    }
    pub fn engaged_start(&self) -> bool {
        self.engaged_start
    }
    pub fn engaged_end(&self) -> bool {
        self.engaged_end
    }
    pub fn set_shape(&mut self, shape: ClickInteractionShape) {
        self.shape = shape;
    }
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}
#[derive(Resource, Default)]
pub(crate) struct InteractiveEntity(pub(crate) Option<Entity>);
#[derive(Resource, Default)]
pub(crate) struct FocusedEntity(pub(crate) Option<Entity>);
#[derive(Copy, Clone, Default)]
pub enum ClickInteractionShape {
    Circle,
    #[default]
    Rectangle,
}
impl ClickInteractionShape {
    pub fn contains(&self, p: Position<LogicalContext>, section: Section<LogicalContext>) -> bool {
        match self {
            ClickInteractionShape::Circle => {
                section.center().distance(p) <= section.area.width() / 2f32
            }
            ClickInteractionShape::Rectangle => section.contains(p),
        }
    }
}
#[derive(Event, Copy, Clone, Default)]
pub struct OnClick {}
pub(crate) fn on_click(
    on_clicks: Query<(Entity, &ClickInteractionListener), Changed<ClickInteractionListener>>,
    mut tree: Tree,
) {
    for (e, listener) in on_clicks.iter() {
        if listener.active {
            tree.trigger_targets(OnClick {}, e);
        }
    }
}
pub(crate) fn disabled_listeners(
    mut listeners: Query<
        (Entity, &mut ClickInteractionListener),
        Changed<ClickInteractionListener>,
    >,
    mut grabbed: ResMut<InteractiveEntity>,
    mut focused: ResMut<FocusedEntity>,
) {
    for (entity, mut listener) in listeners.iter_mut() {
        if listener.disabled {
            if let Some(g) = grabbed.0 {
                if g == entity {
                    grabbed.0.take();
                    focused.0.take();
                    listener.engaged = false;
                    listener.engaged_end = false;
                }
            }
        }
    }
}
#[derive(Resource, Default)]
pub(crate) struct PassThroughInteractions {
    ps: Vec<Entity>,
}
pub(crate) fn listen_for_interactions(
    mut listeners: Query<(
        Entity,
        &mut ClickInteractionListener,
        &Section<LogicalContext>,
        &RenderLayer,
        &ClippingContext,
    )>,
    mut draggable: Query<&mut Draggable>,
    clip_sections: Query<&Section<LogicalContext>>,
    mut events: EventReader<ClickInteraction>,
    mut grabbed: ResMut<InteractiveEntity>,
    mut focused: ResMut<FocusedEntity>,
    mut pass_through: ResMut<PassThroughInteractions>,
) {
    for event in events.read() {
        match event.click_phase {
            ClickPhase::Start => {
                pass_through.ps.clear();
                if grabbed.0.is_none() {
                    let mut grab_info: Option<(Entity, RenderLayer)> = None;
                    for (entity, listener, section, layer, clip_context) in listeners.iter_mut() {
                        if listener.shape.contains(event.position, *section) && !listener.disabled {
                            if event.from_scroll && !listener.listen_scroll {
                                continue;
                            }
                            match clip_context {
                                ClippingContext::Screen => {}
                                ClippingContext::Entity(e) => {
                                    if let Ok(sec) = clip_sections.get(*e) {
                                        if !sec.contains(event.position) {
                                            continue;
                                        }
                                    }
                                }
                            }
                            if listener.pass_through {
                                pass_through.ps.push(entity);
                            } else {
                                if grab_info.is_none() || *layer > grab_info.unwrap().1 {
                                    grab_info.replace((entity, *layer));
                                }
                            }
                        }
                    }
                    if let Some(grab) = grab_info {
                        if let Some(entity) = focused.0.replace(grab.0) {
                            if let Ok(mut l) = listeners.get_mut(entity) {
                                l.1.focused = false;
                            }
                        }
                        grabbed.0.replace(grab.0);
                        listeners.get_mut(grab.0).expect("starting").1.click =
                            Click::new(event.position);
                        listeners.get_mut(grab.0).unwrap().1.focused = true;
                        listeners.get_mut(grab.0).unwrap().1.engaged = true;
                        listeners.get_mut(grab.0).unwrap().1.engaged_start = true;
                    } else if let Some(entity) = focused.0.take() {
                        if let Ok(mut l) = listeners.get_mut(entity) {
                            l.1.focused = false;
                        }
                    }
                    for e in pass_through.ps.iter().copied() {
                        listeners.get_mut(e).expect("starting").1.click =
                            Click::new(event.position);
                        listeners.get_mut(e).unwrap().1.engaged = true;
                        listeners.get_mut(e).unwrap().1.engaged_start = true;
                    }
                }
            }
            ClickPhase::Moved => {
                if !pass_through.ps.is_empty() {
                    if let Some(g) = grabbed.0 {
                        let delta =
                            (event.position - listeners.get(g).unwrap().1.click.start).abs();
                        if delta.x() > ClickInteractionListener::DRAG_THRESHOLD
                            || delta.y() > ClickInteractionListener::DRAG_THRESHOLD
                        {
                            grabbed.0.take();
                            listeners.get_mut(g).unwrap().1.engaged_end = true;
                            listeners.get_mut(g).unwrap().1.engaged = false;
                            move_pass_through(
                                &mut listeners,
                                &mut draggable,
                                &pass_through,
                                event,
                                true,
                            );
                        } else {
                            move_grabbed(&mut listeners, &grabbed, event);
                        }
                    } else {
                        move_pass_through(
                            &mut listeners,
                            &mut draggable,
                            &pass_through,
                            event,
                            false,
                        );
                    }
                } else {
                    move_grabbed(&mut listeners, &grabbed, event);
                }
            }
            ClickPhase::End => {
                if let Some(g) = grabbed.0.take() {
                    end_interaction(&mut listeners, &clip_sections, event, g);
                }
                for ps in pass_through.ps.drain(..) {
                    end_interaction(&mut listeners, &clip_sections, event, ps);
                }
            }
            ClickPhase::Cancel => {
                if let Some(g) = grabbed.0.take() {
                    listeners.get_mut(g).unwrap().1.engaged_end = true;
                    listeners.get_mut(g).unwrap().1.engaged = false;
                }
                for ps in pass_through.ps.drain(..) {
                    listeners.get_mut(ps).unwrap().1.engaged_end = true;
                    listeners.get_mut(ps).unwrap().1.engaged = false;
                }
            }
        }
    }
}

fn move_pass_through(
    listeners: &mut Query<(
        Entity,
        &mut ClickInteractionListener,
        &Section<LogicalContext>,
        &RenderLayer,
        &ClippingContext,
    )>,
    draggable: &mut Query<&mut Draggable>,
    pass_through: &ResMut<PassThroughInteractions>,
    event: &ClickInteraction,
    is_first: bool,
) {
    for p in pass_through.ps.iter() {
        if is_first {
            listeners.get_mut(*p).unwrap().1.click.start = event.position;
            if let Ok(mut drag) = draggable.get_mut(*p) {
                drag.last = event.position;
            }
        }
        listeners.get_mut(*p).unwrap().1.click.current = event.position;
        listeners.get_mut(*p).unwrap().1.moved = true;
    }
}

fn move_grabbed(
    listeners: &mut Query<(
        Entity,
        &mut ClickInteractionListener,
        &Section<LogicalContext>,
        &RenderLayer,
        &ClippingContext,
    )>,
    grabbed: &ResMut<InteractiveEntity>,
    event: &ClickInteraction,
) {
    if let Some(g) = grabbed.0 {
        listeners.get_mut(g).unwrap().1.click.current = event.position;
        listeners.get_mut(g).unwrap().1.moved = true;
    }
}

fn end_interaction(
    listeners: &mut Query<(
        Entity,
        &mut ClickInteractionListener,
        &Section<LogicalContext>,
        &RenderLayer,
        &ClippingContext,
    )>,
    clip_sections: &Query<&Section<LogicalContext>>,
    event: &ClickInteraction,
    g: Entity,
) {
    let section = *listeners.get(g).unwrap().2;
    if listeners
        .get(g)
        .unwrap()
        .1
        .shape
        .contains(event.position, section)
    {
        match listeners.get(g).unwrap().4 {
            ClippingContext::Screen => {
                listeners.get_mut(g).unwrap().1.active = true;
            }
            ClippingContext::Entity(e) => {
                if let Ok(sec) = clip_sections.get(*e) {
                    if sec.contains(event.position) {
                        listeners.get_mut(g).unwrap().1.active = true;
                    }
                }
            }
        }
    }
    listeners.get_mut(g).expect("ending").1.click.current = event.position;
    listeners
        .get_mut(g)
        .expect("ending")
        .1
        .click
        .end
        .replace(event.position);
    listeners.get_mut(g).unwrap().1.engaged_end = true;
    listeners.get_mut(g).unwrap().1.engaged = false;
    listeners.get_mut(g).unwrap().1.moved = true;
}

pub(crate) fn reset_click_listener_flags(mut listeners: Query<&mut ClickInteractionListener>) {
    for mut listener in listeners.iter_mut() {
        listener.engaged_start = false;
        listener.engaged_end = false;
        listener.active = false;
        listener.moved = false;
    }
}
#[derive(Event, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct InputSequence {
    key: Key,
    mods: ModifiersState,
}
impl InputSequence {
    pub fn new(key: Key, mods: ModifiersState) -> Self {
        Self { key, mods }
    }
}
#[derive(Resource, Default)]
pub(crate) struct KeyboardAdapter {
    cache: HashMap<Key, ElementState>,
    pub(crate) mods: ModifiersState,
}
impl KeyboardAdapter {
    pub(crate) fn parse(&mut self, key: Key, state: ElementState) -> Option<InputSequence> {
        if let Some(cached) = self.cache.insert(key.clone(), state) {
            if cached != state && state.is_pressed() {
                return Some(InputSequence::new(key, self.mods));
            }
        }
        None
    }
}
#[derive(Component, Copy, Clone, Default)]
pub struct Draggable {
    pub(crate) last: Position<LogicalContext>, // for relative setting from absolute drag
}
pub(crate) fn draggable(
    mut listeners: Query<(
        Entity,
        &ClickInteractionListener,
        &mut Draggable,
        &Section<LogicalContext>,
        &mut ScrollView,
        &ScrollExtent,
    )>,
    mut tree: Tree,
) {
    for (entity, listener, mut draggable, section, mut view, extent) in listeners.iter_mut() {
        if listener.engaged_start {
            draggable.last = listener.click.start;
        }
        if listener.moved {
            let diff = draggable.last - listener.click.current;
            let mut to_set = diff;
            if view.position.x() + diff.x() + section.area.width()
                > extent.horizontal_extent.vertical()
            {
                to_set.set_x(
                    (extent.horizontal_extent.vertical()
                        - (view.position.x() + section.area.width()))
                    .max(0.0),
                );
            };
            if view.position.x() + diff.x() < extent.horizontal_extent.horizontal() {
                to_set.set_x(extent.horizontal_extent.horizontal() - view.position.x());
            }
            if view.position.y() + diff.y() + section.area.height()
                > extent.vertical_extent.vertical()
            {
                let set_y = (extent.vertical_extent.vertical()
                    - (view.position.y() + section.area.height()))
                .max(0.0);
                to_set.set_y(set_y);
            }
            if view.position.y() + diff.y() < extent.vertical_extent.horizontal() {
                let set_y = extent.vertical_extent.horizontal() - view.position.y();
                to_set.set_y(set_y);
            }
            view.position += to_set;
            tree.entity(entity)
                .insert(EvaluateLocation::skip_extent_check());
            draggable.last = listener.click.current;
        }
    }
}
impl Root for ClickInteractionListener {
    fn attach(elm: &mut Elm) {
        elm.scheduler.main.add_systems((
            (disabled_listeners, listen_for_interactions, on_click)
                .chain()
                .in_set(InternalStage::External),
            draggable.in_set(InternalStage::Apply),
            reset_click_listener_flags.after(InternalStage::Resolve),
        ));
        elm.ecs.insert_resource(PassThroughInteractions::default());
        elm.enable_event::<ClickInteraction>();
        elm.enable_event::<InputSequence>();
    }
}
