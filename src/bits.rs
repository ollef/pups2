use std::ops::{Add, BitAnd, BitOr, Bound, Not, RangeBounds, Shl, Shr, Sub};
pub trait Bits<Index = Self>
where
    Self: Copy,
{
    fn mask(range: impl RangeBounds<Index>) -> Self;
    fn bits(self, range: impl RangeBounds<Index>) -> Self;
    fn bit(self, index: Index) -> bool;
    fn set_bits(self, range: impl RangeBounds<Index>, value: Self) -> Self;
    fn set_bit(self, index: Index, value: bool) -> Self;
}

impl<T, Index> Bits<Index> for T
where
    T: BitAnd<Output = T>,
    T: BitOr<Output = T>,
    T: Copy,
    T: Eq,
    T: From<u8>,
    T: Not<Output = T>,
    T: Shl<Index, Output = T>,
    T: Shr<Index, Output = T>,
    T: Sub<Output = T>,
    Index: From<u8>,
    Index: Add<Output = Index>,
    Index: Sub<Output = Index>,
    Index: Copy,
{
    fn mask(range: impl RangeBounds<Index>) -> T {
        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + Index::from(1),
            Bound::Unbounded => Index::from(0),
        };
        let end = match range.end_bound() {
            Bound::Included(&end) => end + Index::from(1),
            Bound::Excluded(&end) => end,
            Bound::Unbounded => Index::from(8 * std::mem::size_of::<T>() as u8),
        };
        let mask = (T::from(1) << (end - start)) - T::from(1);
        mask << start
    }

    fn bits(self, range: impl RangeBounds<Index>) -> Self {
        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + Index::from(1),
            Bound::Unbounded => Index::from(0),
        };
        let end = match range.end_bound() {
            Bound::Included(&end) => end + Index::from(1),
            Bound::Excluded(&end) => end,
            Bound::Unbounded => Index::from(8 * std::mem::size_of::<T>() as u8),
        };
        let mask = (Self::from(1) << (end - start)) - Self::from(1);
        (self >> start) & mask
    }

    fn bit(self, index: Index) -> bool {
        self & (Self::from(1) << index) != Self::from(0)
    }

    fn set_bits(self, range: impl RangeBounds<Index>, value: T) -> Self {
        let mask = Self::mask(range);
        self & !mask | value & mask
    }

    fn set_bit(self, index: Index, value: bool) -> Self {
        let mask = Self::from(1) << index;
        self & !mask | (if value { mask } else { Self::from(0) })
    }
}

pub trait SignExtend<T> {
    fn sign_extend(self) -> T;
}

impl<T> SignExtend<T> for u16
where
    i16: SignExtend<T>,
{
    fn sign_extend(self) -> T {
        (self as i16).sign_extend()
    }
}

impl<T> SignExtend<T> for u32
where
    i32: SignExtend<T>,
{
    fn sign_extend(self) -> T {
        (self as i32).sign_extend()
    }
}

impl SignExtend<u32> for i16 {
    fn sign_extend(self) -> u32 {
        self as i32 as u32
    }
}

impl SignExtend<u64> for i16 {
    fn sign_extend(self) -> u64 {
        self as i64 as u64
    }
}

impl SignExtend<u64> for i32 {
    fn sign_extend(self) -> u64 {
        self as i64 as u64
    }
}
