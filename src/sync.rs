//! Lightweight synchronisation primitives.
//!
//! These are intentionally small.  The kernel runs with interrupts disabled
//! while a lock is held, so a fair ticket lock is unnecessary; a test-and-set
//! spin lock is both smaller and faster.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

/// A test-and-set spin lock protecting a value of type `T`.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: access to `data` is serialised by `locked`.
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Create a new unlocked spin lock.
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquire the lock, spinning until it becomes available.
    #[inline]
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }
        SpinLockGuard { lock: self }
    }

    /// Try to acquire the lock without spinning.
    #[inline]
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }

    /// Access the protected value without locking.  Requires exclusive access
    /// to the lock, which statically rules out concurrent users.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: `&mut self` guarantees that no guard exists.
        unsafe { &mut *self.data.get() }
    }

    /// Consume the lock and return its value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Returns `true` if the lock is currently held.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }
}

impl<T: Default> Default for SpinLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// RAII guard returned by [`SpinLock::lock`].
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: the guard holds the lock.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the guard holds the lock exclusively.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

const ONCE_UNINIT: u8 = 0;
const ONCE_RUNNING: u8 = 1;
const ONCE_READY: u8 = 2;

/// A primitive that can be initialised exactly once.
pub struct Once<T> {
    state: AtomicU8,
    data: UnsafeCell<MaybeUninit<T>>,
}

// SAFETY: the state machine ensures initialisation happens once and is
// published with release/acquire ordering before any reader observes READY.
unsafe impl<T: Send + Sync> Sync for Once<T> {}
unsafe impl<T: Send> Send for Once<T> {}

impl<T> Once<T> {
    /// Create a new, uninitialised `Once`.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(ONCE_UNINIT),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Initialise the value if needed and return a reference to it.
    pub fn call_once<F: FnOnce() -> T>(&self, f: F) -> &T {
        match self.state.compare_exchange(
            ONCE_UNINIT,
            ONCE_RUNNING,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                let value = f();
                // SAFETY: we won the initialisation race.
                unsafe {
                    (*self.data.get()).write(value);
                }
                self.state.store(ONCE_READY, Ordering::Release);
            }
            Err(_) => {
                while self.state.load(Ordering::Acquire) != ONCE_READY {
                    spin_loop();
                }
            }
        }
        // SAFETY: READY means the value was written and published.
        unsafe { (*self.data.get()).assume_init_ref() }
    }

    /// Return the value if it has been initialised.
    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == ONCE_READY {
            // SAFETY: READY means the value was written and published.
            Some(unsafe { (*self.data.get()).assume_init_ref() })
        } else {
            None
        }
    }
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A reader/writer spin lock.  Writers are preferred, which prevents a stream
/// of readers from starving a writer.
pub struct RwSpinLock<T> {
    state: AtomicUsize,
    data: UnsafeCell<T>,
}

const WRITER: usize = 1 << (usize::BITS - 1);

// SAFETY: access to `data` is serialised by the state machine.
unsafe impl<T: Send> Send for RwSpinLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwSpinLock<T> {}

impl<T> RwSpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn read(&self) -> RwReadGuard<'_, T> {
        loop {
            let state = self.state.load(Ordering::Relaxed);
            if state & WRITER != 0 {
                spin_loop();
                continue;
            }
            if self
                .state
                .compare_exchange_weak(state, state + 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return RwReadGuard { lock: self };
            }
        }
    }

    pub fn write(&self) -> RwWriteGuard<'_, T> {
        loop {
            if self
                .state
                .compare_exchange(0, WRITER, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return RwWriteGuard { lock: self };
            }
            while self.state.load(Ordering::Relaxed) != 0 {
                spin_loop();
            }
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: `&mut self` guarantees exclusive access.
        unsafe { &mut *self.data.get() }
    }
}

impl<T: Default> Default for RwSpinLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub struct RwReadGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for RwReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: a read guard keeps the reader count non-zero.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for RwReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct RwWriteGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for RwWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: a write guard owns the lock.
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for RwWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: a write guard owns the lock.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for RwWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}

/// A simple one-way flag with acquire/release semantics.
#[derive(Debug, Default)]
pub struct AtomicFlag {
    flag: AtomicBool,
}

impl AtomicFlag {
    pub const fn new() -> Self {
        Self {
            flag: AtomicBool::new(false),
        }
    }

    pub fn set(&self) {
        self.flag.store(true, Ordering::Release);
    }

    pub fn clear(&self) {
        self.flag.store(false, Ordering::Release);
    }

    pub fn is_set(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }
}

/// Spin until `predicate` returns `true`.
#[inline]
pub fn spin_until<F: FnMut() -> bool>(mut predicate: F) {
    while !predicate() {
        spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn spinlock_serialises_increments() {
        let lock = Arc::new(SpinLock::new(0u64));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let lock = lock.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1_000 {
                    *lock.lock() += 1;
                }
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(*lock.lock(), 8_000);
    }

    #[test]
    fn once_runs_once() {
        let once = Once::new();
        let start = std::time::Instant::now();
        assert_eq!(*once.call_once(|| 42), 42);
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert_eq!(*once.call_once(|| 99), 42);
        assert!(start.elapsed().as_millis() < 1_000);
    }
}
