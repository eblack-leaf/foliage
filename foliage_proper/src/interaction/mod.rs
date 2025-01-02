use crate::coordinate::position::Position;
use crate::coordinate::Logical;
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::{Event, EventReader};
use bevy_ecs::prelude::{Component, IntoSystemConfigs};
use bevy_ecs::system::{Query, ResMut, Resource};
use bevy_ecs::world::DeferredWorld;

mod adapter;
use crate::ash::clip::ClipSection;
use crate::grid::{viewport_changed, View};
use crate::{Attachment, CoordinateUnit, Foliage, ResolvedElevation, Section, Tree};
pub use adapter::InputSequence;
pub(crate) use adapter::{KeyboardAdapter, MouseAdapter, TouchAdapter};

impl Attachment for Interaction {
    fn attach(foliage: &mut Foliage) {
        foliage
            .main
            .add_systems(interactive_elements.after(viewport_changed));
        foliage.world.insert_resource(KeyboardAdapter::default());
        foliage.world.insert_resource(MouseAdapter::default());
        foliage.world.insert_resource(TouchAdapter::default());
        foliage.enable_queued_event::<Interaction>();
    }
}
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum InteractionPhase {
    Start,
    Moved,
    End,
    Cancel,
}
#[derive(Event, Debug, Copy, Clone)]
pub struct Interaction {
    click_phase: InteractionPhase,
    position: Position<Logical>,
    from_scroll: bool,
}
impl Interaction {
    pub fn new(
        click_phase: InteractionPhase,
        position: Position<Logical>,
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
    pub start: Position<Logical>,
    pub current: Position<Logical>,
    pub end: Option<Position<Logical>>,
}
impl Click {
    pub fn new(start: Position<Logical>) -> Self {
        Self {
            start,
            current: start,
            end: None,
        }
    }
}
#[derive(Resource, Default)]
pub(crate) struct CurrentInteraction {
    pub(crate) primary: Option<Entity>,
    pub(crate) pass_through: Vec<Entity>,
}
#[derive(Event, Copy, Clone, Default)]
pub struct OnClick {}
#[derive(Component, Copy, Clone)]
#[component(on_replace = Self::on_replace)]
pub struct InteractionListener {
    pub click: Click,
    pub disabled: bool,
    pub scroll: bool,
    pub pass_through: bool,
    pub shape: InteractionShape,
    pub(crate) last_drag: Position<Logical>,
}
impl InteractionListener {
    pub const DRAG_THRESHOLD: CoordinateUnit = 40.0;
    pub fn new() -> Self {
        Self {
            click: Default::default(),
            disabled: false,
            scroll: false,
            pass_through: false,
            shape: Default::default(),
            last_drag: Default::default(),
        }
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn circle(mut self) -> Self {
        self.shape = InteractionShape::Circle;
        self
    }
    pub fn scroll(mut self, s: bool) -> Self {
        self.scroll = s;
        self
    }
    pub fn is_contained(
        &self,
        section: Section<Logical>,
        clip: Option<ClipSection>,
        event: Position<Logical>,
    ) -> bool {
        let section_contained = match self.shape {
            InteractionShape::Rectangle => section.contains(event),
            InteractionShape::Circle => section.center().distance(event) <= section.width() / 2f32,
        };
        let clip_contained = if let Some(clip) = clip {
            if let Some(c) = clip.0 {
                c.contains(event)
            } else {
                true
            }
        } else {
            true
        };
        section_contained && clip_contained
    }
    fn on_replace(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        if let Some(mut current) = world.get_resource_mut::<CurrentInteraction>() {
            if let Some(p) = current.primary {
                if p == this {
                    current.primary.take();
                }
            }
        }
    }
}
#[derive(Copy, Clone, Default)]
pub enum InteractionShape {
    #[default]
    Rectangle,
    Circle,
}
pub(crate) fn interactive_elements(
    mut reader: EventReader<Interaction>,
    mut listeners: Query<(
        Entity,
        &mut InteractionListener,
        &Section<Logical>,
        &ResolvedElevation,
        Option<&ClipSection>,
        Option<&mut View>,
    )>,
    mut current: ResMut<CurrentInteraction>,
    mut tree: Tree,
) {
    let events = reader.read().copied().collect::<Vec<_>>();
    if events
        .iter()
        .find(|e| e.click_phase == InteractionPhase::Cancel)
        .is_some()
    {
        current.primary.take();
        current.pass_through.clear();
    } else {
        let started = events
            .iter()
            .copied()
            .filter(|i| i.click_phase == InteractionPhase::Start)
            .collect::<Vec<_>>();
        let moved = events
            .iter()
            .copied()
            .filter(|i| i.click_phase == InteractionPhase::Moved)
            .collect::<Vec<_>>();
        let ended = events
            .iter()
            .copied()
            .filter(|i| i.click_phase == InteractionPhase::End)
            .collect::<Vec<_>>();
        if let Some(event) = started.last() {
            current.primary.take();
            current.pass_through.clear();
            let mut grabbed_elevation = ResolvedElevation::new(100.0);
            for (entity, mut listener, section, elevation, clip, _) in listeners.iter_mut() {
                if !listener.scroll && event.from_scroll || listener.disabled {
                    continue;
                }
                if listener.is_contained(*section, clip.copied(), event.position) {
                    if listener.pass_through {
                        current.pass_through.push(entity);
                    } else {
                        if elevation >= &grabbed_elevation {
                            current.primary.replace(entity);
                            grabbed_elevation = *elevation;
                        }
                    }
                }
            }
            if let Some(p) = current.primary {
                listeners.get_mut(p).unwrap().1.click = Click::new(event.position);
                // TODO tree.trigger_targets(EngagedBegin::default(), p);
            }
            for ps in current.pass_through.iter() {
                let mut listener = listeners.get_mut(*ps).unwrap().1;
                listener.click = Click::new(event.position);
                listener.last_drag = event.position;
            }
        }
        if let Some(event) = moved.last() {
            if let Some(p) = current.primary {
                let scroll_delta = event.position - listeners.get(p).unwrap().1.click.start;
                if scroll_delta.coordinates.a() > InteractionListener::DRAG_THRESHOLD
                    || scroll_delta.coordinates.b() > InteractionListener::DRAG_THRESHOLD
                {
                    // TODO tree.trigger_targets(EngagedEnd::default(), p);
                    current.primary.take();
                    for ps in current.pass_through.iter() {
                        let mut listener = listeners.get_mut(*ps).unwrap();
                        listener.1.click = Click::new(event.position);
                        listener.1.last_drag = event.position;
                    }
                }
            }
            if let Some(p) = current.primary {
                listeners.get_mut(p).unwrap().1.click.current = event.position;
            } else {
                for ps in current.pass_through.iter() {
                    let mut listener = listeners.get_mut(*ps).unwrap();
                    listener.1.click.current = event.position;
                    let diff = listener.1.last_drag - event.position;
                    if listener.1.scroll {
                        if let Some(mut view) = listener.5 {
                            view.offset += diff;
                        }
                    }
                    listener.1.last_drag = event.position;
                }
            }
        }
        if let Some(event) = ended.last() {
            if let Some(p) = current.primary {
                let mut listener = listeners.get_mut(p).unwrap();
                if listener
                    .1
                    .is_contained(*listener.2, listener.4.copied(), event.position)
                {
                    listener.1.click.end.replace(event.position);
                    tree.trigger_targets(OnClick::default(), p);
                } else {
                    // TODO tree.trigger_targets(EngagedEnd::default(), p);
                }
            }
            for ps in current.pass_through.drain(..) {
                let mut listener = listeners.get_mut(ps).unwrap();
                if listener
                    .1
                    .is_contained(*listener.2, listener.4.copied(), event.position)
                {
                    listener.1.click.end.replace(event.position);
                    tree.trigger_targets(OnClick::default(), ps);
                } else {
                    // TODO tree.trigger_targets(EngagedEnd::default(), ps);
                }
            }
        }
    }
}
