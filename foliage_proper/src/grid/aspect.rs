use crate::grid::token::{
    AspectDescriptor, AspectToken, AspectTokenUnit, AspectValue, AspectValueWrapper, TokenOp,
};

#[derive(Default, Clone)]
pub(crate) struct ConfigurationDescriptor {
    pub(crate) aspects: [AspectDescriptor; 2],
    pub(crate) count: u32,
}

impl ConfigurationDescriptor {
    pub(crate) fn set<LAD: Into<AspectValueWrapper>>(&mut self, aspect: GridAspect, desc: LAD) {
        if self.count == 0 {
            self.aspects[0] = AspectDescriptor::new(aspect, desc.into());
            self.count += 1;
        } else if self.count == 1 {
            let mut slot = 1;
            if aspect < self.aspects[0].aspect {
                self.aspects[1] = self.aspects[0].clone();
                slot = 0;
            }
            self.aspects[slot] = AspectDescriptor::new(aspect, desc.into());
            self.count += 1;
        } else {
            panic!("too many dimensions");
        }
    }
    pub fn new() -> ConfigurationDescriptor {
        ConfigurationDescriptor {
            aspects: Default::default(),
            count: 0,
        }
    }
    pub(crate) fn top<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::Top, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_top(mut self) -> Self {
        self.set(GridAspect::Top, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn bottom<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::Bottom, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_bottom(mut self) -> Self {
        self.set(GridAspect::Bottom, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn left<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::Left, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_left(mut self) -> Self {
        self.set(GridAspect::Left, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn right<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::Right, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_right(mut self) -> Self {
        self.set(GridAspect::Right, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn width<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::Width, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_width(mut self) -> Self {
        self.set(GridAspect::Width, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn height<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::Height, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_height(mut self) -> Self {
        self.set(GridAspect::Height, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn center_x<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::CenterX, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_center_x(mut self) -> Self {
        self.set(GridAspect::CenterX, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn center_y<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::CenterY, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_center_y(mut self) -> Self {
        self.set(GridAspect::CenterY, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_ax<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointAX, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_ax(mut self) -> Self {
        self.set(GridAspect::PointAX, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_ay<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointAY, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_ay(mut self) -> Self {
        self.set(GridAspect::PointAY, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_bx<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointBX, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_bx(mut self) -> Self {
        self.set(GridAspect::PointBX, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_by<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointBY, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_by(mut self) -> Self {
        self.set(GridAspect::PointBY, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_cx<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointCX, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_cx(mut self) -> Self {
        self.set(GridAspect::PointCX, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_cy<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointCY, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_cy(mut self) -> Self {
        self.set(GridAspect::PointCY, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_dx<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointDX, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_dx(mut self) -> Self {
        self.set(GridAspect::PointDX, AspectValueWrapper::Existing);
        self
    }
    pub(crate) fn point_dy<LAD: Into<AspectValue>>(mut self, t: LAD) -> Self {
        self.set(GridAspect::PointDY, AspectValueWrapper::Specified(t.into()));
        self
    }
    pub(crate) fn existing_point_dy(mut self) -> Self {
        self.set(GridAspect::PointDY, AspectValueWrapper::Existing);
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
    fn context_token(self, aspect: GridAspect) -> AspectToken {
        AspectToken::new(TokenOp::Add, self, AspectTokenUnit::ContextAspect(aspect))
    }
    pub fn top(self) -> AspectToken {
        self.context_token(GridAspect::Top)
    }
    pub fn bottom(self) -> AspectToken {
        self.context_token(GridAspect::Bottom)
    }
    pub fn left(self) -> AspectToken {
        self.context_token(GridAspect::Left)
    }
    pub fn right(self) -> AspectToken {
        self.context_token(GridAspect::Right)
    }
    pub fn width(self) -> AspectToken {
        self.context_token(GridAspect::Width)
    }
    pub fn height(self) -> AspectToken {
        self.context_token(GridAspect::Height)
    }
    pub fn center_x(self) -> AspectToken {
        self.context_token(GridAspect::CenterX)
    }
    pub fn center_y(self) -> AspectToken {
        self.context_token(GridAspect::CenterY)
    }
    pub fn point_ax(self) -> AspectToken {
        self.context_token(GridAspect::PointAX)
    }
    pub fn point_ay(self) -> AspectToken {
        self.context_token(GridAspect::PointAY)
    }
    pub fn point_bx(self) -> AspectToken {
        self.context_token(GridAspect::PointBX)
    }
    pub fn point_by(self) -> AspectToken {
        self.context_token(GridAspect::PointBY)
    }
    pub fn point_cx(self) -> AspectToken {
        self.context_token(GridAspect::PointCX)
    }
    pub fn point_cy(self) -> AspectToken {
        self.context_token(GridAspect::PointCY)
    }
    pub fn point_dx(self) -> AspectToken {
        self.context_token(GridAspect::PointDX)
    }
    pub fn point_dy(self) -> AspectToken {
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
pub(crate) enum Configuration {
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
impl Configuration {
    pub(crate) fn value(self) -> usize {
        match self {
            Configuration::Horizontal => 0,
            Configuration::Vertical => 1,
        }
    }
}
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default, PartialOrd)]
pub(crate) enum GridAspect {
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
