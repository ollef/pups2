use std::ops::{Add, Div, Mul, Shl, Shr, Sub};

use num_traits::{AsPrimitive, One};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct Fix<T, const F: usize>(T);

impl<T, const F: usize> Fix<T, F> {
    pub fn from_integer<S>(value: S) -> Self
    where
        S: Shl<usize, Output = T>,
    {
        Fix(value << F)
    }

    pub fn from_raw(raw: T) -> Self {
        Fix(raw)
    }

    pub fn raw(self) -> T {
        self.0
    }

    pub fn round(self) -> T
    where
        T: One,
        T: Add<T, Output = T>,
        T: Shl<usize, Output = T>,
        T: Shr<usize, Output = T>,
    {
        (self.0 + (T::one() << (F - 1))) >> F
    }

    pub fn floor(self) -> T
    where
        T: Shr<usize, Output = T>,
    {
        self.0 >> F
    }

    pub fn ceil(self) -> T
    where
        T: One,
        T: Add<T, Output = T>,
        T: Sub<T, Output = T>,
        T: Shl<usize, Output = T>,
        T: Shr<usize, Output = T>,
    {
        (self.0 + (T::one() << F) - T::one()) >> F
    }
}

impl<T, const F: usize> From<Fix<T, F>> for f32
where
    T: Into<f32>,
{
    fn from(value: Fix<T, F>) -> f32 {
        value.0.into() / 2f32.powi(F as i32)
    }
}

impl<T, const F: usize> From<f32> for Fix<T, F>
where
    T: 'static + Copy,
    f32: AsPrimitive<T>,
{
    fn from(value: f32) -> Self {
        Fix((value * 2f32.powi(F as i32)).as_())
    }
}

impl<S, T, const F: usize> Add<Fix<T, F>> for Fix<S, F>
where
    S: Add<T>,
{
    type Output = Fix<<S as Add<T>>::Output, F>;

    fn add(self, rhs: Fix<T, F>) -> Self::Output {
        Fix(self.0 + rhs.0)
    }
}

impl<S, T, const F: usize> Sub<Fix<T, F>> for Fix<S, F>
where
    S: Sub<T>,
{
    type Output = Fix<<S as Sub<T>>::Output, F>;

    fn sub(self, rhs: Fix<T, F>) -> Self::Output {
        Fix(self.0 - rhs.0)
    }
}

impl<S, T, const F: usize> Mul<Fix<T, F>> for Fix<S, F>
where
    S: Mul<T>,
    <S as Mul<T>>::Output: Shr<usize>,
{
    type Output = Fix<<<S as Mul<T>>::Output as Shr<usize>>::Output, F>;

    fn mul(self, rhs: Fix<T, F>) -> Self::Output {
        Fix((self.0 * rhs.0) >> F)
    }
}

impl<S, T, const F: usize> Div<Fix<T, F>> for Fix<S, F>
where
    S: Shl<usize>,
    <S as Shl<usize>>::Output: Div<T>,
{
    type Output = Fix<<<S as std::ops::Shl<usize>>::Output as std::ops::Div<T>>::Output, F>;

    fn div(self, rhs: Fix<T, F>) -> Self::Output {
        Fix((self.0 << F) / rhs.0)
    }
}

impl<T, const F: usize> Shl<usize> for Fix<T, F>
where
    T: Shl<usize>,
{
    type Output = Fix<<T as Shl<usize>>::Output, F>;

    fn shl(self, rhs: usize) -> Self::Output {
        Fix(self.0 << rhs)
    }
}

impl<T, const F: usize> Shr<usize> for Fix<T, F>
where
    T: Shr<usize>,
{
    type Output = Fix<<T as Shr<usize>>::Output, F>;

    fn shr(self, rhs: usize) -> Self::Output {
        Fix(self.0 >> rhs)
    }
}
