use crate::composite::list::{List, ListItems, RowHeight, SelectedIndex};
use crate::composite::text_input::action::InputAction;
use crate::composite::text_input::HintColor;
use crate::composite::{Root, Selected};
use crate::handle_replace;
use crate::IntoTargets;
use crate::TextInputAction;
use crate::{
    Attachment, Component, Composite, EcsExtension, Elevation, FocusBehavior, Foliage, FontSize,
    Grid, GridExt, HintText, Icon, IconValue, InteractionPropagation, Location, Outline, Panel,
    Primary, Rounding, Secondary, Stem, Tertiary, TextChanged, TextInput, TextValue, Tree, Update,
    Visibility,
};
use bevy_ecs::entity::Entity;
use crate::Trigger;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::lifecycle::{HookContext, Insert};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

/// Search-box composite: outlined panel + leading icon + [`TextInput`] + a suggestion [`List`]
/// fed by deterministic prefix-matching over `SuggestionProvider`. Up/Down move the highlighted
/// suggestion, Tab commits it into the input, Enter emits [`Submitted`], Escape hides the
/// suggestions. Theming: `Primary` = text/icon, `Secondary` = panel, `Tertiary` = cursor/
/// selection/suggestion highlight.
#[derive(Component, Clone)]
#[component(on_add = Self::on_add)]
#[component(on_insert = Self::on_insert)]
#[require(SuggestionProvider, HintText, HintColor, Primary, Secondary, Tertiary)]
#[require(IconValue, FontSize, RowHeight)]
pub struct Prompt {}

/// The full set of committable values suggestions are prefix-matched against.
#[derive(Component, Clone, Default)]
pub struct SuggestionProvider(pub Vec<String>);

/// Emitted at the Prompt root when Enter is pressed in its input.
#[derive(EntityEvent, Clone)]
pub struct Submitted {
    entity: Entity,
    pub value: String,
}
impl Submitted {
    pub fn new<S: Into<String>>(value: S) -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            value: value.into(),
        }
    }
}
crate::targeted_event!(Submitted);

impl Prompt {
    const MAX_SUGGESTIONS: usize = 5;
    pub fn new() -> Prompt {
        Prompt {}
    }
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        world
            .commands()
            .entity(this)
            .observe(Self::forward_primary)
            .observe(Self::update_primary)
            .observe(Self::forward_secondary)
            .observe(Self::update_secondary)
            .observe(Self::update_tertiary)
            .observe(Self::forward_tertiary)
            .observe(Self::forward_icon)
            .observe(Self::update_icon)
            .observe(Self::forward_hint)
            .observe(Self::update_hint);
    }
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        let this = ctx.entity;
        let font_size = *world.get::<FontSize>(this).unwrap();
        let row_height = world.get::<RowHeight>(this).unwrap().0;
        world
            .commands()
            .entity(this)
            .insert((Grid::default(), CurrentlyTyped::default()));
        let panel = world.commands().leaf((
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                0.pct().top().with(100.pct().bottom()),
            ),
            Panel::new(),
            Rounding::Md,
            Outline::new(3),
            Elevation::up(1),
            Stem::some(this),
            Grid::default(),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        let icon_value = *world.get::<IconValue>(this).unwrap();
        let icon = world.commands().leaf((
            Icon::new(icon_value.0),
            Elevation::up(2),
            Location::new().xs(
                8.px().left().with(24.px().width()),
                50.pct().center_y().with(24.px().height()),
            ),
            Stem::some(panel),
            InteractionPropagation::pass_through(),
            FocusBehavior::ignore(),
            Root(this),
        ));
        let input = world.commands().leaf((
            TextInput::new(),
            font_size,
            Location::new().xs(
                40.px().left().with(100.pct().right().adjust(-8)),
                50.pct().center_y().with(1.letters().height().adjust(8)),
            ),
            Stem::some(panel),
            Elevation::up(2),
            Root(this),
        ));
        world
            .commands()
            .entity(input)
            .observe(Self::listen_input)
            .observe(Self::text_changed);
        let suggestions = world.commands().leaf((
            List::new(),
            RowHeight(row_height),
            font_size,
            Stem::some(this),
            Elevation::up(10),
            Location::new().xs(
                0.pct().left().with(100.pct().right()),
                100.pct().top().adjust(4).with(0.px().height()),
            ),
            Visibility::new(false),
            Root(this),
        ));
        world
            .commands()
            .entity(suggestions)
            .observe(Self::suggestion_clicked);
        let handle = Handle {
            panel,
            icon,
            input,
            suggestions,
        };
        world.commands().entity(this).insert(handle);
    }
    /// Deterministic prefix match, first `MAX_SUGGESTIONS` in provider order.
    fn matches(provider: &SuggestionProvider, typed: &str) -> Vec<String> {
        if typed.is_empty() {
            return vec![];
        }
        provider
            .0
            .iter()
            .filter(|v| v.starts_with(typed) && v.as_str() != typed)
            .take(Self::MAX_SUGGESTIONS)
            .cloned()
            .collect()
    }
    fn text_changed(
        trigger: Trigger<TextChanged>,
        mut tree: Tree,
        roots: Query<&Root>,
        values: Query<&TextValue>,
        providers: Query<&SuggestionProvider>,
        handles: Query<&Handle>,
        mut typed: Query<&mut CurrentlyTyped>,
        row_heights: Query<&RowHeight>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let handle = handles.get(root).unwrap();
        let value = values.get(handle.input).unwrap().0.clone();
        typed.get_mut(root).unwrap().value = value.clone();
        let matches = Self::matches(providers.get(root).unwrap(), &value);
        let row_height = row_heights.get(root).unwrap().0;
        let height = (matches.len() as i32) * (row_height + 2) + 2;
        tree.entity(handle.suggestions)
            .insert(Visibility::new(!matches.is_empty()))
            .insert(Location::new().xs(
                0.pct().left().with(100.pct().right()),
                100.pct().top().adjust(4).with(height.px().height()),
            ))
            .insert(ListItems(matches))
            .insert(SelectedIndex(None));
    }
    fn listen_input(
        trigger: Trigger<InputAction>,
        mut tree: Tree,
        roots: Query<&Root>,
        handles: Query<&Handle>,
        values: Query<&TextValue>,
        items: Query<&ListItems>,
        selected: Query<&SelectedIndex>,
        visibilities: Query<&Visibility>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let handle = handles.get(root).unwrap();
        let suggestions_visible = visibilities.get(handle.suggestions).unwrap().visible();
        let count = items.get(handle.suggestions).unwrap().0.len();
        let current = selected.get(handle.suggestions).unwrap().0;
        match trigger.event().action {
            TextInputAction::Up if suggestions_visible && count > 0 => {
                let next = match current {
                    Some(0) | None => count - 1,
                    Some(i) => i - 1,
                };
                tree.entity(handle.suggestions)
                    .insert(SelectedIndex(Some(next)));
            }
            TextInputAction::Down if suggestions_visible && count > 0 => {
                let next = match current {
                    Some(i) if i + 1 < count => i + 1,
                    _ => 0,
                };
                tree.entity(handle.suggestions)
                    .insert(SelectedIndex(Some(next)));
            }
            TextInputAction::Tab if suggestions_visible => {
                let index = current.unwrap_or(0);
                if let Some(value) = items.get(handle.suggestions).unwrap().0.get(index) {
                    Self::commit(&mut tree, handle, value.clone());
                }
            }
            TextInputAction::Escape => {
                tree.entity(handle.suggestions)
                    .insert(Visibility::new(false));
            }
            TextInputAction::Enter => {
                // a highlighted suggestion wins over the raw typed value
                let highlighted = if suggestions_visible { current } else { None };
                let value = highlighted
                    .and_then(|i| items.get(handle.suggestions).unwrap().0.get(i).cloned())
                    .unwrap_or_else(|| values.get(handle.input).unwrap().0.clone());
                if highlighted.is_some() {
                    Self::commit(&mut tree, handle, value.clone());
                }
                tree.trigger_targets(Submitted::new(value), root);
            }
            _ => {}
        }
    }
    fn commit(tree: &mut Tree, handle: &Handle, value: String) {
        tree.entity(handle.input).insert(TextValue(value));
        tree.entity(handle.suggestions)
            .insert(Visibility::new(false));
    }
    fn suggestion_clicked(
        trigger: Trigger<Selected>,
        mut tree: Tree,
        roots: Query<&Root>,
        handles: Query<&Handle>,
        items: Query<&ListItems>,
    ) {
        let root = roots.get(trigger.event_target()).unwrap().0;
        let handle = handles.get(root).unwrap();
        if let Some(value) = items
            .get(handle.suggestions)
            .unwrap()
            .0
            .get(trigger.event().index)
        {
            Self::commit(&mut tree, handle, value.clone());
        }
    }
    fn handle_trigger(trigger: Trigger<Insert, Handle>, mut tree: Tree) {
        let this = trigger.event_target();
        tree.trigger_targets(Update::<Primary>::new(), this);
        tree.trigger_targets(Update::<Secondary>::new(), this);
        tree.trigger_targets(Update::<Tertiary>::new(), this);
        tree.trigger_targets(Update::<HintColor>::new(), this);
        tree.trigger_targets(Update::<IconValue>::new(), this);
    }
    fn forward_primary(trigger: Trigger<Insert, Primary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Primary>::new(), trigger.event_target());
    }
    fn update_primary(
        trigger: Trigger<Update<Primary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        primaries: Query<&Primary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let primary = *primaries.get(this).unwrap();
        tree.entity(handle.icon).insert(primary.0);
        tree.entity(handle.input).insert(primary);
        tree.entity(handle.suggestions).insert(primary);
    }
    fn forward_secondary(trigger: Trigger<Insert, Secondary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Secondary>::new(), trigger.event_target());
    }
    fn update_secondary(
        trigger: Trigger<Update<Secondary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        secondaries: Query<&Secondary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let secondary = *secondaries.get(this).unwrap();
        tree.entity(handle.panel).insert(secondary.0);
        tree.entity(handle.input).insert(secondary);
        tree.entity(handle.suggestions).insert(secondary);
    }
    fn forward_tertiary(trigger: Trigger<Insert, Tertiary>, mut tree: Tree) {
        tree.trigger_targets(Update::<Tertiary>::new(), trigger.event_target());
    }
    fn update_tertiary(
        trigger: Trigger<Update<Tertiary>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        tertiaries: Query<&Tertiary>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        let tertiary = *tertiaries.get(this).unwrap();
        tree.entity(handle.input).insert(tertiary);
        tree.entity(handle.suggestions).insert(tertiary);
    }
    fn forward_icon(trigger: Trigger<Insert, IconValue>, mut tree: Tree) {
        tree.trigger_targets(Update::<IconValue>::new(), trigger.event_target());
    }
    fn update_icon(
        trigger: Trigger<Update<IconValue>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        icons: Query<&IconValue>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        tree.entity(handle.icon)
            .insert(Icon::new(icons.get(this).unwrap().0));
    }
    fn forward_hint(trigger: Trigger<Insert, HintText>, mut tree: Tree) {
        tree.trigger_targets(Update::<HintColor>::new(), trigger.event_target());
    }
    fn update_hint(
        trigger: Trigger<Update<HintColor>>,
        mut tree: Tree,
        handles: Query<&Handle>,
        hints: Query<&HintText>,
        hint_colors: Query<&HintColor>,
    ) {
        let this = trigger.event_target();
        let handle = handles.get(this).unwrap();
        tree.entity(handle.input)
            .insert(hints.get(this).unwrap().clone())
            .insert(hint_colors.get(this).unwrap().clone());
    }
}
impl Default for Prompt {
    fn default() -> Self {
        Self::new()
    }
}
impl Attachment for Prompt {
    fn attach(foliage: &mut Foliage) {
        foliage.define(Self::handle_trigger);
    }
}
/// What the user has actually typed (as opposed to a committed suggestion).
#[derive(Component, Clone, Default)]
pub(crate) struct CurrentlyTyped {
    pub(crate) value: String,
}
#[derive(Component, Clone)]
#[component(on_discard = handle_replace::<Prompt>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub input: Entity,
    pub suggestions: Entity,
}
impl Composite for Prompt {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl IntoTargets + Send + Sync + 'static {
        [handle.panel, handle.input, handle.icon, handle.suggestions]
    }
}
