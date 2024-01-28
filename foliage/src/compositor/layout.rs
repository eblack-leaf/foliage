use crate::coordinate::area::Area;
use crate::coordinate::{CoordinateUnit, InterfaceContext, NumericalContext};
#[derive(Copy, Clone, Default, Debug)]
pub struct AspectRatio(pub f32);

impl AspectRatio {
    pub fn new(area: Area<NumericalContext>) -> Self {
        Self(area.width / area.height)
    }
    pub fn is_width_major(&self) -> bool {
        self.0 >= 1f32
    }
    pub fn reciprocal(&self) -> f32 {
        1f32 / self.0
    }
}
impl From<f32> for AspectRatio {
    fn from(value: f32) -> Self {
        AspectRatio(value)
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Orientation {
    Portrait,
    Landscape,
}
impl Orientation {
    pub fn from_area(area: Area<InterfaceContext>) -> Self {
        if area.width > area.height {
            Self::Landscape
        } else {
            Self::Portrait
        }
    }
}
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Threshold {
    Mobile,
    Tablet,
    Desktop,
    Workstation,
}
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct Layout {
    pub orientation: Orientation,
    pub threshold: Threshold,
}
impl Layout {
    pub const PORTRAIT_MOBILE: Self = Layout::new(Orientation::Portrait, Threshold::Mobile);
    pub const PORTRAIT_TABLET: Self = Layout::new(Orientation::Portrait, Threshold::Tablet);
    pub const PORTRAIT_DESKTOP: Self = Layout::new(Orientation::Portrait, Threshold::Desktop);
    pub const PORTRAIT_WORKSTATION: Self =
        Layout::new(Orientation::Portrait, Threshold::Workstation);
    pub const LANDSCAPE_MOBILE: Self = Layout::new(Orientation::Landscape, Threshold::Mobile);
    pub const LANDSCAPE_TABLET: Self = Layout::new(Orientation::Landscape, Threshold::Tablet);
    pub const LANDSCAPE_DESKTOP: Self = Layout::new(Orientation::Landscape, Threshold::Desktop);
    pub const LANDSCAPE_WORKSTATION: Self =
        Layout::new(Orientation::Landscape, Threshold::Workstation);
    pub fn from_area<A: Into<Area<InterfaceContext>>>(area: A) -> Self {
        let area = area.into();
        let orientation = Orientation::from_area(area);
        match orientation {
            Orientation::Portrait => Self::threshold_check(area, Self::PORTRAIT),
            Orientation::Landscape => Self::threshold_check(area, Self::LANDSCAPE),
        }
    }
    fn threshold_check(area: Area<InterfaceContext>, layouts: [Layout; 4]) -> Layout {
        let mut layout = Layout::new(Orientation::from_area(area), Threshold::Mobile);
        for l in layouts {
            let threshold = l.threshold();
            if threshold.horizontal_bound.satisfied(area.width)
                && threshold.vertical_bound.satisfied(area.height)
            {
                layout = l;
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
                Threshold::Mobile => LayoutThreshold::new(
                    ThresholdBound::new(0.0, 415.0),
                    ThresholdBound::new(0.0, 915.0),
                ),
                Threshold::Tablet => LayoutThreshold::new(
                    ThresholdBound::new(415.0, 815.0),
                    ThresholdBound::new(0.0, 1440.0),
                ),
                Threshold::Desktop => LayoutThreshold::new(
                    ThresholdBound::new(815.0, 1215.0),
                    ThresholdBound::new(0.0, 1920.0),
                ),
                Threshold::Workstation => LayoutThreshold::new(
                    ThresholdBound::new(1215.0, 3840.0),
                    ThresholdBound::new(0.0, 2160.0),
                ),
            },
            Orientation::Landscape => match self.threshold {
                Threshold::Mobile => LayoutThreshold::new(
                    ThresholdBound::new(0.0, 915.0),
                    ThresholdBound::new(0.0, 415.0),
                ),
                Threshold::Tablet => LayoutThreshold::new(
                    ThresholdBound::new(0.0, 1920.0),
                    ThresholdBound::new(415.0, 615.0),
                ),
                Threshold::Desktop => LayoutThreshold::new(
                    ThresholdBound::new(0.0, 1920.0),
                    ThresholdBound::new(615.0, 815.0),
                ),
                Threshold::Workstation => LayoutThreshold::new(
                    ThresholdBound::new(0.0, 3840.0),
                    ThresholdBound::new(815.0, 2160.0),
                ),
            },
        }
    }
}
pub struct ThresholdBound {
    pub min: CoordinateUnit,
    pub max: CoordinateUnit,
}
impl ThresholdBound {
    pub fn satisfied(&self, target: CoordinateUnit) -> bool {
        target < self.max && target >= self.min
    }
    pub fn new(min: CoordinateUnit, max: CoordinateUnit) -> Self {
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

#[cfg(test)]
#[test]
fn test_from_area() {
    let mut actual = Layout::from_area((360, 800));
    let mut expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((360, 640));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((375, 812));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((390, 844));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((393, 873));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((360, 780));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((414, 896));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((412, 915));
    expected = Layout::new(Orientation::Portrait, Threshold::Mobile);
    assert_eq!(actual, expected);
    actual = Layout::from_area((428, 926));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((601, 962));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((768, 1024));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((800, 1280));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((810, 1080));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((820, 1180));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((834, 1194));
    expected = Layout::new(Orientation::Portrait, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((962, 601));
    expected = Layout::new(Orientation::Landscape, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1024, 768));
    expected = Layout::new(Orientation::Landscape, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1024, 1366));
    expected = Layout::new(Orientation::Portrait, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1280, 720));
    expected = Layout::new(Orientation::Landscape, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1280, 800));
    expected = Layout::new(Orientation::Landscape, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1280, 1024));
    expected = Layout::new(Orientation::Landscape, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1366, 768));
    expected = Layout::new(Orientation::Landscape, Threshold::Tablet);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1366, 1024));
    expected = Layout::new(Orientation::Landscape, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1440, 900));
    expected = Layout::new(Orientation::Landscape, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1536, 864));
    expected = Layout::new(Orientation::Landscape, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1600, 900));
    expected = Layout::new(Orientation::Landscape, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((1920, 1080));
    expected = Layout::new(Orientation::Landscape, Threshold::Desktop);
    assert_eq!(actual, expected);
    actual = Layout::from_area((2560, 1440));
    expected = Layout::new(Orientation::Landscape, Threshold::Workstation);
    assert_eq!(actual, expected);
}