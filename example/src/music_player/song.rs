use foliage::bevy_ecs::prelude::{Bundle, Commands, IntoSystemConfigs};
use foliage::bevy_ecs::query::{With, Without};
use foliage::bevy_ecs::system::{Query, Res, ResMut, SystemParamItem};
use foliage::color::Color;
use foliage::coordinate::area::Area;
use foliage::coordinate::InterfaceContext;
use foliage::elm::config::{ElmConfiguration, ExternalSet};
use foliage::elm::leaf::{Leaf, Tag};
use foliage::elm::Elm;
use foliage::rectangle::Rectangle;
use foliage::scene::align::SceneAligner;
use foliage::scene::{Anchor, Scene, SceneBinder, SceneBinding, SceneCoordinator, SceneHandle};
use foliage::text::font::MonospacedFont;
use foliage::text::{FontSize, MaxCharacters, Text, TextValue};
use foliage::texture::factors::Progress;
use foliage::window::ScaleFactor;
use foliage::{bevy_ecs, set_descriptor};
#[derive(Bundle)]
pub struct SongInfo {
    tag: Tag<Self>,
}
pub enum SongBindings {
    Artist,
    Divider,
    Song,
}
impl From<SongBindings> for SceneBinding {
    fn from(value: SongBindings) -> Self {
        SceneBinding(value as i32)
    }
}
pub struct SongInfoArgs {
    artist: TextValue,
    song: TextValue,
    color: Color,
}
set_descriptor!(
    pub enum SongInfoSets {
        Area,
    }
);
impl Leaf for SongInfo {
    type SetDescriptor = SongInfoSets;

    fn config(elm_configuration: &mut ElmConfiguration) {
        elm_configuration.configure_hook::<Self>(ExternalSet::Configure, Self::SetDescriptor::Area);
    }

    fn attach(elm: &mut Elm) {
        elm.main().add_systems((resize
            .in_set(Self::SetDescriptor::Area)
            .before(<Text as Leaf>::SetDescriptor::Area),));
    }
}
fn resize(
    scenes: Query<(&SceneHandle, &Area<InterfaceContext>), With<Tag<SongInfo>>>,
    mut coordinator: ResMut<SceneCoordinator>,
    font: Res<MonospacedFont>,
    scale_factor: Res<ScaleFactor>,
    mut texts: Query<&mut FontSize, Without<Tag<SongInfo>>>,
    mut rectangles: Query<&mut Area<InterfaceContext>, Without<Tag<SongInfo>>>,
) {
    for (handle, area) in scenes.iter() {
        let (size, area) = font.best_fit(
            MaxCharacters(25),
            *area / (2, 1).into() - (8, 0).into(),
            &scale_factor,
        );
        let artist =
            coordinator.binding_entity(&handle.access_chain().target(SongBindings::Artist));
        let divider =
            coordinator.binding_entity(&handle.access_chain().target(SongBindings::Divider));
        let song = coordinator.binding_entity(&handle.access_chain().target(SongBindings::Song));
        *texts.get_mut(artist).unwrap() = size;
        *texts.get_mut(song).unwrap() = size;
        rectangles.get_mut(divider).unwrap().height = area.height;
    }
}
impl Scene for SongInfo {
    type Bindings = SongBindings;
    type Args<'a> = SongInfoArgs;
    type ExternalArgs = (Res<'static, MonospacedFont>, Res<'static, ScaleFactor>);

    fn bind_nodes(
        cmd: &mut Commands,
        anchor: Anchor,
        args: &Self::Args<'_>,
        external_args: &SystemParamItem<Self::ExternalArgs>,
        mut binder: SceneBinder<'_>,
    ) -> Self {
        let (size, area) = external_args.0.best_fit(
            MaxCharacters(25),
            anchor.0.section.area / (2, 1).into() - (8, 0).into(),
            &external_args.1,
        );
        binder.bind(
            SongBindings::Artist,
            (0.near(), 0.center(), 0),
            Text::new(MaxCharacters(25), size, TextValue::new(""), args.color),
            cmd,
        );
        binder.bind(
            SongBindings::Divider,
            (0.center(), 0.near(), 0),
            Rectangle::new(
                (4f32, anchor.0.section.area.height).into(),
                args.color,
                Progress::full(),
            ),
            cmd,
        );
        binder.bind(
            SongBindings::Song,
            (8.center(), 0.near(), 0),
            Text::new(MaxCharacters(25), size, TextValue::new(""), args.color),
            cmd,
        );
        Self { tag: Tag::new() }
    }
}
