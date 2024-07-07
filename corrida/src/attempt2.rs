use std::ptr::NonNull;
use std::cell::{Cell,UnsafeCell};
//use std::marker::PhantomData;
use std::fmt::{Debug, Display, Formatter, Error};

const BLOCK_SIZE:usize = 1024;

trait Fighter<T>{
    fn new(ele:T) -> Self;
}

/// Using the Arena with this Fighter allows you just to allocate values, not references between them.
pub struct IsolatedFighter<T> {
    pub ele: T,
    //_boo: PhantomData<&'arena Arena<IsolatedFighter<'arena, T>>>
}

impl<T: Display> Debug for IsolatedFighter<T> {
    fn fmt(&self,  fmt: &mut Formatter) -> Result<(),Error> {
        fmt.write_fmt(format_args!("{}",self.ele))
    }
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
pub struct ConnectedFighter<T> {
    ele: T,
    edges: Vec<NonNull<ConnectedFighter<T>>>
}

// impl<T> ConnectedFighter<T> {
//     fn edges_extend(&mut self, slice: &[ConnectedFighter<T>]) {
//         self.edges.extend(slice.into_iter().map(|fighter_ref| {
//             NonNull::new(fighter_ref as *const ConnectedFighter<T> as *mut ConnectedFighter<T>)
//         }))
//     }
// }

impl<T> Fighter<T> for ConnectedFighter<T> {
    fn new(ele: T) -> Self {
        Self {
            ele,
            edges: Vec::new()
        }
    }
}

// Using the Arena with this Fighter allows you to create directed graph (eg represent dependencies), it evens upports cyclic references.
pub struct DirectedFighter<T> {
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
pub struct Block<F> {
    prev: BlockLink<F>,
    data: Vec<F>
}

impl<F> Block<F> {
    fn new(prev: BlockLink<F>) -> Self {
        Self {
            prev,
            data: Vec::with_capacity(BLOCK_SIZE)
        }
    }
}

pub struct Arena<F> {
    cur_block: UnsafeCell<NonNull<Block<F>>>,
    idx: Cell<(usize, usize)>,
    //_boo: PhantomData<& F>
}

impl<F> Arena<F> {
    pub fn new() -> Self {
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

    /// Returns the amount of fighters in the arena
    pub fn len(&self) -> usize {
        let idx = self.idx.get();
        idx.0 * BLOCK_SIZE + idx.1
    }
}

impl<T> Arena<IsolatedFighter<T>> {
    pub fn alloc(&self, ele: T) -> &mut IsolatedFighter<T>{
        let (mut block_index, mut slot_index) = self.idx.get();

        unsafe {
            let block_ptr = if slot_index == BLOCK_SIZE {
                block_index += 1;
                slot_index = 0;

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
            slot_index += 1;
            self.idx.set((block_index, slot_index));
    
            &mut *slot
        }
    } 
    
    pub fn free(&mut self) {
        self.idx.set((0,0))
    }
}

impl<T> Arena<ConnectedFighter<T>> {
    pub fn alloc(&self, ele: T, edges: Vec<&ConnectedFighter<T>>) -> &mut ConnectedFighter<T> {
        
        
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::{Arena, IsolatedFighter};

    #[test]
    fn test_isolated_arena() {
        let mut arena = Arena::<IsolatedFighter<u32>>::new();
        {
            let a = arena.alloc(1);
            let b = arena.alloc(2);
            let c = arena.alloc(3);
            assert_eq!(a.ele, 1);
            assert_eq!(b.ele, 2);
            assert_eq!(c.ele, 3);
            assert_eq!(arena.len(),3);
        }
        arena.free();
        assert_eq!(arena.len(), 0);
        {
            let a = arena.alloc(4);
            let b = arena.alloc(5);
            let c = arena.alloc(6);
            a.ele = 0;
            c.ele = 10;
            assert_eq!(a.ele, 0);
            assert_eq!(b.ele, 5);
            assert_eq!(c.ele, 10);
            assert_eq!(arena.len(),3);
        }
    }
}