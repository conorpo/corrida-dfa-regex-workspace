use std::ptr::NonNull;
use std::cell::{Cell,UnsafeCell};
use std::marker::PhantomData;
use std::fmt::{Debug, Display, Formatter};

use crate::arena::ArenaFighter;

const BLOCK_SIZE:usize = 1024;

trait Fighter<T>: std::fmt::Debug {
    fn new(ele:T) -> Self;
}



/// Using the Arena with this Fighter allows you just to allocate values, not references between them.
struct IsolatedFighter<T> {
    ele: T,
    //_boo: PhantomData<&'arena Arena<IsolatedFighter<'arena, T>>>
}

impl<T: Display> Debug for IsolatedFighter<T> {
    fn fmt(&self, )
}

impl<T> Fighter<T> for IsolatedFighter<T> {
    fn new(ele: T) -> Self {
        Self { 
            ele,
           // _boo: PhantomData
        }
    }
}

// Using the Arena with this Fighters allows you to create undirected graphs, with shared mutable references between elements
struct ConnectedFighter<T> {
    ele: T,
    edges: Vec<NonNull<ConnectedFighter<T>>>
}

impl<T> Fighter<T> for ConnectedFighter<T> {
    fn new(ele: T) -> Self {
        Self {
            ele,
            edges: Vec::new()
        }
    }
}

// Using the Arena with this Fighter allows you to create directed graph (eg represent dependencies), it evens upports cyclic references.
struct DirectedFighter<T> {
    ele: T,
    outgoing: Vec<NonNull<DirectedFighter<T>>>,
    incoming:Vec<NonNull<DirectedFighter<T>>>,
}

impl<T> Fighter<T> for DirectedFighter<T> {
    fn new(ele: T) -> Self {
        Self {
            ele,
            outgoing: Vec::new(),
            incoming: Vec::new()
        }
    }
}


type BlockLink<F> = Option<NonNull<Block<F>>>;
struct Block<F> {
    prev: BlockLink<F>,
    data: Vec<F>
}

impl<F> Block<F> {
    fn new(prev: BlockLink<F>) -> Self {
        Self {
            prev: None,
            data: Vec::new()
        }
    }
}

struct Arena<F> {
    cur_block: UnsafeCell<NonNull<Block<F>>>,
    idx: Cell<(usize, usize)>,
    //_boo: PhantomData<& F>
}

impl<F> Arena<F> {
    fn new() -> Self {
        Self {
            cur_block: UnsafeCell::new(Self::new_block(None)),
            idx: Cell::new((0,0))
        }
    }

    fn new_block(prev: Option<NonNull<Block<F>>>) -> NonNull<Block<F>> {
        NonNull::new(Box::into_raw(Box::new(
            Block::new(prev)
        ))).unwrap()
    }
}

impl<'arena, T> Arena<IsolatedFighter<T>> {
    fn alloc(&self, ele: T) -> &mut IsolatedFighter<T>{
        let (_,slot_index) = self.idx.get();

        let block_ptr = if slot_index == BLOCK_SIZE {
            let cur = self.cur_block.get();
            let new_block = Self::new_block(Some(*cur));
            *cur = new_block;
            new_block.as_ptr()
        } else {
            (*self.cur_block.get()).as_ptr()
        };


        let new_figter = IsolatedFighter::new(ele);

        let slot = (*block_ptr).data.as_mut_ptr().add(slot_index);

        *slot = new_figter;

        &mut *slot
    } 

    fn free(self) {

    }
}

impl<T> Arena<ConnectedFighter<T>> {
    fn alloc(&self, ele: T, edges: Vec<&ConnectedFighter<T>>) -> &mut ConnectedFighter<T> {
        
        
        todo!()
    }
}

mod test {
    use super::{Arena, IsolatedFighter};

    #[test]
    fn test_isolated_arena() {
        let arena = Arena::<IsolatedFighter<u32>>::new();

        let x = arena.alloc(1);
        let y = arena.alloc(2);
        let z = arena.alloc(3);

        arena.free();

        let x = arena.alloc(4);
        let y = arena.alloc(5);
        let z = arena.alloc(6);

        dbg!(x,y,z,arena.len());
    }
}