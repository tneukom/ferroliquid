use num_traits::{ConstOne, ConstZero, Euclid, Float, Inv, Signed};
use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

pub trait Dot<Rhs = Self> {
    type Output;

    fn dot(self, rhs: Rhs) -> Self::Output;
}

pub trait MinMax {
    fn min(self, rhs: Self) -> Self;
    fn max(self, rhs: Self) -> Self;
}

macro_rules! impl_min_max_ord {
    ($t: ty) => {
        impl MinMax for $t {
            fn min(self, rhs: Self) -> Self {
                std::cmp::min(self, rhs)
            }

            fn max(self, rhs: Self) -> Self {
                std::cmp::max(self, rhs)
            }
        }
    };
}

impl_min_max_ord!(u8);
impl_min_max_ord!(u16);
impl_min_max_ord!(u32);
impl_min_max_ord!(u64);
impl_min_max_ord!(u128);
impl_min_max_ord!(usize);
impl_min_max_ord!(i8);
impl_min_max_ord!(i16);
impl_min_max_ord!(i32);
impl_min_max_ord!(i64);
impl_min_max_ord!(i128);
impl_min_max_ord!(isize);

macro_rules! impl_min_max_primitive {
    ($t: ty) => {
        impl MinMax for $t {
            fn min(self, rhs: Self) -> Self {
                <$t>::min(self, rhs)
            }

            fn max(self, rhs: Self) -> Self {
                <$t>::max(self, rhs)
            }
        }
    };
}

impl_min_max_primitive!(f32);
impl_min_max_primitive!(f64);

pub trait ConstTwo {
    const TWO: Self;
}

macro_rules! impl_const_two {
    ($t: ty, $expr: expr) => {
        impl ConstTwo for $t {
            const TWO: Self = $expr;
        }
    };
}

impl_const_two!(f64, 2.0);
impl_const_two!(f32, 2.0);
impl_const_two!(i128, 2);
impl_const_two!(i64, 2);
impl_const_two!(i32, 2);
impl_const_two!(i16, 2);
impl_const_two!(i8, 2);
impl_const_two!(u128, 2);
impl_const_two!(u64, 2);
impl_const_two!(u32, 2);
impl_const_two!(u16, 2);
impl_const_two!(u8, 2);
impl_const_two!(isize, 2);
impl_const_two!(usize, 2);

pub trait Num:
    Copy
    + Display
    + Sized
    + Debug
    + PartialOrd
    + ConstZero
    + ConstOne
    + ConstTwo
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Euclid
    + MinMax
{
}

impl Num for u8 {}

impl Num for i8 {}

impl Num for u16 {}

impl Num for i16 {}

impl Num for u32 {}

impl Num for i32 {}

impl Num for u64 {}

impl Num for i64 {}

impl Num for u128 {}

impl Num for i128 {}

impl Num for usize {}

impl Num for isize {}

impl Num for f64 {}

impl Num for f32 {}

pub trait SignedNum: Num + Neg<Output = Self> + Signed {}

impl SignedNum for f64 {}

impl SignedNum for f32 {}

impl SignedNum for i8 {}

impl SignedNum for i16 {}

impl SignedNum for i32 {}

impl SignedNum for i64 {}

impl SignedNum for isize {}

pub trait FloatNum: SignedNum + Div<Output = Self> + Inv<Output = Self> + Float {}

impl FloatNum for f64 {}

impl FloatNum for f32 {}
