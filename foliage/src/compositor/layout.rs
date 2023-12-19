use crate::coordinate::area::Area;
use crate::coordinate::InterfaceContext;

pub enum Orientation {
    Portrait,
    Landscape,
}
impl Orientation {
    pub fn from_area(area: Area<InterfaceContext>) -> Self {
        todo!()
    }
}
pub enum Threshold {
    Mobile,
    Tablet,
    Desktop,
    Workstation,
}
pub struct Layout {
    orientation: Orientation,
    threshold: Threshold,
}
impl Layout {
    pub fn new(orientation: Orientation, threshold: Threshold) -> Self {
        Self {
            orientation,
            threshold,
        }
    }
    pub fn threshold(&self) -> LayoutThreshold {
        match self.orientation {
            Orientation::Portrait => match self.threshold {
                Threshold::Mobile => {
                    LayoutThreshold::new(ThresholdBound::new(0, 400), ThresholdBound::new(0, 900))
                }
                Threshold::Tablet => LayoutThreshold::new(
                    ThresholdBound::new(401, 800),
                    ThresholdBound::new(0, 1000),
                ),
                Threshold::Desktop => LayoutThreshold::new(
                    ThresholdBound::new(801, 1200),
                    ThresholdBound::new(0, 1100),
                ),
                Threshold::Workstation => LayoutThreshold::new(
                    ThresholdBound::new(1201, 3840),
                    ThresholdBound::new(0, 2160),
                ),
            },
            Orientation::Landscape => match self.threshold {
                Threshold::Mobile => {
                    LayoutThreshold::new(ThresholdBound::new(0, 900), ThresholdBound::new(0, 400))
                }
                Threshold::Tablet => LayoutThreshold::new(
                    ThresholdBound::new(0, 1000),
                    ThresholdBound::new(401, 800),
                ),
                Threshold::Desktop => LayoutThreshold::new(
                    ThresholdBound::new(0, 1100),
                    ThresholdBound::new(801, 1200),
                ),
                Threshold::Workstation => LayoutThreshold::new(
                    ThresholdBound::new(0, 2160),
                    ThresholdBound::new(1201, 3840),
                ),
            },
        }
    }
}
pub struct ThresholdBound {
    min: u32,
    max: u32,
}
impl ThresholdBound {
    pub fn satisfied(&self, target: u32) -> bool {
        todo!()
    }
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }
}
pub struct LayoutThreshold {
    horizontal_bound: ThresholdBound,
    vertical_bound: ThresholdBound,
}
impl LayoutThreshold {
    pub fn new(hb: ThresholdBound, vb: ThresholdBound) -> Self {
        Self {
            horizontal_bound: hb,
            vertical_bound: vb,
        }
    }
}
pub struct Layouts(pub Vec<Layout>);
impl Layouts {
    pub fn full() -> Self {
        Self(vec![
            Layout::new(Orientation::Portrait, Threshold::Mobile),
            Layout::new(Orientation::Landscape, Threshold::Mobile),
            Layout::new(Orientation::Portrait, Threshold::Tablet),
            Layout::new(Orientation::Landscape, Threshold::Tablet),
            Layout::new(Orientation::Portrait, Threshold::Desktop),
            Layout::new(Orientation::Landscape, Threshold::Desktop),
            Layout::new(Orientation::Portrait, Threshold::Workstation),
            Layout::new(Orientation::Landscape, Threshold::Workstation),
        ])
    }
}