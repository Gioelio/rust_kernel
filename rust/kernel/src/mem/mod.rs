use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

pub enum Error {
    Full 
}

pub struct RingBuffer<T: Sized + Copy, const COUNT: usize> {
    buffer: [UnsafeCell<T>; COUNT],
    idx_in: AtomicUsize,
    idx_out: AtomicUsize,
}

// Safety: SPSC queue guarantees producer and consumer access distinct elements
unsafe impl<T: Send + Copy, const COUNT: usize> Sync for RingBuffer<T, COUNT> {}

impl<T: Sized + Copy, const COUNT: usize> RingBuffer<T, COUNT> {

    /// Create a new RingBuffer to handle queue, COUNT must be power of 2
    pub const fn new(init_val: T) -> Self {
        assert!(COUNT.is_power_of_two(), "COUNT must be a power of two");

        let mut buffer: [MaybeUninit<UnsafeCell<T>>; COUNT] =
            unsafe {MaybeUninit::uninit().assume_init() };

        let mut i = 0;
        while i < COUNT {
            buffer[i] = MaybeUninit::new(UnsafeCell::new(init_val));
            i += 1;
        }

        let buffer = unsafe {
            core::mem::transmute_copy::<
                [MaybeUninit<UnsafeCell<T>>; COUNT],
                [UnsafeCell<T>; COUNT],
            >(&buffer)
        };

        RingBuffer {
            buffer: buffer,
            idx_in: AtomicUsize::new(0),
            idx_out: AtomicUsize::new(0)
        }
    }

    pub fn push(&self, value: T) -> Result<(), Error> {
        let head = self.idx_in.load(Ordering::Relaxed);
        let tail = self.idx_out.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= COUNT {
            return Err(Error::Full);
        }

        let slot = head & (COUNT - 1);

        unsafe {
            *self.buffer[slot].get() = value;
        }

        self.idx_in.store(head.wrapping_add(1), Ordering::Release);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        let tail = self.idx_out.load(Ordering::Relaxed);
        let head = self.idx_in.load(Ordering::Acquire);

        if head == tail {
            return None;
        }

        let slot = tail & (COUNT - 1);

        let value = unsafe {
            *self.buffer[slot].get()
        };

        self.idx_out.store(tail.wrapping_add(1), Ordering::Release);

        Some(value)
    }

    pub fn len(&self) -> usize {
        let head = self.idx_in.load(Ordering::Relaxed);
        let tail = self.idx_out.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    /// Check buffer emptiness
    ///
    /// More optimized to execute pop and check for presence/absence
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }


}
