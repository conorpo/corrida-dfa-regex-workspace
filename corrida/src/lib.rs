#![warn(missing_docs)]
#![feature(allocator_api)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(inline_const)]
#![feature(adt_const_params)]
//#![feature(allocate_api)]

//! Typed Bump Allocator
//! 
//! - Useful as for cache-friendly accesses of collection.
//! 

pub mod r#final;

pub mod toy_structures;

use std::ptr::{self, NonNull};
use std::cell::Cell;

type BlockLink<F> = Option<NonNull<Block<F>>>;
struct Block<F> {
    prev: BlockLink<F>,
    data: Vec<F>
}

const BLOCK_SIZE: usize = 1024;

impl<T> Block<T> {

    fn new(prev: BlockLink<T>) -> Self {
        Self {
            prev,
            data: Vec::with_capacity(BLOCK_SIZE)
        }
    }
}

/// One time use bump allocator.
/// Useful for many allocations of the same type
/// Self references are easy because all lifetimes are garunteed to live for the whole arena.n
pub struct Arena<T> {
    cur_block: Cell<NonNull<Block<T>>>,
    idx: Cell<(usize, usize)>,
    //_boo: PhantomData<& F>
}

impl<'a, T> Arena<T> {
    /// Creates a new arena of the fighter type provided in the type parameter, allocates one block to start off with.
    pub fn new() -> Self {
        Self {
            cur_block: Cell::new(Self::new_block(None)),
            idx: Cell::new((0,0))
        }
    }

    // Allocates a block on the heap, and returns a pointer to it.
    fn new_block(prev: Option<NonNull<Block<T>>>) -> NonNull<Block<T>> {
        NonNull::new(Box::into_raw(Box::new(
            Block::new(prev)
        ))).unwrap()
    }

    /// Returns the amount of fighters in the arena
    pub fn len(&self) -> usize { 
        let idx = self.idx.get();
        idx.0 * BLOCK_SIZE + idx.1
    }

    /// True until the arena is first fighter is added
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Allocates a type T in the current slot and returns an exlusive
    /// references to it.
    pub fn alloc(&self, fighter: T) -> &'a mut T {
        unsafe {
            let slot = self.alloc_core();
            ptr::write(slot.as_ptr(), fighter);
            &mut *slot.as_ptr()
        }
    }

    // Gets a pointer to the current slot, then increments the index
    // Handles creating a new block if needed.
    unsafe fn alloc_core(&self) -> NonNull<T> {
        let (mut block_index, mut slot_index) = self.idx.get();

        let block_ptr = if slot_index == BLOCK_SIZE {
            block_index += 1;
            slot_index = 0;

            let prev = self.cur_block.get();
            let new_block = Self::new_block(Some(prev));
            self.cur_block.set(new_block);

            new_block.as_ptr()
        } else {
            (self.cur_block.get()).as_ptr()
        };

        let slot = unsafe {
            NonNull::new((*block_ptr).data.as_mut_ptr().add(slot_index)).unwrap()
        };

        slot_index += 1;
        self.idx.set((block_index, slot_index));

        slot
    }
}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        unsafe {
            let mut cur_block_opt = Some(self.cur_block.get());
            while let Some(cur_block_ptr) = cur_block_opt {
                let owned = Box::from_raw(cur_block_ptr.as_ptr());
                cur_block_opt = owned.prev;
            }   

            self.idx.take(); //take that shit before we leave
        }
    }
}

#[cfg(test)]
mod test {
    use super::Arena;

    #[test]
    fn test_isolated_arena() {
        let arena = Arena::<u32>::new();
        {
            let a = arena.alloc(1);
            let b = arena.alloc(2);
            let c = arena.alloc(3);
            assert_eq!(*a, 1);
            assert_eq!(*b, 2);
            assert_eq!(*c, 3);
            assert_eq!(arena.len(),3);
        }
    }

    // Well its slower than General Allocator...
    #[test]
    fn test_large() {
        use std::time::*;
        // Each fighter is 4*16, 64 bytes
        let start = Instant::now();
        let arena = Arena::<[u32;16]>::new();
        for i in 0..5_000_000 {
            let _my_ref = arena.alloc([i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
        }
        dbg!(start.elapsed());
        assert!(start.elapsed() < Duration::from_millis(500))
    }

}