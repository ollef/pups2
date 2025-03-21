use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

pub struct Fifo<T> {
    data: Box<[MaybeUninit<T>]>,
    capacity: usize,
    front: usize,
    len: usize,
}

impl<T> Drop for Fifo<T> {
    fn drop(&mut self) {
        while self.pop_back().is_some() {}
    }
}

impl<T> Fifo<T> {
    pub fn with_capacity(capacity: usize) -> Fifo<T>
    where
        T: Clone,
    {
        let mut data = Vec::new();
        data.resize_with(capacity, MaybeUninit::uninit);
        Fifo {
            data: data.into_boxed_slice(),
            capacity,
            front: 0,
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    fn wrap_index(&self, index: usize) -> usize {
        if index < self.capacity {
            index
        } else {
            index - self.capacity
        }
    }

    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    pub fn push_back(&mut self, value: T) {
        assert!(self.len < self.capacity);
        self.data[self.wrap_index(self.front + self.len)].write(value);
        self.len += 1;
    }

    pub fn push_front(&mut self, value: T) {
        assert!(self.len < self.capacity);
        self.len += 1;
        self.front = if self.front == 0 {
            self.capacity - 1
        } else {
            self.front - 1
        };
        self.data[self.front].write(value);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let index = self.wrap_index(self.front + self.len);
        Some(unsafe { self.data[index].assume_init_read() })
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let value = unsafe { self.data[self.front].assume_init_read() };
        self.front = self.wrap_index(self.front + 1);
        Some(value)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> + '_ {
        (0..self.len).map(|i| &self[i])
    }
}

impl<T> Index<usize> for Fifo<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        assert!(index < self.len);
        let index = self.wrap_index(self.front + index);
        unsafe { &*self.data[index].as_ptr() }
    }
}

impl<T> IndexMut<usize> for Fifo<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        assert!(index < self.len);
        let index = self.wrap_index(self.front + index);
        unsafe { &mut *self.data[index].as_mut_ptr() }
    }
}
