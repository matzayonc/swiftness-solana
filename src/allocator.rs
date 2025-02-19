// Sourced from: https://github.com/blockworks-foundation/mango-v4/blob/91d61ad33c42e07374f7782b056b55f27eb1740a/programs/mango-v4/src/allocator.rs
#![allow(dead_code)]

use std::alloc::{GlobalAlloc, Layout};

#[cfg(not(feature = "no-entrypoint"))]
#[global_allocator]
pub static ALLOCATOR: BumpAllocator = BumpAllocator {};

pub fn heap_used() -> usize {
    #[cfg(not(feature = "no-entrypoint"))]
    return ALLOCATOR.used();

    #[cfg(feature = "no-entrypoint")]
    return 0;
}

/// Custom bump allocator for on-chain operations
///
/// The default allocator is also a bump one, but grows from a fixed
/// HEAP_START + 32kb downwards and has no way of making use of extra
/// heap space requested for the transaction.
///
/// This implementation starts at HEAP_START and grows upward, producing
/// a segfault once out of available heap memory.
pub struct BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let heap_start = solana_program::entrypoint::HEAP_START_ADDRESS as usize;
        let pos_ptr = heap_start as *mut usize;

        let mut pos = *pos_ptr;
        if pos == 0 {
            // First time, override the current position to be just past the location
            // where the current heap position is stored.
            pos = heap_start + 8;
        }

        // The result address needs to be aligned to layout.align(),
        // which is guaranteed to be a power of two.
        // Find the first address >=pos that has the required alignment.
        // Wrapping ops are used for performance.
        let mask = layout.align().wrapping_sub(1);
        let begin = pos.wrapping_add(mask) & (!mask);

        // Update allocator state
        let end = begin.checked_add(layout.size()).unwrap();
        *pos_ptr = end;

        // Write a byte to trigger heap overflow errors early
        let end_ptr = end as *mut u8;
        *end_ptr = 0;

        begin as *mut u8
    }
    #[inline]
    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        // I'm a bump allocator, I don't free
    }
}

impl BumpAllocator {
    #[inline]
    pub fn used(&self) -> usize {
        let heap_start = solana_program::entrypoint::HEAP_START_ADDRESS as usize;
        unsafe {
            let pos_ptr = heap_start as *mut usize;

            let pos = *pos_ptr;
            if pos == 0 {
                return 0;
            }
            return pos - heap_start;
        }
    }
}
