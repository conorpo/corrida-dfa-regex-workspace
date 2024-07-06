// Don't want to implement the Allocator trait, because that expects tos support allocating arbitrary blocks of data. Our allocator will only support values of the same type.

use std::cell::UnsafeCell;
use std::cell::Cell;

struct ArenaFighter<'arena,T> {
    ele: T,
    refs     : Vec<&'arena ArenaFighter<'arena, T>>, 
    back_refs: Vec<&'arena ArenaFighter<'arena, T>>
}

struct Arena<'arena, T> {
    fighters: UnsafeCell<Vec<ArenaFighter<'arena,T>>>,
    len: Cell<usize>
}

impl<'arena, T> Arena<'arena, T> {
    pub fn new() -> Arena<'arena,T> {
        Arena {
            fighters: UnsafeCell::new(Vec::new()),
            len: 0
        }
    }

    /// Insert, Add, Append
    pub fn push(&'arena self, ele: T) -> &'_ ArenaFighter<'_, T> {
        let new_fighter = ArenaFighter {
            ele,
            refs: Vec::new(),
            back_refs: Vec::new()
        };

        unsafe {
            let fighters = self.fighters.get();
            (*fighters).push(new_fighter);
            
            &(*fighters)[(*fighters).len() - 1]
        }
    }

    /// Destroy, Clear
    pub fn free(&'arena self) {
        self.len.set(0);
    
    }
}

