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
            .with_leaf::<r_scenes::circle_button::CircleButton>()
            .with_leaf::<r_scenes::icon_button::IconButton>()
            .with_leaf::<r_scenes::text_button::TextButton>()
            .with_leaf::<r_scenes::progress_bar::ProgressBar>()
            .with_leaf::<r_scenes::circle_progress_bar::CircleProgressBar>()
            .with_leaf::<r_scenes::dropdown::Dropdown>()
            .with_leaf::<r_scenes::ellipsis::Ellipsis>()
            .with_leaf::<r_scenes::paged::scene::PageStructure>()
    }
}
