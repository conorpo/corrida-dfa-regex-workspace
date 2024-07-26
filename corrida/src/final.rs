use std::{alloc::{AllocError, Allocator, Global, GlobalAlloc, Layout}, cell::Cell, marker::{PhantomData, PhantomPinned}, mem::{self, MaybeUninit}, ops::Index, ptr::NonNull, slice::{self, from_raw_parts_mut}};

enum Assert<const CHECK: bool> {}
trait IsTrue {}
impl IsTrue for Assert<true> {}

struct Block
{
    data:[u8; BLOCK_DATA_SIZE],
    prev: Option<Box<Block>>
}

const BLOCK_DATA_SIZE: usize = 8192;

impl Block
{
    fn new(prev: Option<Box<Block>>) -> Box<Block> {
        //SAFETY, we are about to immedieatly initialize prev, data will always be initalized
        let mut new_block = unsafe { 
            println!("BLOCK");
            Box::<Block>::new_uninit().assume_init()
        };

        new_block.prev = prev;
        new_block
    }
}


/// One time use bump allocator.
/// Useful for many allocations of the same type
/// Self references are easy because all lifetimes are garunteed to live for the whole arena.
pub struct Corrida 
where
{
    arena: Cell<Option<Box<Block>>>,
    cur_offset: Cell<usize>,
    _boo: PhantomData<Block>
}



impl Corrida
{
    pub fn new() -> Self {
        Self {
            arena:Cell::new(Some(Block::new(None))),
            cur_offset: Cell::new(0),
            _boo: PhantomData
        }
    }

    pub fn alloc<F>(&self, fighter: F) -> &mut F
    {
        let mut cur_offset = self.cur_offset.get();

        let mut block = if cur_offset + size_of::<F>() > BLOCK_DATA_SIZE {
            let old = self.arena.take().unwrap();
            cur_offset = 0;
            
            Block::new(Some(old))   
        } else {
            self.arena.take().unwrap()
        };

        // SAFETY: We are creating a reference to unintialized memory, but we are instantly initializing it.
        // We are also incrementing current offset to make sure we dont reallocate memory.
        let slot = unsafe { &mut *(block.data.as_mut_ptr().add(cur_offset) as *mut F) };
        *slot = fighter;

        self.cur_offset.set(cur_offset + size_of::<F>());
        self.arena.set(Some(block));

        slot
    }
}


#[cfg(test)]
mod test {
    use super::Corrida;

    #[test]
    fn test_isolated_arena() {
        let arena = Corrida::new();
        {
            let a = arena.alloc(1);
            let b = arena.alloc(2);
            let c = arena.alloc(3);
            assert_eq!(*a, 1);
            assert_eq!(*b, 2);
            assert_eq!(*c, 3);
        }
    }

    // Well its slower than General Allocator...
    #[test]
    fn test_large() {
        use std::time::*;
        // Each fighter is 4*16, 64 bytes
        let start = Instant::now();
        let arena = Corrida::new();
        for i in 0..5_000_000 {
            if i % 1_000_000 == 0 {
                println!("{i}");
            }
            let _my_ref = arena.alloc([i,i,i,i,i,i,i,i,i,i,i,i,i,i,i,i]);
        }
        dbg!(start.elapsed());
        assert!(start.elapsed() < Duration::from_millis(500))
    }

}