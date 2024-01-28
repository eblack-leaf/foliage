use crate::coordinate::CoordinateUnit;

impl From<&str> for SegmentUnit {
    fn from(value: &str) -> Self {
        todo!()
    }
}

#[derive(Copy, Clone, Default)]
pub struct SegmentUnit {
    pub base: CoordinateUnit,
    pub fixed: bool,
    pub min: Option<CoordinateUnit>,
    pub max: Option<CoordinateUnit>,
    pub offset: CoordinateUnit,
}

impl SegmentUnit {
    pub fn new(base: CoordinateUnit) -> Self {
        Self {
            base,
            fixed: false,
            min: None,
            max: None,
            offset: 0.0,
        }
    }
    pub fn relative(mut self) -> Self {
        self.fixed = false;
        self
    }
    pub fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
    pub fn max(mut self, m: CoordinateUnit) -> Self {
        self.max.replace(m);
        self
    }
    pub fn min(mut self, m: CoordinateUnit) -> Self {
        self.min.replace(m);
        self
    }
    pub fn offset(mut self, o: CoordinateUnit) -> Self {
        self.offset = o;
        self
    }
}
