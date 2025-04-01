use std::ops::{Add, BitAnd, BitOr, Bound, Not, RangeBounds, Shl, Shr, Sub};
pub trait Bits<Index = Self>
where
    Self: Copy,
{
    fn mask(range: impl RangeBounds<Index>) -> Self;
    fn bits(self, range: impl RangeBounds<Index>) -> Self;
    fn bit(self, index: Index) -> bool;
    fn set_bits<Value>(&mut self, range: impl RangeBounds<Index>, value: Value)
    where
        Self: From<Value>;
    fn set_bit(&mut self, index: Index, value: bool);
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

    fn bits(self, range: impl RangeBounds<Index>) -> T {
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
        (self >> start) & mask
    }

    fn bit(self, index: Index) -> bool {
        self & (T::from(1) << index) != T::from(0)
    }

    fn set_bits<Value>(&mut self, range: impl RangeBounds<Index>, value: Value)
    where
        T: From<Value>,
    {
        let start = match range.start_bound() {
            Bound::Included(&start) => start,
            Bound::Excluded(&start) => start + Index::from(1),
            Bound::Unbounded => Index::from(0),
        };
        let mask = T::mask(range);
        *self = *self & !mask | (T::from(value) << start) & mask
    }

    fn set_bit(&mut self, index: Index, value: bool) {
        let mask = T::from(1) << index;
        *self = *self & !mask | (if value { mask } else { T::from(0) })
    }
}

pub trait SignExtend<T> {
    fn sign_extend(self) -> T;
}

impl<T> SignExtend<T> for u8
where
    i8: SignExtend<T>,
{
    fn sign_extend(self) -> T {
        (self as u8).sign_extend()
    }
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

impl SignExtend<u64> for i8 {
    fn sign_extend(self) -> u64 {
        self as i32 as u64
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
