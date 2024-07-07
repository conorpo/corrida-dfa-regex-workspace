// Don't want to implement the Allocator trait, because that expects tos support allocating arbitrary blocks of data. Our allocator will only support values of the same type.

use std::{cell::{Cell, RefCell, UnsafeCell}, marker::PhantomData};
use std::fmt::Display;
struct Directed<'arena,T: Display> {
    ele: T,
    refs     : UnsafeCell<Vec<&'arena Directed<'arena, T>>>, 
    back_refs: UnsafeCell<Vec<&'arena Directed<'arena, T>>>
}

impl<'arena, T: Display> Directed<'arena, T> {
    pub fn new(ele: T, refs: Vec<&'arena Directed<'arena, T>>, back_refs: Vec<&'arena Directed<'arena, T>>) -> Self {
        Self {
            ele,
            refs: UnsafeCell::new(refs),
            back_refs: UnsafeCell::new(back_refs)
        }
    }
}

impl<'arena,T: Display> From<T> for Directed<'arena, T> {
    fn from(ele: T) -> Self {
        Self {
            ele,
            refs: UnsafeCell::new(Vec::new()),
            back_refs: UnsafeCell::new(Vec::new())
        }
    }
}

impl<'arena, T: Display> std::fmt::Debug for Directed<'arena, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.ele))
    }
}

trait ArenaFighter<'arena> {
    fn insertion_behavior(&'arena self);
}

impl<'arena, T: Display> ArenaFighter<'arena> for Directed<'arena, T> {
    fn insertion_behavior(&'arena self) {
        unsafe {
            for forward_ref in (*self.refs.get()).iter() {
                (*forward_ref.back_refs.get()).push(self);
            }
            for back_ref in (*self.back_refs.get()).iter() {
                (*back_ref.refs.get()).push(self);
            }
        }
        
    }
}


struct Arena<'arena, F: ArenaFighter<'arena>> {
    blocks: Vec<Vec<F>>,
    idx: (usize, usize),
    _boo: PhantomData<&'arena u32>
}

const BLOCK_SIZE:usize = 1024;

impl<'arena, F: ArenaFighter<'arena>> Arena<'arena, F> {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            idx: (0,0),
            _boo: PhantomData
        }
    }

    /// Insert, Add, Append
    fn push(&mut self, ele: F) -> &'arena F {
        let new_fighter = ele;

        let (block_index, slot_index) = self.idx;

        if self.blocks.len() <= block_index {
            self.blocks.push(Vec::with_capacity(BLOCK_SIZE));
        }

        assert!(self.blocks.len() > block_index);

        let slot = &mut (self.blocks[block_index][slot_index]);

        if slot_index == BLOCK_SIZE - 1 {
            self.idx = (block_index + 1, 0);
        } else {
            self.idx = (block_index, slot_index + 1);
        }

        *slot = new_fighter;

        //(&*slot).insertion_behavior();

        &*slot
    }

    fn free(self) -> Self {
        Self {
            blocks: self.blocks,
            idx: (0,0),
            _boo: PhantomData
        }    
    }

    fn len(&self) -> usize {
        self.idx.0 * BLOCK_SIZE + self.idx.1
    }
}

mod test {
    use super::Arena;

    #[cfg(test)]
    fn test_reusability() {
        use super::Directed;

        let mut arena = Arena::<Directed<u32>>::new();
        {
            let x = arena.push(1.into());
            let y = arena.push(2.into());
            let z = arena.push(3.into());
            dbg!(x,y,z,arena.len());
        }
        let mut arena = arena.free();
        {
            let x = arena.push(1.into());
            let y = arena.push(2.into());
            let z = arena.push(3.into());

            dbg!(x,y,z);
        }
    }
}

