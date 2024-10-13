use crate::coordinate::section::Section;
use crate::coordinate::LogicalContext;
use crate::icon::{Icon, IconId};
use crate::interaction::ClickInteractionListener;
use crate::leaf::Leaf;
use crate::panel::{Panel, Rounding};
use crate::style::{Coloring, InteractiveColor};
use crate::text::{FontSize, Text, TextValue};
use crate::twig::Configure;
use bevy_ecs::component::StorageType::SparseSet;
use bevy_ecs::component::{ComponentHooks, ComponentId, StorageType};
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Component, Trigger};
use bevy_ecs::system::Query;
use bevy_ecs::world::DeferredWorld;

#[derive(Copy, Clone)]
pub(crate) enum ButtonShape {
    Circle,
    Square,
}
#[derive(Clone)]
pub struct Button {
    circle_square: ButtonShape,
    coloring: Coloring,
    rounding: Rounding,
    icon_id: IconId,
    text_value: Option<TextValue>,
    font_size: Option<FontSize>,
    outline: u32,
}
impl Button {
    pub fn new<ID: Into<IconId>, C: Into<Coloring>>(id: ID, c: C) -> Button {
        Button {
            circle_square: ButtonShape::Square,
            coloring: c.into(),
            rounding: Default::default(),
            icon_id: id.into(),
            text_value: None,
            font_size: None,
            outline: 0,
        }
    }
    pub fn with_text<T: Into<TextValue>, FS: Into<FontSize>>(mut self, t: T, fs: FS) -> Self {
        self.text_value.replace(t.into());
        self.font_size.replace(fs.into());
        self
    }
    pub fn circle(mut self) -> Self {
        self.rounding = Rounding::all(1.0);
        self.circle_square = ButtonShape::Circle;
        self
    }
    pub fn square(mut self) -> Self {
        self.circle_square = ButtonShape::Square;
        self
    }
    pub fn rounded<R: Into<Rounding>>(mut self, rounding: R) -> Self {
        self.rounding = rounding.into();
        self
    }
    pub fn outline(mut self, outline: u32) -> Self {
        self.outline = outline;
        self
    }
    pub(crate) fn on_insert(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
        // create button using this as root
        let args = world.get::<Button>(entity).unwrap().clone();
        let interaction_listener = match args.circle_square {
            ButtonShape::Circle => ClickInteractionListener::new().as_circle(),
            ButtonShape::Square => ClickInteractionListener::new(),
        };
        world
            .commands()
            .entity(entity)
            .insert(Panel::new(args.rounding, args.coloring.background).outline(args.outline))
            .insert(InteractiveColor::new(
                args.coloring.background,
                args.coloring.foreground,
            ))
            .insert(interaction_listener)
            .observe(configure);
        let icon = world
            .commands()
            .spawn(Leaf::default().stem(entity).elevation(-1))
            .insert(Icon::new(args.icon_id, args.coloring.foreground))
            .id();
        let text = world.commands().spawn(Leaf::new()).id();
        if args.font_size.is_some() {
            world.commands().entity(text).insert(Text::new(
                args.text_value.unwrap_or_default().0,
                args.font_size.unwrap(),
                args.coloring.foreground,
            ));
        }
        world
            .commands()
            .entity(entity)
            .insert(ButtonBindings { icon, text });
    }
}
impl Component for Button {
    const STORAGE_TYPE: StorageType = SparseSet;
    fn register_component_hooks(_hooks: &mut ComponentHooks) {
        _hooks.on_insert(Button::on_insert);
        _hooks.on_remove(|mut world: DeferredWorld, entity: Entity, _| {
            if let Some(bindings) = world.get::<ButtonBindings>(entity).copied() {
                world.commands().entity(bindings.icon).despawn();
                world.commands().entity(bindings.text).despawn();
            }
        });
    }
}
#[derive(Component, Copy, Clone)]
pub struct ButtonBindings {
    pub icon: Entity,
    pub text: Entity,
}
pub(crate) fn configure(
    trigger: Trigger<Configure>,
    mut sections: Query<&mut Section<LogicalContext>>,
    bindings: Query<&ButtonBindings>,
) {
    if let Ok(binding) = bindings.get(trigger.entity()) {
        let main = sections.get(trigger.entity()).copied().unwrap();
        *sections.get_mut(binding.icon).unwrap() = Section::default();
        *sections.get_mut(binding.text).unwrap() = Section::default();
    }
}

// impl Branch for Button {
//     type Handle = ButtonBindings;
//
//     fn grow(self, tree: &mut Tree) -> Self::Handle {
//         let panel = tree.spawn(Leaf::new());
//         let icon = tree.add_leaf();
//         let text = tree.add_leaf();
//         let linked = vec![icon, text];
//
//         tree.stem(panel, twig.stem);
//         tree.location(panel, twig.location);
//         tree.elevation(panel, twig.elevation);
//         tree.entity(panel)
//             .insert(Panel::new(twig.t.rounding, twig.t.coloring.background).outline(twig.t.outline))
//             .insert(
//                 InteractiveColor::new(twig.t.coloring.background, twig.t.coloring.foreground)
//                     .with_linked(linked),
//             )
//             .insert(interaction_listener);
//         let value = twig.t.text_value.unwrap_or_default().0;
//         let icon_location = if value.is_empty() {
//             GridLocation::new()
//                 .center_x(stem().center_x())
//                 .center_y(stem().center_y())
//                 .width(24.px())
//                 .height(24.px())
//         } else {
//             GridLocation::new()
//                 .left(stem().left() + 16.px())
//                 .width(24.px())
//                 .height(24.px())
//                 .center_y(stem().center_y())
//         };
//         tree.stem(icon, Some(panel));
//         tree.location(icon, icon_location);
//         tree.elevation(icon, -1);
//         tree.entity(icon)
//             .insert(Icon::new(twig.t.icon_id, twig.t.coloring.foreground));
//         tree.stem(text, Some(panel));
//         tree.location(
//             text,
//             GridLocation::new()
//                 .left(stem().left() + 48.px())
//                 .right(stem().right() - 16.px())
//                 .center_y(stem().center_y())
//                 .height(90.percent().height().of(stem())),
//         );
//         tree.elevation(text, -1);
//         if twig.t.font_size.is_some() {
//             tree.entity(text).insert(Text::new(
//                 value,
//                 twig.t.font_size.unwrap(),
//                 twig.t.coloring.foreground,
//             ));
//         }
//         tree.flush(panel);
//         ButtonBindings { panel, icon, text }
//     }
// }
