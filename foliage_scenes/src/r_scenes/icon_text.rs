use foliage_macros::InnerSceneBinding;
use foliage_proper::bevy_ecs::entity::Entity;
use foliage_proper::bevy_ecs::prelude::Commands;
use foliage_proper::bevy_ecs::system::SystemParamItem;
use foliage_proper::color::Color;
use foliage_proper::compositor::segment::{Grid, Segment, SegmentUnitDesc};
use foliage_proper::coordinate::{Coordinate, InterfaceContext};
use foliage_proper::icon::{Icon, IconId};
use foliage_proper::scene::{Alignment, Binder, Bindings, Scene, SceneComponents};
use foliage_proper::text::{MaxCharacters, Text, TextValue};

pub struct IconText {
    pub icon_id: IconId,
    pub icon_color: Color,
    pub max_chars: MaxCharacters,
    pub text_value: TextValue,
    pub text_color: Color,
}
impl IconText {
    pub fn new<ID: Into<IconId>, C: Into<Color>, TV: Into<TextValue>, MC: Into<MaxCharacters>>(
        id: ID,
        ic: C,
        mc: MC,
        tv: TV,
        tc: C,
    ) -> Self {
        Self {
            icon_id: id.into(),
            icon_color: ic.into(),
            max_chars: mc.into(),
            text_value: tv.into(),
            text_color: tc.into(),
        }
    }
}
#[derive(InnerSceneBinding)]
pub enum IconTextBindings {
    Icon,
    Text,
}
impl Scene for IconText {
    type Params = ();
    type Filter = ();
    type Components = ();

    fn config(
        entity: Entity,
        coordinate: Coordinate<InterfaceContext>,
        ext: &mut SystemParamItem<Self::Params>,
        bindings: &Bindings,
    ) {
        todo!()
    }

    fn create(self, cmd: &mut Commands) -> SceneComponents<Self::Components> {
        let mut binder = Binder::new(cmd);
        binder.bind(
            IconTextBindings::Icon,
            Alignment::new(
                Segment::new(
                    0.relative().to(0.25.relative()),
                    0.relative().to(1.relative()),
                )
                .with_aspect(1.0),
                0,
            ),
            Icon::new(self.icon_id, self.icon_color),
            cmd,
        );
        binder.bind(
            IconTextBindings::Text,
            Alignment::new(
                Segment::new(
                    0.3.relative().to(1.relative()),
                    0.relative().to(1.relative()),
                ),
                0,
            ),
            Text::new(self.max_chars, self.text_value, self.text_color),
            cmd,
        );
        SceneComponents::new(Grid::new(1, 1), binder.bindings(), ())
    }
}