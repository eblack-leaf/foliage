use crate::Justify::Far;
use crate::{handle_replace, Color, EcsExtension, Elevation, FontSize, Grid, GridExt, HorizontalAlignment, Icon, IconId, Location, Panel, ResolvedElevation, Rounding, Stem, Text, VerticalAlignment};
use crate::{Component, Composite};
use bevy_ecs::component::ComponentId;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::TriggerTargets;
use bevy_ecs::world::DeferredWorld;

#[derive(Component, Clone)]
#[component(on_insert = Self::on_insert)]
pub struct Button {
    icon_id: IconId,
    shape: ButtonShape,
    text: String,
    background: Color,
    foreground: Color,
    font_size: FontSize,
}
#[derive(Copy, Clone, PartialEq, Default)]
pub enum ButtonShape {
    Circle,
    #[default]
    Rectangle,
}
impl Button {
    pub fn new() -> Self {
        Self {
            icon_id: 0,
            shape: Default::default(),
            text: "".to_string(),
            background: Default::default(),
            foreground: Default::default(),
            font_size: Default::default(),
        }
    }
    pub fn icon(mut self, icon_id: IconId) -> Self {
        self.icon_id = icon_id;
        self
    }
    pub fn circle(mut self) -> Self {
        self.shape = ButtonShape::Circle;
        self
    }
    pub fn rect(mut self) -> Self {
        self.shape = ButtonShape::Rectangle;
        self
    }
    pub fn text<S: AsRef<str>>(mut self, text: S) -> Self {
        self.text = text.as_ref().to_string();
        self
    }
    pub fn foreground(mut self, c: Color) -> Self {
        self.foreground = c;
        self
    }
    pub fn background(mut self, c: Color) -> Self {
        self.background = c;
        self
    }
    pub fn font_size(mut self, size: FontSize) -> Self {
        self.font_size = size;
        self
    }
    fn on_insert(mut world: DeferredWorld, this: Entity, _c: ComponentId) {
        let args = world.get::<Button>(this).unwrap().clone();
        let elevation = *world.get::<Elevation>(this).unwrap();
        let res = *world.get::<ResolvedElevation>(this).unwrap();
        println!("{:?}, {:?}", elevation, res);
        world
            .commands()
            .entity(this)
            .insert(Grid::new(3.col(), 1.row()));
        let panel = world.commands().leaf((
            Panel::new(),
            match args.shape {
                ButtonShape::Circle => Rounding::Full,
                ButtonShape::Rectangle => Rounding::Sm,
            },
            Stem::some(this),
            Location::new().xs(0.pct().to(100.pct()), 0.pct().to(100.pct())),
            Elevation::up(1),
            args.background,
        ));
        let icon_location = match args.shape {
            ButtonShape::Circle => Location::new().xs(
                0.pct().to(100.pct()).min(24.px()).max(24.px()),
                0.pct().to(100.pct()).min(24.px()).max(24.px()),
            ),
            ButtonShape::Rectangle => Location::new().xs(
                1.col().to(1.col()).min(24.px()).max(24.px()).justify(Far),
                1.row().to(1.row()).min(24.px()).max(24.px()),
            ),
        };
        let icon = world.commands().leaf((
            Icon::new(args.icon_id),
            icon_location,
            args.foreground,
            Elevation::up(2),
            Stem::some(this),
        ));
        let text = world.commands().leaf((
            Text::new(&args.text),
            args.font_size,
            Elevation::up(2),
            Stem::some(this),
            args.foreground,
            HorizontalAlignment::Center,
            VerticalAlignment::Middle,
            Location::new().xs(2.col().to(3.col()), 1.row().to(1.row())),
        ));
        println!("{:?}, {:?}, {:?}", panel, icon, text);
        let handle = Handle { panel, icon, text };
        world.commands().entity(this).insert(handle);
    }
}
impl Composite for Button {
    type Handle = Handle;
    fn remove(handle: &Self::Handle) -> impl TriggerTargets + Send + Sync + 'static {
        [handle.panel, handle.text, handle.icon]
    }
}
#[derive(Component, Copy, Clone)]
#[component(on_replace = handle_replace::<Button>)]
pub struct Handle {
    pub panel: Entity,
    pub icon: Entity,
    pub text: Entity,
}
