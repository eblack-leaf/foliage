use foliage_proper::elm::leaf::Leaves;
use foliage_proper::Foliage;

// pub mod aspect_ratio_image;
// pub mod button;
// pub mod circle_button;
// pub mod circle_progress_bar;
// pub mod icon_button;
// pub mod icon_text;
// pub mod interactive_progress_bar;
// pub mod progress_bar;
pub mod r_scenes;
// pub mod text_input;

pub struct SceneExtensions;
impl Leaves for SceneExtensions {
    fn leaves(f: Foliage) -> Foliage {
        f.with_leaf::<r_scenes::icon_text::IconText>()
            .with_leaf::<r_scenes::button::Button>()
    }
}