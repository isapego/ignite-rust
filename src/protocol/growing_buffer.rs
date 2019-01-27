use std::alloc;
use std::mem;
use std::ptr;
use std::cmp;
use std::cell::Cell;

/// Default reserved memory capacity
const DEFAULT_CAPACITY: usize = 1024;

/// Max capacity of the underlying memory
const MAX_CAPACITY: usize = ::std::i32::MAX as usize;

/// Writing stream abstraction
pub struct GrowingBuffer {
    mem: ptr::NonNull<u8>,
    len: Cell<usize>,
}

impl GrowingBuffer {
    /// Make new instance
    pub fn new() -> Self {
        Self {
            mem: ptr::NonNull::dangling(),
            len: Cell::new(0),
        }
    }

    /// Get filled memory in specified range.
    pub fn into_memory(self, len: usize) -> Box<[u8]> {
        let cap = self.len.get();
        assert!(len <= cap, "Invalid length");

        unsafe {
            let res = Vec::from_raw_parts(self.mem.as_ptr(), len, cap).into_boxed_slice();

            mem::forget(self);

            res
        }
    }

    /// Ensure that capacity is enough to fit the required number of bytes
    pub fn ensure_len(&self, need_len: usize) {
        let old_len = self.len.get();
        if old_len > need_len {
            return;
        }

        assert!(need_len <= MAX_CAPACITY, "Capacity overflow");

        let mut new_len = cmp::max(DEFAULT_CAPACITY, old_len);

        while new_len < need_len {
            new_len *= 2;

            if new_len >= MAX_CAPACITY {
                new_len = MAX_CAPACITY;

                break;
            }
        }

        self.realloc_mem(new_len);
    }

    /// Reallocate internal memory buffer.
    fn realloc_mem(&self, new_len: usize) {
        let old_len = self.len.get();

        assert!(new_len > old_len, "Buffer may only grow!");

        unsafe {
            let new_mem = if old_len == 0 {
                let layout = Self::layout_for_len(new_len);

                alloc::alloc(layout)
            } else {
                let layout = Self::layout_for_len(old_len);

                alloc::realloc(self.mut_ptr(), layout, new_len)
            };

            assert!(!new_mem.is_null(), "Out of memory");

            let me = self as *const Self as *mut Self;
            (*me).mem = ptr::NonNull::new_unchecked(new_mem);

            self.len.set(new_len);
        }
    }

    /// Get layout for the memory of the specified length
    /// Unchecked
    #[inline(always)]
    unsafe fn layout_for_len(len: usize) -> alloc::Layout {
        alloc::Layout::from_size_align_unchecked(len, mem::align_of::<u8>())
    }

    /// Get mutable pointer
    #[inline(always)]
    pub fn mut_ptr(&self) -> *mut u8 {
        unsafe {
            self.mem.as_ref() as *const u8 as *mut u8
        }
    }
}

/// Implementing drop for GrowingBuffer to deal with memory deallocation
impl Drop for GrowingBuffer {
    fn drop(&mut self) {
        let len = self.len.get();
        if len != 0 {
            unsafe {
                let layout = Self::layout_for_len(len);

                alloc::dealloc(self.mut_ptr(), layout);
            }
        }
    }
}
