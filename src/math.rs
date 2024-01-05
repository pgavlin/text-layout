use core::cmp::{Ordering, PartialOrd};
use core::fmt::{self, Debug};
use core::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use fixed::traits::{FixedSigned, ToFixed};

/// A trait that describes the operations necessary for this crate's layout algorithms.
pub trait Num
where
    Self: Default
        + Debug
        + Copy
        + PartialOrd<Self>
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + SubAssign
        + Mul<Output = Self>
        + Div<Output = Self>,
{
    const INFINITY: Self;
    const NEG_INFINITY: Self;

    fn from(i: i16) -> Self;
    fn abs(self) -> Self;
    fn powi(self, y: u32) -> Self;

    fn rat(num: i16, denom: i16) -> Self {
        Self::from(num) / Self::from(denom)
    }
}

#[cfg(feature = "std")]
impl Num for f32 {
    const INFINITY: Self = f32::INFINITY;
    const NEG_INFINITY: Self = f32::NEG_INFINITY;

    fn from(i: i16) -> f32 {
        i.into()
    }

    fn abs(self) -> f32 {
        self.abs()
    }

    fn powi(self, y: u32) -> f32 {
        self.powi(y as i32)
    }
}

#[cfg(all(not(feature = "std"), feature = "libm"))]
impl Num for f32 {
    const INFINITY: Self = f32::INFINITY;
    const NEG_INFINITY: Self = f32::NEG_INFINITY;

    fn from(i: i16) -> f32 {
        i.into()
    }

    fn abs(self) -> f32 {
        libm::fabsf(self)
    }

    fn powi(self, y: u32) -> f32 {
        libm::powf(self, y as f32)
    }
}

/// Wraps a signed fixed-point number. All operations are saturating so that the underlying
/// representation's minimum and maximum values are able to stand in for -∞ and +∞.
#[derive(Default, Clone, Copy)]
pub struct Fixed<F: FixedSigned>(F);

impl<F: FixedSigned> Debug for Fixed<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        Debug::fmt(&self.0, f)
    }
}

impl<F: FixedSigned> PartialEq for Fixed<F> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<F: FixedSigned> PartialOrd for Fixed<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<F: FixedSigned> Add for Fixed<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Fixed(self.0.saturating_add(rhs.0))
    }
}

impl<F: FixedSigned> AddAssign for Fixed<F> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<F: FixedSigned> Sub for Fixed<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Fixed(self.0.saturating_sub(rhs.0))
    }
}

impl<F: FixedSigned> SubAssign for Fixed<F> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<F: FixedSigned> Mul for Fixed<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Fixed(self.0.saturating_mul(rhs.0))
    }
}

impl<F: FixedSigned> Div for Fixed<F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Fixed(self.0.saturating_div(rhs.0))
    }
}

impl<F: FixedSigned> Fixed<F> {
    pub const MAX: Self = Self(F::MAX);
    pub const MIN: Self = Self(F::MIN);

    pub fn from_num<Src: ToFixed>(src: Src) -> Self {
        Fixed(F::from_num(src))
    }
}

impl<F: FixedSigned> Num for Fixed<F> {
    const INFINITY: Self = Self::MAX;
    const NEG_INFINITY: Self = Self::MIN;

    fn from(i: i16) -> Self {
        Self::from_num(i)
    }

    fn abs(self) -> Self {
        Fixed(self.0.abs())
    }

    fn powi(self, y: u32) -> Self {
        let mut result: Self = Self::from_num(1);
        for _ in 0..y {
            result = result * self;
        }
        result
    }
}
