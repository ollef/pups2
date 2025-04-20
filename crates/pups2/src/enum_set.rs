use enum_map::Enum;
use num_traits::PrimInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumSet<Carrier: PrimInt, T: Enum + Copy> {
    bits: Carrier,
    _marker: std::marker::PhantomData<T>,
}

impl<Carrier: PrimInt, T: Enum + Copy> Default for EnumSet<Carrier, T> {
    fn default() -> Self {
        EnumSet::new()
    }
}

impl<Carrier: PrimInt, T: Enum + Copy> EnumSet<Carrier, T> {
    pub fn new() -> Self {
        assert!(T::LENGTH <= std::mem::size_of::<Carrier>() * 8);
        EnumSet {
            bits: Carrier::zero(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits == Carrier::zero()
    }

    pub fn len(&self) -> usize {
        self.bits.count_ones() as usize
    }

    pub fn contains(&self, value: T) -> bool {
        self.bits & Carrier::one() << value.into_usize() != Carrier::zero()
    }

    pub fn insert(&mut self, value: T) {
        self.bits = self.bits | (Carrier::one() << value.into_usize());
    }

    pub fn remove(&mut self, value: T) {
        self.bits = self.bits & !(Carrier::one() << value.into_usize());
    }
}

pub struct Iter<Carrier: PrimInt, T: Enum + Copy> {
    bits: Carrier,
    next: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<Carrier: PrimInt, T: Enum + Copy> Iterator for Iter<Carrier, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let zeros = self.bits.trailing_zeros() as usize;
        if zeros >= T::LENGTH {
            return None;
        }
        let index = self.next + zeros;
        self.bits = self.bits >> (zeros + 1);
        self.next += zeros + 1;
        Some(T::from_usize(index))
    }
}

impl<Carrier: PrimInt, T: Enum + Copy> IntoIterator for EnumSet<Carrier, T> {
    type Item = T;
    type IntoIter = Iter<Carrier, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            bits: self.bits,
            next: 0,
            _marker: std::marker::PhantomData,
        }
    }
}
