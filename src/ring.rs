//! A fixed-capacity ring buffer.

use core::mem::MaybeUninit;

/// A lock-free-by-construction ring buffer.  Callers provide their own
/// synchronisation when sharing one between threads or interrupt handlers.
pub struct RingBuffer<T, const N: usize> {
    slots: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    /// Create an empty ring buffer.  `N` must be greater than zero for the
    /// buffer to be useful, but a zero-sized buffer is representable.
    pub const fn new() -> Self {
        Self {
            // SAFETY: an array of `MaybeUninit` is always valid, even when
            // uninitialised.
            slots: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            head: 0,
            len: 0,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    #[inline]
    fn physical_index(&self, logical: usize) -> usize {
        if N == 0 {
            0
        } else {
            (self.head + logical) % N
        }
    }

    /// Push a value, returning it back if the buffer is full.
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            return Err(value);
        }
        let index = self.physical_index(self.len);
        // SAFETY: the slot is uninitialised because `len < N`.
        self.slots[index].write(value);
        self.len += 1;
        Ok(())
    }

    /// Push a value, overwriting the oldest entry if the buffer is full.
    /// Returns the overwritten value, if any.
    pub fn push_overwrite(&mut self, value: T) -> Option<T> {
        if N == 0 {
            return Some(value);
        }
        if self.len == N {
            let index = self.head;
            // SAFETY: the slot contains an initialised value.
            let old = unsafe { self.slots[index].assume_init_read() };
            // SAFETY: the slot is now uninitialised and gets overwritten.
            self.slots[index].write(value);
            self.head = (self.head + 1) % N;
            Some(old)
        } else {
            let _ = self.push(value);
            None
        }
    }

    /// Remove and return the oldest value.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let index = self.head;
        // SAFETY: the slot contains an initialised value.
        let value = unsafe { self.slots[index].assume_init_read() };
        self.head = if N == 0 { 0 } else { (self.head + 1) % N };
        self.len -= 1;
        Some(value)
    }

    /// Look at the oldest value without removing it.
    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            // SAFETY: the slot contains an initialised value.
            Some(unsafe { self.slots[self.head].assume_init_ref() })
        }
    }

    /// Look at the newest value without removing it.
    pub fn peek_back(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            let index = self.physical_index(self.len - 1);
            // SAFETY: the slot contains an initialised value.
            Some(unsafe { self.slots[index].assume_init_ref() })
        }
    }

    /// Access a value by logical index, where zero is the oldest entry.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let physical = self.physical_index(index);
        // SAFETY: the slot contains an initialised value.
        Some(unsafe { self.slots[physical].assume_init_ref() })
    }

    /// Drop every element and reset the buffer.
    pub fn clear(&mut self) {
        while self.pop().is_some() {}
        self.head = 0;
    }

    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter {
            ring: self,
            index: 0,
        }
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, T, const N: usize> {
    ring: &'a RingBuffer<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.ring.get(self.index);
        self.index += 1;
        value
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.ring.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl<T, const N: usize> ExactSizeIterator for Iter<'_, T, N> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_operations() {
        let mut ring = RingBuffer::<u32, 3>::new();
        assert!(ring.push(1).is_ok());
        assert!(ring.push(2).is_ok());
        assert!(ring.push(3).is_ok());
        assert_eq!(ring.push(4), Err(4));
        assert_eq!(ring.pop(), Some(1));
        assert!(ring.push(4).is_ok());
        assert_eq!(ring.iter().copied().collect::<alloc::vec::Vec<_>>(), alloc::vec![2, 3, 4]);
    }

    #[test]
    fn overwrite() {
        let mut ring = RingBuffer::<u32, 2>::new();
        ring.push(1).unwrap();
        ring.push(2).unwrap();
        assert_eq!(ring.push_overwrite(3), Some(1));
        assert_eq!(ring.pop(), Some(2));
        assert_eq!(ring.pop(), Some(3));
    }
}
