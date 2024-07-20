//! An Arena implemenation using Rust's Allocator trait, allowing it to be as an allocator for other structs.
use std::alloc::{Layout, alloc, AllocError, Allocator};
use std::cell::Cell;
use std::{mem, ptr, slice};
use std::ptr::{NonNull, from_raw_parts_mut};

/// Bump allocates data into mostly concurrent blocks for the purposes of fewer allocations and better cache locality.
/// Dropping data is not allowed, instead all values in the Arena have the same lifetime, and are only dropped when the Arena is dropped.

const BLOCK_DATA_SIZE_BYTES: usize = 1024;
const BLOCK_SIZE_BYTES: usize = BLOCK_DATA_SIZE_BYTES + mem::size_of::<Cell<NonNull<u8>>>();
struct Block {
    prev: Option<NonNull<Block>>,
    data: [u8; BLOCK_DATA_SIZE_BYTES]
}

impl Block {
    fn new(prev: Option<NonNull<Block>>) -> NonNull<Block> {
        unsafe {
            let block_ptr = alloc(Layout::from_size_align(
                BLOCK_SIZE_BYTES,
                1
            ).unwrap()) as *mut ();

            ptr::write(block_ptr as *mut Option<NonNull<Block>>, prev);

            NonNull::new_unchecked(from_raw_parts_mut::<Block>(
                block_ptr,
                ()
            ))
        }
    }
}



pub struct ArenaAllocator {
    last_
    cur_index: Cell<usize>
}

impl ArenaAllocator {
    /// Creates a new arena with one block to start off with.
    pub fn new() -> Self {
        let block_non_null = Block::new(None);

        unsafe {
            Self {
                cur_block: Cell::new(block_non_null),
                cur_index: Cell::new(NonNull::new_unchecked(block_non_null.get_ptr() as *mut u8))
            }
        }
    }
}

unsafe impl Allocator for ArenaAllocator {
    fn allocate(&self, layout: Layout<>) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let cur_index = self.cur_index.get();
            let next_index = cur_index + layout.size();
            (*self.cur_block.get().as_ptr()).data[cur_index..next_index]

            self.cur_index.set(next_index);
        }
        todo!();
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        todo!();
    }
}

#[cfg(test)]
mod test {
    #[test]
    use super::ArenaAllocator;
    fn test_basics() {
        let my_alloc = ArenaAllocator::new();

        let x = Box::new_in(32, my_alloc).as_mut();
        let y = Box::new_in(32, my_alloc).as_mut();
        let z = Box::new_in(32, my_alloc).as_mut();
    
    }
}