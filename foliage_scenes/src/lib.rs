use foliage_proper::elm::leaf::Leaves;
use foliage_proper::Foliage;

pub mod aspect_ratio_image;
pub mod button;
pub mod circle_button;
pub mod circle_progress_bar;
pub mod icon_button;
pub mod icon_text;
pub mod interactive_progress_bar;
pub mod progress_bar;
pub mod text_input;

pub struct SceneExtensions;
impl Leaves for SceneExtensions {
    fn leaves(f: Foliage) -> Foliage {
        f.with_leaf::<aspect_ratio_image::AspectRatioImage>()
            .with_leaf::<button::Button>()
            .with_leaf::<circle_button::CircleButton>()
            .with_leaf::<circle_progress_bar::CircleProgressBar>()
            .with_leaf::<icon_button::IconButton>()
            .with_leaf::<icon_text::IconText>()
            .with_leaf::<interactive_progress_bar::InteractiveProgressBar>()
            .with_leaf::<progress_bar::ProgressBar>()
            .with_leaf::<text_input::TextInput>()
    }
}