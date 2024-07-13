use std::marker::PhantomData;
use std::ptr::{self, NonNull};
use std::cell::{Cell,UnsafeCell};
//use std::marker::PhantomData;
use std::fmt::{Debug, Display, Formatter, Error};

const BLOCK_SIZE:usize = 1024;

type BlockLink<F> = Option<NonNull<Block<F>>>;
pub struct Block<T> {
    prev: BlockLink<T>,
    data: Vec<T>
}

impl<T> Block<T> {
    fn new(prev: BlockLink<T>) -> Self {
        Self {
            prev,
            data: Vec::with_capacity(BLOCK_SIZE)
        }
    }
}


pub struct Arena<T> {
    cur_block: Cell<NonNull<Block<T>>>,
    idx: Cell<(usize, usize)>,
    //_boo: PhantomData<& F>
}

impl<T> Arena<T> {
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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn alloc(&self, fighter: T) -> &mut T {
        unsafe {
            let slot = self.alloc_core();
            ptr::write(slot.as_ptr(), fighter);
            &mut *slot.as_ptr()
        }
    }


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

    /// Resets the Arena for reuse, deallocating all blocks but the first one, and setting the index back to (0,0). Borrows self exclusively, so all borrows must end before this is called.
    pub fn reset(&mut self) {
        unsafe {
            let mut cur_block_ptr = self.cur_block.get().as_ptr();
            while let Some(prev) = (*cur_block_ptr).prev.take() {
                let _ = Box::from_raw(cur_block_ptr);
                cur_block_ptr = prev.as_ptr();
            }
            
            self.cur_block.set(NonNull::new(cur_block_ptr).unwrap());
            self.idx.set((0,0));
        
        }
    }

    // pub fn iter(&mut self) -> Iter<'_, T> {
    //     if self.is_empty() {
    //         Iter {
    //             front: None,
    //             back: None,
    //             len: 0,
    //             _boo: PhantomData
    //         }
    //     } else {



    //         Iter {
    //             front
    //         }
    //     }

    //     Iter::<'_, T> {
    //         front: 
    //     }
    //}

}

impl<T> Drop for Arena<T> {
    fn drop(&mut self) {
        self.reset();

        unsafe { 
            let _ = Box::from_raw(self.cur_block.get().as_ptr());
            self.idx.take(); //take that shit before we leave
        };
    }
}

#[cfg(test)]
mod test {
    use super::{Arena};

    #[test]
    fn test_isolated_arena() {
        let mut arena = Arena::<u32>::new();
        {
            let a = arena.alloc(1);
            let b = arena.alloc(2);
            let c = arena.alloc(3);
            assert_eq!(*a, 1);
            assert_eq!(*b, 2);
            assert_eq!(*c, 3);
            assert_eq!(arena.len(),3);
        }
        arena.reset();
        assert_eq!(arena.len(), 0);
        {
            let a = arena.alloc(4);
            let b = arena.alloc(5);
            let c = arena.alloc(6);
            *a = 0;
            *c = 10;
            assert_eq!(*a, 0);
            assert_eq!(*b, 5);
            assert_eq!(*c, 10);
            assert_eq!(arena.len(),3);
        }
    }
}