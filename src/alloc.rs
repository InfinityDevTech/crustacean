use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

pub struct CountingAllocator<A> {
    inner: A,
    allocated_now: AtomicIsize,
}

impl<A> CountingAllocator<A> {
    const fn new(inner: A) -> Self {
        Self {
            inner,
            allocated_now: AtomicIsize::new(0),
        }
    }

    pub fn allocated_now(&self) -> usize {
        self.allocated_now
            .load(Ordering::Relaxed)
            .try_into()
            .unwrap_or(0)
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for CountingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocated_now
            .fetch_add(layout.size() as isize, Ordering::Relaxed);
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocated_now
            .fetch_sub(layout.size() as isize, Ordering::Relaxed);
        self.inner.dealloc(ptr, layout);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.allocated_now
            .fetch_add(layout.size() as isize, Ordering::Relaxed);
        self.inner.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.allocated_now.fetch_add(
            new_size as isize - layout.size() as isize,
            Ordering::Relaxed,
        );
        self.inner.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
pub static ALLOCATOR: CountingAllocator<System> = CountingAllocator::new(System);