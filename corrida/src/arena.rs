// Don't want to implement the Allocator trait, because that expects tos support allocating arbitrary blocks of data. Our allocator will only support values of the same type.

use std::cell::UnsafeCell;
use std::cell::Cell;
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Error, Display};

const BLOCK_SIZE : usize = 1024;

struct ArenaVec<T> (UnsafeCell<Vec<T>>);

impl<T> ArenaVec<T> {
    fn new() -> Self {
        Self (UnsafeCell::new(Vec::new()))
    }

    fn with_capacity(capacity: usize) -> Self {
        Self (UnsafeCell::new(Vec::with_capacity(capacity)))
    }
}



pub trait ArenaFighter<'arena> {
    // fn insertion_behaviour(&self, arena: &'arena SingleUseArena<'arena,T, Self>);
    fn new() -> Self {
        
    }
}

struct Fighter<T> {
    ele:T
}

impl<T> ArenaFighter<'_> for Fighter<T> {}
impl<T> From<T> for Fighter<T> {
    fn from(ele:T) -> Self {
        Self { ele }
    }
}
impl<T: Display> Debug for Fighter<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("{}", self.ele))
    }
}

// impl<T,'arena> ArenaFighter for Fighter<T> {
//     fn insertion_behaviour(&self, &mut Arena< ) {
        
//     }
// }

struct EdgeFighter {}
struct DirectedFighter<'arena, T> {
    ele: T,
    out: ArenaVec<&'arena Self>,
    r#in: ArenaVec<&'arena Self>
}

impl<T> ArenaFighter<'_> for DirectedFighter<'_, T> {}

struct SingleUseArena<'arena, F:ArenaFighter<'arena>> {
    blocks: Vec<ArenaVec<F>>,
    _boo: PhantomData<&'arena F>
}

impl<'arena, F:ArenaFighter<'arena>> SingleUseArena<'arena, F> {
    fn new() -> Self {
        Self {
            blocks: Vec::new(),
            _boo: PhantomData
        }
    }
}

struct Arena<'arena, F: ArenaFighter<'arena>> {
    arena: SingleUseArena<'arena, F>,
    idx: (usize, usize),
    //_boo: PhantomData<T>
}

impl<'arena, T> Arena<'arena, Fighter<T>> {
    pub fn new() -> Self {
        Arena {
            arena: SingleUseArena::<Fighter<T>>::new(),
            idx: (0,0)
        }
    }

    /// Insert, Add, Append
    pub fn push<IF: Into<Fighter<T>>>(&mut self, fighter: IF) -> &'arena Fighter<T> {
        let new_fighter: Fighter<T> = fighter.into();

        let (block_index, slot_index) = self.idx;

        let block = match self.arena.blocks.len() < block_index {
            true => &self.arena.blocks[block_index],
            false => {
                self.arena.blocks.push(ArenaVec::with_capacity(BLOCK_SIZE));
                &self.arena.blocks[block_index]
            }
        };

        assert_eq!(block);

        let slot = unsafe {&mut (*block.0.get())[slot_index] };
        *slot = new_fighter;

        if slot_index == BLOCK_SIZE - 1 {
            self.idx = (block_index + 1, 0);
        } else {
            self.idx = (block_index, slot_index + 1);
        }
        
        slot
    }

    /// Destroy, Clear
    pub fn free(&mut self) {
        self.idx = (0,0);
    
    }

    pub fn len(&self) -> usize {
        self.idx.0 * BLOCK_SIZE + self.idx.1
    }
}

#[cfg(test)]
mod test{
    use super::{Fighter, Arena};
    
    #[test]
    fn test_reusability() {
        let mut arena = Arena::<Fighter<u32>>::new();
        {
            let x = arena.push(1);
            let y = arena.push(2);
            let z = arena.push(3);

            dbg!(x,y,z, arena.len());
        }
        let f = arena.push(4);
        arena.free();
        dbg!(f);
        dbg!(arena.len());
        {
            let x = arena.push(4);
            let y = arena.push(5);
            let z = arena.push(6);

            dbg!(x,y,z, arena.len());
        }
    }
}