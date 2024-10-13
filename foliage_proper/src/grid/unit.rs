use crate::coordinate::CoordinateUnit;
use crate::grid::aspect::GridContext;
use crate::grid::token::{AspectToken, AspectTokenUnit, TokenOp};

pub trait TokenUnit {
    fn px(self) -> AspectToken;
    fn percent(self) -> PercentDescriptor;
    fn column(self) -> ColumnDescriptor;
    fn row(self) -> RowDescriptor;
}
macro_rules! impl_token_unit_for {
    ($impl_type:ty) => {
        impl TokenUnit for $impl_type {
            fn px(self) -> AspectToken {
                AspectToken::new(
                    TokenOp::Add,
                    GridContext::Absolute,
                    AspectTokenUnit::Absolute(self as CoordinateUnit),
                )
            }
            fn percent(self) -> PercentDescriptor {
                PercentDescriptor {
                    value: self as CoordinateUnit / 100.0,
                    use_width: false,
                }
            }
            fn column(self) -> ColumnDescriptor {
                ColumnDescriptor {
                    value: self as i32,
                    is_end: false,
                }
            }
            fn row(self) -> RowDescriptor {
                RowDescriptor {
                    value: self as i32,
                    is_end: false,
                }
            }
        }
    };
}
impl_token_unit_for!(i32);
impl_token_unit_for!(f32);
pub struct RowDescriptor {
    value: i32,
    is_end: bool,
}

impl RowDescriptor {
    pub fn begin(mut self) -> Self {
        self.is_end = false;
        self
    }
    pub fn end(mut self) -> Self {
        self.is_end = true;
        self
    }
    pub fn of<GC: Into<GridContext>>(self, gc: GC) -> AspectToken {
        AspectToken::new(
            TokenOp::Add,
            gc.into(),
            AspectTokenUnit::Relative(RelativeUnit::Row(self.value, self.is_end)),
        )
    }
}

pub struct ColumnDescriptor {
    value: i32,
    is_end: bool,
}

impl ColumnDescriptor {
    pub fn begin(mut self) -> Self {
        self.is_end = false;
        self
    }
    pub fn end(mut self) -> Self {
        self.is_end = true;
        self
    }
    pub fn of<GC: Into<GridContext>>(self, gc: GC) -> AspectToken {
        AspectToken::new(
            TokenOp::Add,
            gc.into(),
            AspectTokenUnit::Relative(RelativeUnit::Column(self.value, self.is_end)),
        )
    }
}

pub struct PercentDescriptor {
    value: CoordinateUnit,
    use_width: bool,
}

impl PercentDescriptor {
    pub fn from<GC: Into<GridContext>>(mut self, gc: GC) -> AspectToken {
        AspectToken::new(
            TokenOp::Add,
            gc.into(),
            AspectTokenUnit::Relative(RelativeUnit::Percent(self.value, self.use_width, true)),
        )
    }
    pub fn width(mut self) -> Self {
        self.use_width = true;
        self
    }
    pub fn height(mut self) -> Self {
        self.use_width = false;
        self
    }
    pub fn of<GC: Into<GridContext>>(self, gc: GC) -> AspectToken {
        AspectToken::new(
            TokenOp::Add,
            gc.into(),
            AspectTokenUnit::Relative(RelativeUnit::Percent(self.value, self.use_width, false)),
        )
    }
}

#[derive(Clone, Copy)]
pub enum RelativeUnit {
    Column(i32, bool),
    Row(i32, bool),
    Percent(f32, bool, bool),
}
