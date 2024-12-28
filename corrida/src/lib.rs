#![warn(missing_docs)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
#![feature(slice_from_ptr_range)]
#![feature(generic_const_exprs)]

//TODO: Check Documenation, Check Safety, Clean up Structure one last Time. Make it faster to do allocate a single object (high priority)

//! Typed Bump Allocator
//!

pub mod basic_structures;

use std::{
    alloc::{AllocError, Allocator, Global, Layout},
    cell::Cell,
    marker::PhantomData,
    ptr::NonNull,
};

pub struct Assert<const CHECK: bool> {}
pub trait IsTrue {}
impl IsTrue for Assert<true> {}

pub const BLOCK_MIN_ALIGN: usize = 128;

#[repr(align(128))]
struct BlockMeta {
    prev: Option<NonNull<BlockMeta>>,
    block_start: NonNull<u8>,
    cur_ptr: NonNull<u8>,
    block_end: NonNull<u8>,
}

impl BlockMeta {
    fn new(prev: Option<NonNull<BlockMeta>>, block_size: usize) -> NonNull<Self>
    {
        //SAFETY,
        unsafe {
            //SAFETY, align is nonzero power of two, size + a little meta is still
            let layout = Layout::from_size_align_unchecked(block_size + size_of::<Self>(), BLOCK_MIN_ALIGN);
            let ptr = Global.allocate(layout).unwrap().as_mut_ptr();

            // SAFETY, ptr will now be at the end of the data, at the start of metadata with exactly the size needed left
            let metadata_nn = { NonNull::new_unchecked(ptr.add(block_size) as *mut Self) };
            metadata_nn.write(BlockMeta {
                prev,
                block_start: NonNull::new_unchecked(ptr),
                cur_ptr: NonNull::new_unchecked(ptr),
                block_end: NonNull::new_unchecked(metadata_nn.as_ptr() as *mut u8),
            });

            metadata_nn
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, AllocError> {
        let align_offset = self.cur_ptr.align_offset(align);
        unsafe {
            // SAFETY, we never access this computed pointer
            if self.cur_ptr.add(align_offset + size) >= self.block_end {
                Err(AllocError)
            } else {
                // SAFETY, We ensured that we have space for data with this size/align in our block,
                // we return the aligned slot pointer and increment the current pointer to be at the end of that "allocation"
                let slot_start = self.cur_ptr.add(align_offset);
                self.cur_ptr = slot_start.add(size);
                Ok(slot_start)
            }
        }
    }
}

/// One time use bump allocator.
/// Useful for many values / objects with the same lifetime.
/// Allocates memory in large blocks all at once, mutable references to values are returned, drops only happen when the whole struct is dropped.
/// Self references are easy because all lifetimes are garunteed to live for the whole arena.
pub struct Corrida<const BLOCK_SIZE: usize> 
{
    cur_block: Cell<NonNull<BlockMeta>>,
    _boo: PhantomData<BlockMeta>,
}


impl<const BLOCK_SIZE: usize> Corrida<BLOCK_SIZE> 
where Assert<{BLOCK_SIZE % BLOCK_MIN_ALIGN == 0}>: IsTrue,
      Assert<{BLOCK_SIZE <= (1 << 22)}> : IsTrue
{
    /// Creates a new arena with a block to start. Generic over the default block size in bytes.
    pub fn new() -> Self {
        Self {
            cur_block: Cell::new(BlockMeta::new(None, BLOCK_SIZE)),
            _boo: PhantomData,
        }
    }

    /// Allocate the given value at the current pointer in the current block.
    /// Will create a new block if the current one does not have enough free space.
    pub fn alloc<F>(&self, fighter: F) -> &mut F 
        where [(); {size_of::<F>() <= BLOCK_SIZE} as usize]: ,
              [(); {BLOCK_MIN_ALIGN % align_of::<F>() == 0} as usize]: ,
    {
        let size: usize = size_of::<F>();
        let layout = align_of::<F>();

        unsafe {
            let slot = match (*self.cur_block.get().as_ptr()).alloc(size, layout) {
                Ok(slot) => slot,
                Err(_) => {
                    let old_block = NonNull::new_unchecked(self.cur_block.get().as_ptr());
                    let mut new_block = BlockMeta::new(Some(old_block), BLOCK_SIZE);

                    self.cur_block.set(new_block);
                    // SAFETY, New Block is a valid Block
                    new_block.as_mut().alloc(size, layout).unwrap()
                }
            };

            let slot = slot.as_ptr() as *mut F;
            //SAFETY, garunteed to have space and align required for F.
            slot.write(fighter);
            &mut *slot
        }
        // }
    }
}

impl<const BLOCK_SIZE: usize> Drop for Corrida<{BLOCK_SIZE}> 
{
    fn drop(&mut self) {
        unsafe {
            let mut cur_block_nn = self.cur_block.get();

            loop {
                let block_metadata = cur_block_nn.as_mut();

                let prev = block_metadata.prev;
                let size = block_metadata
                    .block_end
                    .offset_from(block_metadata.block_start) as usize
                    + size_of::<BlockMeta>();
                let layout = Layout::from_size_align_unchecked(size, BLOCK_MIN_ALIGN);

                let block_ptr = block_metadata.block_start;
                // drop(block_metadata);

                Global.deallocate(block_ptr, layout);

                match prev {
                    Some(prev) => {
                        cur_block_nn = prev;
                    }
                    None => {
                        break;
                    }
                }
            }
        };
    }
}

#[cfg(test)]
mod test {
    use std::hint::black_box;

    use super::{BlockMeta, Corrida};

    #[test]
    fn test_isolated_arena() {
        dbg!(size_of::<BlockMeta>());
        dbg!(align_of::<BlockMeta>());

        let arena = Corrida::<{1 << 16}>::new();
        {
            let a = arena.alloc(1);
            let b = arena.alloc(2);
            let c = arena.alloc(3);
            assert_eq!(*a, 1);
            assert_eq!(*b, 2);
            assert_eq!(*c, 3);
        }
    }

    #[test]
    fn test_large() {
        use std::time::*;
        // Each fighter is 4*16, 64 bytes
        let start = Instant::now();
        let arena = Corrida::<{1 << 20}>::new();
        for i in 0..5_000_000 {
            let _my_ref = arena.alloc([i, i, i, i, i, i, i, i, i, i, i, i, i, i, i, i]);
            black_box(_my_ref);
        }
        println!("Arena: {}", start.elapsed().as_millis());
    }

    #[test]
    fn test_mixed() {
        let arena = Corrida::<{1 << 16}>::new();

        let arr = arena.alloc([1; 100]);
        let char = arena.alloc('c');
        let i32 = arena.alloc(1);
        let arena_inside_arena = arena.alloc(Corrida::<{1 << 12}>::new());

        arena_inside_arena.alloc(*arr);
        arena_inside_arena.alloc(*char);
        arena_inside_arena.alloc(*i32);

        println!("{:?}", arr);
        println!("{}", char);
        println!("{}", i32);
    }
}
