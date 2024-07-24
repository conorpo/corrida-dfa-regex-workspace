
//! An Arena implemenation using Rust's Allocator trait, allowing it to be as an allocator for other structs.

use std::alloc::{Layout, alloc, AllocError, Allocator};
use std::cell::Cell;
use std::rc::Rc;
use std::{mem, ptr, slice};
use std::ptr::{NonNull, from_raw_parts_mut};



/// Bump allocates data into mostly concurrent blocks for the purposes of fewer allocations and better cache locality.
/// Dropping data is not allowed, instead all values in the Arena have the same lifetime, and are only dropped when the Arena is dropped.

const BLOCK_DATA_SIZE_BYTES: usize = 1024;
const BLOCK_SIZE_BYTES: usize = BLOCK_DATA_SIZE_BYTES + mem::size_of::<Cell<NonNull<u8>>>();
struct Block {
    prev: Option<NonNull<Block>>,
    data: [u8; BLOCK_DATA_SIZE_BYTES],
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

#[derive(Clone)]


pub struct ArenaAllocator {
    cur_block: Cell<NonNull<Block>>,
    cur_index: Cell<usize>
}

impl ArenaAllocator {
    /// Creates a new arena with one block to start off with.
    pub fn new() -> Self {
        let block_non_null = Block::new(None);

        Self  {
                cur_block: Cell::new(block_non_null),
                cur_index: Cell::new(0)
            }
    }
}

unsafe impl Allocator for &ArenaAllocator {
    fn allocate(&self, layout: Layout<>) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            // Update cur block and index if needed
            if self.cur_index.get() + layout.size() > BLOCK_DATA_SIZE_BYTES {
                self.cur_block.update(|old_block| {
                    Block::new(Some(old_block))
                });
                self.cur_index.set(0);
            };

            let start = self.cur_block.get().as_ptr().add(
                self.cur_index.get()
            ) as *mut u8;

            let end = start.add(layout.size());

            Ok(NonNull::new_unchecked(
                slice::from_mut_ptr_range(start..end) as *mut [u8])
            )
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        
    }
}

pub struct NfaTest<'a> {
    arena: &'a ArenaAllocator
}

pub struct NfaVertex {
}

impl<'a> NfaTest<'a> {
    pub fn new(arena: &'a ArenaAllocator) -> Self {
        Self {
            arena
        }
    }

    pub fn insert_vertex(&mut self) -> Box<NfaVertex, &ArenaAllocator> {
        unsafe {
            Box::new_in(NfaVertex {}, self.arena)
        }
    }
}

mod test {
    use super::*;

    struct TestFighter<'a> {
        child: Option<&'a TestFighter<'a>>
    }

    impl<'a> TestFighter<'a> {
        pub fn new() -> Self {
            Self {
                child: None
            }
        }
    }

    #[cfg(test)]
    fn test() {
        use std::pin::Pin;

        let arena = ArenaAllocator::new();
        let mut nfa = NfaTest::new(&arena);

        let a = nfa.insert_vertex();
        let b = nfa.insert_vertex();
    }
}