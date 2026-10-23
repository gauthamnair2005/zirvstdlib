//! Bitmaps and bitsets.

/// A dynamic bitmap over a caller-provided slice of 64-bit words.
pub struct Bitmap<'a> {
    words: &'a mut [u64],
    bits: usize,
}

impl<'a> Bitmap<'a> {
    /// Wrap an existing word slice.  All bits start in their current state.
    pub fn new(words: &'a mut [u64], bits: usize) -> Self {
        debug_assert!(bits <= words.len() * 64);
        Self { words, bits }
    }

    /// Wrap a slice and clear every bit.
    pub fn zeroed(words: &'a mut [u64]) -> Self {
        for word in words.iter_mut() {
            *word = 0;
        }
        let bits = words.len() * 64;
        Self { words, bits }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bits
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        if index >= self.bits {
            return false;
        }
        (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    #[inline]
    pub fn set(&mut self, index: usize) {
        if index < self.bits {
            self.words[index / 64] |= 1u64 << (index % 64);
        }
    }

    #[inline]
    pub fn clear(&mut self, index: usize) {
        if index < self.bits {
            self.words[index / 64] &= !(1u64 << (index % 64));
        }
    }

    #[inline]
    pub fn toggle(&mut self, index: usize) {
        if index < self.bits {
            self.words[index / 64] ^= 1u64 << (index % 64);
        }
    }

    pub fn set_all(&mut self, value: bool) {
        let fill = if value { u64::MAX } else { 0 };
        for word in self.words.iter_mut() {
            *word = fill;
        }
        self.mask_tail();
    }

    pub fn clear_all(&mut self) {
        self.set_all(false);
    }

    /// Find the first clear bit at or after `start`.
    pub fn find_first_zero(&self, start: usize) -> Option<usize> {
        if start >= self.bits {
            return None;
        }
        let mut word_index = start / 64;
        let offset = start % 64;
        let mut word = !self.words[word_index] & (u64::MAX << offset);
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let index = word_index * 64 + bit;
                return if index < self.bits { Some(index) } else { None };
            }
            word_index += 1;
            if word_index >= self.words.len() {
                return None;
            }
            word = !self.words[word_index];
        }
    }

    /// Find the first set bit at or after `start`.
    pub fn find_first_set(&self, start: usize) -> Option<usize> {
        if start >= self.bits {
            return None;
        }
        let mut word_index = start / 64;
        let offset = start % 64;
        let mut word = self.words[word_index] & (u64::MAX << offset);
        loop {
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                let index = word_index * 64 + bit;
                return if index < self.bits { Some(index) } else { None };
            }
            word_index += 1;
            if word_index >= self.words.len() {
                return None;
            }
            word = self.words[word_index];
        }
    }

    /// Count the number of set bits.
    pub fn count_ones(&self) -> usize {
        let mut total = 0;
        for (index, word) in self.words.iter().enumerate() {
            let value = if index + 1 == self.words.len() {
                // Mask the unused tail.
                let used = self.bits % 64;
                if used == 0 {
                    *word
                } else {
                    word & ((1u64 << used) - 1)
                }
            } else {
                *word
            };
            total += value.count_ones() as usize;
        }
        total
    }

    /// Count the number of clear bits.
    pub fn count_zeros(&self) -> usize {
        self.bits - self.count_ones()
    }

    fn mask_tail(&mut self) {
        let used = self.bits % 64;
        if used != 0 {
            if let Some(last) = self.words.last_mut() {
                *last &= (1u64 << used) - 1;
            }
        }
    }
}

/// A fixed-capacity bitset.
#[derive(Clone)]
pub struct BitSet<const N: usize> {
    words: [u64; N],
}

impl<const N: usize> BitSet<N> {
    pub const fn new() -> Self {
        Self { words: [0; N] }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N * 64
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        index < self.capacity() && (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    #[inline]
    pub fn set(&mut self, index: usize) -> bool {
        if index >= self.capacity() {
            return false;
        }
        self.words[index / 64] |= 1u64 << (index % 64);
        true
    }

    #[inline]
    pub fn clear(&mut self, index: usize) -> bool {
        if index >= self.capacity() {
            return false;
        }
        self.words[index / 64] &= !(1u64 << (index % 64));
        true
    }

    pub fn iter_set(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(word_index, word)| {
            let mut value = *word;
            core::iter::from_fn(move || {
                if value == 0 {
                    None
                } else {
                    let bit = value.trailing_zeros() as usize;
                    value &= value - 1;
                    Some(word_index * 64 + bit)
                }
            })
        })
    }
}

impl<const N: usize> Default for BitSet<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_basics() {
        let mut words = [0u64; 2];
        let mut bitmap = Bitmap::new(&mut words, 100);
        assert_eq!(bitmap.find_first_zero(0), Some(0));
        bitmap.set(3);
        bitmap.set(64);
        assert!(bitmap.get(3));
        assert!(bitmap.get(64));
        assert_eq!(bitmap.find_first_zero(0), Some(0));
        assert_eq!(bitmap.find_first_zero(3), Some(4));
        assert_eq!(bitmap.find_first_set(0), Some(3));
        assert_eq!(bitmap.count_ones(), 2);
    }

    #[test]
    fn bitset_iter() {
        let mut set = BitSet::<2>::new();
        set.set(1);
        set.set(70);
        let values: alloc::vec::Vec<_> = set.iter_set().collect();
        assert_eq!(values, alloc::vec![1, 70]);
    }
}
