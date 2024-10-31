use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{IntoSystemConfigs, Or};
use bevy_ecs::query::Changed;
use bevy_ecs::system::{Query, Res};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::coordinate::section::{GpuSection, Section};
use crate::elm::{Elm, InternalStage};
use crate::ginkgo::ScaleFactor;
use crate::Root;

pub mod area;
pub mod elevation;
pub mod placement;
pub mod points;
pub mod position;
pub mod section;

pub trait CoordinateContext
where
    Self: Send + Sync + 'static + Copy + Clone + Default,
{
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct DeviceContext;

#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct LogicalContext;

#[derive(Copy, Clone, PartialOrd, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct NumericalContext;

impl CoordinateContext for DeviceContext {}

impl CoordinateContext for LogicalContext {}

impl CoordinateContext for NumericalContext {}

pub type CoordinateUnit = f32;

#[repr(C)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Pod, Zeroable, Debug, Default)]
pub struct Coordinates(pub [CoordinateUnit; 2]);
impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.horizontal(), self.vertical()))
    }
}
impl Coordinates {
    pub const fn new(a: CoordinateUnit, b: CoordinateUnit) -> Self {
        Self([a, b])
    }
    pub const fn horizontal(&self) -> CoordinateUnit {
        self.0[0]
    }
    pub const fn vertical(&self) -> CoordinateUnit {
        self.0[1]
    }
    pub fn normalized<C: Into<Coordinates>>(&self, c: C) -> Self {
        let c = c.into();
        Self::new(
            self.horizontal() / c.horizontal(),
            self.vertical() / c.vertical(),
        )
    }
    pub fn set_horizontal(&mut self, h: f32) {
        self.0[0] = h;
    }
    pub fn set_vertical(&mut self, v: f32) {
        self.0[1] = v;
    }
    pub fn clamped(&self, min: CoordinateUnit, max: CoordinateUnit) -> Self {
        Self::new(
            self.horizontal().clamp(min, max),
            self.vertical().clamp(min, max),
        )
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

// TODO fn to distill Position / Area => GpuPosition / GpuArea w/ ScaleFactor
impl Root for Coordinates {
    fn attach(elm: &mut Elm) {
        elm.scheduler
            .main
            .add_systems(coordinate_resolve.in_set(InternalStage::FinalizeCoordinate));
    }
}
fn coordinate_resolve(
    mut placed_pos: Query<
        (&mut GpuSection, &Section<LogicalContext>),
        Or<(Changed<Section<LogicalContext>>,)>,
    >,
    scale_factor: Res<ScaleFactor>,
) {
    for (mut gpu, section) in placed_pos.iter_mut() {
        gpu.pos = section.position.to_device(scale_factor.value()).to_gpu();
        gpu.area = section.area.to_device(scale_factor.value()).to_gpu();
    }
}

impl Sub for Coordinates {
    type Output = Coordinates;

    fn sub(self, rhs: Self) -> Self::Output {
        Coordinates::new(
            self.horizontal() - rhs.horizontal(),
            self.vertical() - rhs.vertical(),
        )
    }
}

impl Div<f32> for Coordinates {
    type Output = Coordinates;

    fn div(self, rhs: f32) -> Self::Output {
        Coordinates::new(self.horizontal() / rhs, self.vertical() / rhs)
    }
}
impl Div<Coordinates> for Coordinates {
    type Output = Coordinates;
    fn div(self, rhs: Coordinates) -> Self::Output {
        Coordinates::new(
            self.horizontal() / rhs.horizontal(),
            self.vertical() / rhs.vertical(),
        )
    }
}
impl Add for Coordinates {
    type Output = Coordinates;

    fn add(self, rhs: Self) -> Self::Output {
        Coordinates::new(
            self.horizontal() + rhs.horizontal(),
            self.vertical() + rhs.vertical(),
        )
    }
}

impl Mul for Coordinates {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        (
            self.horizontal() * rhs.horizontal(),
            self.vertical() * rhs.vertical(),
        )
            .into()
    }
}
impl AddAssign for Coordinates {
    fn add_assign(&mut self, rhs: Self) {
        self.0[0] += rhs.horizontal();
        self.0[1] += rhs.vertical();
    }
}
impl SubAssign for Coordinates {
    fn sub_assign(&mut self, rhs: Self) {
        self.0[0] -= rhs.horizontal();
        self.0[1] -= rhs.vertical();
    }
}
