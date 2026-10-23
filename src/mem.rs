//! Memory helpers and address newtypes.

use core::sync::atomic::{Ordering, compiler_fence};

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;
pub const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;
pub const CACHE_LINE: usize = 64;

#[inline]
pub const fn align_up_usize(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}

#[inline]
pub const fn align_down_usize(value: usize, align: usize) -> usize {
    value & !(align - 1)
}

#[inline]
pub const fn size_of<T>() -> usize {
    core::mem::size_of::<T>()
}

#[inline]
pub const fn align_of<T>() -> usize {
    core::mem::align_of::<T>()
}

/// Read a volatile value, preventing the compiler from eliding the access.
#[inline]
pub unsafe fn read_volatile<T: Copy>(address: *const T) -> T {
    // SAFETY: the caller guarantees that `address` is valid for a read.
    unsafe { core::ptr::read_volatile(address) }
}

/// Write a volatile value.
#[inline]
pub unsafe fn write_volatile<T>(address: *mut T, value: T) {
    // SAFETY: the caller guarantees that `address` is valid for a write.
    unsafe { core::ptr::write_volatile(address, value) };
}

/// Read an unaligned value.
#[inline]
pub unsafe fn read_unaligned<T: Copy>(address: *const T) -> T {
    // SAFETY: the caller guarantees that `address` is valid for a read.
    unsafe { core::ptr::read_unaligned(address) }
}

/// Write an unaligned value.
#[inline]
pub unsafe fn write_unaligned<T>(address: *mut T, value: T) {
    // SAFETY: the caller guarantees that `address` is valid for a write.
    unsafe { core::ptr::write_unaligned(address, value) };
}

/// Zero a region of memory.
#[inline]
pub unsafe fn zero<T>(address: *mut T, count: usize) {
    // SAFETY: the caller guarantees the region is writable for `count` items.
    unsafe {
        core::ptr::write_bytes(address as *mut u8, 0, count * core::mem::size_of::<T>());
    }
}

/// Copy a non-overlapping region.
#[inline]
pub unsafe fn copy<T>(source: *const T, destination: *mut T, count: usize) {
    // SAFETY: the caller guarantees both regions are valid and non-overlapping.
    unsafe {
        core::ptr::copy_nonoverlapping(source, destination, count);
    }
}

/// A physical address.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }

    pub const fn align_up(self, align: u64) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }

    pub const fn align_down(self, align: u64) -> Self {
        Self(self.0 & !(align - 1))
    }

    pub const fn offset(self, bytes: i64) -> Self {
        Self(self.0.wrapping_add(bytes as u64))
    }
}

impl core::fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

/// A virtual address.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub const fn as_ptr<T>(self) -> *mut T {
        self.0 as usize as *mut T
    }

    pub const fn is_aligned(self, align: u64) -> bool {
        self.0 & (align - 1) == 0
    }

    pub const fn offset(self, bytes: i64) -> Self {
        Self(self.0.wrapping_add(bytes as u64))
    }
}

impl core::fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

/// A 4 KiB page number.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Page(u64);

impl Page {
    pub const fn containing(address: PhysAddr) -> Self {
        Self(address.0 >> PAGE_SHIFT)
    }

    pub const fn start_address(self) -> PhysAddr {
        PhysAddr(self.0 << PAGE_SHIFT)
    }

    pub const fn number(self) -> u64 {
        self.0
    }
}

/// Issue a compiler fence.  On x86, locked instructions and MMIO accesses are
/// ordered by the hardware; the compiler fence prevents reordering in Rust.
#[inline]
pub fn fence() {
    compiler_fence(Ordering::SeqCst);
}

/// A single-producer/single-consumer memory fence for device rings.
#[inline]
pub fn device_fence() {
    compiler_fence(Ordering::SeqCst);
    core::sync::atomic::fence(Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses() {
        let address = PhysAddr::new(0x1234);
        assert_eq!(address.align_up(4096), PhysAddr::new(0x2000));
        assert_eq!(address.align_down(4096), PhysAddr::new(0x1000));
    }
}
