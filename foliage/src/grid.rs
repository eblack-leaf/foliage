use crate::coordinate::area::Area;
use crate::coordinate::section::Section;
use crate::coordinate::{CoordinateUnit, NumericalContext};
#[derive(Copy, Clone)]
pub struct Grid {
    section: Section<NumericalContext>,
    cols: u32,
    rows: u32,
    gap: (CoordinateUnit, CoordinateUnit),
    col_width: CoordinateUnit,
    row_height: CoordinateUnit,
}
impl Default for Grid {
    fn default() -> Self {
        Self::new(Section::default(), 1, 1)
    }
}
impl Grid {
    pub const DEFAULT_GAP: CoordinateUnit = 4.0;
    pub fn new(section: Section<NumericalContext>, cols: u32, rows: u32) -> Self {
        let gap = (Self::DEFAULT_GAP, Self::DEFAULT_GAP);
        let grid_dims = Self::metrics(section, cols, rows, gap);
        Self {
            section,
            cols,
            rows,
            gap,
            col_width: grid_dims.width,
            row_height: grid_dims.height,
        }
    }

    fn metrics(
        section: Section<NumericalContext>,
        cols: u32,
        rows: u32,
        gap: (CoordinateUnit, CoordinateUnit),
    ) -> Area<NumericalContext> {
        let area_minus_gap = section.area - Area::from(gap) * (cols, rows).into();
        let grid_dims = area_minus_gap / (cols, rows).into();
        grid_dims
    }
    pub fn with_gap_x(mut self, gap: CoordinateUnit) -> Self {
        self.gap.0 = gap;
        self.set_metrics();
        self
    }
    pub fn col_width(&self) -> CoordinateUnit {
        self.col_width
    }
    pub fn row_height(&self) -> CoordinateUnit {
        self.row_height
    }
    fn set_metrics(&mut self) {
        let metrics = Self::metrics(self.section, self.cols, self.rows, self.gap);
        self.col_width = metrics.width;
        self.row_height = metrics.height;
    }
    pub fn with_gap_y(mut self, gap: CoordinateUnit) -> Self {
        self.gap.1 = gap;
        self.set_metrics();
        self
    }
    pub fn place(&self, col: u32, row: u32) -> Option<Section<NumericalContext>> {
        if col > self.cols || row > self.rows {
            return None;
        }
        let x = self.col_width * col as CoordinateUnit + self.gap.0 * (col + 1) as CoordinateUnit;
        let y = self.row_height * row as CoordinateUnit + self.gap.1 * (row + 1) as CoordinateUnit;
        Option::from(Section::new((x, y), (self.col_width, self.row_height)))
    }
}