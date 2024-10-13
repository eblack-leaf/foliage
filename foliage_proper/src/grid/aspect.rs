use crate::grid::token::{
    LocationAspectDescriptor, LocationAspectDescriptorValue, LocationAspectToken,
    LocationAspectTokenOp, LocationAspectTokenValue, SpecifiedDescriptorValue,
};
use smallvec::SmallVec;

#[derive(Default, Clone)]
pub(crate) struct LocationAspect {
    pub(crate) aspects: SmallVec<[LocationAspectDescriptor; 2]>,
    pub(crate) count: u32,
}

impl LocationAspect {
    pub(crate) fn set<LAD: Into<LocationAspectDescriptorValue>>(
        &mut self,
        aspect: GridAspect,
        desc: LAD,
    ) {
        if self.count == 0 {
            self.aspects[0] = LocationAspectDescriptor::new(aspect, desc.into());
            self.count += 1;
        } else if self.count == 1 {
            let mut slot = 1;
            if aspect < self.aspects[0].aspect {
                self.aspects[1] = self.aspects[0].clone();
                slot = 0;
            }
            self.aspects[slot] = LocationAspectDescriptor::new(aspect, desc.into());
            self.count += 1;
        } else if self.count == 2 {
            todo!()
        } else if self.count == 3 {
            todo!()
        } else {
            panic!("too many dimensions");
        }
    }
    pub fn new() -> LocationAspect {
        let mut aspects = SmallVec::new();
        aspects.push(LocationAspect::new());
        aspects.push(LocationAspect::new());
        LocationAspect { aspects, count: 0 }
    }
    pub(crate) fn top<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Top,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_top(mut self) -> Self {
        self.set(GridAspect::Top, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn bottom<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Bottom,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_bottom(mut self) -> Self {
        self.set(GridAspect::Bottom, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn left<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Left,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_left(mut self) -> Self {
        self.set(GridAspect::Left, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn right<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Right,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_right(mut self) -> Self {
        self.set(GridAspect::Right, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn width<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Width,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_width(mut self) -> Self {
        self.set(GridAspect::Width, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn height<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::Height,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_height(mut self) -> Self {
        self.set(GridAspect::Height, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn center_x<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::CenterX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_center_x(mut self) -> Self {
        self.set(GridAspect::CenterX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn center_y<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::CenterY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_center_y(mut self) -> Self {
        self.set(GridAspect::CenterY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_ax<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointAX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_ax(mut self) -> Self {
        self.set(GridAspect::PointAX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_ay<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointAY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_ay(mut self) -> Self {
        self.set(GridAspect::PointAY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_bx<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointBX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_bx(mut self) -> Self {
        self.set(GridAspect::PointBX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_by<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointBY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_by(mut self) -> Self {
        self.set(GridAspect::PointBY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_cx<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointCX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_cx(mut self) -> Self {
        self.set(GridAspect::PointCX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_cy<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointCY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_cy(mut self) -> Self {
        self.set(GridAspect::PointCY, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_dx<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointDX,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_dx(mut self) -> Self {
        self.set(GridAspect::PointDX, LocationAspectDescriptorValue::Existing);
        self
    }
    pub(crate) fn point_dy<LAD: Into<SpecifiedDescriptorValue>>(mut self, t: LAD) -> Self {
        self.set(
            GridAspect::PointDY,
            LocationAspectDescriptorValue::Specified(t.into()),
        );
        self
    }
    pub(crate) fn existing_point_dy(mut self) -> Self {
        self.set(GridAspect::PointDY, LocationAspectDescriptorValue::Existing);
        self
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, PartialOrd)]
pub enum GridContext {
    Screen,
    Stem,
    Absolute,
}

impl GridContext {
    fn context_token(self, aspect: GridAspect) -> LocationAspectToken {
        LocationAspectToken::new(
            LocationAspectTokenOp::Add,
            self,
            LocationAspectTokenValue::ContextAspect(aspect),
        )
    }
    pub fn top(self) -> LocationAspectToken {
        self.context_token(GridAspect::Top)
    }
    pub fn bottom(self) -> LocationAspectToken {
        self.context_token(GridAspect::Bottom)
    }
    pub fn left(self) -> LocationAspectToken {
        self.context_token(GridAspect::Left)
    }
    pub fn right(self) -> LocationAspectToken {
        self.context_token(GridAspect::Right)
    }
    pub fn width(self) -> LocationAspectToken {
        self.context_token(GridAspect::Width)
    }
    pub fn height(self) -> LocationAspectToken {
        self.context_token(GridAspect::Height)
    }
    pub fn center_x(self) -> LocationAspectToken {
        self.context_token(GridAspect::CenterX)
    }
    pub fn center_y(self) -> LocationAspectToken {
        self.context_token(GridAspect::CenterY)
    }
    pub fn point_ax(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointAX)
    }
    pub fn point_ay(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointAY)
    }
    pub fn point_bx(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointBX)
    }
    pub fn point_by(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointBY)
    }
    pub fn point_cx(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointCX)
    }
    pub fn point_cy(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointCY)
    }
    pub fn point_dx(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointDX)
    }
    pub fn point_dy(self) -> LocationAspectToken {
        self.context_token(GridAspect::PointDY)
    }
}

pub fn screen() -> GridContext {
    GridContext::Screen
}

pub fn stem() -> GridContext {
    GridContext::Stem
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Default, Debug)]
pub(crate) enum SectionAspectConfiguration {
    #[default]
    Horizontal,
    Vertical,
}
#[derive(Hash, PartialEq, Eq, Clone, Copy, Default, Debug)]
pub(crate) enum PointAspectConfiguration {
    #[default]
    PointA,
    PointB,
    PointC,
    PointD,
}
impl PointAspectConfiguration {
    pub(crate) fn value(self) -> usize {
        match self {
            PointAspectConfiguration::PointA => 0,
            PointAspectConfiguration::PointB => 1,
            PointAspectConfiguration::PointC => 2,
            PointAspectConfiguration::PointD => 3,
        }
    }
}
impl SectionAspectConfiguration {
    pub(crate) fn value(self) -> usize {
        match self {
            SectionAspectConfiguration::Horizontal => 0,
            SectionAspectConfiguration::Vertical => 1,
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, PartialOrd)]
pub enum GridAspect {
    #[default]
    Top,
    Height,
    CenterY, // Dependent => Top | Height | Bottom
    Bottom,  // Dependent => Height | Top | CenterY
    Left,
    Width,
    CenterX, // Dependent => Right | Width | Left
    Right,   // Dependent => Width | Left | CenterX
    PointAX,
    PointAY,
    PointBX,
    PointBY,
    PointCX,
    PointCY,
    PointDX,
    PointDY,
}
