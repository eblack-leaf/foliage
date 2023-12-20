use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, InterfaceContext};
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum Orientation {
    Portrait,
    Landscape,
}
impl Orientation {
    pub fn from_area(area: Area<InterfaceContext>) -> Self {
        todo!()
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum Threshold {
    Mobile,
    Tablet,
    Desktop,
    Workstation,
}
#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct Layout {
    orientation: Orientation,
    threshold: Threshold,
}
impl Layout {
    pub fn from_area(area: Area<InterfaceContext>) -> Self {
        let orientation = Orientation::from_area(area);
        match orientation {
            Orientation::Portrait => {
                Self::threshold_check(area, Self::PORTRAIT)
            }
            Orientation::Landscape => {
                Self::threshold_check(area, Self::LANDSCAPE)
            }
        }
    }
    fn threshold_check(area: Area<InterfaceContext>, layouts: [Layout; 4]) -> Layout {
        let mut layout = Layout::new(Orientation::from_area(area), Threshold::Mobile);
        for l in layouts {
            let threshold = l.threshold();
            if threshold.horizontal_bound.satisfied(area.width)
                && threshold.vertical_bound.satisfied(area.height)
            {
                layout = *l;
                break;
            }
        }
        layout
    }
    pub const PORTRAIT: [Layout; 4] = [
        Layout::new(Orientation::Portrait, Threshold::Mobile),
        Layout::new(Orientation::Portrait, Threshold::Tablet),
        Layout::new(Orientation::Portrait, Threshold::Desktop),
        Layout::new(Orientation::Portrait, Threshold::Workstation),
    ];
    pub const LANDSCAPE: [Layout; 4] = [
        Layout::new(Orientation::Landscape, Threshold::Mobile),
        Layout::new(Orientation::Landscape, Threshold::Tablet),
        Layout::new(Orientation::Landscape, Threshold::Desktop),
        Layout::new(Orientation::Landscape, Threshold::Workstation),
    ];
    pub const fn new(orientation: Orientation, threshold: Threshold) -> Self {
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
    pub min: u32,
    pub max: u32,
}
impl ThresholdBound {
    pub fn satisfied(&self, target: CoordinateUnit) -> bool {
        todo!()
    }
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }
}
pub struct LayoutThreshold {
    pub horizontal_bound: ThresholdBound,
    pub vertical_bound: ThresholdBound,
}
impl LayoutThreshold {
    pub fn new(hb: ThresholdBound, vb: ThresholdBound) -> Self {
        Self {
            horizontal_bound: hb,
            vertical_bound: vb,
        }
    }
}
