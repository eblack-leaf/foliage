use crate::r_scenes::paged::scene::PageStructure;
use crate::r_scenes::{Colors, Direction};
use foliage_proper::icon::FeatherIcon;
use foliage_proper::procedure::Procedure;
use foliage_proper::scene::Scene;
use foliage_proper::segment::ResponsiveSegment;
use foliage_proper::view::ViewBuilder;

pub(crate) mod scene;

pub struct Paged<P> {
    pub elements: Vec<P>,
    pub colors: Colors,
    pub direction: Direction,
    pub responsive: ResponsiveSegment,
    pub increment_icon: FeatherIcon,
    pub decrement_icon: FeatherIcon,
}
impl<P: Scene> Paged<P> {
    pub fn new(
        elements: Vec<P>,
        colors: Colors,
        direction: Direction,
        responsive_segment: ResponsiveSegment,
        decrement_icon: FeatherIcon,
        increment_icon: FeatherIcon,
    ) -> Self {
        Self {
            elements,
            colors,
            direction,
            responsive: responsive_segment,
            increment_icon,
            decrement_icon,
        }
    }
}
impl<P: Scene + Clone> Procedure for Paged<P> {
    fn steps(self, view_builder: &mut ViewBuilder) {
        let handle = view_builder.add_scene(
            PageStructure::new(
                self.decrement_icon,
                self.increment_icon,
                self.colors,
                self.direction,
                self.elements.len() as u32,
            ),
            self.responsive,
        );
        for (i, element) in self.elements.iter().enumerate() {
            view_builder
                .place_conditional_scene_on(handle.bindings().get(i as i32 + 3), element.clone());
        }
    }
}
