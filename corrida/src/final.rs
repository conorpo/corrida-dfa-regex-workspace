use std::{alloc::{Allocator, Global, GlobalAlloc, Layout}, cell::Cell, marker::{PhantomData, PhantomPinned}, mem, ops::Index, ptr::NonNull};

enum Assert<const CHECK: bool> {}
trait IsTrue {}
impl IsTrue for Assert<true> {}

struct Block<const ALIGN: usize> 
{
    prev: Option<Box<Block<ALIGN>>>,
    data: NonNull<[u8]>
}

const BLOCK_SIZE: usize = 8192;

impl<const ALIGN: usize> Block<ALIGN> 
{
    const BLOCK_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(BLOCK_SIZE, ALIGN) };

    fn new(prev: Option<Box<Block<ALIGN>>>) -> Self {
        let data = (Global {}).allocate(Self::BLOCK_LAYOUT).unwrap();
        
        Self {
            prev,
            data
        }
    }
}

trait IsBlock {}
impl<const ALIGN: usize> IsBlock for Block<ALIGN> 
{}

/// One time use bump allocator.
/// Useful for many allocations of the same type
/// Self references are easy because all lifetimes are garunteed to live for the whole arena.
pub struct Corrida<> {
    aligned_arenas: Vec<NonNull<()>>,
    _boo: PhantomData<Block<2>>
}

impl<T0: Sized, T1, T2, T3, T4, T5, T6, T7, T8, T9> Corrida 
where
    T0: Sized
{
    pub fn new<const >() -> Self {
        // assert!(aligns.len() <= 10);

        // let arenas = Vec::new();
        // let i = 2;
        // arenas.push(NonNull::from());
        // Self {
        //     aligned_arenas: 

        // }
    }

    pub fn alloc<F>(fighter: F) where Assert<{F == T0}>: IsTrue {
    }
    pub fn alloc<F>(fighter: F) where Assert<{F == T1}>: IsTrue {
        
    }

    pub fn _phantom_constraint() where
        T0: Copy
    {

    }
}


// macro_rules! impl_blocktuple {
//     ($) => {
        
//     };
// }

// Im so sorry
impl<const A1: usize> Corrida for (Box<Block<A1>>,) {
    const A1: usize = A1;
}



// impl<const A1: usize, const A2: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize, const A5: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>, Box<Block<A5>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize, const A5: usize, const A6: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>, Box<Block<A5>>, Box<Block<A6>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize, const A5: usize, const A6: usize, const A7: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>, Box<Block<A5>>, Box<Block<A6>>, Box<Block<A7>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize, const A5: usize, const A6: usize, const A7: usize, const A8: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>, Box<Block<A5>>, Box<Block<A6>>, Box<Block<A7>>, Box<Block<A8>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize, const A5: usize, const A6: usize, const A7: usize, const A8: usize, const A9: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>, Box<Block<A5>>, Box<Block<A6>>, Box<Block<A7>>, Box<Block<A8>>, Box<Block<A9>>) {}
// impl<const A1: usize, const A2: usize, const A3: usize, const A4: usize, const A5: usize, const A6: usize, const A7: usize, const A8: usize, const A9: usize, const A10: usize> Corrida for (Box<Block<A1>>, Box<Block<A2>>, Box<Block<A3>>, Box<Block<A4>>, Box<Block<A5>>, Box<Block<A6>>, Box<Block<A7>>, Box<Block<A8>>, Box<Block<A9>>, Box<Block<A10>>) {}

macro_rules! corrida {
    ($T0:ty $(,)?) => {
        Arena::<(Box<Block<{ std::mem::align_of::< $T0 >() }>>,)>::new();
    };
    ($T0:ty, $($T:ty),*) => {
        Arena::new((
            Box::new(Block::<{ std::mem::align_of::< $T0 >() }>::new(None)),
            $(Box::new(Block::<{ std::mem::align_of::< $T >() }>::new(None))),*   
        ))
    };
}



// pub struct Arena<const = (20, 30)> {
//     aligned_arenas: T
// }

impl<T: Corrida> Arena<T> {
    pub fn new(aligned_arenas: T) -> Self {
        Self {
            aligned_arenas
        }
    }



}

pub fn test() {
    let arena = corrida!(u8, u16);

}



// impl<T> Arena {
//     /// Creates a new arena of the fighter type provided in the type parameter, allocates one block to start off with.
//     pub fn from_enum(_enum: T) 
//     where 
//         T: Enum,
//     {
    
//     }

//     // Allocates a block on the heap, and returns a pointer to it.
//     fn new_block(prev: Option<NonNull<Block<T>>>) -> NonNull<Block<T>> {
//         NonNull::new(Box::into_raw(Box::new(
//             Block::new(prev)
//         ))).unwrap()
//     }

//     /// Returns the amount of fighters in the arena
//     pub fn len(&self) -> usize { 
//         let idx = self.idx.get();
//         idx.0 * BLOCK_SIZE + idx.1
//     }

//     /// True until the arena is first fighter is added
//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }

//     /// Allocates a type T in the current slot and returns an exlusive
//     /// references to it.
//     pub fn alloc(&self, fighter: T) -> &'a mut T {
//         unsafe {
//             let slot = self.alloc_core();
//             ptr::write(slot.as_ptr(), fighter);
//             &mut *slot.as_ptr()
//         }
//     }

//     // Gets a pointer to the current slot, then increments the index
//     // Handles creating a new block if needed.
//     unsafe fn alloc_core(&self) -> NonNull<T> {
//         let (mut block_index, mut slot_index) = self.idx.get();

//         let block_ptr = if slot_index == BLOCK_SIZE {
//             block_index += 1;
//             slot_index = 0;

//             let prev = self.cur_block.get();
//             let new_block = Self::new_block(Some(prev));
//             self.cur_block.set(new_block);

//             new_block.as_ptr()
//         } else {
//             (self.cur_block.get()).as_ptr()
//         };

//         let slot = unsafe {
//             NonNull::new((*block_ptr).data.as_mut_ptr().add(slot_index)).unwrap()
//         };

//         slot_index += 1;
//         self.idx.set((block_index, slot_index));

//         slot
//     }
// }

// impl<T> Drop for Arena<T> {
//     fn drop(&mut self) {
//         unsafe {
//             let mut cur_block_opt = Some(self.cur_block.get());
//             while let Some(cur_block_ptr) = cur_block_opt {
//                 let owned = Box::from_raw(cur_block_ptr.as_ptr());
//                 cur_block_opt = owned.prev;
//             }   

//             self.idx.take(); //take that shit before we leave
//         }
//     }
// }

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