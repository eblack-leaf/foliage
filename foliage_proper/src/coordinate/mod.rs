use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub mod area;
pub mod elevation;
pub mod points;
pub mod position;
pub mod section;

/// Marker for which coordinate space a [`Position`](position::Position),
/// [`Area`](area::Area), [`Section`](section::Section) or [`Points`](points::Points) is
/// expressed in.
///
/// Carried as a type parameter so the two real spaces cannot be mixed by accident -- a
/// `Section<Logical>` will not go where a `Section<Physical>` is wanted. Convert
/// deliberately with `to_physical`/`to_logical` and the current
/// [`ScaleFactor`](crate::ScaleFactor).
pub trait CoordinateContext
where
    Self: Send + Sync + 'static + Copy + Clone + Default,
{
}

/// Device pixels -- what the GPU and the windowing system deal in. Logical units times
/// the display's scale factor.
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Physical;

/// Scale-factor-independent pixels, and the space all layout is authored in. A
/// `Location` written in `px` means these, so a UI keeps its size across displays.
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Logical;

/// Neither space: a bare pair of numbers reusing the same vector arithmetic, for
/// quantities that are not positions on screen.
#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct Numerical;

impl CoordinateContext for Physical {}

impl CoordinateContext for Logical {}

impl CoordinateContext for Numerical {}

/// The scalar every coordinate type is built from.
pub type CoordinateUnit = f32;

/// Two [`CoordinateUnit`]s in GPU-upload layout -- the shared backing for
/// [`Position`](position::Position) and [`Area`](area::Area), and the arithmetic they
/// both inherit. Its fields are named `a`/`b` rather than x/y or width/height because
/// which they mean depends on the wrapper.
#[repr(C)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Pod, Zeroable, Default)]
pub struct Coordinates(pub [CoordinateUnit; 2]);
impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{} {}]", self.a(), self.b()))
    }
}
// Delegated to `Display` so `{:?}` -- what every trace call here uses -- logs the compact
// `[12 -31092]` rather than the derived `Coordinates([12.0, -31092.0])`.
impl std::fmt::Debug for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
impl Coordinates {
    pub const fn new(a: CoordinateUnit, b: CoordinateUnit) -> Self {
        Self([a, b])
    }
    pub const fn a(&self) -> CoordinateUnit {
        self.0[0]
    }
    pub const fn b(&self) -> CoordinateUnit {
        self.0[1]
    }
    pub fn normalized<C: Into<Coordinates>>(&self, c: C) -> Self {
        let c = c.into();
        Self::new(self.a() / c.a(), self.b() / c.b())
    }
    pub fn set_horizontal(&mut self, h: f32) {
        self.0[0] = h;
    }
    pub fn set_vertical(&mut self, v: f32) {
        self.0[1] = v;
    }
    pub fn clamped(&self, min: CoordinateUnit, max: CoordinateUnit) -> Self {
        Self::new(self.a().clamp(min, max), self.b().clamp(min, max))
    }
    pub fn rounded(self) -> Self {
        Self([self.0[0].round(), self.0[1].round()])
    }
}

macro_rules! permutation_coordinate_impl {
    ($a:ty, $b:ty) => {
        impl From<($a, $b)> for Coordinates {
            fn from(value: ($a, $b)) -> Self {
                Self([value.0 as CoordinateUnit, value.1 as CoordinateUnit])
            }
        }
        impl From<($b, $a)> for Coordinates {
            fn from(value: ($b, $a)) -> Self {
                Self([value.0 as CoordinateUnit, value.1 as CoordinateUnit])
            }
        }
    };
}
macro_rules! single_coordinate_impl {
    ($a:ty) => {
        impl From<($a, $a)> for Coordinates {
            fn from(value: ($a, $a)) -> Self {
                Self([value.0 as CoordinateUnit, value.1 as CoordinateUnit])
            }
        }
    };
}
single_coordinate_impl!(f32);
single_coordinate_impl!(f64);
permutation_coordinate_impl!(f32, f64);
single_coordinate_impl!(i32);
permutation_coordinate_impl!(f32, i32);
permutation_coordinate_impl!(f64, i32);
single_coordinate_impl!(u32);
permutation_coordinate_impl!(f32, u32);
permutation_coordinate_impl!(i32, u32);
permutation_coordinate_impl!(f64, u32);
single_coordinate_impl!(usize);
permutation_coordinate_impl!(f32, usize);
permutation_coordinate_impl!(i32, usize);
permutation_coordinate_impl!(u32, usize);
permutation_coordinate_impl!(f64, usize);
impl Sub for Coordinates {
    type Output = Coordinates;

    fn sub(self, rhs: Self) -> Self::Output {
        Coordinates::new(self.a() - rhs.a(), self.b() - rhs.b())
    }
}

impl Div<f32> for Coordinates {
    type Output = Coordinates;

    fn div(self, rhs: f32) -> Self::Output {
        Coordinates::new(self.a() / rhs, self.b() / rhs)
    }
}
impl Div<Coordinates> for Coordinates {
    type Output = Coordinates;
    fn div(self, rhs: Coordinates) -> Self::Output {
        Coordinates::new(self.a() / rhs.a(), self.b() / rhs.b())
    }
}
impl Add for Coordinates {
    type Output = Coordinates;

    fn add(self, rhs: Self) -> Self::Output {
        Coordinates::new(self.a() + rhs.a(), self.b() + rhs.b())
    }
}

impl Mul for Coordinates {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        (self.a() * rhs.a(), self.b() * rhs.b()).into()
    }
}
impl AddAssign for Coordinates {
    fn add_assign(&mut self, rhs: Self) {
        self.0[0] += rhs.a();
        self.0[1] += rhs.b();
    }
}
impl SubAssign for Coordinates {
    fn sub_assign(&mut self, rhs: Self) {
        self.0[0] -= rhs.a();
        self.0[1] -= rhs.b();
    }
}
